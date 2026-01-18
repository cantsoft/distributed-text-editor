use crate::types::{PeerIdType, TimestampType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Side {
    peer_id: PeerIdType,
    time: TimestampType,
}

impl Side {
    pub fn new(peer_id: PeerIdType) -> Self {
        Self { peer_id, time: 0 }
    }

    pub fn peer_id(&self) -> PeerIdType {
        self.peer_id
    }

    pub fn time_inc(&mut self) -> TimestampType {
        let ret = self.time;
        self.time += 1;
        ret
    }
}
