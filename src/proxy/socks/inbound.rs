use crate::{
    net::ProxyStream,
    proxy::{
        socks::handshake_as_server, CommonStream, ConnectionSession, Inbound, TransportNetwork,
    },
};
use anyhow::Result;
use async_trait::async_trait;

pub struct SocksTcpInboundHandler;

#[async_trait]
impl Inbound for SocksTcpInboundHandler {
    async fn handle(
        &self,
        stream: CommonStream,
        network: TransportNetwork,
    ) -> Result<ConnectionSession> {
        let mut stream = stream;
        let session = handshake_as_server(&mut stream).await?;
        Ok(session)
    }
}
