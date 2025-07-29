use entities::{ServiceKind, ServiceTemplate};

#[derive(Clone)]
pub struct ListServiceTemplatesUseCase;

impl ListServiceTemplatesUseCase {
    pub async fn execute(&self) -> Vec<ServiceTemplate> {
        ServiceKind::variants()
            .into_iter()
            .cloned()
            .map(Into::into)
            .collect()
    }
}
