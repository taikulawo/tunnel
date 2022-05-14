use std::{sync::Arc, collections::HashMap};

use log::info;

use crate::{config::Outbound, proxy::{OutboundHandler, socks}};

// 管理全部的传出协议 outbound
pub struct OutboundManager {
    pub handlers: HashMap<String, Arc<OutboundHandler>>,
}

impl OutboundManager {
    pub fn new(outbounds: &Vec<Outbound>) -> OutboundManager {
        let mut handlers = HashMap::new();
        for outbound in outbounds.iter() {
            let handler = match &*outbound.protocol {
                "socks" => {
                    let tcp = Arc::new(socks::TcpOutboundHandler{});
                    let udp = Arc::new(socks::UdpOutboundHandler{});
                    Arc::new(OutboundHandler::new(outbound.tag.clone(), Some(tcp), Some(udp)))
                },
                "shadowsocks" => {
                    todo!()
                },
                _ => {
                    info!("found unsupported outbound {}", outbound.tag);
                    continue
                }
            };
            handlers.insert(outbound.tag.clone(), handler);
            
        }
        OutboundManager { 
            handlers
        }
    }
}