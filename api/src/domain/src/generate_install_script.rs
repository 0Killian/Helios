use common::CONFIG;
use ports::repositories::{RepositoryError, ServicesRepository, UnitOfWorkProvider};
use serde::Deserialize;
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

// FIXME: This endpoint exposes the token, we should ensure that this is only called once for installation, or re-generate the token.

#[derive(Error, Debug, PartialEq, Eq)]
pub enum GenerateInstallScriptError {
    #[error("The requested service was not found.")]
    ServiceNotFound,

    #[error("A database error occurred: {0}.")]
    DatabaseError(#[from] RepositoryError),
}

#[derive(Deserialize, Copy, Clone, Debug)]
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

    #[instrument(skip(self), name = "GenerateInstallScriptUseCase::execute")]
    pub async fn execute(
        &self,
        os: OperatingSystem,
        service_id: Uuid,
    ) -> Result<InstallationScript, GenerateInstallScriptError> {
        let mut uow = self.uow_provider.begin_transaction().await?;
        let service = match SR::fetch_one(&mut uow, service_id).await {
            Ok(service) => service,
            Err(RepositoryError::NotFound) => {
                return Err(GenerateInstallScriptError::ServiceNotFound);
            }
            Err(err) => return Err(GenerateInstallScriptError::DatabaseError(err)),
        };

        Ok(match os {
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
        })
    }
}
