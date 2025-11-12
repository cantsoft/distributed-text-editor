use crate::node::{Node, NodeKey, NodeKind};
use crate::types::{MAX_POSITION_DIGIT, MIN_POSITION_DIGIT};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct Tree {
    pub root: Node,
}

impl Default for Tree {
    fn default() -> Self {
        let mut new = Self {
            root: Node {
                kind: NodeKind::Root,
                children: BTreeMap::new(),
                subtree_size: 0,
            },
        };
        new.root.children.insert(
            NodeKey {
                digit: MIN_POSITION_DIGIT,
                peer_id: std::u8::MAX,
                time: 0,
            },
            Box::new(Node {
                kind: NodeKind::Bos,
                children: BTreeMap::new(),
                subtree_size: 0,
            }),
        );
        new.root.children.insert(
            NodeKey {
                digit: MAX_POSITION_DIGIT,
                peer_id: std::u8::MAX,
                time: 0,
            },
            Box::new(Node {
                kind: NodeKind::Eos,
                children: BTreeMap::new(),
                subtree_size: 0,
            }),
        );
        new
    }
}

impl Tree {
    pub fn bos_path(&self) -> Arc<[NodeKey]> {
        Arc::from([NodeKey {
            digit: MIN_POSITION_DIGIT,
            peer_id: 0,
            time: 0,
        }])
    }

    pub fn eos_path(&self) -> Arc<[NodeKey]> {
        Arc::from([NodeKey {
            digit: MAX_POSITION_DIGIT,
            peer_id: 0,
            time: 0,
        }])
    }

    // assumse path is valid and not exists yet
    pub fn insert(&mut self, path: &[NodeKey], data: char) {
        self.root.insert(path, data);
    }

    // assumse path is valid and exists yet
    pub fn remove(&mut self, path: &[NodeKey]) {
        self.root.remove(path);
    }

    pub fn collect_string(&self) -> String {
        self.root
            .children
            .iter()
            .flat_map(|(_, node)| node.iter())
            .filter_map(|node| match node.kind {
                NodeKind::Char(ch) => Some(ch),
                _ => None,
            })
            .collect()
    }
}
