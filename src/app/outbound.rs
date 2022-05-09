use crate::config::Outbound;

// 管理全部的传出协议 outbound
pub struct OutboundManager {

}

impl OutboundManager {
    fn new(outbounds: Vec<Outbound>) -> OutboundManager {
        for outbound in outbounds.iter() {
            let handler = match &*outbound.protocol {
                "socks" => {
                    
                },
                "shadowsocks" => {

                },
                _ => {

                }
            };
            
        }
        OutboundManager {  }
    }
}