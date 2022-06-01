use log::error;
use std::{io, net::SocketAddr, sync::Arc};

use crate::{
    proxy::{
        socks::handshake_as_server, Session, InboundResult, TcpInboundHandlerTrait,
        UdpInboundHandlerTrait, InboundDatagramTrait, Address,
    },
};
use async_trait::async_trait;
use tokio::net::{TcpStream, UdpSocket};

pub struct TcpInboundHandler;

#[async_trait]
impl TcpInboundHandlerTrait for TcpInboundHandler {
    async fn handle(&self, _conn: Session, mut stream: TcpStream) -> io::Result<InboundResult> {
        let session = match handshake_as_server(&mut stream).await {
            Ok(session) => session,
            Err(err) => {
                error!("failed to process socks inbound {}", err);
                return Err(io::Error::new(io::ErrorKind::Other, "unknown"));
            }
        };
        Ok(InboundResult::Stream(stream, session))
    }
}

pub struct UdpInboundHandler;

#[async_trait]
impl UdpInboundHandlerTrait for UdpInboundHandler {
    async fn handle(&self, socket: UdpSocket) -> io::Result<InboundResult> {
        // socks5 对 udp 会有单独的连接流程
        // 由于 udp 的connectionless 特性，所以 client 只发送一次，header， data 都包含在其中
        // https://datatracker.ietf.org/doc/html/rfc1928#section-7

        // 由于 IP 层不可靠，server收到的包可能丢失，可能乱序，可能重复，所以 socks5 的 UDP 提供 FRAG 对收到的 UDP 数据重组
        // 但实现这个功能不是强制的
        // Implementation of fragmentation is optional; an implementation that
        // does not support fragmentation MUST drop any datagram whose FRAG
        // field is other than X'00'.
        // https://github.com/iamwwc/v2ray-core/blob/02f251ebecbf21095c7b74cb3f0feaed0927d3f9/proxy/socks/protocol.go#L321
        let udp = Socks5Datagram {
            socket
        };
        Ok(InboundResult::Datagram(Arc::new(udp)))
    }
}

struct Socks5Datagram {
    socket: UdpSocket
}

#[async_trait]
impl InboundDatagramTrait for Socks5Datagram {
    async fn recv_from(&self) ->io::Result<(Vec<u8> ,SocketAddr, Address)>  {
        let mut buf = vec![0u8; 1024];
        match self.socket.recv_from(&mut buf).await {
            Ok((n, source_addr)) => {

            },
            Err(err) => {

            }
        }
        todo!()
    }
    async fn send_to(&self, buf: Vec<u8>, dest: SocketAddr) ->io::Result<usize>  {
        match self.socket.send_to(&buf, dest).await {
            Ok(n) => {

            }
            Err(err) => {

            }
        }
        todo!()
    }
}
