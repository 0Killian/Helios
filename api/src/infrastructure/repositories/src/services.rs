use entities::{ApplicationProtocol, Service, ServiceKind, ServicePort, TransportProtocol};
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
    pub service_kind: ServiceKind,
    pub service_is_managed: bool,
    #[sqlx(try_from = "i64")]
    pub port_port: u16,
    pub port_transport_protocol: TransportProtocol,
    pub port_application_protocol: ApplicationProtocol,
    pub port_is_online: bool,
}

fn service_with_port_group_to_service(services_with_port: Vec<&ServiceWithPort>) -> Service {
    let mut service = Service {
        service_id: services_with_port[0].service_id,
        device_mac: services_with_port[0].service_device_mac,
        display_name: services_with_port[0].service_display_name.clone(),
        kind: services_with_port[0].service_kind,
        is_managed: services_with_port[0].service_is_managed,
        ports: Vec::new(),
    };

    for service_with_port in services_with_port {
        service.ports.push(ServicePort {
            port: service_with_port.port_port,
            transport_protocol: service_with_port.port_transport_protocol,
            application_protocol: service_with_port.port_application_protocol,
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
            SELECT * FROM core.services s
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
            SELECT * FROM core.services s
            INNER JOIN core.service_ports sp ON s.service_id = sp.service_id
            WHERE service_id = $1
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

    async fn create<'a>(connection: &'a mut PostgresUoW<'_>, service: Service) {
        sqlx::query(
            r#"
            INSERT INTO core.services (
                service_id,
                device_mac,
                display_name,
                kind,
                is_managed
            ) VALUES ($1, $2, $3, $4, $5)
        "#,
        )
        .bind(service.service_id)
        .bind(service.device_mac)
        .bind(service.display_name)
        .bind(service.kind)
        .bind(service.is_managed)
        .execute((&mut *connection) as &mut PgConnection)
        .await
        .unwrap();

        for port in service.ports {
            sqlx::query(
                r#"
                INSERT INTO core.service_ports (
                    service_id,
                    port,
                    transport_protocol,
                    application_protocol,
                    is_online
                ) VALUES ($1, $2, $3, $4, $5)
            "#,
            )
            .bind(service.service_id)
            .bind(port.port as i64)
            .bind(port.transport_protocol)
            .bind(port.application_protocol)
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
        .bind(service.kind)
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
                    port,
                    transport_protocol,
                    application_protocol,
                    is_online
                ) VALUES ($1, $2, $3, $4, $5)
            "#,
            )
            .bind(service.service_id)
            .bind(port.port as i64)
            .bind(port.transport_protocol)
            .bind(port.application_protocol)
            .bind(port.is_online)
            .execute((&mut *connection) as &mut PgConnection)
            .await
            .unwrap();
        }
    }
}
