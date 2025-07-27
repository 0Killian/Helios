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
}
