use crate::types::{PeerId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Side {
    pub(crate) peer_id: PeerId,
    pub(crate) time: Timestamp,
}

impl Side {
    pub(crate) fn new(peer_id: PeerId) -> Self {
        Self { peer_id, time: 0 }
    }

    pub(crate) fn time_inc(&mut self) -> Timestamp {
        let ret = self.time;
        self.time += 1;
        ret
    }
}
