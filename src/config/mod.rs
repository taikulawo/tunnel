use std::{collections::HashMap, fs::{self, File}, io::BufReader, net::SocketAddr, path::Path};
use anyhow::Result;
use serde_json::{Map, value::RawValue};
use serde_derive::{
    Serialize,
    Deserialize,
};
#[derive(Clone,Deserialize)]
pub struct Outbound {
    protocol: String,

    // shadowsocks | socks5
    password: Option<String>,
    method: Option<String>,
}

#[derive(Clone,Deserialize)]
pub struct GeneralSettings {
    pub prefer_ipv6: bool,
    pub use_ipv6: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SocksInboundSettings {
    pub address: String,
    pub method: String,
    pub port: u16,
}
#[derive(Clone, Deserialize)]
pub struct Inbound {
    pub protocol: String,
    pub tag: String,
    // domain or socket addr
    pub settings: Option<Box<RawValue>>
}

#[derive(Clone,Deserialize)]
pub struct Rule {
    ip: Option<Vec<String>>,
    portRange: Option<Vec<String>>,
    domain: Option<Vec<String>>,
    domainSuffix: Option<Vec<String>>,
    domainKeyword: Option<Vec<String>>,
    target: String,
}
#[derive(Clone, Deserialize)]
pub struct Config {
    pub general: GeneralSettings,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
    pub routes: Vec<Rule>,
    pub dns: Option<DnsConfig>,
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

pub fn parse_from_str(p: &str) -> Result<Config>{
    let json = serde_json::from_str(p)?;
    Ok(json)
}

pub fn load_from_file(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    parse_from_str(&*content)
}