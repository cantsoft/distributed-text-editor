pub type PeerIdType = u8;
pub(super) type DigitType = u32;
pub(super) type TimestampType = u64;
pub(super) const MIN_POSITION_DIGIT: DigitType = 0;
pub(super) const MAX_POSITION_DIGIT: DigitType = u32::MAX;
pub(super) const RESERVED_PEER: PeerIdType = 0;
pub(super) const DEFAULT_BOUNDARY: DigitType = 16;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeKey {
    pub(super) digit: DigitType,
    pub(super) peer_id: PeerIdType,
    pub(super) time: TimestampType,
}

impl NodeKey {
    pub(super) fn new(digit: DigitType, peer_id: PeerIdType, time: TimestampType) -> Self {
        Self {
            digit: digit,
            peer_id: peer_id,
            time: time,
        }
    }
}
