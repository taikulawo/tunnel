use std::{error::Error, io::{self, ErrorKind}, net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr}};
use log::{
    error
};
use etherparse::{IpHeader, PacketHeaders, ReadError, TransportHeader};
use tokio::io::AsyncReadExt;
use tun::{AsyncDevice, Device, Layer};

pub struct Tun {
    device: AsyncDevice
}


impl Tun {
    pub fn new() -> io::Result<Tun> {
        let mut config = tun::Configuration::default();
        config.address("10.0.0.1").netmask("255.255.255.0").layer(Layer::L3).up();
        let device = match tun::create_as_async(&config) {
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
            Ok(x) => x
        };
        Ok(Tun {
            device
        })
    }
    pub async fn run(mut self) -> io::Result<()> {
        let mtu = self.device.get_mut().mtu().expect("mtu");
        let mut packet = vec![0u8; mtu as usize].into_boxed_slice();
        // self.device.read(buf)
        loop {
            tokio::select! {
                n = self.device.read(&mut packet) => {
                    let n = n?;
                    println!("{} bytes read", n);
                }
            }
        }
    }
    async fn handle_ip_packet(&self, packet: &mut[u8]) -> io::Result<bool> {
        let mut ipPacket = match PacketHeaders::from_ip_slice(packet) {
            Ok(ip) => ip,
            Err(ReadError::IoError(err)) => return Err(err),
            Err(err) => return Err(io::Error::new(ErrorKind::Other, err))
        };
        let payload_len = ipPacket.payload.len();
        let header = match ipPacket.ip {
            Some(ref header) => header,
            None => {
                error!("unknown ethernet packet {:?}", ipPacket);
                return Err(io::Error::new(ErrorKind::Other, "unknown ethernet packet"))
            }
        };
        let (src_ip, destination_ip): (IpAddr, IpAddr) = match header{
            IpHeader::Version4(v4) => (Ipv4Addr::from(v4.source).into(), Ipv4Addr::from(v4.destination).into()),
            IpHeader::Version6(v6) => (Ipv6Addr::from(v6.source).into(), Ipv6Addr::from(v6.destination).into())
        };
        // mapping ip
        match ipPacket.transport {
            Some(TransportHeader::Tcp(ref mut tcp_header)) => {

            },
            Some(TransportHeader::Udp(ref mut udp_header)) => {
                
            },
            None => {

            }
        };
        Ok(true)
    }
}