use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use futures_util::{future::BoxFuture, FutureExt};
use tokio::net::{TcpListener, UdpSocket};

use crate::{
    app::Context,
    proxy::{NetworkType, TransportNetwork},
};

pub struct InboundListener {
    transport: TransportNetwork,
    ctx: Arc<Context>,
}
type TaskFuture = BoxFuture<'static, Result<()>>;
impl InboundListener {
    pub async fn listen(self) -> Result<TaskFuture> {
        let TransportNetwork {
            ref addr,
            ref net_type,
        } = self.transport;
        let addr = addr.clone();
        let task = match net_type {
            NetworkType::TCP => self.tcp_listener(addr).boxed(),
            NetworkType::UDP => self.udp_listener(addr).boxed(),
        };
        Ok(task)
    }
    async fn tcp_listener(&self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        let dispatcher = Arc::clone(&self.ctx.dispatcher);
        tokio::spawn(async move {
            for (conn, ..) in listener.accept().await {
                let dispatcher = Arc::clone(&dispatcher);
            }
        });
        Ok(())
    }
    async fn udp_listener(&self, addr: SocketAddr) -> Result<()> {
        let listener = UdpSocket::bind(addr).await?;
        // todo!("udp listener")
        Ok(())
    }
}
