#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Side {
    pub peer_id: u8,
    pub time: u64,
}

impl Side {
    pub fn new(peer_id: u8) -> Self {
        Self { peer_id, time: 0 }
    }

    pub fn time_inc(&mut self) -> u64 {
        let ret = self.time;
        self.time += 1;
        ret
    }
}
