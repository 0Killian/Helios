mod agent_ping;
mod create_service;
mod fetch_network_status;
mod generate_install_script;
mod handle_agent_websocket;
mod list_devices;
mod list_service_templates;
mod list_services;
mod sync_devices;

use std::time::Instant;

pub use agent_ping::*;
pub use create_service::*;
pub use fetch_network_status::*;
pub use generate_install_script::*;
pub use handle_agent_websocket::*;
pub use list_devices::*;
pub use list_service_templates::*;
pub use list_services::*;
pub use sync_devices::*;

#[async_trait::async_trait]
pub trait PeriodicUseCase {
    fn next_execution(&self) -> Option<Instant>;
    async fn execute(&self);
}
