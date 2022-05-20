mod common;
mod net;
pub mod config;
pub mod app;
pub mod proxy;

use std::{sync::{Arc, Once}};

use app::{Dispatcher, DnsClient, InboundManager, OutboundManager, Router};
use futures::future::BoxFuture;

use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
};
use anyhow::{
    anyhow
};
use tokio::{sync::{RwLock}};

pub use self::config::{load_from_file, parse_from_str};

pub struct Context {
    dns_client: Arc<RwLock<DnsClient>>,
}

impl Context {
    pub fn new(dns_client: Arc<RwLock<DnsClient>>) -> Self {
        Context { dns_client }
    }
}

pub fn newRuntime() -> tokio::runtime::Runtime {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    runtime
}

pub fn start(config: config::Config, shutdown_handler: BoxFuture<'static, ()>) -> anyhow::Result<()> {
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
            .build(log::LevelFilter::Error),
        )
        .unwrap();
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            let _handler = log4rs::init_config(logger_config).unwrap();
        });
        
        let inbound_manager = InboundManager::new(config.inbounds.clone());
        let outbound_manager = Arc::new(OutboundManager::new(config.outbounds.clone())?);
        let router = Arc::new(Router::new(config.routes.clone()));
        let dns_client = Arc::new(RwLock::new(DnsClient::new(config.clone())));
        let context = Arc::new(Context::new(dns_client.clone()));
        
        let dispatcher = Arc::new(Dispatcher::new(
            context.clone(),
            router.clone(),
            dns_client.clone(),
            outbound_manager.clone(),
            config.clone(),
        ));
        
    let inbound_futures = match inbound_manager.listen(dispatcher.clone()) {
        Ok(x) => x,
        Err(err) => {
            return Err(anyhow!("{}", err));
        }
    };
    tasks.push(shutdown_handler);
    tasks.push(inbound_futures);
    let runtime = newRuntime();
    runtime.block_on(futures::future::select_all(tasks));
    Ok(())
}
