use entities::{ServiceKind, ServiceTemplate};
use tracing::instrument;

#[derive(Clone)]
pub struct ListServiceTemplatesUseCase;

impl ListServiceTemplatesUseCase {
    #[instrument(skip(self), name = "ListServiceTemplatesUseCase::execute")]
    pub async fn execute(&self) -> Vec<ServiceTemplate> {
        ServiceKind::variants()
            .into_iter()
            .cloned()
            .map(Into::into)
            .collect()
    }
}
