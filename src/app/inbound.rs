use anyhow::Result;
use futures::FutureExt;
use futures_util::future::BoxFuture;
use log::{error, info};
use std::str::FromStr;
use std::sync::Arc;
use std::{collections::HashMap, net::SocketAddr};

use crate::{
    config::{Inbound},
    proxy::{
        socks::{TcpInboundHandler, UdpInboundHandler}, InboundHandler,
    },
};

use super::{Dispatcher, InboundListener, UdpAssociationManager};
// 统一管理全部 inbound 协议
pub struct InboundManager {
    handlers: HashMap<String, Arc<InboundHandler>>,
    configs: Vec<Inbound>,
    nat: Arc<UdpAssociationManager>,
}

impl InboundManager {
    pub fn new(config: Vec<Inbound>, nat: Arc<UdpAssociationManager>) -> InboundManager {
        let mut handlers: HashMap<String, Arc<InboundHandler>> = HashMap::new();

        // 迭代全部的inbound协议，并创建listener
        for inbound in config.iter() {
            let handler = match &*inbound.protocol {
                "socks" => {
                    let tcp = Arc::new(TcpInboundHandler);
                    let udp = Arc::new(UdpInboundHandler);
                    InboundHandler::new(inbound.tag.clone(), Some(tcp), Some(udp))
                }
                _ => {
                    info!("unknown protocol: {} tag: {}", inbound.protocol, inbound.tag);
                    continue;
                }
            };
            handlers.insert(inbound.tag.clone(), Arc::new(handler));
        }
        InboundManager {
            handlers,
            configs: config,
            nat
        }
    }
    pub fn listen(mut self, dispatcher: Arc<Dispatcher>) -> Result<BoxFuture<'static, ()>> {
        let mut tasks = Vec::new();
        for config in self.configs {
            let dispatcher = dispatcher.clone();
            if let Some(handler) = self.handlers.get_mut(&config.tag) {
                let Inbound { port, listen, protocol, .. } = config;
                // 除 tun 外，其他protocol都必须有port
                let mut future = match protocol.as_str() {
                    "tun" => {
                        todo!()
                    },
                    _ => {
                        let addr = match SocketAddr::from_str(format!("{}:{}", listen.unwrap(), port.unwrap()).as_str()) {
                            Ok(x) => x,
                            Err(err) => {
                                error!("invalid listen or port field {}", err);
                                continue;
                            }
                        };
                        InboundListener::listen(dispatcher, handler.clone(), addr, self.nat.clone())?
                    }
                };
                tasks.append(&mut future);
            }
        }
        Ok(async {
            futures::future::join_all(tasks).await;
        }.boxed())
    }
}
