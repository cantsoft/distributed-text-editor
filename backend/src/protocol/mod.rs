use crate::{state, types};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/dte.rs"));
}
pub use generated::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct PeerBeacon {
    pub id: types::PeerIdType,
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
        id: types::PeerIdType,
        addr: SocketAddr,
    },
    PeerConnected {
        id: types::PeerIdType,
        sender: mpsc::Sender<PeerMessage>,
    },
    PeerDisconnected {
        id: types::PeerIdType,
    },

    User(LocalOperation),

    Network(PeerMessage),
}
