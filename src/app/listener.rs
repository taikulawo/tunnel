use std::{io::Result, net::SocketAddr, sync::Arc};

use futures_util::{future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt};
use tokio::{
    net::{TcpListener, UdpSocket},
    sync::futures,
};

use crate::{
    app::Context,
    config,
    proxy::{AnyInboundHandler, NetworkType, TransportNetwork},
};

use super::dispatcher::Dispatcher;

pub struct InboundListener {
    transport: TransportNetwork,
    ctx: Arc<Context>,
}
type TaskFuture = BoxFuture<'static, Result<()>>;
impl InboundListener {
    pub async fn listen(
        self,
        handle: AnyInboundHandler,
        addr: SocketAddr,
    ) -> Result<Vec<TaskFuture>> {
        let mut tasks: Vec<TaskFuture> = vec![];
        let dispatcher = self.ctx.dispatcher.clone();
        if (handle.has_tcp()) {
            // 最初是 self.tcp_listener 写法，但会报
            // `self` does not live long enough, borrowed value does not live long enough. rustcE0597
            // 是因为 boxed() 返回 'static，tcp_listener 内部临时借用 self 来获取 dispatcher，从而产生对 self 的间接依赖。
            // 而 self 却不是 'static
            // 解决办法两种
            // 1. 将 tcp_listener 代码移动到 listen，listen 开头 clone dispatcher，也就没有 tcp_listener, udp_listener
            // 2. tcp_listener 不能依赖self，listen调用 dispatcher.clone() 后将 cloned dispatcher 传给 tcp_listener
            // 这就要求 tcp_listener 改为 InboundListener
            // 实在不想在 listen 糅合一堆代码，我在这里采用 2
            let f = InboundListener::tcp_listener(dispatcher.clone(), addr).boxed();
            tasks.push(f);
        }
        if (handle.has_udp()) {
            let f = InboundListener::udp_listener(dispatcher.clone(), addr).boxed();
            tasks.push(f);
        }
        Ok(tasks)
    }
    async fn tcp_listener(dispatcher: Arc<Dispatcher>, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        tokio::spawn(async move {
            for (conn, ..) in listener.accept().await {
                let dispatcher = Arc::clone(&dispatcher);
            }
        });
        Ok(())
    }
    async fn udp_listener(dispatcher: Arc<Dispatcher>, addr: SocketAddr) -> Result<()> {
        let listener = UdpSocket::bind(addr).await?;
        // todo!("udp listener")
        Ok(())
    }
}
