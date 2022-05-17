use std::{io, net::{IpAddr, Ipv4Addr}};

use log::error;
use tokio::io::{AsyncRead, AsyncReadExt};
use anyhow::{
    anyhow
};

type Reader = AsyncRead + Unpin;

pub async fn handshake(stream: &mut Reader)
{
    // https://datatracker.ietf.org/doc/html/rfc1928#section-7
    let mut buf: Vec<u8> = Vec::new();
    buf.resize(4, 0);
    match stream.read_exact(&mut*buf).await {
        Ok(x) => x,
        Err(err) => {
            error!("{}", err);
            return;
        }
    };
    if buf[..2] != [0x00, 0x00] {
        // https://stackoverflow.com/a/27650405/7529562
        error!("Reserved should be X'0000'. actual: {:#04X?}", &buf[..2]);
        return;
    }
    if buf[2] != 0x00 {
        error!("FRAG is not implemented");
        return;
    } 
}
const ATYP_IPV4: u8 = 0x01;
const ATYP_DOMAIN: u8 = 0x03;
const ATYP_IPV6: u8 = 0x04;
pub async fn Socks5AddrReader(reader: &mut Reader) -> anyhow::Result<()> {
    let mut buf = vec![0;1];
    reader.read_exact(&mut buf).await?;
    match buf[0] {
        ATYP_IPV4 => {
            buf.resize(4, 0);
            reader.read_exact(&mut buf).await?;
            let v4 = Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]);
        }
        ATYP_DOMAIN => {

        }
        ATYP_IPV6 => {

        }
        _ => {
            return Err(anyhow!("unknown cmd {}", buf[0]));
        }
    }
    Ok(())
}