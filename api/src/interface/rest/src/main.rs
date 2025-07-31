use std::sync::Arc;

use axum_distributed_routing::{create_router, route_group};
use common::{CONFIG, RouterKind};
use domain::{
    CreateServiceUseCase, FetchNetworkStatusUseCase, GenerateInstallScriptUseCase,
    ListDevicesUseCase, ListServiceTemplatesUseCase,
};
use ports::repositories::{DevicesRepository, ServicesRepository, UnitOfWorkProvider};
use repositories::{PostgresDevicesRepository, PostgresServicesRepository, PostgresUWP};
use router_api::bouygues::BboxRouterApi;
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
    list_service_templates: ListServiceTemplatesUseCase,
    create_service: CreateServiceUseCase<SR, UWP>,
    generate_install_script: GenerateInstallScriptUseCase<SR, UWP>,
}

type PostgresAppState =
    AppState<PostgresDevicesRepository, PostgresServicesRepository, PostgresUWP>;

route_group!(pub Base, PostgresAppState);
route_group!(pub RestV1, PostgresAppState, Base, "/api/v1");

mod agents;
mod devices;
mod extractors;
mod network;
mod response;
mod service_templates;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router_api = Arc::new(match CONFIG.router_api.kind {
        RouterKind::Bbox => {
            BboxRouterApi::new(
                CONFIG.router_api.base_url.clone(),
                CONFIG.router_api.password.clone(),
            )
            .await?
        }
    });

    let pg_pool = Arc::new(Mutex::new(
        PgPool::connect(CONFIG.database.url.as_str()).await?,
    ));

    let unit_of_work_provider = PostgresUWP::new(pg_pool);

    let app_state = AppState {
        list_devices: ListDevicesUseCase::new(unit_of_work_provider.clone()),
        fetch_network_status: FetchNetworkStatusUseCase::new(router_api),
        list_service_templates: ListServiceTemplatesUseCase,
        create_service: CreateServiceUseCase::new(unit_of_work_provider.clone()),
        generate_install_script: GenerateInstallScriptUseCase::new(unit_of_work_provider.clone()),
    };

    let router = create_router!(Base).with_state(app_state).layer(
        tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any),
    );

    Ok(axum::serve(
        TcpListener::bind((CONFIG.api.listen_address, CONFIG.api.listen_port))
            .await
            .map(|listener| {
                println!("Listening on {}", listener.local_addr().unwrap());
                listener
            })?,
        router,
    )
    .await?)
}
