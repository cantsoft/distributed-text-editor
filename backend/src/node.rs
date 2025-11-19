use crate::types::{IdType, PeerId, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeKey {
    pub digit: IdType,
    pub peer_id: PeerId,
    pub time: Timestamp,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeKind {
    Char(char),
    Bos,
    Eos,
}
