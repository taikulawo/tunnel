use std::sync::Arc;

use crate::{app::dns_client::DnsClient, config::AppConfig};

mod dns_client;
mod listener;
mod dispatcher;
pub struct TunnelContext {
    resolver: DnsClient,
    config: AppConfig,
    dispatcher: Arc<Dispatcher>
}

impl TunnelContext {
    pub fn new() -> Self{
        let config = AppConfig::default();
        let resolver = DnsClient::new(config.clone());
        let ctx = TunnelContext {
            config,
            resolver
        };
        ctx
    }
}