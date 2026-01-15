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
        id: Vec<state::NodeKey>,
        value: char,
    },

    RemoteRemove {
        char_id: Vec<state::NodeKey>,
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
    PeerConnected {
        id: PeerIdType,
        sender: mpsc::Sender<PeerMessage>,
    },
    PeerDisconnected {
        id: PeerIdType,
    },

    User(LocalOperation),

    Network(PeerMessage),
}
