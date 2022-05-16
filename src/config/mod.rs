use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::{
    collections::HashMap,
    fs::{self, File},
    net::SocketAddr,
};

// https://v2ray.com/chapter_02/01_overview.html
#[derive(Clone, Deserialize)]
pub struct Config {
    pub general: GeneralSettings,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
    pub routes: Vec<Rule>,
    pub dns: Option<DnsConfig>,
}

#[derive(Clone, Deserialize)]
pub struct Outbound {
    pub protocol: String,
    pub settings: Option<Box<RawValue>>,
    pub tag: String,
}

#[derive(Clone, Deserialize)]
pub struct GeneralSettings {
    pub prefer_ipv6: bool,
    pub use_ipv6: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Socks5InboundSettings {}

#[derive(Clone, Serialize, Deserialize)]
pub struct Socks5OutboundSettings {
    pub address: String,
    pub port: u16
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ShadowsocksInboundSettings {
    pub address: String,
    pub port: u16,
    pub method: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ShadowsocksOutboundSettings {
    pub address: String,
    pub port: u16,
    pub password: String,
    pub method: String,
}

#[derive(Clone, Deserialize)]
pub struct Inbound {
    pub port: u16, 
    pub listen: String,
    pub protocol: String,
    pub tag: String,
    // domain or socket addr
    pub settings: Option<Box<RawValue>>,
}

#[derive(Clone, Deserialize)]
pub struct Rule {
    pub ip: Option<Vec<String>>,
    pub portRange: Option<Vec<String>>,
    pub domain: Option<Vec<String>>,
    pub domainSuffix: Option<Vec<String>>,
    pub domainKeyword: Option<Vec<String>>,
    pub target: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub ip: Option<SocketAddr>,
    pub bind: String,
    pub servers: Option<Vec<String>>,
    pub hosts: Option<HashMap<String, Vec<String>>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: GeneralSettings {
                prefer_ipv6: false,
                use_ipv6: false,
            },
            inbounds: Vec::new(),
            outbounds: Vec::new(),
            routes: Vec::new(),
            dns: None,
        }
    }
}

pub fn parse_from_str(p: &str) -> Result<Config> {
    let json = serde_json::from_str(p)?;
    Ok(json)
}

pub fn load_from_file(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    parse_from_str(&*content)
}
