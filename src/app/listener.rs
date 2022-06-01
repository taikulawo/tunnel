use futures_util::{future::BoxFuture, FutureExt, StreamExt};
use log::{debug, error, info};
use std::{io::Result, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, UdpSocket},
    sync::oneshot,
};

use crate::{
    app::udp_association_manager::UdpPacket,
    proxy::{
        Address, AnyInboundHandler, InboundResult, Network, Session, TcpInboundHandlerTrait,
        UdpInboundHandlerTrait,
    },
};

use super::{dispatcher::Dispatcher, UdpAssociationManager};

pub struct InboundListener {}
type TaskFuture = BoxFuture<'static, ()>;
impl InboundListener {
    pub fn listen(
        dispatcher: Arc<Dispatcher>,
        handler: AnyInboundHandler,
        addr: SocketAddr,
        nat: Arc<UdpAssociationManager>,
    ) -> Result<Vec<TaskFuture>> {
        let mut tasks: Vec<TaskFuture> = vec![];
        if handler.has_tcp() {
            // 最初是 self.tcp_listener 写法，但会报
            // `self` does not live long enough, borrowed value does not live long enough. rustcE0597
            // 是因为 boxed() 返回 'static，tcp_listener 内部临时借用 self 来获取 dispatcher，从而产生对 self 的间接依赖。
            // 而 self 却不是 'static
            // 解决办法两种
            // 1. 将 tcp_listener 代码移动到 listen，listen 开头 clone dispatcher，也就没有 tcp_listener, udp_listener
            // 2. tcp_listener 不能依赖self，listen调用 dispatcher.clone() 后将 cloned dispatcher 传给 tcp_listener
            // 这就要求 tcp_listener 改为 InboundListener
            // 实在不想在 listen 糅合一堆代码，我在这里采用 2
            let f = InboundListener::tcp_listener(handler.clone(), dispatcher.clone(), addr);
            tasks.push(f);
        }
        if handler.has_udp() {
            let f = InboundListener::udp_listener(handler.clone(), dispatcher.clone(), addr, nat);
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
            loop {
                match listener.accept().await {
                    Ok((conn, _)) => {
                        let dispatcher = Arc::clone(&dispatcher);
                        let handler = handler.clone();
                        tokio::spawn(async move {
                            let addr = conn.peer_addr().expect("peer");
                            let local = conn.local_addr().expect("local");
                            let session = Session {
                                destination: Address::Ip(addr),
                                network: Network::TCP,
                                local_peer: local,
                                peer_address: conn.peer_addr().expect("peer"),
                            };
                            match TcpInboundHandlerTrait::handle(&*handler, session, conn).await {
                                Ok(InboundResult::Stream(stream, mut sess)) => {
                                    dispatcher.dispatch_tcp(stream, &mut sess).await;
                                }
                                Ok(InboundResult::Datagram(socket)) => {
                                    // dispatcher.dispatch_udp(socket, sess).await;
                                }
                                Ok(InboundResult::NotSupported) => {
                                    error!("not supported");
                                }
                                Err(err) => {
                                    error!("handle tcp inbound failed err {}", err);
                                }
                            }
                        });
                    }
                    Err(err) => {
                        error!("accept error {}", err);
                        return;
                    }
                }
            }
        }
        .boxed();
        task
    }
    fn udp_listener(
        handler: AnyInboundHandler,
        dispatcher: Arc<Dispatcher>,
        addr: SocketAddr,
        nat: Arc<UdpAssociationManager>,
    ) -> TaskFuture {
        let future = async move {
            let socket = UdpSocket::bind(addr).await.unwrap();
            info!("Udp listening at {}", addr);
            match UdpInboundHandlerTrait::handle(handler.as_ref(), socket).await {
                Ok(res) => {
                    match res {
                        InboundResult::Datagram(send_recv_socket) => {
                            tokio::spawn(async move {
                                let mut buf = vec![0u8; 1024];
                                let (source_addr, real_addr) =
                                    match send_recv_socket.recv_from(&mut buf).await {
                                        Ok(x) => x,
                                        Err(err) => {
                                            debug!("{}", err);
                                            return;
                                        }
                                    };
                                let (mut sender, mut receiver) =
                                    tokio::sync::mpsc::channel::<Vec<u8>>(10);
                                let send_recv_socket1 = send_recv_socket.clone();
                                tokio::spawn(async move {
                                    loop {
                                        let res = match receiver.recv().await {
                                            Some(x) => x,
                                            None => {
                                                // closed
                                                debug!("{} closed channel", &source_addr);
                                                return;
                                            }
                                        };
                                        match send_recv_socket1
                                            .send_to(res.as_ref(), source_addr.clone())
                                            .await
                                        {
                                            Ok(_) => {}
                                            Err(err) => {
                                                debug!("{}", err);
                                                return;
                                            }
                                        }
                                    }
                                });
                                let packet = UdpPacket {
                                    data: buf,
                                    dest: real_addr.clone(),
                                };
                                match nat
                                    .send_packet(real_addr, source_addr, packet, sender)
                                    .await
                                {
                                    Ok(_) => {}
                                    Err(err) => {
                                        debug!("{}", err);
                                    }
                                }
                            });
                        }
                        InboundResult::Stream(stream, sess) => {}
                        InboundResult::NotSupported => {}
                    }
                }
                Err(err) => {}
            }
        }
        .boxed();
        future
    }
}
