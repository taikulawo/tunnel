use std::sync::Arc;

use crate::{
    app::{dispatcher::Dispatcher, dns_client::DnsClient},
    config::AppConfig,
};

mod dispatcher;
mod dns_client;
mod listener;
pub struct Context {
    resolver: DnsClient,
    config: AppConfig,
    dispatcher: Arc<Dispatcher>,
}

impl Context {
    pub fn new() -> Self {
        let config = AppConfig::default();
        let resolver = DnsClient::new(config.clone());
        let dispatcher = Dispatcher {};
        let ctx = Context {
            config,
            resolver,
            dispatcher: Arc::new(dispatcher),
        };
        ctx
    }
}
