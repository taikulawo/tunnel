use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

use crate::{Config, config::SocksInboundSettings, proxy::socks::SocksInbound};

pub async fn inboundInit(config: Arc<Config>) -> Result<()> {
    let mut inbounds = &config.inbounds;
    let mut handlers = HashMap::new();
    for inbound in inbounds {
        match &*inbound.protocol {
            "socks" => {
                if let Some(ref settings) = inbound.settings {
                    let socks_settings: SocksInboundSettings = serde_json::from_str(settings.get())?;
                    let handler = SocksInbound{};
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