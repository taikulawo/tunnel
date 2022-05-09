use std::{io, net::SocketAddr};

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::{
    proxy::{Address, ConnSession, OutboundResult, TcpOutboundHandlerTrait},
};

pub struct TcpOutboundHandler {}

#[async_trait]
impl TcpOutboundHandlerTrait for TcpOutboundHandler {
    async fn handle(&self, session: ConnSession) -> io::Result<OutboundResult> {
        let ConnSession { ref host, ref port } = session;
        match host {
            Address::Domain(name) => {}
            Address::Ip(ip) => {}
        }
        todo!()
    }
}

pub struct UdpOutboundHandler {}

#[async_trait]
impl TcpOutboundHandlerTrait for UdpOutboundHandler {
    async fn handle(&self, session: ConnSession) -> io::Result<OutboundResult> {
        todo!()
    }
}