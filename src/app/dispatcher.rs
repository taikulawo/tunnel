use tokio::net::TcpStream;

// 负责将请求分发给不同的 代理协议 处理
pub struct Dispatcher {}
impl Dispatcher {
    async fn dispatch_tcp(stream: TcpStream, ) {
        
    }

    async fn dispatch_udp() {}
}
