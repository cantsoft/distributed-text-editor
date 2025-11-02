use super::node_crdt::{NodeCRDT, Position};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct TreeCRDT {
    root_childrens: BTreeMap<Position, Box<NodeCRDT>>,
}

impl Default for TreeCRDT {
    fn default() -> Self {
        let mut new = Self {
            root_childrens: BTreeMap::new(),
        };
        new.root_childrens.insert(
            Position {
                digit: 0,
                peer_id: 0,
                time: 0,
            },
            Box::new(NodeCRDT {
                depth: 1,
                data: '\0',
                childrens: BTreeMap::new(),
            }),
        );
        new.root_childrens.insert(
            Position {
                digit: std::u32::MAX,
                peer_id: 0,
                time: 0,
            },
            Box::new(NodeCRDT {
                depth: 1,
                data: '\0',
                childrens: BTreeMap::new(),
            }),
        );
        new
    }
}

impl TreeCRDT {
    pub fn insert(&mut self, path: &Vec<Position>, data: char) {
        let mut it = &mut self.root_childrens;
        for key in &path[..path.len().saturating_sub(1)] {
            if !it.contains_key(key) {
                break;
            }
            let node = it.get_mut(key).unwrap();
            it = &mut node.childrens;
        }
        it.insert(
            path.last().unwrap().clone(),
            Box::new(NodeCRDT {
                depth: path.len() as u8,
                data: data,
                childrens: BTreeMap::new(),
            }),
        );
    }
    pub fn collect_string(&self) -> String {
        let mut ret = String::new();
        for (_, node) in &self.root_childrens {
            ret += &node.collect_string();
        }
        ret
    }
}
