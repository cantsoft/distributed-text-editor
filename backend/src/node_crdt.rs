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
    pub depth: Depth,
    pub data: Option<char>,
    pub children: BTreeMap<Position, Box<NodeCRDT>>,
}

impl NodeCRDT {
    pub fn new(depth: Depth, data: Option<char>) -> Self {
        Self {
            depth,
            data,
            children: BTreeMap::new(),
        }
    }

    pub fn max_digit(depth: Depth) -> IdType {
        1 << (4 + depth)
    }

    pub fn iter<'a>(&'a self) -> NodeCRDTIterator<'a> {
        NodeCRDTIterator::new(self)
    }

    pub fn insert(&mut self, path: &[Position], data: char) {
        match path {
            [head] => {
                self.children
                    .insert(*head, Box::new(NodeCRDT::new(1 + self.depth, Some(data))));
            }
            // we don't handle empty chars in tree now
            [head, tail @ ..] => {
                let child = self.children.get_mut(head).unwrap();
                child.insert(tail, data);
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
