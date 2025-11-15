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

    pub fn save_remove_char(&mut self, key: &NodeKey) {
        let to_remove = self.children.get_mut(key).unwrap();
        if to_remove.children.is_empty() {
            self.children.remove(key);
        } else {
            to_remove.kind = crate::node::NodeKind::Empty;
        }
    }

    // we don't handle empty chars in tree now
    pub fn insert_id(&mut self, id: &[NodeKey], data: char) {
        self.subtree_size += 1;
        match id {
            [head] => {
                self.children.insert(*head, Box::new(Node::new(data)));
            }
            [head, tail @ ..] => {
                let child = self.children.get_mut(head).unwrap(); // this asummes that path is correct
                child.insert_id(tail, data);
            }
            [] => panic!("Path cannot be empty"),
        }
    }

    pub fn remove_id(&mut self, path: &[NodeKey]) {
        match path {
            [head] => {
                self.save_remove_char(head);
            }
            [head, tail @ ..] => {
                let child = self.children.get_mut(head).unwrap(); // this asummes that path is correct
                child.subtree_size -= 1;
                child.remove_id(tail);
            }
            [] => panic!("Path cannot be empty"),
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
