mod common;
mod config;
mod net;
mod proxy;
pub mod app;

use std::sync::Arc;

use app::DnsClient;
use tokio::sync::RwLock;

pub use self::config::{
    Config,
    parse_from_str,
    load_from_file
};

pub struct Context {
    dns_client: Arc<RwLock<DnsClient>>,
}

impl Context {
    pub fn new(dns_client: Arc<RwLock<DnsClient>>) -> Self{
        Context {
            dns_client
        }
    }
}