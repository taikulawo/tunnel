use std::{io, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use log::debug;
use tokio::{io::AsyncWriteExt, net::{TcpStream, UdpSocket}};

use crate::{
    proxy::{
        connect_to_remote_tcp, Address, Error, OutboundConnect, Session, TcpOutboundHandlerTrait,
        UdpOutboundHandlerTrait,
    },
    Context,
};

use super::handshake_as_client;

pub struct TcpOutboundHandler {
    pub addr: String,
    pub port: u16,
}

#[async_trait]
impl TcpOutboundHandlerTrait for TcpOutboundHandler {
    async fn handle(&self, ctx: Arc<Context>, session: &Session) -> Result<TcpStream, Error> {
        let mut stream =
            match connect_to_remote_tcp(ctx.dns_client.clone(), &self.addr, self.port).await {
                Ok(stream) => stream,
                Err(err) => {
                    debug!("{}", err);
                    return Err(Error::ConnectError(self.addr.clone(), self.port));
                }
            };
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
    async fn handle(&self, ctx: Arc<Context>, session: &Session) -> Result<UdpSocket, Error> {
        todo!()
    }
}
