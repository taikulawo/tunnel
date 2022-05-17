use std::{io, sync::Arc};

use async_trait::async_trait;
use tokio::net::{TcpStream, UdpSocket};

use crate::Context;

use super::{TcpOutboundHandlerTrait, Session, Error, UdpOutboundHandlerTrait, connect_to_remote_tcp, connect_to_remote_udp};

pub struct TcpOutboundHandler{}

#[async_trait]
impl TcpOutboundHandlerTrait for TcpOutboundHandler {
    async fn handle(&self, ctx: Arc<Context>, sess: &Session) -> anyhow::Result<TcpStream> {
        connect_to_remote_tcp(ctx.dns_client.clone(), sess.destination.clone()).await
    }
}

pub struct UdpOutboundHandler{}

#[async_trait]
impl UdpOutboundHandlerTrait for UdpOutboundHandler {
    async fn handle(&self, ctx: Arc<Context>, sess: &Session) -> anyhow::Result<UdpSocket> {
        connect_to_remote_udp(ctx.dns_client.clone(), sess.local_peer, sess.destination.clone()
    ).await
    }
}