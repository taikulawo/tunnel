use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use tokio::net::{TcpListener, UdpSocket};

use crate::{app::TunnelContext, proxy::{NetworkType, TransportNetwork}};

pub struct InboundListener {
    transport: TransportNetwork,
    ctx: Arc<TunnelContext>
}

impl InboundListener {
    pub async fn listen(self) {
        let TransportNetwork {
            ref addr,
            ref net_type   
        }= self.transport;
        let task = match net_type {
            NetworkType::TCP => {
                self.tcp_listener(&addr)
            },
            NetworkType::UDP => {
                self.udp_listener(&addr)
            }
        };
    }
    async fn tcp_listener(&self, addr: &SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        tokio::spawn(async move {
            for (conn, ..) in listener.accept().await {
                let dispatcher = self.ctx.
            }
        });
        Ok(())
    }
    async fn udp_listener(&self, addr: &SocketAddr) -> Result<()> {
        let listener = UdpSocket::bind(addr).await?;
        // todo!("udp listener")
        Ok(())
    }
}