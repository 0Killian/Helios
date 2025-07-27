use std::sync::Arc;

use axum_distributed_routing::{create_router, route_group};
use common::{CONFIG, InternetProvider};
use ip_api_adapter::InternetProviderApiAdapter;
use ip_api_port::bouygues::BboxInternetProviderApiPort;
use tokio::{net::TcpListener, sync::Mutex};

#[derive(Clone)]
pub struct Services {
    internet_provider_api: Arc<Mutex<dyn InternetProviderApiAdapter>>,
}

route_group!(pub Base, Services);
route_group!(pub RestV1, Services, Base, "/api/v1");

mod devices;
mod network;

#[tokio::main]
async fn main() {
    let router = create_router!(Base)
        .with_state(Services {
            internet_provider_api: match CONFIG.internet_provider.kind {
                InternetProvider::Bouygues => Arc::new(Mutex::new(
                    BboxInternetProviderApiPort::new(
                        CONFIG.internet_provider.base_url.clone(),
                        CONFIG.internet_provider.password.clone(),
                    )
                    .await,
                )),
            },
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
