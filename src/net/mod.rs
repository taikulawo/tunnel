use std::{io, net::{IpAddr, SocketAddr}, ops::{Deref, DerefMut}};

use tokio::net::TcpListener;

mod stream;
mod sys;
pub struct ProxyTcpListener {
    inner: TcpListener,
}

impl ProxyTcpListener {
    pub async fn new(ip_addr: IpAddr, port: u16) -> io::Result<ProxyTcpListener> {
        let listener = TcpListener::bind(SocketAddr::new(ip_addr, port)).await?;
        Ok(ProxyTcpListener {
            inner: listener
        })
    }
}

impl Deref for ProxyTcpListener {
    type Target = TcpListener;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ProxyTcpListener {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}


pub use self::{
    stream::ProxyStream,
    sys::bind_to_device
};