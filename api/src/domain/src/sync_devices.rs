use std::{collections::HashMap, sync::Arc, time::Instant};

use ports::{
    api::RouterApi,
    repositories::{DevicesRepository, UnitOfWorkProvider},
};

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

    async fn execute(&self) {
        let mut uow = match self.uow_provider.begin_transaction().await {
            Ok(uow) => uow,
            Err(err) => {
                println!("Failed to begin transaction: {}", err);
                return;
            }
        };

        let scanned_devices = match self.router_api.list_devices().await {
            Ok(devices) => devices,
            Err(err) => {
                println!("Failed to fetch devices: {}", err);
                return;
            }
        };

        let known_devices = match DR::fetch_all(&mut uow, None).await {
            Ok(devices) => devices,
            Err(err) => {
                println!("Failed to fetch devices: {}", err);
                return;
            }
        };

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
                Op::Update => match DR::update(&mut uow, device).await {
                    Ok(_) => (),
                    Err(err) => println!("Failed to update device: {}", err),
                },
                Op::Create => match DR::create(&mut uow, device).await {
                    Ok(_) => (),
                    Err(err) => println!("Failed to create device: {}", err),
                },
            }
        }

        for device in known_map.into_values().map(|mut d| {
            d.is_online = false;
            d
        }) {
            match DR::update(&mut uow, device).await {
                Ok(_) => (),
                Err(err) => println!("Failed to update device: {}", err),
            };
        }

        match self.uow_provider.commit(uow).await {
            Ok(_) => (),
            Err(err) => println!("Failed to commit transaction: {}", err),
        };
    }
}
