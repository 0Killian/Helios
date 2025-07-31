use entities::{FullDevice, Pagination};
use ports::repositories::{
    DevicesRepository, RepositoryResult, ServicesRepository, UnitOfWorkProvider,
};
use tracing::instrument;

#[derive(Clone)]
pub struct ListDevicesUseCase<
    DR: DevicesRepository<UWP>,
    SR: ServicesRepository<UWP>,
    UWP: UnitOfWorkProvider,
> {
    uow_provider: UWP,
    _marker: std::marker::PhantomData<(DR, SR)>,
}

impl<DR: DevicesRepository<UWP>, SR: ServicesRepository<UWP>, UWP: UnitOfWorkProvider>
    ListDevicesUseCase<DR, SR, UWP>
{
    pub fn new(uow_provider: UWP) -> Self {
        Self {
            uow_provider,
            _marker: std::marker::PhantomData,
        }
    }

    #[instrument(skip(self), name = "ListDevicesUseCase::execute")]
    pub async fn execute(
        &self,
        pagination: Option<Pagination>,
        full: bool,
    ) -> RepositoryResult<Vec<FullDevice>> {
        let mut uow = self.uow_provider.begin_transaction().await?;
        let devices = DR::fetch_all(&mut uow, pagination).await?;

        if devices.is_empty() {
            return Ok(vec![]);
        }

        let mut devices = devices
            .into_iter()
            .map(|device| FullDevice {
                device,
                services: None,
            })
            .collect::<Vec<_>>();

        if full {
            for device in &mut devices {
                device.services =
                    Some(SR::fetch_all_of_device(&mut uow, device.device.mac_address).await?);
            }
        }

        Ok(devices)
    }
}
