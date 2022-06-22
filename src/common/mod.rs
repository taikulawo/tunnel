#[cfg(target_os = "linux")]
pub mod linux;
use std::net::SocketAddr;

use lazy_static::lazy_static;

lazy_static!{
    pub static ref UNKNOWN_SOCKET_ADDR: SocketAddr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
}