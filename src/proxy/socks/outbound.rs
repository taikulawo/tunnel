use tokio::io::AsyncWriteExt;

use crate::{net::ProxyStream, proxy::{AnyStream, AnyStreamTrait, ConnectionSession, TcpOutbound}};

pub struct OutboundHandler {

}

impl TcpOutbound for OutboundHandler {
    fn handle(session: ConnectionSession) -> ProxyStream {
        match session {
            
        }
        ProxyStream::connect(addr)
        todo!()
    }
}

// as client
async fn handshake<T>(stream: &mut T) where T: AnyStreamTrait {
    stream.write_all(&[0x05, 0x01, 0x00]).await;
    todo!()
}