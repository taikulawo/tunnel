use std::{io, net::SocketAddr};

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::{
    proxy::{Address, Session, OutboundResult, UdpOutboundHandlerTrait, TcpOutboundHandlerTrait, OutboundConnect},
};

pub struct TcpOutboundHandler {
    pub addr: String,
    pub port: u16,
}

#[async_trait]
impl TcpOutboundHandlerTrait for TcpOutboundHandler {
    async fn handle(&self, session: Session) -> io::Result<OutboundResult> {
        
        todo!()
    }
    fn remote_addr(&self) -> OutboundConnect{
        OutboundConnect::Proxy(self.addr.clone(), self.port)
    }
}

pub struct UdpOutboundHandler {
    pub addr: String,
    pub port: u16,
}

#[async_trait]
impl UdpOutboundHandlerTrait for UdpOutboundHandler {
    async fn handle(&self, session: Session) -> io::Result<OutboundResult> {
        todo!()
    }
}