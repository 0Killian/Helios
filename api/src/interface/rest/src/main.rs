use std::sync::Arc;

use axum_distributed_routing::{create_router, route_group};
use common::{CONFIG, InternetProvider};
use domain::{FetchNetworkStatusUseCase, ListDevicesUseCase};
use internet_provider_api::bouygues::BboxInternetProviderApi;
use ports::repositories::{DevicesRepository, ServicesRepository, UnitOfWorkProvider};
use repositories::{PostgresDevicesRepository, PostgresServicesRepository, PostgresUWP};
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex};

#[derive(Clone)]
pub struct AppState<DR, SR, UWP>
where
    DR: DevicesRepository<UWP>,
    SR: ServicesRepository<UWP>,
    UWP: UnitOfWorkProvider,
{
    list_devices: ListDevicesUseCase<DR, SR, UWP>,
    fetch_network_status: FetchNetworkStatusUseCase,
}

type PostgresAppState =
    AppState<PostgresDevicesRepository, PostgresServicesRepository, PostgresUWP>;

route_group!(pub Base, PostgresAppState);
route_group!(pub RestV1, PostgresAppState, Base, "/api/v1");

mod devices;
mod network;

#[tokio::main]
async fn main() {
    let internet_provider_api = Arc::new(match CONFIG.internet_provider.kind {
        InternetProvider::Bouygues => {
            BboxInternetProviderApi::new(
                CONFIG.internet_provider.base_url.clone(),
                CONFIG.internet_provider.password.clone(),
            )
            .await
        }
    });

    let pg_pool = Arc::new(Mutex::new(
        PgPool::connect(CONFIG.database.url.as_str()).await.unwrap(),
    ));

    let unit_of_work_provider = PostgresUWP::new(pg_pool);

    let app_state = AppState {
        list_devices: ListDevicesUseCase::new(unit_of_work_provider.clone()),
        fetch_network_status: FetchNetworkStatusUseCase::new(internet_provider_api),
    };

    let router = create_router!(Base).with_state(app_state).layer(
        tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any),
    );

    axum::serve(TcpListener::bind("127.0.0.1:3000").await.unwrap(), router)
        .await
        .unwrap();
}
