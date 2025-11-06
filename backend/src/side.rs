#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Side {
    pub peer_id: u8, // unique id for each peer
    pub time: u64,   // logical clock
}

impl Side {
    pub fn new(peer_id: u8) -> Self {
        Self { peer_id, time: 0 }
    }

    // use this to get time and increment logical clock
    pub fn time_inc(&mut self) -> u64 {
        let ret = self.time;
        self.time += 1;
        ret
    }
}
