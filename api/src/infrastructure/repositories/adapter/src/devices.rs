use entities::{Device, Pagination};
use sqlx::{PgConnection, types::mac_address::MacAddress};

#[async_trait::async_trait]
pub trait DevicesRepositoryAdapter: Send + Sync {
    async fn fetch_all(
        &self,
        connection: &mut PgConnection,
        pagination: Option<Pagination>,
    ) -> Vec<Device>;

    async fn fetch_one(
        &self,
        connection: &mut PgConnection,
        mac_address: MacAddress,
    ) -> Option<Device>;

    async fn create(&self, connection: &mut PgConnection, device: Device);
    async fn update(&self, connection: &mut PgConnection, device: Device);
}
