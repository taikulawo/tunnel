use std::{
    net::{IpAddr, SocketAddr},
    ops::Add,
    str::FromStr,
};

use anyhow::{anyhow, bail, Result};
use ipnet::IpAdd;
use tokio::{io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt}, net::TcpStream};
use trust_dns_proto::rr::rdata::name;

use crate::proxy::{
    Address, GeneralConn, GeneralConnTrait, ConnSession, DomainSession, Inbound,
    ProxyStream,
};

mod inbound;
mod outbound;

pub use self::inbound::SocksTcpInboundHandler;
pub use self::inbound::SocksUdpInboundHandler;
pub use self::outbound::OutboundHandler;
const NO_AUTHENTICATION_REQUIRED: u8 = 0x01;
const CMD_CONNECT: u8 = 0x01;
const CMD_BIND: u8 = 0x02;
const CMD_UDP_ASSOCIATE: u8 = 0x03;
const TYPE_IPV4: u8 = 0x01;
const TYPE_DOMAIN: u8 = 0x03;
const TYPE_IPV6: u8 = 0x04;
// as client
async fn handshake_as_client<T>(stream: &mut T, session: &ConnSession) -> Result<()>
where
    T: GeneralConnTrait,
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

fn build_request(buf: &mut Vec<u8>, session: &ConnSession) {
    buf.extend(&[0x05, 0x01, 0x00]);
    buf.extend(&[CMD_CONNECT]); // TODO support more ATYP instead of only CONNECT
    match session.host {
        Address::Domain(ref name) => {
            buf.push(TYPE_DOMAIN);
            buf.push(name.len() as u8);
            buf.extend_from_slice(name.as_bytes());
        }
        Address::Ip(ref ip) => match *ip {
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
    let port = session.port;
    buf.push((port >> 8) as u8);
    buf.push(port as u8);
}

// as server
async fn handshake_as_server(stream: &mut TcpStream) -> Result<ConnSession> {
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
            Address::Domain(name.to_string())
        }
        TYPE_IPV4 => {
            buf.resize(4, 0);
            stream.read_exact(&mut buf).await?;
            let str = String::from_utf8_lossy(&buf);
            let ipv4 = match IpAddr::from_str(&str) {
                Ok(x) => x,
                Err(err) => bail!("should be ipv4 {} {:?}", err, &buf),
            };
            Address::Ip(ipv4)
        }
        TYPE_IPV6 => {
            buf.resize(16, 0);
            stream.read_exact(&mut buf).await?;
            let str = String::from_utf8_lossy(&buf);
            let ipv6 = match IpAddr::from_str(&str) {
                Ok(x) => x,
                Err(err) => bail!("should be ipv6 {} {:?}", err, &buf),
            };
            Address::Ip(ipv6)
        }
        _ => bail!("unknown atyp {}", buf[3]),
    };
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await?;
    let port = unsafe { u16::from_be(*(buf.as_ptr() as *const u16)) };
    let res = ConnSession {
        host: address,
        port,
    };
    Ok(res)
}
