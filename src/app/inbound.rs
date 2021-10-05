use std::sync::Arc;

use anyhow::Result;

use crate::{Config, config::SocksInboundSettings, proxy::socks::SocksInbound};

pub async fn inboundInit(config: Arc<Config>) -> Result<()> {
    let inbounds = &config.inbounds;
    let mut handlers = Vec::new();
    for inbound in inbounds {
        let inbound_handler = match &*inbound.protocol {
            "socks" => {
                if let Some(ref settings) = inbound.settings {
                    let socks_settings: SocksInboundSettings = serde_json::from_str(settings.get())?;
                    SocksInbound{
                        
                    }
                }
            }
            _ => {

            }
        };
    }
    todo!()
}