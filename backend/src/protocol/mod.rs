use crate::state;
use crate::state::PeerIdType;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/dte.rs"));
}
pub use generated::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct PeerBeacon {
    pub id: PeerIdType,
    pub tcp_port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RemoteOp {
    RemoteInsert {
        key: Vec<state::NodeKey>,
        value: char,
    },

    RemoteDelete {
        key: Vec<state::NodeKey>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerMessage {
    SyncOp(RemoteOp),
}

pub enum NodeEvent {
    PeerDiscovered {
        id: PeerIdType,
        addr: SocketAddr,
    },

    User(LocalOperation),

    PeerConnected {
        id: PeerIdType,
        sender: mpsc::Sender<PeerMessage>,
    },
    PeerDisconnected {
        id: PeerIdType,
    },

    Network {
        from: PeerIdType,
        payload: PeerMessage,
    },
}
