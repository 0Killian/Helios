use std::{collections::HashSet, str::FromStr};

use entities::{
    ApplicationProtocol, Service, ServiceKind, ServicePort, ServicePortTemplate, TransportProtocol,
};
use ports::repositories::{Repository, RepositoryError, RepositoryResult, ServicesRepository};
use sqlx::{PgConnection, Postgres, prelude::FromRow, types::mac_address::MacAddress};
use tracing::{error, instrument};
use uuid::Uuid;

use crate::{PostgresUWP, PostgresUoW, map_sqlx_error};

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

fn service_with_port_group_to_service(
    services_with_port: &[ServiceWithPort],
) -> RepositoryResult<Service> {
    let map_parse_err = |field: &str, value: &str| {
        error!("Failed to parse {} from {}", field, value);
        RepositoryError::Unknown
    };

    let mut service = Service {
        service_id: services_with_port[0].service_id,
        device_mac: services_with_port[0].service_device_mac,
        display_name: services_with_port[0].service_display_name.clone(),
        kind: ServiceKind::from_str(&services_with_port[0].service_kind)
            .map_err(|_| map_parse_err("kind", &services_with_port[0].service_kind))?,
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
            .map_err(|_| {
                map_parse_err(
                    "transport_protocol",
                    &service_with_port.port_transport_protocol,
                )
            })?,
            application_protocol: ApplicationProtocol::from_str(
                &service_with_port.port_application_protocol,
            )
            .map_err(|_| {
                map_parse_err(
                    "application_protocol",
                    &service_with_port.port_application_protocol,
                )
            })?,
            is_online: service_with_port.port_is_online,
        });
    }

    Ok(service)
}

impl Repository<PostgresUWP> for PostgresServicesRepository {}

#[async_trait::async_trait]
impl ServicesRepository<PostgresUWP> for PostgresServicesRepository {
    #[instrument(skip(connection))]
    async fn fetch_all_of_device<'a>(
        connection: &'a mut PostgresUoW<'_>,
        mac_address: MacAddress,
    ) -> RepositoryResult<Vec<Service>> {
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
        .map_err(map_sqlx_error)?
        .chunk_by(|s1, s2| s1.service_id == s2.service_id)
        .into_iter()
        .map(|chunk| service_with_port_group_to_service(chunk))
        .collect()
    }

    #[instrument(skip(connection))]
    async fn fetch_one<'a>(
        connection: &'a mut PostgresUoW<'_>,
        service_id: Uuid,
    ) -> RepositoryResult<Service> {
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
        .map_err(map_sqlx_error)?;

        if services.len() == 0 {
            Err(RepositoryError::NotFound)
        } else {
            Ok(service_with_port_group_to_service(&services)?)
        }
    }

    #[instrument(skip(connection))]
    async fn find_one<'a>(
        connection: &'a mut PostgresUoW<'_>,
        mac_address: MacAddress,
        kind: ServiceKind,
        ports: &[ServicePortTemplate],
    ) -> RepositoryResult<Option<Service>> {
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
        .map_err(map_sqlx_error)?
        .chunk_by(|s1, s2| s1.service_id == s2.service_id)
        .into_iter()
        .map(|chunk| service_with_port_group_to_service(chunk))
        .collect::<Result<Vec<_>, _>>()?;

        // Check that the ports are the same
        for service in services {
            let existing_ports_set: HashSet<_> = service
                .ports
                .iter()
                .map(|port| {
                    (
                        port.port,
                        &port.name,
                        port.transport_protocol,
                        port.application_protocol,
                    )
                })
                .collect();

            let input_ports_set: HashSet<_> = ports
                .iter()
                .map(|port| {
                    (
                        port.port,
                        &port.name,
                        port.transport_protocol,
                        port.application_protocol,
                    )
                })
                .collect();

            if existing_ports_set == input_ports_set {
                return Ok(Some(service));
            }
        }

        Ok(None)
    }

    #[instrument(skip(connection))]
    async fn create<'a>(
        connection: &'a mut PostgresUoW<'_>,
        service: Service,
    ) -> RepositoryResult<()> {
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
        .map_err(map_sqlx_error)?;

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
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }

    #[instrument(skip(connection))]
    async fn update<'a>(
        connection: &'a mut PostgresUoW<'_>,
        service: Service,
    ) -> RepositoryResult<()> {
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
        .map_err(map_sqlx_error)?;

        sqlx::query(
            r#"
            DELETE FROM core.service_ports WHERE service_id = $1
            "#,
        )
        .bind(service.service_id)
        .execute((&mut *connection) as &mut PgConnection)
        .await
        .map_err(map_sqlx_error)?;

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
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }
}
