use entities::Service;
use sqlx::{PgConnection, types::mac_address::MacAddress};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait ServicesRepositoryAdapter {
    async fn fetch_all_of_device(
        &self,
        connection: &mut PgConnection,
        mac_address: MacAddress,
    ) -> Vec<Service>;
    async fn fetch_one(&self, connection: &mut PgConnection, service_id: Uuid) -> Option<Service>;
}
