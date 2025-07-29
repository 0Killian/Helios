use common::{BaseAgentConfig, CONFIG};
use mac_address::MacAddress;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};
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
    pub token: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicePort {
    pub name: String,
    pub port: u16,
    pub transport_protocol: TransportProtocol,
    pub application_protocol: ApplicationProtocol,
    pub is_online: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
pub enum TransportProtocol {
    TCP,
    UDP,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
pub enum ApplicationProtocol {
    HTTP,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceTemplate {
    pub kind: ServiceKind,
    pub ports: Vec<ServicePortTemplate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicePortTemplate {
    pub name: String,
    pub port: u16,
    pub transport_protocol: TransportProtocol,
    pub application_protocol: ApplicationProtocol,
}

impl ServicePortTemplate {
    pub fn matches(&self, port: &ServicePort) -> bool {
        self.port == port.port
            && self.transport_protocol == port.transport_protocol
            && self.application_protocol == port.application_protocol
            && self.name == port.name
    }
}

impl From<ServiceKind> for ServiceTemplate {
    fn from(kind: ServiceKind) -> Self {
        match kind {
            ServiceKind::HelloWorld => ServiceTemplate {
                kind,
                ports: vec![ServicePortTemplate {
                    name: "HTTP".to_string(),
                    port: 80,
                    transport_protocol: TransportProtocol::TCP,
                    application_protocol: ApplicationProtocol::HTTP,
                }],
            },
            ServiceKind::HelloWorld2 => ServiceTemplate {
                kind,
                ports: vec![
                    ServicePortTemplate {
                        name: "HTTP/2".to_string(),
                        port: 8080,
                        transport_protocol: TransportProtocol::TCP,
                        application_protocol: ApplicationProtocol::HTTP,
                    },
                    ServicePortTemplate {
                        name: "HTTP/3".to_string(),
                        port: 8081,
                        transport_protocol: TransportProtocol::UDP,
                        application_protocol: ApplicationProtocol::HTTP,
                    },
                ],
            },
        }
    }
}

macro_rules! enum_with_variant_list {
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident),* $(,)?
        }
    ) => {
        $(#[$attr])*
        $vis enum $name {
            $($variant),*
        }

        impl $name {
            pub fn variants() -> &'static [Self] {
                &[$(Self::$variant),*]
            }
        }
    };
}

enum_with_variant_list!(
    #[derive(Debug, Serialize, Deserialize, Clone, Copy, EnumString, Display)]
    #[serde(rename_all = "kebab-case")]
    #[strum(serialize_all = "kebab-case")]
    pub enum ServiceKind {
        HelloWorld,
        HelloWorld2,
    }
);

impl ServiceKind {
    pub fn base_config(&self) -> &BaseAgentConfig {
        match self {
            ServiceKind::HelloWorld => &CONFIG.agents.hello_world,
            ServiceKind::HelloWorld2 => &CONFIG.agents.hello_world2,
        }
    }
}
