use tokio::io::AsyncWriteExt;

use crate::proxy::{AnyStreamTrait, ConnectionSession, ProxyStream, TcpInbound};

mod inbound;
mod outbound;

pub struct Socks {
    
}

// as client
async fn handshake_as_client<T>(stream: &mut T)
where
    T: AnyStreamTrait,
{
    stream.write_all(&[0x05, 0x01, 0x00]).await;
    todo!()
}

// as server
async fn handshake_as_server<T>(stream: &mut T) -> ConnectionSession
where
    T: AnyStreamTrait
{
    todo!()
}
