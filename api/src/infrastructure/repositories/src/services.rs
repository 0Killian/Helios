use std::str::FromStr;

use entities::{
    ApplicationProtocol, Service, ServiceKind, ServicePort, ServicePortTemplate, TransportProtocol,
};
use itertools::Itertools;
use ports::repositories::{Repository, ServicesRepository};
use sqlx::{PgConnection, Postgres, prelude::FromRow, types::mac_address::MacAddress};
use uuid::Uuid;

use crate::{PostgresUWP, PostgresUoW};

#[derive(Clone)]
pub struct PostgresServicesRepository;

#[derive(FromRow)]
struct ServiceWithPort {
    pub service_id: Uuid,
    pub service_device_mac: MacAddress,
    pub service_display_name: String,
    pub service_kind: String,
    pub service_is_managed: bool,
    pub service_token: String,
    pub port_name: String,
    #[sqlx(try_from = "i32")]
    pub port_port: u16,
    pub port_transport_protocol: String,
    pub port_application_protocol: String,
    pub port_is_online: bool,
}

fn service_with_port_group_to_service(services_with_port: Vec<&ServiceWithPort>) -> Service {
    let mut service = Service {
        service_id: services_with_port[0].service_id,
        device_mac: services_with_port[0].service_device_mac,
        display_name: services_with_port[0].service_display_name.clone(),
        kind: ServiceKind::from_str(&services_with_port[0].service_kind).unwrap(),
        is_managed: services_with_port[0].service_is_managed,
        token: services_with_port[0].service_token.clone(),
        ports: Vec::new(),
    };

    for service_with_port in services_with_port {
        service.ports.push(ServicePort {
            name: service_with_port.port_name.clone(),
            port: service_with_port.port_port,
            transport_protocol: TransportProtocol::from_str(
                &service_with_port.port_transport_protocol,
            )
            .unwrap(),
            application_protocol: ApplicationProtocol::from_str(
                &service_with_port.port_application_protocol,
            )
            .unwrap(),
            is_online: service_with_port.port_is_online,
        });
    }

    service
}

impl Repository<PostgresUWP> for PostgresServicesRepository {}

#[async_trait::async_trait]
impl ServicesRepository<PostgresUWP> for PostgresServicesRepository {
    async fn fetch_all_of_device<'a>(
        connection: &'a mut PostgresUoW<'_>,
        mac_address: MacAddress,
    ) -> Vec<Service> {
        sqlx::query_as::<Postgres, ServiceWithPort>(
            r#"
            SELECT
                s.service_id as service_id,
                s.device_mac as service_device_mac,
                s.display_name as service_display_name,
                s.kind as service_kind,
                s.is_managed as service_is_managed,
                s.token as service_token,
                sp.name as port_name,
                sp.port as port_port,
                sp.transport_protocol as port_transport_protocol,
                sp.application_protocol as port_application_protocol,
                sp.is_online as port_is_online
            FROM core.services s
            INNER JOIN core.service_ports sp ON s.service_id = sp.service_id
            WHERE device_mac = $1
        "#,
        )
        .bind(mac_address)
        .fetch_all(connection as &'a mut PgConnection)
        .await
        .unwrap()
        .iter()
        .chunk_by(|service| service.service_id)
        .into_iter()
        .map(|chunk| service_with_port_group_to_service(chunk.1.collect_vec()))
        .collect()
    }

    async fn fetch_one<'a>(
        connection: &'a mut PostgresUoW<'_>,
        service_id: Uuid,
    ) -> Option<Service> {
        let services = sqlx::query_as::<Postgres, ServiceWithPort>(
            r#"
            SELECT
                s.service_id as service_id,
                s.device_mac as service_device_mac,
                s.display_name as service_display_name,
                s.kind as service_kind,
                s.is_managed as service_is_managed,
                s.token as service_token,
                sp.name as port_name,
                sp.port as port_port,
                sp.transport_protocol as port_transport_protocol,
                sp.application_protocol as port_application_protocol,
                sp.is_online as port_is_online
            FROM core.services s
            INNER JOIN core.service_ports sp ON s.service_id = sp.service_id
            WHERE s.service_id = $1
        "#,
        )
        .bind(service_id)
        .fetch_all(connection as &'a mut PgConnection)
        .await
        .unwrap();

        if services.len() == 0 {
            None
        } else {
            Some(service_with_port_group_to_service(
                services.iter().collect_vec(),
            ))
        }
    }

    async fn find_one<'a>(
        connection: &'a mut PostgresUoW<'_>,
        mac_address: MacAddress,
        kind: ServiceKind,
        ports: &[ServicePortTemplate],
    ) -> Option<Service> {
        let services: Vec<Service> = sqlx::query_as::<Postgres, ServiceWithPort>(
            r#"
            SELECT
                s.service_id as service_id,
                s.device_mac as service_device_mac,
                s.display_name as service_display_name,
                s.kind as service_kind,
                s.is_managed as service_is_managed,
                s.token as service_token,
                sp.name as port_name,
                sp.port as port_port,
                sp.transport_protocol as port_transport_protocol,
                sp.application_protocol as port_application_protocol,
                sp.is_online as port_is_online
            FROM core.services s
            INNER JOIN core.service_ports sp ON s.service_id = sp.service_id
            WHERE s.device_mac = $1 AND s.kind = $2
        "#,
        )
        .bind(mac_address)
        .bind(kind.to_string())
        .fetch_all(connection as &'a mut PgConnection)
        .await
        .unwrap()
        .iter()
        .chunk_by(|service| service.service_id)
        .into_iter()
        .map(|chunk| service_with_port_group_to_service(chunk.1.collect_vec()))
        .collect();

        // Check that the ports are the same
        for service in services {
            let mut ports_match = true;
            for port in ports {
                if service.ports.iter().all(|p| port.matches(p)) {
                    ports_match = false;
                    break;
                }
            }

            if ports_match {
                return Some(service);
            }
        }

        None
    }

    async fn create<'a>(connection: &'a mut PostgresUoW<'_>, service: Service) {
        sqlx::query(
            r#"
            INSERT INTO core.services (
                service_id,
                device_mac,
                display_name,
                kind,
                is_managed,
                token
            ) VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        )
        .bind(service.service_id)
        .bind(service.device_mac)
        .bind(service.display_name)
        .bind(service.kind.to_string())
        .bind(service.is_managed)
        .bind(service.token)
        .execute((&mut *connection) as &mut PgConnection)
        .await
        .unwrap();

        for port in service.ports {
            sqlx::query(
                r#"
                INSERT INTO core.service_ports (
                    service_id,
                    name,
                    port,
                    transport_protocol,
                    application_protocol,
                    is_online
                ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            )
            .bind(service.service_id)
            .bind(port.name)
            .bind(port.port as i64)
            .bind(port.transport_protocol.to_string())
            .bind(port.application_protocol.to_string())
            .bind(port.is_online)
            .execute((&mut *connection) as &mut PgConnection)
            .await
            .unwrap();
        }
    }

    async fn update<'a>(connection: &'a mut PostgresUoW<'_>, service: Service) {
        sqlx::query(
            r#"
            UPDATE core.services
            SET device_mac = $2,
                display_name = $3,
                kind = $4,
                is_managed = $5
            WHERE service_id = $1
            "#,
        )
        .bind(service.service_id)
        .bind(service.device_mac)
        .bind(service.display_name)
        .bind(service.kind.to_string())
        .bind(service.is_managed)
        .execute((&mut *connection) as &mut PgConnection)
        .await
        .unwrap();

        sqlx::query(
            r#"
            DELETE FROM core.service_ports WHERE service_id = $1
            "#,
        )
        .bind(service.service_id)
        .execute((&mut *connection) as &mut PgConnection)
        .await
        .unwrap();

        for port in service.ports {
            sqlx::query(
                r#"
                INSERT INTO core.service_ports (
                    service_id,
                    name,
                    port,
                    transport_protocol,
                    application_protocol,
                    is_online
                ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            )
            .bind(service.service_id)
            .bind(port.name)
            .bind(port.port as i64)
            .bind(port.transport_protocol.to_string())
            .bind(port.application_protocol.to_string())
            .bind(port.is_online)
            .execute((&mut *connection) as &mut PgConnection)
            .await
            .unwrap();
        }
    }
}
