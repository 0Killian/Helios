use common::CONFIG;
use ports::repositories::{ServicesRepository, UnitOfWorkProvider};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperatingSystem {
    Linux,
}

pub struct InstallationScript {
    pub content: String,
    pub file_format: String,
    pub file_name: String,
}

#[derive(Clone)]
pub struct GenerateInstallScriptUseCase<SR: ServicesRepository<UWP>, UWP: UnitOfWorkProvider> {
    uow_provider: UWP,
    _marker: std::marker::PhantomData<SR>,
}

impl<SR: ServicesRepository<UWP>, UWP: UnitOfWorkProvider> GenerateInstallScriptUseCase<SR, UWP> {
    pub fn new(uow_provider: UWP) -> Self {
        Self {
            uow_provider,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn execute(&self, os: OperatingSystem, service_id: Uuid) -> InstallationScript {
        let mut uow = self.uow_provider.begin_transaction().await;
        let service = SR::fetch_one(&mut uow, service_id).await.unwrap();

        match os {
            OperatingSystem::Linux => InstallationScript {
                content: format!(
                    include_str!("../../../assets/install_script_linux.sh"),
                    agent_binary_base_url = service.kind.base_config().download_base_url,
                    token = service.token,
                    custom_config = "",
                    helios_base_url = CONFIG.api.base_url
                )
                .replace("\r", ""),
                file_format: "text/x-shellscript".to_string(),
                file_name: "install_script.sh".to_string(),
            },
        }
    }
}
