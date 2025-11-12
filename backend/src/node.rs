use std::collections::BTreeMap;

use crate::types::{Depth, IdType, PeerId, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeKey {
    pub digit: IdType,
    pub peer_id: PeerId,
    pub time: Timestamp,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeKind {
    Root,
    Char(char),
    Bos,
    Eos,
    Empty,
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub children: BTreeMap<NodeKey, Box<Node>>,
    pub subtree_size: usize,
}

impl Node {
    pub fn new(data: char) -> Self {
        Self {
            kind: NodeKind::Char(data),
            children: BTreeMap::new(),
            subtree_size: 1,
        }
    }

    pub fn max_digit(depth: Depth) -> IdType {
        1 << (4 + depth)
    }

    pub fn iter<'a>(&'a self) -> PreOrderIterator<'a> {
        PreOrderIterator::new(self)
    }

    // we don't handle empty chars in tree now
    pub fn insert(&mut self, path: &[NodeKey], data: char) {
        self.subtree_size += 1;
        match path {
            [head] => {
                self.children.insert(*head, Box::new(Node::new(data)));
            }
            [head, tail @ ..] => {
                let child = self.children.get_mut(head).unwrap(); // this asummes that path is correct
                child.insert(tail, data);
            }
            [] => panic!("Path cannot be empty"),
        }
    }

    pub fn remove(&mut self, path: &[NodeKey]) {
        match path {
            [head] => {
                let to_remove = self.children.get_mut(head).unwrap();
                if to_remove.children.is_empty() {
                    self.children.remove(head);
                } else {
                    to_remove.kind = NodeKind::Empty;
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

pub struct PreOrderIterator<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> PreOrderIterator<'a> {
    fn new(root: &'a Node) -> Self {
        let mut stack = Vec::new();
        stack.push(root);
        Self { stack }
    }
}

impl<'a> Iterator for PreOrderIterator<'a> {
    type Item = &'a Node;

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
