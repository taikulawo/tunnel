use std::{io, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::{
    proxy::{
        connect_to_remote_tcp, Address, Error, OutboundConnect, OutboundResult, Session,
        TcpOutboundHandlerTrait, UdpOutboundHandlerTrait,
    },
    Context,
};

pub struct TcpOutboundHandler {
    pub addr: String,
    pub port: u16,
}

#[async_trait]
impl TcpOutboundHandlerTrait for TcpOutboundHandler {
    async fn handle(&self, ctx: Arc<Context>, session: &Session) -> Result<OutboundResult, Error> {
        let stream = match connect_to_remote_tcp(ctx.dns_client.clone(), &self.addr, self.port).await {
            Ok(stream) => stream,
            Err(err) => return Err(Error::ConnectError(self.addr.clone(), self.port)),
        };
        todo!()
    }
}

pub struct UdpOutboundHandler {
    pub addr: String,
    pub port: u16,
}

#[async_trait]
impl UdpOutboundHandlerTrait for UdpOutboundHandler {
    async fn handle(&self, ctx: Arc<Context>, session: &Session) -> Result<OutboundResult, Error> {
        todo!()
    }
}
