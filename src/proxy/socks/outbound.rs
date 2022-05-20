use std::{sync::Arc};

use async_trait::async_trait;
use log::debug;
use tokio::{net::{TcpStream, UdpSocket}};

use crate::{
    proxy::{
        connect_to_remote_tcp, Address, Session, TcpOutboundHandlerTrait,
        UdpOutboundHandlerTrait,
    },
    Context,
};

use super::handshake_as_client;

pub struct TcpOutboundHandler {
    pub address: Address
}

#[async_trait]
impl TcpOutboundHandlerTrait for TcpOutboundHandler {
    async fn handle(&self, ctx: Arc<Context>, session: &Session) -> anyhow::Result<TcpStream> {
        let mut stream = connect_to_remote_tcp(ctx.dns_client.clone(), self.address.clone()).await?;
        match handshake_as_client(&mut stream, &session).await {
            Err(err) => {
                debug!("{}", err);
            }
            _ => {}
        }
        Ok(stream)
    }
}

pub struct UdpOutboundHandler {
    pub addr: String,
    pub port: u16,
}

#[async_trait]
impl UdpOutboundHandlerTrait for UdpOutboundHandler {
    async fn handle(&self, _ctx: Arc<Context>, _session: &Session) -> anyhow::Result<UdpSocket> {
        todo!()
    }
}
