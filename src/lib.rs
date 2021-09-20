async fn run_tun() {
    let config = tun::Configuration::default();
    tun::create_as_async(&config);
}