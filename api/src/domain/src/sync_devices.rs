use std::{collections::HashMap, sync::Arc, time::Instant};

use ports::{
    api::RouterApi,
    repositories::{DevicesRepository, UnitOfWorkProvider},
};
use tracing::{error, info, instrument};

use crate::PeriodicUseCase;

pub struct SyncDevicesUseCase<DR: DevicesRepository<UWP>, UWP: UnitOfWorkProvider> {
    _marker: std::marker::PhantomData<DR>,
    router_api: Arc<dyn RouterApi>,
    uow_provider: UWP,
}

impl<DR: DevicesRepository<UWP>, UWP: UnitOfWorkProvider> SyncDevicesUseCase<DR, UWP> {
    pub fn new(uow_provider: UWP, router_api: Arc<dyn RouterApi>) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            router_api,
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

    #[instrument(skip(self), name = "SyncDevicesUseCase::execute")]
    async fn execute(&self) {
        let mut uow = match self.uow_provider.begin_transaction().await {
            Ok(uow) => uow,
            Err(err) => {
                error!("Failed to begin transaction: {}", err);
                return;
            }
        };

        let scanned_devices = match self.router_api.list_devices().await {
            Ok(devices) => devices,
            Err(err) => {
                error!("Failed to fetch devices: {}", err);
                return;
            }
        };

        let known_devices = match DR::fetch_all(&mut uow, None).await {
            Ok(devices) => devices,
            Err(err) => {
                error!("Failed to fetch devices: {}", err);
                return;
            }
        };

        info!(
            scanned_devices = scanned_devices.len(),
            known_devices = known_devices.len(),
            "Fetched devices from router and database"
        );

        let mut known_map = known_devices
            .into_iter()
            .map(|d| (d.mac_address.clone(), d))
            .collect::<HashMap<_, _>>();

        let mut new_devices = Vec::new();
        let mut disconnected_devices = Vec::new();
        let mut reconnected_devices = Vec::new();

        for update in scanned_devices.into_iter().map(|scanned| {
            known_map
                .remove(&scanned.mac_address)
                .map(|device| (Some(device.clone()), device.update(scanned.clone())))
                .unwrap_or_else(|| (None, scanned))
        }) {
            match update {
                (Some(old), new) => match DR::update(&mut uow, new.clone()).await {
                    Ok(_) if new.is_online != old.is_online => {
                        if !new.is_online {
                            disconnected_devices.push(new);
                        } else {
                            reconnected_devices.push(new);
                        }
                    }
                    Ok(_) => (),
                    Err(err) => error!("Failed to update device: {}", err),
                },
                (None, device) => match DR::create(&mut uow, device.clone()).await {
                    Ok(_) => new_devices.push(device),
                    Err(err) => error!("Failed to create device: {}", err),
                },
            }
        }

        for device in known_map.into_values().map(|mut d| {
            d.is_online = false;
            d
        }) {
            match DR::update(&mut uow, device.clone()).await {
                Ok(_) => disconnected_devices.push(device),
                Err(err) => error!("Failed to update device: {}", err),
            };
        }

        match self.uow_provider.commit(uow).await {
            Ok(_) => (),
            Err(err) => error!("Failed to commit transaction: {}", err),
        };

        info!(
            new_devices = ?new_devices.iter().map(|d| d.mac_address.to_string()).collect::<Vec<_>>(),
            disconnected_devices = ?disconnected_devices.iter().map(|d| d.mac_address.to_string()).collect::<Vec<_>>(),
            reconnected_devices = ?reconnected_devices.iter().map(|d| d.mac_address.to_string()).collect::<Vec<_>>(),
            "Finished syncing devices"
        );
    }
}
