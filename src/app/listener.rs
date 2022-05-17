use futures_util::{future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt};
use log::{error, info};
use std::{io::Result, net::SocketAddr, sync::Arc, process::Output, future::Future};
use tokio::{
    net::{TcpListener, UdpSocket},
    sync::futures,
};

use crate::{
    config,
    proxy::{
        Address, AnyInboundHandler, InboundHandler, InboundResult, Network, NetworkType, Session,
        TcpInboundHandlerTrait, TransportNetwork,
    },
};

use super::dispatcher::Dispatcher;

pub struct InboundListener {}
type TaskFuture = BoxFuture<'static, ()>;
impl InboundListener {
    pub fn listen(
        dispatcher: Arc<Dispatcher>,
        handler: AnyInboundHandler,
        addr: SocketAddr,
    ) -> Result<Vec<TaskFuture>> {
        let mut tasks: Vec<TaskFuture> = vec![];
        if (handler.has_tcp()) {
            // 最初是 self.tcp_listener 写法，但会报
            // `self` does not live long enough, borrowed value does not live long enough. rustcE0597
            // 是因为 boxed() 返回 'static，tcp_listener 内部临时借用 self 来获取 dispatcher，从而产生对 self 的间接依赖。
            // 而 self 却不是 'static
            // 解决办法两种
            // 1. 将 tcp_listener 代码移动到 listen，listen 开头 clone dispatcher，也就没有 tcp_listener, udp_listener
            // 2. tcp_listener 不能依赖self，listen调用 dispatcher.clone() 后将 cloned dispatcher 传给 tcp_listener
            // 这就要求 tcp_listener 改为 InboundListener
            // 实在不想在 listen 糅合一堆代码，我在这里采用 2
            let f =
                InboundListener::tcp_listener(handler.clone(), dispatcher.clone(), addr);
            tasks.push(f);
        }
        if (handler.has_udp()) {
            let f =
                InboundListener::udp_listener(handler.clone(), dispatcher.clone(), addr);
            tasks.push(f);
        }
        Ok(tasks)
    }
    fn tcp_listener(
        handler: AnyInboundHandler,
        dispatcher: Arc<Dispatcher>,
        addr: SocketAddr,
    ) -> TaskFuture {
        let task = async move {
            let listener = TcpListener::bind(addr).await.unwrap();
            info!("Tcp listening at {}", addr);
            for (conn, ..) in listener.accept().await {
                let dispatcher = Arc::clone(&dispatcher);
                let addr = conn.peer_addr().expect("peer");
                let local = conn.local_addr().expect("local");
                let session = Session {
                    destination: Address::Ip(addr),
                    network: Network::TCP,
                    local_peer: local,
                };
                match TcpInboundHandlerTrait::handle(&*handler, session, conn).await {
                    Ok(InboundResult::Stream(stream, mut sess)) => {
                        dispatcher.dispatch_tcp(stream, &mut sess).await;
                    }
                    Ok(InboundResult::Datagram(socket, sess)) => {
                        dispatcher.dispatch_udp(socket, sess).await;
                    }
                    Err(err) => {
                        error!("handle tcp inbound failed{}", err);
                    }
                }
            }
        }.boxed();
        task
    }
    fn udp_listener(
        handler: AnyInboundHandler,
        dispatcher: Arc<Dispatcher>,
        addr: SocketAddr,
    ) -> TaskFuture {
        let future = async move {
            let listener = UdpSocket::bind(addr).await.unwrap();
            info!("Udp listen at {}", addr);
            ()
        }.boxed();
        // todo!("udp listener")
        future
    }
}
