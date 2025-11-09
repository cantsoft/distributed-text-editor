use crate::node_crdt::{NodeCRDT, Position};
use crate::types::{MAX_POSITION_DIGIT, MIN_POSITION_DIGIT};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct TreeCRDT {
    pub root: NodeCRDT,
}

impl Default for TreeCRDT {
    fn default() -> Self {
        let mut new = Self {
            root: NodeCRDT {
                data: None,
                children: BTreeMap::new(),
                depth: 0,
                subtree_size: 0,
            },
        };
        new.root.children.insert(
            Position {
                digit: MIN_POSITION_DIGIT,
                peer_id: std::u8::MAX,
                time: 0,
            },
            Box::new(NodeCRDT {
                data: None,
                children: BTreeMap::new(),
                depth: 1,
                subtree_size: 0,
            }),
        );
        new.root.children.insert(
            Position {
                digit: MAX_POSITION_DIGIT,
                peer_id: std::u8::MAX,
                time: 0,
            },
            Box::new(NodeCRDT {
                data: None,
                children: BTreeMap::new(),
                depth: 1,
                subtree_size: 0,
            }),
        );
        new
    }
}

impl TreeCRDT {
    pub fn bos_path(&self) -> Arc<[Position]> {
        Arc::from([Position {
            digit: MIN_POSITION_DIGIT,
            peer_id: 0,
            time: 0,
        }])
    }

    pub fn eos_path(&self) -> Arc<[Position]> {
        Arc::from([Position {
            digit: MAX_POSITION_DIGIT,
            peer_id: 0,
            time: 0,
        }])
    }

    // assumse path is valid and not exists yet
    pub fn insert(&mut self, path: &[Position], data: char) {
        self.root.insert(path, data);
    }

    // assumse path is valid and exists yet
    pub fn remove(&mut self, path: &[Position]) {
        self.root.remove(path);
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
