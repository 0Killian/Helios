use std::collections::HashSet;

use entities::{Service, ServiceKind, ServicePort, ServicePortTemplate, ServiceTemplate};
use mac_address::MacAddress;
use ports::repositories::{ServicesRepository, UnitOfWorkProvider};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateService {
    pub device_mac: MacAddress,
    pub display_name: String,
    pub kind: ServiceKind,
    pub ports: Vec<ServicePortTemplate>,
}

impl CreateService {
    fn validate(&self) -> Result<(), &'static str> {
        let template = ServiceTemplate::from(self.kind);

        let mut port_numbers = HashSet::new();
        for port in &self.ports {
            if !port_numbers.insert(port.port) {
                return Err("Duplicate port number");
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
            return Err("Duplicate port type");
        }

        if input_port_types.len() != template_port_types.len() {
            return Err("Missing required ports");
        }

        if template_port_types != input_port_types {
            return Err("Invalid port configuration");
        }

        Ok(())
    }
}

impl Into<Service> for CreateService {
    fn into(self) -> Service {
        if let Err(e) = self.validate() {
            panic!("{}", e);
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

        Service {
            service_id: uuid::Uuid::now_v7(),
            device_mac: self.device_mac,
            display_name: self.display_name,
            kind: self.kind,
            is_managed: true,
            ports: service_ports,
            token: common::generate_token(),
        }
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

    pub async fn execute(&self, service: CreateService) -> Service {
        let mut uow = self.uow_provider.begin_transaction().await;

        if SR::find_one(&mut uow, service.device_mac, service.kind, &service.ports)
            .await
            .is_some()
        {
            panic!("Service already exists");
        }

        let service: Service = service.into();

        SR::create(&mut uow, service.clone()).await;
        self.uow_provider.commit(uow).await;
        service
    }
}
