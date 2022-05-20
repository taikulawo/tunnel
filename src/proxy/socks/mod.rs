use std::{
    convert::TryInto,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    slice,
    str::FromStr,
};

use anyhow::{anyhow, bail, Result};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::proxy::{Address, Session};

mod inbound;
mod outbound;
mod udp;

pub use self::inbound::TcpInboundHandler;
pub use self::inbound::UdpInboundHandler;
pub use self::outbound::TcpOutboundHandler;
pub use self::outbound::UdpOutboundHandler;

use super::{Network, StreamWrapperTrait};
const NO_AUTHENTICATION_REQUIRED: u8 = 0x00;
const CMD_CONNECT: u8 = 0x01;
const CMD_BIND: u8 = 0x02;
const CMD_UDP_ASSOCIATE: u8 = 0x03;
const TYPE_IPV4: u8 = 0x01;
const TYPE_DOMAIN: u8 = 0x03;
const TYPE_IPV6: u8 = 0x04;
// as client
pub async fn handshake_as_client<T>(stream: &mut T, session: &Session) -> Result<()>
where
    T: StreamWrapperTrait,
{
    stream.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut buf = vec![0u8; 2];
    stream.read_exact(&mut buf).await?;
    if buf[1] != NO_AUTHENTICATION_REQUIRED {
        return Err(anyhow!("only no authentication supported {:?}", &buf));
    }
    let mut buf = Vec::new();
    build_request(&mut buf, session);
    stream.write_all(&*buf).await?;
    buf.resize(10, 0);
    stream.read_exact(&mut buf).await?;
    if buf[..2] != [0x05, 0x00] {
        bail!("unexpected reply from server {:?}", buf);
    }
    Ok(())
}

fn build_request(buf: &mut Vec<u8>, session: &Session) {
    // TODO support more ATYP instead of only CONNECT
    buf.extend(&[0x05, 0x01, 0x00]);
    match session.destination {
        Address::Domain(ref name, _) => {
            buf.push(TYPE_DOMAIN);
            buf.push(name.len() as u8);
            buf.extend_from_slice(name.as_bytes());
        }
        Address::Ip(ref addr) => match addr.ip() {
            IpAddr::V4(ref v4) => {
                buf.push(TYPE_IPV4);
                buf.extend(v4.octets());
            }
            IpAddr::V6(ref v6) => {
                buf.push(TYPE_IPV6);
                buf.extend(v6.octets());
            }
        },
    };
    let port = match session.destination {
        Address::Domain(_, port) => port,
        Address::Ip(addr) => addr.port(),
    };
    buf.push((port >> 8) as u8);
    buf.push(port as u8);
}

// as server
pub async fn handshake_as_server(stream: &mut TcpStream) -> Result<Session> {
    let mut buf = vec![0; 3];
    stream.read_exact(&mut buf).await?;
    let version = buf[0];
    if version != 0x05 {
        bail!("only version 5 supported {}", &version)
    };
    stream.write_all(&[0x05, 0x00]).await?;
    buf.resize(4, 0);
    stream.read_exact(&mut buf).await?;
    let address: Address = match buf[3] {
        TYPE_DOMAIN => {
            let mut len_buf = [0; 1];
            stream.read(&mut len_buf).await?;
            let len = len_buf[0];
            buf.resize(len.into(), 0);
            stream.read_exact(&mut buf).await?;
            let name = String::from_utf8_lossy(&buf);
            Address::Domain(name.to_string(), 0)
        }
        TYPE_IPV4 => {
            buf.resize(4, 0);
            stream.read_exact(&mut buf).await?;
            let ipv4 = Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]);
            Address::Ip(SocketAddr::new(IpAddr::V4(ipv4), 0))
        }
        TYPE_IPV6 => {
            buf.resize(16, 0);
            stream.read_exact(&mut buf).await?;
            let buf: &[u16] = unsafe { slice::from_raw_parts(buf.as_ptr() as *const _, 8) };
            let ipv6 = Ipv6Addr::new(
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            );
            Address::Ip(SocketAddr::new(IpAddr::V6(ipv6), 0))
        }
        _ => bail!("unknown atyp {}", buf[3]),
    };
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await?;
    let port = unsafe { u16::from_be(*(buf.as_ptr() as *const u16)) };
    let address = match address {
        Address::Domain(domain, _) => Address::Domain(domain, port),
        Address::Ip(addr) => match addr {
            SocketAddr::V4(mut v4) => {
                v4.set_port(port);
                Address::Ip(SocketAddr::V4(v4))
            }
            SocketAddr::V6(mut v6) => {
                v6.set_port(port);
                Address::Ip(SocketAddr::V6(v6))
            }
        },
    };
    let res = Session {
        destination: address,
        network: Network::TCP,
        local_peer: stream.local_addr().expect("local"),
    };
    Ok(res)
}
