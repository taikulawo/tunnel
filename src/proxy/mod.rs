use std::{io, net::{IpAddr, SocketAddr}, os::unix::prelude::{FromRawFd, IntoRawFd}};

use anyhow::Result;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{io::{AsyncRead, AsyncWrite}, net::{TcpSocket, UdpSocket}};
use async_trait::async_trait;

use crate::{common::get_default_interface, net::{ProxyStream, bind_to_device}};

mod socks;
pub trait CommonStreamTrait: AsyncRead + AsyncWrite + Unpin + Send + Sync{}
pub type CommonStream = Box<dyn CommonStreamTrait>;


pub enum NetworkType {
    TCP,
    UDP,
}

pub struct TransportNetwork {
    pub addr: SocketAddr,
    pub net_type: NetworkType
}
#[async_trait]
pub trait Inbound {
    async fn handle(&self, stream: CommonStream, network: TransportNetwork) -> Result<ConnectionSession>;
}

#[async_trait]
pub trait Outbound {
    async fn handle(&self, stream: CommonStream, session: ConnectionSession) -> Result<CommonStream>;
}


pub struct DomainSession {
    name: String,
    port: u16,
}

pub enum Address {
    Domain(String),
    Ip(IpAddr)
}
pub struct ConnectionSession{
    host: Address,
    port: u16,
}

pub fn create_bounded_udp_socket(addr: IpAddr) -> io::Result<UdpSocket>{
    let socket = match addr {
        IpAddr::V4(..) => Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?,
        IpAddr::V6(..) => Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?
    };
    // let s: SockAddr = ;
    socket.bind(&SockAddr::from(SocketAddr::new(addr, 0)));
    socket.set_nonblocking(true);
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