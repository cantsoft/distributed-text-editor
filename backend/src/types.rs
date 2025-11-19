pub(crate) type DigitType = u32;
pub(crate) type PeerIdType = u8;
pub(crate) type TimestampType = u64;
pub(crate) const MIN_POSITION_DIGIT: DigitType = 0;
pub(crate) const MAX_POSITION_DIGIT: DigitType = u32::MAX;
pub(crate) const RESERVED_PEER: PeerIdType = 0;
pub(crate) const DEFAULT_BOUNDARY: DigitType = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NodeKey {
    pub(crate) digit: DigitType,
    pub(crate) peer_id: PeerIdType,
    pub(crate) time: TimestampType,
}

impl NodeKey {
    pub(crate) fn new(digit: DigitType, peer_id: PeerIdType, time: TimestampType) -> Self {
        Self {
            digit: digit,
            peer_id: peer_id,
            time: time,
        }
    }
}
