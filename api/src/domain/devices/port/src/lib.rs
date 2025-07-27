use devices_adapter::DevicesAdapter;
use entities::{Device, MacAddress, Pagination, Service, SharedReference};
use ip_api_adapter::InternetProviderApiAdapter;

pub struct DevicesPort {
    ip_api: SharedReference<dyn InternetProviderApiAdapter>,
}

impl DevicesPort {
    pub fn new(ip_api: SharedReference<dyn InternetProviderApiAdapter>) -> Self {
        DevicesPort { ip_api }
    }
}

#[async_trait::async_trait]
impl DevicesAdapter for DevicesPort {
    async fn list_devices(&self, _: Option<Pagination>) -> Vec<Device> {
        todo!()
    }

    async fn list_services(&self, _: MacAddress) -> Vec<Service> {
        todo!()
    }

    async fn scan_devices(&self) {
        todo!()
    }
}
