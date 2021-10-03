use std::net::SocketAddr;

use tokio::io::AsyncWriteExt;

use crate::{
    net::ProxyStream,
    proxy::{AnyStream, AnyStreamTrait, ConnectionSession, DomainSession, TcpOutbound},
};

pub struct OutboundHandler {}

impl TcpOutbound for OutboundHandler {
    fn handle(stream: ProxyStream, session: ConnectionSession) -> ProxyStream {
        let addr: SocketAddr = match session {
            ConnectionSession::Domain(..) => {
                todo!()
            }
            ConnectionSession::IP(..) => {
                todo!()
            }
        };
        ProxyStream::connect(addr);
        todo!()
    }
}

