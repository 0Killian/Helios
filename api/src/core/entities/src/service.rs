use mac_address::MacAddress;
use serde::Serialize;
use sqlx::prelude::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
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

#[derive(Debug, Serialize)]
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
