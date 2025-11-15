use crate::types::{PeerId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Side {
    pub peer_id: PeerId,
    pub time: Timestamp,
}

impl Side {
    pub fn new(peer_id: PeerId) -> Self {
        Self { peer_id, time: 0 }
    }

    pub fn time_inc(&mut self) -> Timestamp {
        let ret = self.time;
        self.time += 1;
        ret
    }
}
