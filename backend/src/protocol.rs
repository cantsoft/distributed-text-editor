use crate::state::{Doc, NodeKey};
use crate::types::PeerId;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/dte.rs"));
}
pub use generated::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerBeacon {
    pub id: PeerId,
    pub tcp_port: u16,
}

pub enum NodeEvent {
    Net(PeerEvent),

    Local(ClientCommand),

    Sync(PeerSyncOp),
}

pub enum PeerEvent {
    Discovered {
        id: PeerId,
        addr: SocketAddr,
    },
    Connection {
        stream: tokio::net::TcpStream,
    },
    Connected {
        id: PeerId,
        sender: mpsc::Sender<PeerSyncOp>,
    },
    Disconnected {
        id: PeerId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerSyncOp {
    Insert { char_id: Vec<NodeKey>, value: u8 },

    Remove { char_id: Vec<NodeKey> },

    FullSync { state: Doc },
}
