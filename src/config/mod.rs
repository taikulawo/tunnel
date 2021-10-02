use std::default::default;

pub struct AppConfig {
    prefer_ipv6: bool,
    use_ipv6: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            prefer_ipv6: false,
            use_ipv6: false
        }
    }
}