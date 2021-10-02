use std::{io, net::{SocketAddr, UdpSocket}, os::unix::prelude::{FromRawFd, IntoRawFd}};

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{io::{AsyncRead, AsyncWrite}, net::{TcpSocket}};

use crate::{common::get_default_interface, net::{ProxyStream, bind_to_device}};

mod socks;

pub trait TcpInbound {
    fn handle(stream: ProxyStream) -> ConnectionSession;
}
pub trait AnyStreamTrait: AsyncRead + AsyncWrite + Unpin {}
pub type AnyStream = Box<dyn AnyStreamTrait>;

pub trait TcpOutbound {
    fn handle(session: ConnectionSession) -> ProxyStream;
}

pub struct DomainSession {
    dest: String,
    port: u16,
}
pub enum ConnectionSession{
    Domain(DomainSession),
    IP(SocketAddr)
}

pub enum AddressFamily {
    TCP,
    UDP,
}

pub enum BoundedSocket {
    Udp(UdpSocket),
    Tcp(TcpSocket)
}
pub fn create_bounded_udp_socket(addr: SocketAddr) -> io::Result<UdpSocket>{
    let socket = match addr {
        SocketAddr::V4(v4) => Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?,
        SocketAddr::V6(v6) => Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?
    };
    // let s: SockAddr = ;
    socket.bind(&addr.into());
    socket.set_nonblocking(true);
    unsafe { Ok(UdpSocket::from_raw_fd(socket.into_raw_fd())) }
}

pub fn create_bounded_tcp_socket(addr: SocketAddr) -> io::Result<TcpSocket> {
    let socket = match addr {
        SocketAddr::V4(..) => Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?,
        SocketAddr::V6(..) => Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))?,
    };
    socket.bind(&addr.into());
    socket.set_nonblocking(true);
    unsafe { Ok(TcpSocket::from_raw_fd(socket.into_raw_fd())) }
}