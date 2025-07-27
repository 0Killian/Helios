mod device;
mod network;
mod service;
mod utils;

use std::sync::Arc;

use tokio::sync::Mutex;

pub use device::*;
pub use network::*;
pub use service::*;
pub use utils::*;

/// Convert the object to an SQL expression (useful for pagination, filtering, etc.)
pub trait ToSql {
    fn to_sql(&self) -> String;
}

pub type SharedLockedReference<T> = Arc<Mutex<T>>;
