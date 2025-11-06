use std::collections::BTreeMap;

pub type IdBase = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub digit: IdBase,
    pub peer_id: u8,
    pub time: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeCRDT {
    pub depth: u8,
    pub data: char,
    pub childrens: BTreeMap<Position, Box<NodeCRDT>>,
}

impl NodeCRDT {
    pub fn max_digit(depth: u8) -> IdBase {
        1 << (4 + depth)
    }

    pub fn collect_string(&self) -> String {
        let mut ret = String::new();
        ret.push(self.data);
        for (_, node) in &self.childrens {
            ret += &node.collect_string();
        }
        ret
    }
}
