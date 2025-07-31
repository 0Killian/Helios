use entities::{Device, Pagination};
use mac_address::MacAddress;

use crate::repositories::{Repository, UnitOfWorkProvider};

use super::RepositoryResult;

#[async_trait::async_trait]
pub trait DevicesRepository<UWP>: Repository<UWP> + Send + Sync + Clone
where
    UWP: UnitOfWorkProvider,
{
    async fn fetch_all<'a>(
        uow: &'a mut UWP::UnitOfWork<'_>,
        pagination: Option<Pagination>,
    ) -> RepositoryResult<Vec<Device>>;

    async fn fetch_one<'a>(
        uow: &'a mut UWP::UnitOfWork<'_>,
        mac_address: MacAddress,
    ) -> RepositoryResult<Option<Device>>;

    async fn create<'a>(uow: &'a mut UWP::UnitOfWork<'_>, device: Device) -> RepositoryResult<()>;
    async fn update<'a>(uow: &'a mut UWP::UnitOfWork<'_>, device: Device) -> RepositoryResult<()>;
}
