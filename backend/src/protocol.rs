use crate::state::NodeKey;
use crate::types::PeerId;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/dte.rs"));
}
pub use generated::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct PeerBeacon {
    pub id: PeerId,
    pub tcp_port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RemoteOp {
    RemoteInsert { id: Vec<NodeKey>, value: char },

    RemoteRemove { char_id: Vec<NodeKey> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerMessage {
    SyncOp(RemoteOp),
}

pub enum NodeEvent {
    PeerDiscovered {
        id: PeerId,
        addr: SocketAddr,
    },
    PeerConnection {
        stream: tokio::net::TcpStream,
    },
    PeerConnected {
        id: PeerId,
        sender: mpsc::Sender<PeerMessage>,
    },
    PeerDisconnected {
        id: PeerId,
    },

    Local(LocalCommand),

    Network(PeerMessage),
}
