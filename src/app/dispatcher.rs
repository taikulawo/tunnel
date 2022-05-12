use std::convert::TryFrom;

use log::debug;
use tokio::net::{TcpStream, UdpSocket};

use crate::proxy::{Session, StreamWrapperTrait, Address};

use super::sniffer::{self, Sniffer};

// 负责将请求分发给不同的 代理协议 处理
pub struct Dispatcher {}
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
    }

    pub async fn dispatch_udp(&self, socket: UdpSocket, sess: Session) {}
}
