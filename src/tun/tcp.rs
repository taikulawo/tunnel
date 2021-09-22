use std::{io, net::{IpAddr, SocketAddr}, sync::{Arc}, time::Duration};

use etherparse::TcpHeader;
use ipnet::{IpNet, Ipv4Net};
use log::error;
use lru_time_cache::LruCache;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

use crate::net::ProxyTcpListener;

pub struct Nat {
    // fake ip to real_src_ip
    mapping: LruCache<(SocketAddr, SocketAddr), SocketAddr>,
    // real_src_ip, real_dest_ip to fake ip
    connections: LruCache<SocketAddr, TcpConnection>
}

impl Nat {
    pub fn new() -> Nat{
        Nat{
            // one day
            mapping: LruCache::with_expiry_duration(Duration::from_secs(60 * 60 * 24)),
            connections: LruCache::with_expiry_duration(Duration::from_secs(60 * 60 * 24)),
        }
    }
}
pub struct TcpTun {
    free_address: Vec<IpAddr>,
    nat: Arc<Mutex<Nat>>,
    listener_addr: SocketAddr,
}

#[derive(Clone, PartialEq, Eq)]
enum State{
    Established,
    FinWait,
    LastAck
}
struct TcpConnection {
    src_addr: SocketAddr,
    dest_addr: SocketAddr,
    fake_addr: SocketAddr,
    state: State
}
impl TcpTun{
    pub async fn new(tun_network: IpNet) -> io::Result<TcpTun>{
        let mut hosts = tun_network.hosts();
        let listener_addr = match hosts.next(){
            Some(addr) => addr,
            None => return Err(io::Error::new(io::ErrorKind::Other, "unexpected listener address allocate failed"))
        };
        let listener = ProxyTcpListener::new(listener_addr, 0).await?;
        let local_addr = listener.local_addr()?;
        let free_src_address = hosts.take(10).collect::<Vec<IpAddr>>();
        let nat = Arc::new(Mutex::new(Nat::new()));
        tokio::spawn(TcpTun::tunnel(listener, nat.clone()));
        Ok(TcpTun {
            free_address: free_src_address,
            nat: nat,
            listener_addr: local_addr,
        })
    }
    pub async fn handle_packet(&self, src_addr: SocketAddr, dest_addr: SocketAddr, tcp_header: &TcpHeader)-> io::Result<Option<(SocketAddr, SocketAddr)>> {
        let Nat {
            ref mut connections,
            ref mut mapping
        } = *self.nat.lock().await;
        let (connection, is_reply) = if tcp_header.syn && !tcp_header.ack {
            // new tcp connection
            let fake_ip = loop {
                let addr_index = rand::random::<usize>() % self.free_address.len();
                // 1024 below are privilege ports
                let port = rand::random::<u16>() % (65535 - 1024) + 1024;
                let fake_addr = SocketAddr::new(self.free_address.get(addr_index).expect("should works").clone(), port);
                if !connections.contains_key(&fake_addr) {
                    // mapping record will be created at first time to establish tcp connection.
                    // so key will always be (original_src_ip, original_dest_ip)
                    mapping.insert((src_addr, dest_addr), fake_addr);
                    connections.insert(fake_addr, TcpConnection {
                        src_addr,
                        dest_addr,
                        fake_addr,
                        state: State::Established
                    });
                    break fake_addr;
                }
            };
            // TcpConnection::get(&*connections, &fake_ip)
            (connections.get_mut(&fake_ip).unwrap(), false)
        }else {
            // existing connections
            match mapping.get(&(src_addr, dest_addr)) {
                Some(fake) => {
                    // isn't reply
                    (connections.get_mut(&fake).unwrap(), false)
                },
                None => {
                    // Does it's a reply packet?
                    match connections.get_mut(&dest_addr) {
                        Some(..) => {
                            // is reply
                            (connections.get_mut(&dest_addr).unwrap(), true)
                        },
                        None => {
                            error!("unknown connection from {} -> {}", src_addr, dest_addr);
                            return Ok(None);
                        }
                    }
                }
            }
        };
        //          nat (fake_ip, listener ip)              SO_BINDTOINTERFACE
        // local ---------------------------------> tun -------------------------> server ---------------------> real remote
        //       <---------------------------------     <-------------------------        <---------------------
        //        (src_ip, dest_ip) nat                    (fake_ip, server ip)
        let (final_src_ip, final_dest_ip) = if is_reply {
            (dest_addr, src_addr)
        }else {
            (connection.fake_addr, self.listener_addr)
        };
        // clean up old connections
        if tcp_header.rst || (tcp_header.ack && connection.state == State::LastAck) {
            mapping.remove(&(connection.src_addr, connection.dest_addr));
            let fake_ip = connection.fake_addr;
            connections.remove(&fake_ip);
        }else if tcp_header.fin {
            // fin, close connection
            // https://users.cs.northwestern.edu/~agupta/cs340/project2/TCPIP_State_Transition_Diagram.pdf
            match connection.state {
                // tcp connection state machine
                State::Established => connection.state = State::FinWait,
                State::FinWait => connection.state = State::LastAck,
                _ => {},
            }
        }
        Ok(Some((final_src_ip, final_dest_ip)))
    }
    async fn tunnel(listener: ProxyTcpListener, translator: Arc<Mutex<Nat>>) {
        loop {
            let (mut stream, remote_addr) = match listener.accept().await {
                Ok(x )=> x,
                Err(err) => {
                    error!("accept error at {}", &err);
                    continue;
                }
            };
            match translator.lock().await.connections.get(&remote_addr) {
                Some(conn) => {
                    tokio::spawn(TcpTun::handle_redir(stream, conn.src_addr, conn.dest_addr));
                },
                None => {
                    error!("unknown connection from tunnel {}", remote_addr);
                    continue;
                }
            };
        }
    }
    // REDIRECT
    // transparent proxy
    async fn handle_redir(mut stream: TcpStream, src_addr: SocketAddr, dest_addr: SocketAddr) {
        // stream is local
        // 
    }
}