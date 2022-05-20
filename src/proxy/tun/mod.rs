use etherparse::{
    IpHeader, PacketHeaders, ReadError, TransportHeader,
};
use ipnet::Ipv4Net;
use log::error;
use std::{
    error::Error,
    io::{self, Cursor, ErrorKind},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tun::{AsyncDevice, Device, Layer};

use tcp::TcpTun;
mod tcp;
pub struct Tun {
    device: AsyncDevice,
    tcp_tun: TcpTun,
}

impl Tun {
    pub async fn new() -> io::Result<Tun> {
        let mut config = tun::Configuration::default();
        let netmask = 24;
        config.address("10.0.0.1").netmask(24).layer(Layer::L3).up();
        let device = match tun::create_as_async(&config) {
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
            Ok(x) => x,
        };
        let tun_address = match device.get_ref().address() {
            Err(err) => return Err(io::Error::new(ErrorKind::Other, err)),
            Ok(x) => x,
        };
        let tun_network = Ipv4Net::new(tun_address, netmask).expect("ipv4 net new");
        let tcp_tun = TcpTun::new(tun_network.into())
            .await
            .expect("tcp tun error");
        Ok(Tun { device, tcp_tun })
    }
    pub async fn run(mut self) -> io::Result<()> {
        let mtu = self.device.get_mut().mtu().expect("mtu");
        let mut packet = vec![0u8; mtu as usize].into_boxed_slice();
        // self.device.read(buf)
        loop {
            tokio::select! {
                n = self.device.read(&mut packet) => {
                    let n = n?;
                    if self.handle_ip_packet(&mut packet).await? {
                        self.device.write_all(&packet).await?;
                    };
                    println!("{} bytes read", n);
                }
            }
        }
    }
    async fn handle_ip_packet(&self, packet: &mut [u8]) -> io::Result<bool> {
        let mut ip_packet = match PacketHeaders::from_ip_slice(packet) {
            Ok(ip) => ip,
            Err(ReadError::IoError(err)) => return Err(err),
            Err(err) => return Err(io::Error::new(ErrorKind::Other, err)),
        };
        // 看内部实现，payload 是 传输层 的 payload
        // 已经排除 传输层 的header
        let payload_len = ip_packet.payload.len();
        let mut ip_header = match ip_packet.ip {
            Some(ref mut header) => header,
            None => {
                error!("unknown ethernet packet {:?}", ip_packet);
                return Err(io::Error::new(ErrorKind::Other, "unknown ethernet packet"));
            }
        };
        let (src_ip, destination_ip): (IpAddr, IpAddr) = match ip_header {
            IpHeader::Version4(v4) => (
                Ipv4Addr::from(v4.source).into(),
                Ipv4Addr::from(v4.destination).into(),
            ),
            IpHeader::Version6(v6) => (
                Ipv6Addr::from(v6.source).into(),
                Ipv6Addr::from(v6.destination).into(),
            ),
        };
        // mapping ip
        match ip_packet.transport {
            Some(TransportHeader::Tcp(ref mut tcp_header)) => {
                // port map
                let src_addr = SocketAddr::new(src_ip, tcp_header.source_port);
                let dest_addr = SocketAddr::new(destination_ip, tcp_header.destination_port);
                let (final_src_addr, final_dest_addr) = match self
                    .tcp_tun
                    .handle_packet(src_addr, dest_addr, tcp_header)
                    .await?
                {
                    Some(x) => x,
                    None => return Ok(false),
                };
                // replace src ip, port
                match (final_src_addr, &mut ip_header) {
                    (SocketAddr::V4(v4), IpHeader::Version4(v4_header)) => {
                        v4_header.source = v4.ip().octets()
                    }
                    (SocketAddr::V6(v6), IpHeader::Version6(v6_header)) => {
                        v6_header.source = v6.ip().octets()
                    }
                    _ => unreachable!("src ip replace unreachable!"),
                };
                // replace dest ip, port
                match (final_dest_addr, &mut ip_header) {
                    (SocketAddr::V4(v4), IpHeader::Version4(v4_header)) => {
                        v4_header.destination = v4.ip().octets()
                    }
                    (SocketAddr::V6(v6), IpHeader::Version6(v6_header)) => {
                        v6_header.destination = v6.ip().octets()
                    }
                    _ => unreachable!("dest ip replace unreachable!"),
                }
                // calculate tcp checksum
                match ip_header {
                    IpHeader::Version4(v4_ip_header) => {
                        tcp_header.checksum = tcp_header
                            .calc_checksum_ipv4(&v4_ip_header, ip_packet.payload)
                            .expect("tcp calculate check sum error")
                    }
                    IpHeader::Version6(v6_ip_header) => {
                        tcp_header.checksum = tcp_header
                            .calc_checksum_ipv6(&v6_ip_header, ip_packet.payload)
                            .expect("tcp calculate check sum error")
                    }
                }
                let (headers, ..) = packet.split_at_mut(packet.len() - payload_len);
                // write ip header and tcp header into transport
                let mut cursor = Cursor::new(headers);
                ip_header
                    .write(&mut cursor)
                    .expect("ip header write failed!");
                tcp_header
                    .write(&mut cursor)
                    .expect("tcp header write failed!");
            }
            Some(TransportHeader::Udp(ref mut _udp_header)) => {}
            None => {}
        };
        Ok(true)
    }
}
