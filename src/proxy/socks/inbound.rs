use crate::{net::ProxyStream, proxy::{ConnectionSession, TcpInbound}};


pub struct SocksTcpInboundHandler;

impl TcpInbound for SocksTcpInboundHandler {
    fn handle(stream: ProxyStream) -> ConnectionSession {
        todo!()
    }
}

async fn handshake(stream: ProxyStream) {
}