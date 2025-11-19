use crate::types::{PeerIdType, TimestampType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Side {
    pub(crate) peer_id: PeerIdType,
    time: TimestampType,
}

impl Side {
    pub(crate) fn new(peer_id: PeerIdType) -> Self {
        Self { peer_id, time: 0 }
    }

    pub(crate) fn time_inc(&mut self) -> TimestampType {
        let ret = self.time;
        self.time += 1;
        ret
    }
}
