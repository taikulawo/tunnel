use std::{collections::HashMap, io, net::SocketAddr, sync::Arc};

use log::{debug, error};
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};

use crate::proxy::{Address, Network, Session};

use super::Dispatcher;

pub struct UdpPacket {
    pub data: Vec<u8>,
    pub dest: Address,
}
type SyncMap = Arc<Mutex<HashMap<SocketAddr, (Sender<UdpPacket>)>>>;
pub struct UdpAssociationManager {
    store: SyncMap,
    dispatcher: Arc<Dispatcher>,
}

impl UdpAssociationManager {
    pub fn new(dispatcher: Arc<Dispatcher>) -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            dispatcher,
        }
    }

    pub async fn send_packet(
        &self,
        dest: Address,
        source_addr: SocketAddr,
        data: UdpPacket,
        local_sender: Sender<Vec<u8>>,
    ) -> anyhow::Result<()> {
        let map = self.store.lock().await;
        if map.contains_key(&source_addr) {
            self.do_send(&source_addr, data).await;
            return Ok(());
        }
        let sess = Session {
            destination: dest,
            network: Network::UDP,
            peer_address: source_addr,
            ..Default::default()
        };
        self.add_association(sess, local_sender).await?;
        self.do_send(&source_addr, data).await;
        Ok(())
    }
    async fn add_association(
        &self,
        sess: Session,
        local_sender: Sender<Vec<u8>>,
    ) -> anyhow::Result<()> {
        let (mut remote_socket_sender, mut remote_socket_receiver) =
            mpsc::channel::<UdpPacket>(100);
        let socket = self.dispatcher.dispatch_udp(sess.clone()).await?;
        let socket1 = socket.clone();
        tokio::spawn(async move {
            loop {
                match remote_socket_receiver.recv().await {
                    Some(msg) => {
                        let UdpPacket { data, dest } = msg;
                        match socket1.send_to(data.as_ref(), dest).await {
                            Ok(res) => {}
                            Err(err) => {
                                debug!("{}", err)
                            }
                        }
                    }
                    None => {
                        debug!("received none, closed");
                        //closed
                        return;
                    }
                };
            }
        });
        let socket2 = socket.clone();
        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];
            loop {
                match socket2.recv_from(&mut buf).await {
                    Ok(x) => match local_sender.try_send(buf.clone()) {
                        Ok(_) => {}
                        Err(err) => {
                            debug!("{}", err);
                            return;
                        }
                    },
                    Err(err) => {}
                }
            }
        });
        let mut map = self.store.lock().await;
        map.insert(sess.peer_address, remote_socket_sender);
        Ok(())
    }
    async fn do_send(&self, source_addr: &SocketAddr, packet: UdpPacket) {
        let map = self.store.lock().await;
        let sender = match map.get(source_addr) {
            Some(x) => x,
            None => {
                error!("no sender for {} found", &source_addr);
                return;
            }
        };
        let sender = sender.clone();
        tokio::spawn(async move {
            match sender.try_send(packet) {
                Ok(_) => {}
                Err(err) => {
                    debug!("send to remote should success but failed with err {}", err);
                    return;
                }
            };
        });
    }
}
