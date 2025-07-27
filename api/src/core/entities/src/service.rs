use serde::Serialize;
use uuid::Uuid;

use crate::MacAddress;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    id: Uuid,
    device_mac: MacAddress,
    display_name: String,
    kind: ServiceKind,
    is_managed: bool,
    ports: Vec<ServicePort>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceKind {
    HelloWorld,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicePort {
    port: u16,
    transport_protocol: TransportProtocol,
    application_protocol: ApplicationProtocol,
    is_online: bool,
}

#[derive(Debug, Serialize)]
pub enum TransportProtocol {
    TCP,
    UDP,
}

#[derive(Debug, Serialize)]
pub enum ApplicationProtocol {
    HTTP,
}
