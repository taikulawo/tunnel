mod server;
#[test]
fn start() {
    let local = r#"
    {
        "general":{
            "prefer_ipv6": false,
            "use_ipv6": false
        },
        "api": {
            "address": "127.0.0.1",
            "port": 9991
        },
        "log": {
            "level": "trace",
            "output": "leaf.log"
        },
        "dns": {
            "bind": "192.168.50.3",
            "servers": [
                "8.8.8.8:53",
                "8.8.4.4:53"
            ],
            "hosts": {
                "example.com": [
                    "192.168.0.1",
                    "192.168.0.2"
                ]
            }
        },
        "inbounds": [
            {
                "port": 1080,
                "listen":"127.0.0.1",
                "protocol": "socks",
                "settings": {},
                "tag": "socks_in"
            }
        ],
        "outbounds": [
            {
                "protocol": "socks",
                "settings": {
                    "address": "127.0.0.1",
                    "port": 1081
                },
                "tag": "socks_out"
            }
        ],
        "routes": [
            {
                "ip": [
                    "8.8.8.8/8",
                    "8.8.4.4/8"
                ],
                "target": "socks_out"
            },
            {
                "domain": [
                    "www.google.com"
                ],
                "target": "socks_out"
            },
            {
                "regexp": [
                    ".*"
                ],
                "target": "socks_out"
            }
        ]
    }
    "#;
    let server = r#"
    {
        "general": {
            "prefer_ipv6": false,
            "use_ipv6": false
        },
        "api": {
            "address": "127.0.0.1",
            "port": 9991
        },
        "log": {
            "level": "trace",
            "output": "leaf.log"
        },
        "dns": {
            "bind": "192.168.50.3",
            "servers": [
                "8.8.8.8:53",
                "8.8.4.4:53"
            ],
            "hosts": {
                "example.com": [
                    "192.168.0.1",
                    "192.168.0.2"
                ]
            }
        },
        "inbounds": [
            {
                "port": 1081,
                "listen": "127.0.0.1",
                "protocol": "socks",
                "settings": {},
                "tag": "socks_in"
            }
        ],
        "outbounds": [
            {
                "protocol": "direct",
                "tag": "direct_out"
            }
        ],
        "routes": [
            {
                "regexp": [
                    ".*"
                ],
                "target": "direct_out"
            }
        ]
    }"#;
    let mut configs = Vec::new();
    for config in vec![local, server] {
        let c = serde_json::from_str(config).unwrap();
        configs.push(c);
    }
    server::start_tunnel(configs, "127.0.0.1:3002","127.0.0.1:1080");
}