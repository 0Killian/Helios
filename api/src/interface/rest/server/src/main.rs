use std::sync::Arc;

use axum_distributed_routing::{create_router, route_group};
use common::{CONFIG, InternetProvider};
use devices_adapter::DevicesAdapter;
use devices_port::DevicesPort;
use entities::SharedLockedReference;
use ip_api_adapter::InternetProviderApiAdapter;
use ip_api_port::bouygues::BboxInternetProviderApiPort;
use repositories_adapter::{DevicesRepositoryAdapter, ServicesRepositoryAdapter};
use repositories_port::{DevicesRepositoryPort, ServicesRepositoryPort};
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex};

#[derive(Clone)]
pub struct Infrastructure {
    pg_pool: SharedLockedReference<PgPool>,
    internet_provider_api: Arc<dyn InternetProviderApiAdapter>,
    devices_repository: Arc<dyn DevicesRepositoryAdapter>,
    services_repository: Arc<dyn ServicesRepositoryAdapter>,
}

#[derive(Clone)]
pub struct Services {
    infrastructure: Infrastructure,
    devices: Arc<dyn DevicesAdapter>,
}

route_group!(pub Base, Services);
route_group!(pub RestV1, Services, Base, "/api/v1");

mod devices;
mod network;

#[tokio::main]
async fn main() {
    let internet_provider_api: Arc<dyn InternetProviderApiAdapter> =
        match CONFIG.internet_provider.kind {
            InternetProvider::Bouygues => Arc::new(
                BboxInternetProviderApiPort::new(
                    CONFIG.internet_provider.base_url.clone(),
                    CONFIG.internet_provider.password.clone(),
                )
                .await,
            ),
        };

    let infrastructure = Infrastructure {
        pg_pool: Arc::new(Mutex::new(
            PgPool::connect(CONFIG.database.url.as_str()).await.unwrap(),
        )),
        internet_provider_api: internet_provider_api,
        devices_repository: Arc::new(DevicesRepositoryPort {}),
        services_repository: Arc::new(ServicesRepositoryPort {}),
    };

    let services = Services {
        infrastructure: infrastructure.clone(),
        devices: Arc::new(DevicesPort::new(
            infrastructure.internet_provider_api,
            infrastructure.devices_repository,
            infrastructure.services_repository,
            infrastructure.pg_pool,
        )),
    };

    let router = create_router!(Base).with_state(services.clone()).layer(
        tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any),
    );

    let server = axum::serve(TcpListener::bind("127.0.0.1:3000").await.unwrap(), router);
    let scan_worker = services
        .devices
        .start_devices_scan(chrono::Duration::seconds(
            CONFIG.scanning.device_scan_delay as i64,
        ));

    let (server, _) = tokio::join!(server, scan_worker);

    server.unwrap();
}
