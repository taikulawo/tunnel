use futures::FutureExt;
use tokio::{runtime::Runtime, net::UdpSocket};

#[test]
pub fn udp_server() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let server_listening_at = "127.0.0.1:8088";
    let mut tasks = Vec::new();
    let udp_server_task = async {
        let socket = UdpSocket::bind(server_listening_at).await.unwrap();
        let mut buf = vec![0u8; 1024];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((n, original_address)) => {
                    let v = &buf[..n];
                    println!("recv from {}, message {:?}", original_address, String::from_utf8_lossy(v));
                },
                Err(err) => {
                    eprintln!("udp server err {}", err);
                    continue;
                }
            }
        }
    }.boxed();
    tasks.push(udp_server_task);
    let udp_client_task = async {
        let socket = UdpSocket::bind("127.0.0.1:8089").await.unwrap();
        let message = "hello,world".as_bytes();
        match socket.send_to(&message, server_listening_at).await {
            Ok((n)) => {
                println!("{} bytes sent", n);
            }
            Err(err) => {
                eprintln!("udp send to {} err{}", server_listening_at, err);
            }
        }
    }.boxed();
    tasks.push(udp_client_task);
    let runner = futures::future::join_all(tasks);
    let abort_handler = async {
        tokio::signal::ctrl_c().await.unwrap();
        println!("ctrl c");
    }.boxed();
    rt.block_on(futures::future::select(runner, abort_handler));
}