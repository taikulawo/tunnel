use std::{convert::TryFrom, sync::Arc};


use log::{debug, error};
use tokio::{
    net::{TcpStream, UdpSocket},
    sync::RwLock,
};

use crate::{
    proxy::{
        Address, Session,
        StreamWrapperTrait, TcpOutboundHandlerTrait,
    },
    config::Config, Context,
};

use super::{sniffer::Sniffer, DnsClient, OutboundManager, Router};

// 负责将请求分发给不同的 代理协议 处理
pub struct Dispatcher {
    ctx: Arc<Context>,
    router: Arc<Router>,
    dns_client: Arc<RwLock<DnsClient>>,
    outbound_manager: Arc<OutboundManager>,
}
impl Dispatcher {
    pub async fn dispatch_tcp(&self, stream: TcpStream, sess: &mut Session) {
        // https://github.com/iamwwc/v2ray-core/blob/8cdd680f5ca8d05c618752eb944a42a7b4d31f6c/app/dispatcher/default.go#L207
        // 由于需要提供 domain routing，所以如果 port == 443，首先尝试嗅探 TLS SNI
        let mut local_stream: Box<dyn StreamWrapperTrait> = if sess.local_peer.port() == 443 {
            // TLS，嗅探 SNI
            let mut sniffer = Sniffer::new(stream);
            match sniffer.sniff().await {
                Ok(s) => {
                    match s {
                        Some(name) => {
                            sess.destination = match Address::try_from((name, sess.port())) {
                                Ok(x) => x,
                                Err(err) => {
                                    debug!("try from failed {}", err);
                                    return;
                                }
                            };
                        }
                        None => {}
                    }
                    Box::new(sniffer)
                }
                Err(_err) => return,
            }
        } else {
            Box::new(stream)
        };
        // starting routing match
        let outbound_handler = match self.router.route(&sess) {
            Some(tag) => match self.outbound_manager.get_handler(&*tag) {
                Some(h) => h,
                None => {
                    error!("no outbound tag found {}", tag);
                    return;
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
        } else {
            error!("tag {} not have tcp handler !", outbound_handler.tag);
            return;
        };
        let mut remote_stream =
            match TcpOutboundHandlerTrait::handle(tcp.as_ref(), self.ctx.clone(), sess).await {
                Ok(res) => res,
                Err(err) => {
                    debug!(
                        "connect to {} failed. connection {} -> {} {}",
                        sess.destination.to_string(), sess.local_peer, sess.destination, err
                    );
                    return;
                }
            };
        // start pipe
        tokio::io::copy_bidirectional(&mut local_stream, &mut remote_stream).await;
    }

    pub async fn dispatch_udp(&self, _socket: UdpSocket, _sess: Session) {}

    pub fn new(
        context: Arc<Context>,
        router: Arc<Router>,
        dns_client: Arc<RwLock<DnsClient>>,
        outbound_manager: Arc<OutboundManager>,
        _config: Config,
    ) -> Dispatcher {
        Dispatcher {
            ctx: context,
            dns_client,
            outbound_manager: outbound_manager,
            router,
        }
    }
}
