use tokio::net::TcpStream;

// 负责将请求分发给不同的 代理协议 处理
pub struct Dispatcher {}
impl Dispatcher {
    async fn dispatch_tcp(stream: TcpStream, ) {
        // https://github.com/iamwwc/v2ray-core/blob/8cdd680f5ca8d05c618752eb944a42a7b4d31f6c/app/dispatcher/default.go#L207
    }

    async fn dispatch_udp() {}
}
