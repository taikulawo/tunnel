use std::{error::Error, io};

use tokio::runtime::Builder;
use tunnel::{AppConfig, Tun};
fn main() {
    let builder = Builder::new_multi_thread().enable_all().build().unwrap();
    let config = AppConfig::default();
        .block_on(async {
        let mut tun = Tun::new().await.unwrap();
        tun.run().await
    });
    ()
}
