use std::sync::Arc;

use crate::{
    app::{dispatcher::Dispatcher, dns_client::DnsClient},
    config::Config,
};

mod dispatcher;
mod dns_client;
mod listener;
mod inbound;
mod outbound;
mod sniffer;
mod router;

pub struct Context {
    resolver: DnsClient,
    config: Config,
    dispatcher: Arc<Dispatcher>,
}

impl Context {
    pub fn new() -> Self {
        let config = Config::default();
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
