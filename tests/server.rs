// 两个实例，某个具体协议的 inbound 就一定对应此具体协议
// 所以 local-proxy inbound 只需要socks就行，
// 其他协议的 inbound，通过 local-proxy#outbound => remote-proxy-server#inbound 测试

use std::{net::{SocketAddr, Ipv4Addr, ToSocketAddrs}, str::FromStr};

use futures::{FutureExt, future::BoxFuture};
use tokio::{net::{
    TcpListener,
    UdpSocket
}, runtime::Builder};
use tunnel::start_instance;
pub async fn tcp_echo_server(addr: SocketAddr) {
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async {
                    let (mut read_half,mut write_half) = stream.into_split();
                    tokio::io::copy(&mut read_half, &mut write_half).await
                });
            }
            Err(err) => {
                eprint!("error occurred at listener#accept {}", err)
            }
        }
    }
}
pub async fn udp_echo_server(bind_addr: SocketAddr) {
    let socket = UdpSocket::bind(bind_addr).await.unwrap();
    let mut buf = Vec::new();
    loop {
        let (n, remote_addr) = match socket.recv_from(&mut buf).await {
            Ok(x) => x,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        match socket.send_to(&buf[..n], remote_addr).await {
            Err(err) => {
                eprintln!("{}", err);
            }
            _ => {}
        }
    }
}

pub fn run_two_of_echo_server(bind_addr: SocketAddr) -> Vec<BoxFuture<'static, ()>>{
    let mut tasks = Vec::new();
    let f = tcp_echo_server(bind_addr.clone()).boxed();
    tasks.push(f);
    let f = udp_echo_server(bind_addr.clone()).boxed();
    tasks.push(f);
    tasks
}
pub async fn start_tunnel(config: tunnel::Config) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let mut tasks = start_instance(config).unwrap();
    
    // echo server is the real remote server we want to connected.
    let mut echo_futures = run_two_of_echo_server(SocketAddr::from_str("127.0.0.1:6666").unwrap());
    tasks.append(&mut echo_futures);
    
    // tasks.extend_from_slice();
    let (abort_future, abort_handler) = futures::future::abortable(futures::future::join_all(tasks));
    let test_future = async {
        
        // call abort handler after test completed
        abort_handler.abort();
    };
    rt.block_on(futures::future::join(abort_future, test_future));
}