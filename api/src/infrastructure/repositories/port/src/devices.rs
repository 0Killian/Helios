use entities::{Device, Pagination, ToSql};
use repositories_adapter::DevicesRepositoryAdapter;
use sqlx::{PgConnection, types::mac_address::MacAddress};

pub struct DevicesRepositoryPort {}

#[async_trait::async_trait]
impl DevicesRepositoryAdapter for DevicesRepositoryPort {
    async fn fetch_all(
        &self,
        connection: &mut PgConnection,
        pagination: Option<Pagination>,
    ) -> Vec<Device> {
        sqlx::query_as(&format!(
            "SELECT * FROM core.devices {}",
            pagination.to_sql()
        ))
        .fetch_all(connection)
        .await
        .unwrap()
    }

    async fn fetch_one(
        &self,
        connection: &mut PgConnection,
        mac_address: MacAddress,
    ) -> Option<Device> {
        sqlx::query_as("SELECT * FROM core.devices WHERE mac_address = $1")
            .bind(mac_address)
            .fetch_optional(connection)
            .await
            .unwrap()
    }

    async fn create(&self, connection: &mut PgConnection, device: Device) {
        sqlx::query(
            r#"
            INSERT INTO core.devices (
                mac_address,
                last_known_ip,
                display_name,
                is_name_custom,
                notes,
                is_online,
                last_seen,
                last_scanned
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(device.mac_address)
        .bind(device.last_known_ip)
        .bind(device.display_name)
        .bind(device.is_name_custom)
        .bind(device.notes)
        .bind(device.is_online)
        .bind(device.last_seen)
        .bind(device.last_scanned)
        .execute(connection)
        .await
        .unwrap();
    }

    async fn update(&self, connection: &mut PgConnection, device: Device) {
        sqlx::query(
            r#"
            UPDATE core.devices
            SET last_known_ip = $2,
                display_name = $3,
                is_name_custom = $4,
                notes = $5,
                is_online = $6,
                last_seen = $7,
                last_scanned = $8
            WHERE mac_address = $1
            "#,
        )
        .bind(device.mac_address)
        .bind(device.last_known_ip)
        .bind(device.display_name)
        .bind(device.is_name_custom)
        .bind(device.notes)
        .bind(device.is_online)
        .bind(device.last_seen)
        .bind(device.last_scanned)
        .execute(connection)
        .await
        .unwrap();
    }
}
