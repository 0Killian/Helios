use entities::Service;
use mac_address::MacAddress;
use uuid::Uuid;

use crate::repositories::{Repository, UnitOfWorkProvider};

#[async_trait::async_trait]
pub trait ServicesRepository<UWP>: Repository<UWP> + Send + Sync + Clone
where
    UWP: UnitOfWorkProvider,
{
    async fn fetch_all_of_device<'a>(
        uow: &'a mut UWP::UnitOfWork<'_>,
        mac_address: MacAddress,
    ) -> Vec<Service>;

    async fn fetch_one<'a>(uow: &'a mut UWP::UnitOfWork<'_>, service_id: Uuid) -> Option<Service>;
    async fn create<'a>(uow: &'a mut UWP::UnitOfWork<'_>, service: Service);
    async fn update<'a>(uow: &'a mut UWP::UnitOfWork<'_>, service: Service);
}
