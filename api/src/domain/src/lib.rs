mod fetch_network_status;
mod list_devices;
mod list_services;
mod sync_devices;

use std::time::Instant;

pub use fetch_network_status::*;
pub use list_devices::*;
pub use list_services::*;
pub use sync_devices::*;

#[async_trait::async_trait]
pub trait PeriodicUseCase {
    fn next_execution(&self) -> Option<Instant>;
    async fn execute(&self);
}
