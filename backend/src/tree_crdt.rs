use crate::types::{MAX_POSITION_DIGIT, MIN_POSITION_DIGIT};

use crate::node_crdt::{NodeCRDT, Position};

#[derive(Debug)]
pub struct TreeCRDT {
    pub root: NodeCRDT,
}

impl Default for TreeCRDT {
    fn default() -> Self {
        let mut new = Self {
            root: NodeCRDT::new(0, None),
        };
        new.root.children.insert(
            Position {
                digit: MIN_POSITION_DIGIT,
                peer_id: 0,
                time: 0,
            },
            Box::new(NodeCRDT::new(1, None)),
        );
        new.root.children.insert(
            Position {
                digit: MAX_POSITION_DIGIT,
                peer_id: 0,
                time: 0,
            },
            Box::new(NodeCRDT::new(1, None)),
        );
        new
    }
}

impl TreeCRDT {
    pub fn bos_path(&self) -> Vec<Position> {
        vec![Position {
            digit: MIN_POSITION_DIGIT,
            peer_id: 0,
            time: 0,
        }]
    }

    pub fn eos_path(&self) -> Vec<Position> {
        vec![Position {
            digit: MAX_POSITION_DIGIT,
            peer_id: 0,
            time: 0,
        }]
    }

    // assumse path is valid and not exists yet
    pub fn insert(&mut self, path: &Vec<Position>, data: char) {
        self.root.insert(path, data);
    }

    pub fn collect_string(&self) -> String {
        self.root
            .children
            .iter()
            .flat_map(|(_, node)| node.iter())
            .filter_map(|node| node.data)
            .collect()
    }
}
