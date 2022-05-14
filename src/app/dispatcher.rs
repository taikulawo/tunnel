use std::{convert::TryFrom, sync::Arc, collections::HashMap, net::{SocketAddr}, io};

use anyhow::{
    Result,
    anyhow
};
use log::{debug, error};
use tokio::{net::{TcpStream, UdpSocket}, sync::RwLock};

use crate::{proxy::{Session, StreamWrapperTrait, Address, OutboundHandler, OutboundResult, socks::TcpOutboundHandler, TcpOutboundHandlerTrait, OutboundConnect}, Config};

use super::{sniffer::{Sniffer}, Router, DnsClient, OutboundManager};

// 负责将请求分发给不同的 代理协议 处理
pub struct Dispatcher {
    router: Arc<Router>,
    dns_client: Arc<RwLock<DnsClient>>,
    outbound_manager: Arc<OutboundManager>
}
impl Dispatcher {
    pub async fn dispatch_tcp(&self, stream:TcpStream, sess: &mut Session) {
        // https://github.com/iamwwc/v2ray-core/blob/8cdd680f5ca8d05c618752eb944a42a7b4d31f6c/app/dispatcher/default.go#L207
        // 由于需要提供 domain routing，所以如果 port == 443，首先尝试嗅探 TLS SNI
        let local_stream: Box<dyn StreamWrapperTrait> = if sess.local_peer.port() == 443 {
            // TLS，嗅探 SNI
            let mut sniffer = Sniffer::new(stream);
            match sniffer.sniff().await {
                Ok(s) => {
                    match s {
                        Some(name) =>  {
                            sess.destination = match Address::try_from((name, sess.port())) {
                                Ok(x) => x,
                                Err(err) =>{
                                    debug!("try from failed {}", err);
                                    return
                                }
                            };
                        },
                        None => {}
                    }
                    Box::new(sniffer)
                },
                Err(err) => {
                    return
                }
            }
        }else {
            Box::new(stream)
        };
        // starting routing match
        let outbound_handler = match self.router.route(&sess) {
            Some(tag) => {
                match self.outbound_manager.get_handler(&*tag) {
                    Some(h) => h,
                    None => {
                        error!("no outbound tag found {}", tag);
                        return;
                    }
                }
            },
            None => {
                error!("no outbound session {:?} found!", &sess);
                return;
            }
        };
        // connect to remote proxy server
        let tcp = if let Some(tcp) = &outbound_handler.tcp_handler {
            tcp
        }else {
            error!("tag {} not have tcp handler !", outbound_handler.tag);
            return;
        };
        let target = TcpOutboundHandlerTrait::remote_addr(tcp.as_ref());
        let proxy_stream = match target {
            OutboundConnect::Proxy(name, port) => {
                connect_remote_tcp(self.dns_client.clone(), name, port).await?
            },
            OutboundConnect::Direct => {

            },
            OutboundConnect::Drop => {

            }
        }
    }

    pub async fn dispatch_udp(&self, socket: UdpSocket, sess: Session) {}

    pub fn new(router: Arc<Router>, dns_client: Arc<RwLock<DnsClient>>, outbound_manager: Arc<OutboundManager>, config: &Config) -> Dispatcher{
        Dispatcher {
            dns_client,
            outbound_manager: outbound_manager,
            router
        }
    }
}

pub async fn connect_remote_tcp(dns_client:Arc<RwLock<DnsClient>>, addr: String, port: u16) -> Result<TcpStream>{
    let socket_addr = match addr.parse::<SocketAddr>() {
        Ok(socket_addr) => socket_addr,
        Err(err) => {
            // maybe domain name
            match dns_client.read().await.lookup(&addr).await {
                Ok(ips) => {
                    // TODO connect to multiple ips
                    let ip = if let Some(ip) = ips.get(0) {
                        ip
                    }else {
                        return Err(anyhow!("dns not ip found"))
                    };
                    SocketAddr::new(ip.clone(), port)
                },
                Err(e) => {
                    return Err(e)
                }
            }
        }
    };
    // 这样可以
    Ok(TcpStream::connect(socket_addr).await?)
    // 但下面不行
    // TcpStream::connect(socket_addr).await
    // 原因是 ? 进行 type conversion, anyhow::Result 实现了 from io::Error 转换
    // https://stackoverflow.com/a/62241599/7529562
}