mod common;
mod config;
mod net;
mod proxy;
pub mod app;

use std::sync::Arc;

use app::{DnsClient, InboundManager, OutboundManager, Router, Dispatcher};
use futures::future::BoxFuture;
use log::error;
use log4rs::{append::console::ConsoleAppender, encode::pattern::PatternEncoder, config::{Appender, Root, Logger}};
use tokio::{sync::RwLock, runtime::Builder};

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

pub fn start_instance(config: config::Config) -> anyhow::Result<Vec<BoxFuture<'static, ()>>> {
    let mut tasks = Vec::new();

    let stdout_logger = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d} {h({l})} {f}:{L} {m} {n}",
        )))
        .build();
    let logger_config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout_logger)))
        .logger(Logger::builder().build("tunnel", log::LevelFilter::Trace))
        .build(
            Root::builder()
                .appender("stdout")
                .build(log::LevelFilter::Warn),
        )
        .unwrap();
    let handler = log4rs::init_config(logger_config).unwrap();

    let inbound_manager = InboundManager::new(config.inbounds.clone());
    let outbound_manager = Arc::new(OutboundManager::new(&config.outbounds)?);
    let router = Arc::new(Router::new(&config.routes));
    let dns_client = Arc::new(RwLock::new(DnsClient::new(config.clone())));

    let context = Arc::new(Context::new(dns_client.clone()));
    let dispatcher = Arc::new(Dispatcher::new(
        context,
        router,
        dns_client.clone(),
        outbound_manager,
        &config,
    ));
    let inbound_futures = match inbound_manager.listen(dispatcher.clone()) {
        Ok(x) => x,
        Err(err) => {
            error!("{}", err);
            return Err(err);
        }
    };
    tasks.push(inbound_futures);
    Ok(tasks)
}