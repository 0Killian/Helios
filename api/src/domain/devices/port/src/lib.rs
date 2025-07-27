use std::sync::Arc;

use devices_adapter::DevicesAdapter;
use entities::{Device, Pagination, Service, SharedLockedReference};
use ip_api_adapter::InternetProviderApiAdapter;
use mac_address::MacAddress;
use repositories_adapter::{DevicesRepositoryAdapter, ServicesRepositoryAdapter};
use sqlx::PgPool;

pub struct DevicesPort {
    ip_api: Arc<dyn InternetProviderApiAdapter>,
    devices_repository: Arc<dyn DevicesRepositoryAdapter>,
    services_repository: Arc<dyn ServicesRepositoryAdapter>,
    pg_pool: SharedLockedReference<PgPool>,
}

impl DevicesPort {
    pub fn new(
        ip_api: Arc<dyn InternetProviderApiAdapter>,
        devices_repository: Arc<dyn DevicesRepositoryAdapter>,
        services_repository: Arc<dyn ServicesRepositoryAdapter>,
        pg_pool: SharedLockedReference<PgPool>,
    ) -> Self {
        DevicesPort {
            ip_api,
            devices_repository,
            services_repository,
            pg_pool,
        }
    }
}

#[async_trait::async_trait]
impl DevicesAdapter for DevicesPort {
    async fn list_devices(&self, pagination: Option<Pagination>) -> Vec<Device> {
        let mut conn = self.pg_pool.lock().await.acquire().await.unwrap();
        let devices = self
            .devices_repository
            .fetch_all(&mut conn, pagination)
            .await;
        devices
    }

    async fn list_services(&self, mac_address: MacAddress) -> Vec<Service> {
        let mut conn = self.pg_pool.lock().await.acquire().await.unwrap();
        let services = self
            .services_repository
            .fetch_all_of_device(&mut conn, mac_address)
            .await;
        services
    }

    async fn start_devices_scan(&self, delay: chrono::Duration) {
        loop {
            println!("Starting device scan");
            let mut tx = self.pg_pool.lock().await.begin().await.unwrap();
            let scanned_devices = self.ip_api.list_devices().await;

            for device in scanned_devices {
                if let Some(dev) = self
                    .devices_repository
                    .fetch_one(&mut tx, device.mac_address)
                    .await
                {
                    let device = dev.update(device);
                    println!("Updating device: {}", device.mac_address);
                    self.devices_repository.update(&mut tx, device).await;
                } else {
                    println!("Creating device: {}", device.mac_address);
                    self.devices_repository.create(&mut tx, device).await;
                }
            }

            tx.commit().await.unwrap();

            tokio::time::sleep(delay.to_std().unwrap()).await;
        }
    }
}
