use std::{collections::HashMap, sync::Arc, time::Instant};

use ports::{
    api::InternetProviderApi,
    repositories::{DevicesRepository, UnitOfWorkProvider},
};

use crate::PeriodicUseCase;

pub struct SyncDevicesUseCase<DR: DevicesRepository<UWP>, UWP: UnitOfWorkProvider> {
    _marker: std::marker::PhantomData<DR>,
    internet_provider_api: Arc<dyn InternetProviderApi>,
    uow_provider: UWP,
}

impl<DR: DevicesRepository<UWP>, UWP: UnitOfWorkProvider> SyncDevicesUseCase<DR, UWP> {
    pub fn new(uow_provider: UWP, internet_provider_api: Arc<dyn InternetProviderApi>) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            internet_provider_api,
            uow_provider,
        }
    }
}

#[async_trait::async_trait]
impl<DR: DevicesRepository<UWP>, UWP: UnitOfWorkProvider + 'static> PeriodicUseCase
    for SyncDevicesUseCase<DR, UWP>
{
    fn next_execution(&self) -> Option<Instant> {
        Some(Instant::now() + std::time::Duration::from_secs(60))
    }

    async fn execute(&self) {
        let mut uow = self.uow_provider.begin_transaction().await;
        let scanned_devices = self.internet_provider_api.list_devices().await;
        let known_devices = DR::fetch_all(&mut uow, None).await;

        let mut known_map = known_devices
            .into_iter()
            .map(|d| (d.mac_address.clone(), d))
            .collect::<HashMap<_, _>>();

        pub enum Op {
            Update,
            Create,
        }

        for (device, op) in scanned_devices.into_iter().map(|scanned| {
            known_map
                .remove(&scanned.mac_address)
                .map(|device| (device.update(scanned.clone()), Op::Update))
                .unwrap_or_else(|| (scanned, Op::Create))
        }) {
            match op {
                Op::Update => DR::update(&mut uow, device).await,
                Op::Create => DR::create(&mut uow, device).await,
            }
        }

        for device in known_map.into_values().map(|mut d| {
            d.is_online = false;
            d
        }) {
            DR::update(&mut uow, device).await;
        }

        self.uow_provider.commit(uow).await;
    }
}
