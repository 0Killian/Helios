use std::sync::Arc;

use entities::NetworkStatus;
use ports::api::{RouterApi, RouterApiResult};
use tracing::instrument;

#[derive(Clone)]
pub struct FetchNetworkStatusUseCase {
    router_api: Arc<dyn RouterApi>,
}

impl FetchNetworkStatusUseCase {
    pub fn new(router_api: Arc<dyn RouterApi>) -> Self {
        Self { router_api }
    }

    #[instrument(skip(self), name = "FetchNetworkStatusUseCase::execute")]
    pub async fn execute(&self) -> RouterApiResult<NetworkStatus> {
        let stats = self.router_api.wan_stats().await?;
        let connectivity = self.router_api.wan_connectivity().await?;

        Ok(NetworkStatus {
            stats,
            connectivity,
        })
    }
}
