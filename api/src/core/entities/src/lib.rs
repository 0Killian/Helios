mod device;
mod network;

pub use device::*;
pub use network::*;
use strum::EnumString;

#[derive(EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum InternetProvider {
    Bouygues,
}
