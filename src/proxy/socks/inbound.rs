use std::io;
use log:: {
    error
};

use crate::{config::SocksInboundSettings, net::ProxyStream, proxy::{GeneralConn, ConnSession, Inbound, NetworkType, TransportNetwork, socks::handshake_as_server, TcpInboundHandlerTrait, InboundResult, UdpInboundHandlerTrait}};
use async_trait::async_trait;
use tokio::net::{TcpStream, UdpSocket};

pub struct SocksTcpInboundHandler;

#[async_trait]
impl TcpInboundHandlerTrait for SocksTcpInboundHandler {
    async fn handle(
        &self,
        conn: ConnSession,
        mut stream: TcpStream,
    ) -> io::Result<InboundResult> {
        let session = match handshake_as_server(&mut stream).await {
            Ok(session) => session,
            Err(err) => {
                error!("failed to process socks inbound {}", err);
                return Err(io::Error::new(io::ErrorKind::Other, "unknown"))
            }
        };
        Ok(InboundResult::Stream(stream, conn))
    }
}




pub struct SocksUdpInboundHandler;

#[async_trait]
impl UdpInboundHandlerTrait for SocksUdpInboundHandler {
    async fn handle(&self, conn: ConnSession, socket: UdpSocket) -> io::Result<InboundResult> {
        Ok(InboundResult::Datagram(socket, conn))
    }
}