use std::{future::Future, io, net::{IpAddr, SocketAddr}, process::Output, str::FromStr};
use futures_util::{FutureExt, future::{self, BoxFuture}};
use rand::{Rng, SeedableRng, random};
use trust_dns_proto::{op::{Message, MessageType, OpCode, Query}, rr::{Name, RecordType}};

use crate::{config::AppConfig, proxy::create_bounded_udp_socket};


macro_rules! random_get {
    ($v:ident) => {
        use rand::random;
        let len = v.len();
        let idx = random::<usize>() % len;
        v.get(idx).expect("never reached!")
    };
}
pub struct DnsClient {
    remote_dns_servers: Vec<SocketAddr>,
    config: AppConfig
}

impl DnsClient {
    pub fn new(ips: Vec<IpAddr>) -> DnsClient {
        DnsClient {
            remote_dns_servers: ips.iter().map(|x| SocketAddr::new(x.clone(), 53)).collect::<Vec<SocketAddr>>(),
            config: Default::default()
        }
    }
    pub async fn new_query(&self, host: String, ty: RecordType) -> Message {
        let mut message =  Message::new();
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
    pub async fn lookup(&self, host: String) -> io::Result<Vec<IpAddr>> {
        let AppConfig {
            prefer_ipv6,
            use_ipv6
        } = self.config;
        let tasks :Vec<BoxFuture<IpAddr>>= match (use_ipv6, prefer_ipv6) {
            (true, true) => {
                // only wait ipv6 result
                let query = self.new_query(host, RecordType::AAAA);
                let addr = random_get!(self.remote_dns_servers);
                let socket = create_bounded_udp_socket()?;
                
                todo!();
            },
            (true, false) => {
                // wait first result
            },
            (false, ..) => {
                // don't use ipv6
                // just use ipv4
            }
        };
        match future::join_all(tasks).await {

        }
        todo!()
    }
}