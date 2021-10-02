pub struct AppConfig {
    pub prefer_ipv6: bool,
    pub use_ipv6: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            prefer_ipv6: false,
            use_ipv6: false
        }
    }
}