use std::net::SocketAddr;

#[derive(Clone)]
pub struct Proxy {
    protocol: String,

    // shadowsocks | socks5
    password: Option<String>,
    method: Option<String>,
}

#[derive(Clone)]
pub struct GeneralSettings {
    pub prefer_ipv6: bool,
    pub use_ipv6: bool,
    pub dns: Vec<DnsConfig>,
}

#[derive(Clone)]
pub struct Route {}
#[derive(Clone)]
pub struct AppConfig {
    pub general: GeneralSettings,
    pub proxys: Vec<Proxy>,
    pub routes: Vec<Route>,
}

#[derive(Clone)]
pub struct DnsConfig {
    pub ip: SocketAddr,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            general: GeneralSettings {
                prefer_ipv6: false,
                use_ipv6: false,
                dns: Vec::new(),
            },
            proxys: Vec::new(),
            routes: Vec::new(),
            ..Default::default()
        }
    }
}
