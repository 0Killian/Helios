use entities::{ServiceKind, ServiceTemplate};

#[derive(Clone)]
pub struct ListServiceTemplatesUseCase;

impl ListServiceTemplatesUseCase {
    pub async fn execute(&self) -> Vec<ServiceTemplate> {
        vec![ServiceKind::HelloWorld.into()]
    }
}
