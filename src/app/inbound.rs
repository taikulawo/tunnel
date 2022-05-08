use std::collections::HashMap;
use std::sync::Arc;
use log::{
    info
};
use anyhow::Result;

use crate::{Config, config::{SocksInboundSettings, Inbound}, proxy::{socks::{SocksTcpInboundHandler, SocksUdpInboundHandler}, InboundHandlerTrait, InboundHandler, AnyInboundHandler}};

pub async fn inboundInit(config: Arc<Config>) -> Result<()> {
    let mut inbounds = &config.inbounds;
    let mut handlers = HashMap::new();
    for inbound in inbounds {
        match &*inbound.protocol {
            "socks" => {
                if let Some(ref settings) = inbound.settings {
                    let socks_settings: SocksInboundSettings = serde_json::from_str(settings.get())?;
                    let handler = SocksTcpInboundHandler{};
                    handlers.insert(inbound.tag.clone(), handler);
                }
            },
            _ => {
                continue
            }
        };
    }
    todo!()
}

// 统一管理全部 inbound 协议
pub struct InboundManager {

}

impl InboundManager {
    pub fn new(config: &Vec<Inbound>) -> InboundManager {
        let mut handlers: HashMap<String, Arc<InboundHandler>> = HashMap::new();

        // 迭代全部的inbound协议，并创建listener
        for inbound in config.iter() {
            let handler = match &*inbound.tag {
                "socks" => {
                    let tcp = Arc::new(SocksTcpInboundHandler);
                    let udp = Arc::new(SocksUdpInboundHandler);
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

        }
    }
}