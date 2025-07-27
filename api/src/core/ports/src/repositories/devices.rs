use entities::{Device, Pagination};
use mac_address::MacAddress;

use crate::repositories::{Repository, UnitOfWorkProvider};

#[async_trait::async_trait]
pub trait DevicesRepository<UWP>: Repository<UWP> + Send + Sync + Clone
where
    UWP: UnitOfWorkProvider,
{
    async fn fetch_all<'a>(
        uow: &'a mut UWP::UnitOfWork<'_>,
        pagination: Option<Pagination>,
    ) -> Vec<Device>;
    async fn fetch_one<'a>(
        uow: &'a mut UWP::UnitOfWork<'_>,
        mac_address: MacAddress,
    ) -> Option<Device>;
    async fn create<'a>(uow: &'a mut UWP::UnitOfWork<'_>, device: Device);
    async fn update<'a>(uow: &'a mut UWP::UnitOfWork<'_>, device: Device);
}
