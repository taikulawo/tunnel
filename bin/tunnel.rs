use std::{error::Error, io};

use clap::Arg;
use log::error;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Logger, Root},
};
use tokio::runtime::Builder;
fn main() {
    let app = clap::App::new("tunnel").arg(
        Arg::with_name("config")
            .short("-c")
            .long("--config")
            .required(true)
            .value_name("FILE"),
    );

    let stdout_logger = ConsoleAppender::builder().build();
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
            return;
        }
    };
    // rt.block_on(async {
    //     let mut tun = Tun::new().await.unwrap();
    //     tun.run().await
    // })
    // .unwrap();
    ()
}
