use std::net::SocketAddr;

use tokio::io::AsyncWriteExt;
use async_trait::async_trait;

use crate::{net::ProxyStream, proxy::{Address, AnyStream, AnyStreamTrait, ConnectionSession, DomainSession, TcpOutbound}};

pub struct OutboundHandler {}

#[async_trait]
impl TcpOutbound for OutboundHandler {
    async fn handle(stream: ProxyStream, session: ConnectionSession) -> ProxyStream {
        let ConnectionSession { ref host, ref port } = session;
        match host {
            Address::Domain(name) => {

            },
            Address::Ip(ip) => {
                
            }
        }
        todo!()
    }
}

