use std::collections::HashSet;

use entities::{Service, ServiceKind, ServicePort, ServicePortTemplate, ServiceTemplate};
use mac_address::MacAddress;
use ports::repositories::{RepositoryError, ServicesRepository, UnitOfWorkProvider};
use serde::Deserialize;
use thiserror::Error;
use tracing::{error, info, instrument, warn};
use validator::Validate;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CreateServiceError {
    #[error("Duplicate port number")]
    DuplicatePortNumber,
    #[error("Duplicate port type")]
    DuplicatePortType,
    #[error("Missing required ports")]
    MissingRequiredPorts,
    #[error("Invalid port configuration")]
    InvalidPortConfiguration,
    #[error("Service already exists")]
    ServiceAlreadyExists,
    #[error("A database error occurred: {0}.")]
    DatabaseError(#[from] RepositoryError),
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateService {
    pub device_mac: MacAddress,

    #[validate(length(min = 1, max = 100))]
    pub display_name: String,
    pub kind: ServiceKind,

    #[validate(nested)]
    pub ports: Vec<ServicePortTemplate>,
}

impl CreateService {
    fn validate(&self) -> Option<CreateServiceError> {
        let template = ServiceTemplate::from(self.kind);

        let mut port_numbers = HashSet::new();
        for port in &self.ports {
            if !port_numbers.insert(port.port) {
                return Some(CreateServiceError::DuplicatePortNumber);
            }
        }

        let input_port_types: HashSet<_> = self
            .ports
            .iter()
            .map(|port| {
                (
                    &port.name,
                    &port.transport_protocol,
                    &port.application_protocol,
                )
            })
            .collect();

        let template_port_types: HashSet<_> = template
            .ports
            .iter()
            .map(|port| {
                (
                    &port.name,
                    &port.transport_protocol,
                    &port.application_protocol,
                )
            })
            .collect();

        if input_port_types.len() != self.ports.len() {
            return Some(CreateServiceError::DuplicatePortNumber);
        }

        if input_port_types.len() != template_port_types.len() {
            return Some(CreateServiceError::DuplicatePortNumber);
        }

        if template_port_types != input_port_types {
            return Some(CreateServiceError::InvalidPortConfiguration);
        }

        None
    }
}

impl TryInto<Service> for CreateService {
    type Error = CreateServiceError;

    fn try_into(self) -> Result<Service, Self::Error> {
        if let Some(e) = self.validate() {
            return Err(e);
        }

        let template = ServiceTemplate::from(self.kind);

        let template_port_map: std::collections::HashMap<_, _> = template
            .ports
            .iter()
            .map(|port| {
                (
                    (
                        &port.name,
                        &port.transport_protocol,
                        &port.application_protocol,
                    ),
                    port,
                )
            })
            .collect();

        let service_ports: Vec<ServicePort> = self
            .ports
            .into_iter()
            .map(|port| {
                let template_port = template_port_map
                    .get(&(
                        &port.name,
                        &port.transport_protocol,
                        &port.application_protocol,
                    ))
                    .expect("Port validation should have caught this");

                ServicePort {
                    port: port.port,
                    name: template_port.name.clone(),
                    transport_protocol: template_port.transport_protocol,
                    application_protocol: template_port.application_protocol,
                    is_online: false,
                }
            })
            .collect();

        Ok(Service {
            service_id: uuid::Uuid::now_v7(),
            device_mac: self.device_mac,
            display_name: self.display_name,
            kind: self.kind,
            is_managed: true,
            ports: service_ports,
            token: common::generate_token(),
        })
    }
}

#[derive(Clone)]
pub struct CreateServiceUseCase<SR: ServicesRepository<UWP>, UWP: UnitOfWorkProvider> {
    uow_provider: UWP,
    _marker: std::marker::PhantomData<SR>,
}

impl<SR: ServicesRepository<UWP>, UWP: UnitOfWorkProvider> CreateServiceUseCase<SR, UWP> {
    pub fn new(uow_provider: UWP) -> Self {
        Self {
            uow_provider,
            _marker: std::marker::PhantomData,
        }
    }

    #[instrument(skip(self), name = "CreateServiceUseCase::execute")]
    pub async fn execute(&self, service: CreateService) -> Result<Service, CreateServiceError> {
        info!(device = %service.device_mac, service = ?service, "Creating a new service");
        let mut uow = self.uow_provider.begin_transaction().await?;

        if SR::find_one(&mut uow, service.device_mac, service.kind, &service.ports)
            .await?
            .is_some()
        {
            warn!("Service already exists");
            return Err(CreateServiceError::ServiceAlreadyExists);
        }

        let service: Service = service.try_into()?;

        match SR::create(&mut uow, service.clone()).await {
            Ok(_) => (),
            Err(RepositoryError::UniqueViolation) => {
                // Should never happen???
                error!("Unexpected unique violation when creating service");
                return Err(CreateServiceError::ServiceAlreadyExists);
            }
            Err(err) => return Err(CreateServiceError::DatabaseError(err)),
        }

        self.uow_provider.commit(uow).await?;

        info!(service = ?service, "Service created successfully");
        Ok(service)
    }
}
