fn main() {
    
    let config = tun::Configuration::default();
    tun::create_as_async(&config);
}