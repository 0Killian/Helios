use std::sync::Arc;

use axum_distributed_routing::{create_router, route_group};
use common::{CONFIG, InternetProvider};
use devices_adapter::DevicesAdapter;
use devices_port::DevicesPort;
use entities::SharedReference;
use ip_api_adapter::InternetProviderApiAdapter;
use ip_api_port::bouygues::BboxInternetProviderApiPort;
use tokio::{net::TcpListener, sync::Mutex};

#[derive(Clone)]
pub struct Infrastructure {
    internet_provider_api: SharedReference<dyn InternetProviderApiAdapter>,
}

#[derive(Clone)]
pub struct Services {
    infrastructure: Infrastructure,
    devices: SharedReference<dyn DevicesAdapter>,
}

route_group!(pub Base, Services);
route_group!(pub RestV1, Services, Base, "/api/v1");

mod devices;
mod network;

#[tokio::main]
async fn main() {
    let internet_provider_api: SharedReference<dyn InternetProviderApiAdapter> =
        match CONFIG.internet_provider.kind {
            InternetProvider::Bouygues => Arc::new(Mutex::new(Box::new(
                BboxInternetProviderApiPort::new(
                    CONFIG.internet_provider.base_url.clone(),
                    CONFIG.internet_provider.password.clone(),
                )
                .await,
            ))),
        };
    let router = create_router!(Base)
        .with_state(Services {
            infrastructure: Infrastructure {
                internet_provider_api: internet_provider_api.clone(),
            },
            devices: Arc::new(Mutex::new(Box::new(DevicesPort::new(
                internet_provider_api,
            )))),
        })
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    axum::serve(TcpListener::bind("127.0.0.1:3000").await.unwrap(), router)
        .await
        .unwrap();
}
