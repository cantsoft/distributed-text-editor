use crate::types::{PeerId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Side {
    pub peer_id: PeerId, // unique id for each peer
    pub time: Timestamp, // logical clock
}

impl Side {
    pub fn new(peer_id: PeerId) -> Self {
        Self { peer_id, time: 0 }
    }

    // use this to get time and increment logical clock
    pub fn time_inc(&mut self) -> Timestamp {
        let ret = self.time;
        self.time += 1;
        ret
    }
}
