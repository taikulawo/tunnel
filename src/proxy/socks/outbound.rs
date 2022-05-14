use std::{io, net::SocketAddr};

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::{
    proxy::{Address, Session, OutboundResult, UdpOutboundHandlerTrait, TcpOutboundHandlerTrait},
};

pub struct TcpOutboundHandler {}

#[async_trait]
impl TcpOutboundHandlerTrait for TcpOutboundHandler {
    async fn handle(&self, session: Session) -> io::Result<OutboundResult> {
        todo!()
    }
}

pub struct UdpOutboundHandler {}

#[async_trait]
impl UdpOutboundHandlerTrait for UdpOutboundHandler {
    async fn handle(&self, session: Session) -> io::Result<OutboundResult> {
        todo!()
    }
}