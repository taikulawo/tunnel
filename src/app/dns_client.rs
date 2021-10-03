use anyhow::{anyhow, Result};
use futures_util::{
    future::{self, BoxFuture},
    FutureExt,
};
use log::trace;
use rand::{random, Rng, SeedableRng};
use std::{
    future::Future,
    io,
    net::{IpAddr, SocketAddr},
    process::Output,
    str::FromStr,
    vec,
};
use tokio::net::UdpSocket;
use trust_dns_proto::{
    op::{Message, MessageType, OpCode, Query, ResponseCode},
    rr::{Name, RData, RecordType},
    serialize::binary::BinDecodable,
};

use crate::{
    common::{get_default_interface, get_default_ipv4_gateway, get_default_ipv6_gateway},
    config::AppConfig,
    proxy::create_bounded_udp_socket,
};

macro_rules! random_get {
    ($v:expr) => {{
        use rand::random;
        let len = $v.len();
        let idx = random::<usize>() % len;
        $v.get(idx).expect("never reached!")
    }};
}
pub struct DnsClient {
    /// should be ipv4 addr
    pub remote_dns_servers: Vec<SocketAddr>,
    pub config: AppConfig,
}

impl DnsClient {
    pub fn new(ips: Vec<IpAddr>) -> DnsClient {
        DnsClient {
            remote_dns_servers: ips
                .iter()
                .map(|x| SocketAddr::new(x.clone(), 53))
                .collect::<Vec<SocketAddr>>(),
            config: Default::default(),
        }
    }
    pub fn new_query(&self, host: &String, ty: RecordType) -> Message {
        let mut message = Message::new();
        let mut query = Query::new();
        let name = Name::from_str(&*host).expect("wrong host!");
        let mut random_generator = rand::rngs::StdRng::from_entropy();
        let random = random_generator.gen();
        query.set_name(name).set_query_type(ty);
        message.add_query(query);
        message.set_message_type(MessageType::Query);
        message.set_id(random);
        message.set_op_code(OpCode::Query);
        message.set_recursion_desired(true);
        message
    }

    /// domain string to ip
    pub async fn lookup(&self, host: String) -> Result<Vec<IpAddr>> {
        let AppConfig {
            prefer_ipv6,
            use_ipv6,
        } = self.config;
        let mut tasks: Vec<BoxFuture<Result<Vec<IpAddr>>>> = Vec::new();
        match (use_ipv6, prefer_ipv6) {
            (true, true) => {
                // only wait ipv6 result
                let query = self.new_query(&host, RecordType::AAAA);
                let server = random_get!(self.remote_dns_servers);
                let v = query.to_vec()?;
                let task = DnsClient::do_lookup(v, &*host, server).boxed();
                tasks.push(task);
            }
            (true, false) => {
                // wait the first result
                let server = random_get!(self.remote_dns_servers);
                let query = self.new_query(&host, RecordType::A);
                let v = query.to_vec()?;
                let task = DnsClient::do_lookup(v, &*host, server).boxed();
                tasks.push(task);
                let query = self.new_query(&host, RecordType::AAAA);
                let v = query.to_vec()?;
                let task = DnsClient::do_lookup(v, &*host, server).boxed();
                tasks.push(task);
            }
            (false, ..) => {
                // don't use ipv6
                // just use ipv4
                let server = random_get!(self.remote_dns_servers);
                let query = self.new_query(&host, RecordType::A);
                let v = query.to_vec()?;
                let task = DnsClient::do_lookup(v, &*host, server).boxed();
                tasks.push(task);
            }
        };
        let mut ips = Vec::new();
        for mut res in future::join_all(tasks).await {
            match res {
                Ok(ref mut x) => {
                    ips.append(x);
                }
                Err(err) => return Err(anyhow!("lookup failed {}", err)),
            }
        }
        Ok(ips)
    }
    pub async fn do_lookup(
        request: Vec<u8>,
        host: &str,
        server: &SocketAddr,
    ) -> Result<Vec<IpAddr>> {
        trace!("look up {} on {}", host, &server);
        let socket = match server {
            SocketAddr::V4(v4) => {
                let bind_addr = get_default_ipv4_gateway()?;
                create_bounded_udp_socket(bind_addr)?
            }
            SocketAddr::V6(v6) => {
                let bind_addr = get_default_ipv6_gateway()?;
                create_bounded_udp_socket(bind_addr)?
            }
        };
        match socket.send_to(&*request, server).await {
            Ok(..) => {
                let mut buf = vec![0u8; 512];
                match socket.recv_from(&mut buf).await {
                    Ok((n, ..)) => {
                        let message = Message::from_bytes(&buf[..n])?;
                        if message.response_code() != ResponseCode::NoError {
                            return Err(anyhow!(
                                "dns lookup response indicate failed {}",
                                message.response_code()
                            ));
                        }
                        let anwsers = message.answers();
                        let mut ips = Vec::new();
                        for anwser in anwsers {
                            let rdata = anwser.rdata();
                            match rdata {
                                RData::A(ip) => ips.push(IpAddr::V4(ip.clone())),
                                RData::AAAA(ipv6) => ips.push(IpAddr::V6(ipv6.clone())),
                                _ => {}
                            };
                        }
                        return Ok(ips);
                    }
                    Err(err) => {
                        todo!()
                    }
                }
            }
            Err(err) => {
                todo!()
            }
        };
    }
}
