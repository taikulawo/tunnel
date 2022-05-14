use std::{collections::HashMap, sync::Arc};
use anyhow::{
    anyhow,
    Result
};
use log::{error, info};

use crate::{
    config::{Outbound, Socks5OutboundSettings},
    proxy::{socks, OutboundHandler},
};

// 管理全部的传出协议 outbound
pub struct OutboundManager {
    pub handlers: HashMap<String, Arc<OutboundHandler>>,
}

impl OutboundManager {
    pub fn new(outbounds: &Vec<Outbound>) -> Result<OutboundManager> {
        let mut handlers = HashMap::new();
        for outbound in outbounds.iter() {
            let handler = match &*outbound.protocol {
                "socks" => {
                    let socks_settings = match &outbound.settings {
                        Some(settings) => match serde_json::from_str::<Socks5OutboundSettings>(settings.get()) {
                            Ok(res) => res,
                            Err(err) => {
                                error!("{}", err);
                                continue
                            }
                        },
                        None => {
                            error!("no socks settings found!");
                            continue;
                        }
                    };
                    let tcp = Arc::new(socks::TcpOutboundHandler {
                        addr: socks_settings.address.clone(),
                        port: socks_settings.port,
                    });
                    let udp = Arc::new(socks::UdpOutboundHandler {
                        addr: socks_settings.address.clone(),
                        port: socks_settings.port,
                    });
                    Arc::new(OutboundHandler::new(
                        outbound.tag.clone(),
                        Some(tcp),
                        Some(udp),
                    ))
                }
                "shadowsocks" => {
                    todo!()
                }
                _ => {
                    info!("found unsupported outbound {}", outbound.tag);
                    continue;
                }
            };
            handlers.insert(outbound.tag.clone(), handler);
        }
        Ok(OutboundManager { handlers })
    }
    pub fn get_handler(&self, tag: &str) -> Option<Arc<OutboundHandler>> {
        self.handlers.get(tag).and_then(|x| Some(x.clone()))
    }
}
