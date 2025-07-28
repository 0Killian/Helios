use mac_address::MacAddress;
use serde::Serialize;
use sqlx::prelude::{FromRow, Type};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub service_id: Uuid,
    pub device_mac: MacAddress,
    pub display_name: String,
    pub kind: ServiceKind,
    pub is_managed: bool,
    pub ports: Vec<ServicePort>,
}

#[derive(Debug, Serialize, Clone, Copy, Type)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceKind {
    HelloWorld,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicePort {
    pub port: u16,
    pub transport_protocol: TransportProtocol,
    pub application_protocol: ApplicationProtocol,
    pub is_online: bool,
}

#[derive(Debug, Serialize, Clone, Copy, Type)]
pub enum TransportProtocol {
    TCP,
    UDP,
}

#[derive(Debug, Serialize, Clone, Copy, Type)]
pub enum ApplicationProtocol {
    HTTP,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceTemplate {
    pub kind: ServiceKind,
    pub ports: Vec<ServicePortTemplate>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicePortTemplate {
    pub port: u16,
    pub transport_protocol: TransportProtocol,
    pub application_protocol: ApplicationProtocol,
}

impl From<ServiceKind> for ServiceTemplate {
    fn from(kind: ServiceKind) -> Self {
        match kind {
            ServiceKind::HelloWorld => ServiceTemplate {
                kind,
                ports: vec![ServicePortTemplate {
                    port: 80,
                    transport_protocol: TransportProtocol::TCP,
                    application_protocol: ApplicationProtocol::HTTP,
                }],
            },
        }
    }
}
