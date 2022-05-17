use std::{error::Error, io, sync::Arc};

use anyhow::{anyhow, Result};
use clap::Arg;
use futures::future;
use log::error;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Logger, Root},
    encode::{pattern::PatternEncoder, Encode},
};
use tokio::{runtime::Builder, sync::RwLock};
use tunnel::{
    app::{Dispatcher, DnsClient, InboundManager, OutboundManager, Router},
    Context,
};

fn load() -> Result<()> {
    let app = clap::App::new("tunnel").arg(
        Arg::with_name("config")
            .short("-c")
            .long("--config")
            .required(true)
            .value_name("FILE"),
    );

    let stdout_logger = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {h({l})} {f}:{L} {m} {n}")))
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
    let rt = Builder::new_multi_thread().enable_all().build().unwrap();

    let matchers = app.get_matches();
    let config_path = matchers
        .value_of("config")
        .expect("config file path required");
    let config = match tunnel::load_from_file(config_path) {
        Ok(x) => x,
        Err(err) => {
            error!("failed to load config file {} {}", config_path, err);
            return Err(err);
        }
    };
    let mut tasks = Vec::new();

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

    rt.block_on(future::join_all(tasks));
    Ok(())
}
fn main() {
    if let Err(err) = load() {
        error!("{}", err);
    }
}
