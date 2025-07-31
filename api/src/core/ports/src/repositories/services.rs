use entities::{Service, ServiceKind, ServicePortTemplate};
use mac_address::MacAddress;
use uuid::Uuid;

use crate::repositories::{Repository, UnitOfWorkProvider};

use super::RepositoryResult;

#[async_trait::async_trait]
pub trait ServicesRepository<UWP>: Repository<UWP> + Send + Sync + Clone
where
    UWP: UnitOfWorkProvider,
{
    async fn fetch_all_of_device<'a>(
        uow: &'a mut UWP::UnitOfWork<'_>,
        mac_address: MacAddress,
    ) -> RepositoryResult<Vec<Service>>;

    async fn fetch_one<'a>(
        uow: &'a mut UWP::UnitOfWork<'_>,
        service_id: Uuid,
    ) -> RepositoryResult<Service>;

    async fn find_one<'a>(
        uow: &'a mut UWP::UnitOfWork<'_>,
        mac_address: MacAddress,
        kind: ServiceKind,
        ports: &[ServicePortTemplate],
    ) -> RepositoryResult<Option<Service>>;

    async fn create<'a>(uow: &'a mut UWP::UnitOfWork<'_>, service: Service)
    -> RepositoryResult<()>;

    async fn update<'a>(uow: &'a mut UWP::UnitOfWork<'_>, service: Service)
    -> RepositoryResult<()>;
}
