use std::collections::HashMap;
use std::sync::Arc;
use log::{
    info
};
use anyhow::Result;

use crate::{Config, config::{Socks5InboundSettings, Inbound}, proxy::{socks::{TcpInboundHandler, UdpInboundHandler}, InboundHandlerTrait, InboundHandler, AnyInboundHandler}};
// 统一管理全部 inbound 协议
pub struct InboundManager {
    handlers: HashMap<String, Arc<InboundHandler>>,
}

impl InboundManager {
    pub fn new(config: &Vec<Inbound>) -> InboundManager {
        let mut handlers: HashMap<String, Arc<InboundHandler>> = HashMap::new();

        // 迭代全部的inbound协议，并创建listener
        for inbound in config.iter() {
            let handler = match &*inbound.tag {
                "socks" => {
                    let tcp = Arc::new(TcpInboundHandler);
                    let udp = Arc::new(UdpInboundHandler);
                    InboundHandler::new(inbound.tag.clone(), Some(tcp), Some(udp))
                },
                _ => {
                    info!("unknown tag {}", inbound.tag);
                    continue;               
                }
            };
            handlers.insert(inbound.tag.clone(), Arc::new(handler));
        }
        InboundManager {
            handlers
        }
    }
    pub async fn listen() {

    }
}