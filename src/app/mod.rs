use std::sync::Arc;

use crate::{
    config::Config,
};

mod dispatcher;
pub use dispatcher::Dispatcher;

mod dns_client;
pub use dns_client::DnsClient;

mod listener;
pub use listener::InboundListener;


mod inbound;
pub use inbound::InboundManager;

mod outbound;
pub use outbound::OutboundManager;

mod sniffer;
pub use sniffer::Sniffer;

mod router;
pub use router::Router;
