use agent_connection::InMemoryAgentConnectionManager;
use axum::http::Request;
use std::sync::Arc;
use tower_http::{
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{Level, info, span};
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

use axum_distributed_routing::{create_router, route_group};
use common::{CONFIG, RouterKind};
use domain::{
    CreateServiceUseCase, FetchNetworkStatusUseCase, GenerateInstallScriptUseCase,
    HandleAgentWebsocketUseCase, ListDevicesUseCase, ListServiceTemplatesUseCase,
};
use ports::repositories::{DevicesRepository, ServicesRepository, UnitOfWorkProvider};
use repositories::{PostgresDevicesRepository, PostgresServicesRepository, PostgresUWP};
use router_api::bouygues::BboxRouterApi;
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex};
use tracing_subscriber::layer::SubscriberExt;

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
    handle_agent_websocket: HandleAgentWebsocketUseCase<SR, UWP>,
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

#[derive(Clone)]
struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::now_v7().to_string().parse().unwrap();
        Some(RequestId::new(request_id))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting REST server");

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
    let acm = Arc::new(InMemoryAgentConnectionManager::new());

    let app_state = AppState {
        list_devices: ListDevicesUseCase::new(unit_of_work_provider.clone()),
        fetch_network_status: FetchNetworkStatusUseCase::new(router_api.clone()),
        list_service_templates: ListServiceTemplatesUseCase,
        create_service: CreateServiceUseCase::new(unit_of_work_provider.clone()),
        generate_install_script: GenerateInstallScriptUseCase::new(unit_of_work_provider.clone()),
        handle_agent_websocket: HandleAgentWebsocketUseCase::new(
            unit_of_work_provider.clone(),
            acm.clone(),
        ),
    };

    let router = create_router!(Base)
        .with_state(app_state)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // Get the request ID from the request extensions
                let request_id = request
                    .extensions()
                    .get::<RequestId>()
                    .map(|id| id.header_value().to_str().unwrap())
                    .unwrap_or_else(|| "unknown".into());

                // Create a span with all the desired fields
                span!(
                    Level::INFO,
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                    request_id = %request_id,
                )
            }),
        )
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));

    let web_server = axum::serve(
        TcpListener::bind((CONFIG.api.listen_address, CONFIG.api.listen_port))
            .await
            .map(|listener| {
                info!("Listening on {}", listener.local_addr().unwrap());
                listener
            })?,
        router,
    );

    let cron = cron::cron::<PostgresDevicesRepository, PostgresUWP>(
        router_api,
        unit_of_work_provider,
        acm,
    )
    .await?;

    Ok(tokio::select! {
        r = web_server => r?,
        _ = cron => (),
    })
}
