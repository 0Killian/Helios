use entities::Service;
use mac_address::MacAddress;
use ports::repositories::{RepositoryResult, ServicesRepository, UnitOfWorkProvider};

#[derive(Clone)]
pub struct ListServicesUseCase<SR: ServicesRepository<UWP>, UWP: UnitOfWorkProvider> {
    uow_provider: UWP,
    _marker: std::marker::PhantomData<SR>,
}

impl<SR: ServicesRepository<UWP>, UWP: UnitOfWorkProvider> ListServicesUseCase<SR, UWP> {
    pub fn new(uow_provider: UWP) -> Self {
        Self {
            uow_provider,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn execute(&self, mac_address: MacAddress) -> RepositoryResult<Vec<Service>> {
        let mut uwo = self.uow_provider.begin_transaction().await?;
        let devices = SR::fetch_all_of_device(&mut uwo, mac_address).await?;
        Ok(devices)
    }
}
