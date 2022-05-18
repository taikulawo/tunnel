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
    Context, start_instance,
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
    let tasks = start_instance(config).unwrap();
    let (abort_future, handler) = futures::future::abortable(futures::future::join_all(tasks));
    let rt = Builder::new_multi_thread().enable_all().build().unwrap();
    rt.spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("ctrl c received");
        handler.abort();
    });
    rt.block_on(abort_future);
    Ok(())
}
fn main() {
    if let Err(err) = load() {
        error!("{}", err);
    }
}
