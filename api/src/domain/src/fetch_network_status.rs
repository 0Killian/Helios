use std::sync::Arc;

use entities::NetworkStatus;
use ports::api::InternetProviderApi;

#[derive(Clone)]
pub struct FetchNetworkStatusUseCase {
    provider_api: Arc<dyn InternetProviderApi>,
}

impl FetchNetworkStatusUseCase {
    pub fn new(provider_api: Arc<dyn InternetProviderApi>) -> Self {
        Self { provider_api }
    }

    pub async fn execute(&self) -> NetworkStatus {
        let stats = self.provider_api.wan_stats().await;
        let connectivity = self.provider_api.wan_connectivity().await;

        NetworkStatus {
            stats,
            connectivity,
        }
    }
}
