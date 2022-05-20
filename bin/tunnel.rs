use std::{error::Error, io, sync::Arc};

use anyhow::{anyhow, Result};
use clap::Arg;
use futures::{future, FutureExt};
use log::error;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Logger, Root},
    encode::{pattern::PatternEncoder, Encode},
};
use tokio::{
    runtime::Builder,
    sync::{mpsc, RwLock},
};
use tunnel::{
    app::{Dispatcher, DnsClient, InboundManager, OutboundManager, Router},
    newRuntime, Context, start,
};

fn load() -> Result<()> {
    let app = clap::App::new("tunnel").arg(
        Arg::with_name("config")
            .short("-c")
            .long("--config")
            .required(true)
            .value_name("FILE"),
    );
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
    let (shutdown_future, shutdown_handler) = futures::future::abortable(futures::future::pending::<bool>());
    let handler = async {
        shutdown_future.await.unwrap();
    }.boxed();
    start(config, handler).unwrap();
    shutdown_handler.abort();
    Ok(())
}
fn main() {
    if let Err(err) = load() {
        error!("{}", err);
    }
}
