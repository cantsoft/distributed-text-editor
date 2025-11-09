use std::collections::BTreeMap;

use crate::types::{Depth, IdType, PeerId, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub digit: IdType,
    pub peer_id: PeerId,
    pub time: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeCRDT {
    pub data: Option<char>,
    pub children: BTreeMap<Position, Box<NodeCRDT>>,
    pub depth: Depth,
    pub subtree_size: usize,
}

impl NodeCRDT {
    pub fn new(data: char, depth: Depth) -> Self {
        Self {
            depth,
            data: Some(data),
            children: BTreeMap::new(),
            subtree_size: 1,
        }
    }

    pub fn max_digit(depth: Depth) -> IdType {
        1 << (4 + depth)
    }

    pub fn iter<'a>(&'a self) -> NodeCRDTIterator<'a> {
        NodeCRDTIterator::new(self)
    }

    // we don't handle empty chars in tree now
    pub fn insert(&mut self, path: &[Position], data: char) {
        self.subtree_size += 1;
        match path {
            [head] => {
                self.children
                    .insert(*head, Box::new(NodeCRDT::new(data, 1 + self.depth)));
            }
            [head, tail @ ..] => {
                let child = self.children.get_mut(head).unwrap(); // this asummes that path is correct
                child.insert(tail, data);
            }
            [] => panic!("Path cannot be empty"),
        }
    }

    pub fn remove(&mut self, path: &[Position]) {
        match path {
            [head] => {
                let to_remove = self.children.get_mut(head).unwrap();
                if to_remove.children.is_empty() {
                    self.children.remove(head);
                } else {
                    to_remove.data = None;
                }
            }
            [head, tail @ ..] => {
                let child = self.children.get_mut(head).unwrap(); // this asummes that path is correct
                child.subtree_size -= 1;
                child.remove(tail);
            }
            [] => unreachable!(),
        }
    }
}

pub struct NodeCRDTIterator<'a> {
    stack: Vec<&'a NodeCRDT>,
}

impl<'a> NodeCRDTIterator<'a> {
    fn new(root: &'a NodeCRDT) -> Self {
        let mut stack = Vec::new();
        stack.push(root);
        Self { stack }
    }
}

impl<'a> Iterator for NodeCRDTIterator<'a> {
    type Item = &'a NodeCRDT;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            for (_, child) in node.children.iter().rev() {
                self.stack.push(child);
            }
            Some(node)
        } else {
            None
        }
    }
}
