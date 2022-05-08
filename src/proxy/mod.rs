use std::{
    io,
    net::{IpAddr, SocketAddr},
    os::unix::prelude::{FromRawFd, IntoRawFd}, sync::Arc,
};

use anyhow::Result;
use async_trait::async_trait;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpSocket, UdpSocket, TcpStream},
};

use crate::{
    common::get_default_interface,
    net::{bind_to_device, ProxyStream},
};

pub mod socks;
pub trait GeneralConnTrait: AsyncRead + AsyncWrite + Unpin + Send + Sync {}
impl<S> GeneralConnTrait for S where S: AsyncRead + AsyncWrite + Unpin + Send + Sync {}
pub type GeneralConn = Box<dyn GeneralConnTrait>;
// pub 
pub enum NetworkType {
    TCP,
    UDP,
}

pub struct TransportNetwork {
    pub addr: SocketAddr,
    pub net_type: NetworkType,
}
#[async_trait]
pub trait Inbound {
    async fn handle(
        &self,
        conn: GeneralConn,
        network: NetworkType,
    ) -> Result<ConnSession>;
    fn network() -> Vec<NetworkType>;
}

#[async_trait]
pub trait Outbound {
    async fn handle(
        &self,
        conn: GeneralConn,
        session: ConnSession,
    ) -> Result<GeneralConn>;
}

pub struct DomainSession {
    name: String,
    port: u16,
}

pub enum Address {
    Domain(String),
    Ip(IpAddr),
}
// connection session
pub struct ConnSession {
    host: Address,
    port: u16,
}

pub fn create_bounded_udp_socket(addr: IpAddr) -> io::Result<UdpSocket> {
    let socket = match addr {
        IpAddr::V4(..) => Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?,
        IpAddr::V6(..) => Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?,
    };
    // let s: SockAddr = ;
    match socket.bind(&SockAddr::from(SocketAddr::new(addr, 0))) {
        Ok(..) => {},
        Err(err) => {
            log::error!("failed to bind socket {}", err.to_string())
        }
    }
    match socket.set_nonblocking(true) {
        Ok(..) => {},
        Err(err) => {
            log::error!("failed to set non blocking {}", err)
        }
    }
    Ok(UdpSocket::from_std(socket.into())?)
}

pub fn create_bounded_tcp_socket(addr: SocketAddr) -> io::Result<TcpSocket> {
    let socket = match addr {
        SocketAddr::V4(..) => Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?,
        SocketAddr::V6(..) => Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))?,
    };
    socket.bind(&addr.into());
    socket.set_nonblocking(true);
    Ok(TcpSocket::from_std_stream(socket.into()))
}



// ----------------------------

pub enum InboundResult {
    Stream(TcpStream, ConnSession),
    Datagram(UdpSocket, ConnSession),
}

pub type AnyTcpInboundHandler = Arc<dyn TcpInboundHandlerTrait>;
pub type AnyUdpInboundHandler = Arc<dyn UdpInboundHandlerTrait>;
pub type AnyInboundHandler = Arc<dyn InboundHandlerTrait>;

pub struct InboundHandler {
    tag: String,
    tcp_handler: Option<AnyTcpInboundHandler>,
    udp_handler: Option<AnyUdpInboundHandler>,
}

impl InboundHandler {
    pub fn new(tag: String, tcp: Option<AnyTcpInboundHandler>, udp: Option<AnyUdpInboundHandler>) -> InboundHandler {
        InboundHandler {
            tag,
            tcp_handler: tcp,
            udp_handler: udp,
        }
    }
}

pub trait InboundHandlerTrait: TcpInboundHandlerTrait + UdpInboundHandlerTrait + Sync + Send {
    fn has_tcp(&self) -> bool;
    fn has_udp(&self) -> bool;
}

#[async_trait]
pub trait TcpInboundHandlerTrait {
    async fn handle(&self, session: ConnSession, stream: TcpStream) -> io::Result<InboundResult>;
}

#[async_trait]
pub trait UdpInboundHandlerTrait {
    async fn handle(&self, session: ConnSession, socket: tokio::net::UdpSocket) -> io::Result<InboundResult>;
}