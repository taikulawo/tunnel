use std::{io, net::SocketAddr, os::unix::prelude::AsRawFd};

use tokio::net::{TcpSocket, TcpStream};

use crate::common::get_default_interface;

use super::sys::bind_to_device;

pub struct Stream {

}

impl Stream {
    pub async fn connect(addr: SocketAddr) -> io::Result<Stream>{
        let socket = match addr {
            SocketAddr::V4(v4) => TcpSocket::new_v4()?,
            SocketAddr::V6(v6) => TcpSocket::new_v6()?
        };
        let default_interface = get_default_interface()?;
        bind_to_device(&socket, &default_interface);
        todo!()
    }
}
