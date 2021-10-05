use crate::{config::SocksInboundSettings, net::ProxyStream, proxy::{GeneralConn, ConnSession, Inbound, NetworkType, TransportNetwork, socks::handshake_as_server}};
use anyhow::Result;
use async_trait::async_trait;

pub struct SocksInbound;

#[async_trait]
impl Inbound for SocksInbound {
    async fn handle(
        &self,
        conn: GeneralConn,
        network: NetworkType,
    ) -> Result<ConnSession> {
        let mut stream = conn;
        let session = handshake_as_server(&mut stream).await?;
        Ok(session)
    }
    fn network() -> Vec<NetworkType>{
        vec![NetworkType::TCP]
    }
}
