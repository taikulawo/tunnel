use std::net::SocketAddr;

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::{
    net::ProxyStream,
    proxy::{Address, GeneralConn, GeneralConnTrait, ConnSession, DomainSession, Outbound},
};

pub struct OutboundHandler {}

#[async_trait]
impl Outbound for OutboundHandler {
    async fn handle(
        &self,
        stream: GeneralConn,
        session: ConnSession,
    ) -> Result<GeneralConn> {
        let ConnSession { ref host, ref port } = session;
        match host {
            Address::Domain(name) => {}
            Address::Ip(ip) => {}
        }
        todo!()
    }
}
