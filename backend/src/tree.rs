use crate::node::{Node, NodeKey, NodeKind};
use crate::types::{MAX_POSITION_DIGIT, MIN_POSITION_DIGIT};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct Tree {
    pub(crate) root: Node,
}

#[macro_export]
macro_rules! traverse_absolute {
    (
        $self:expr,
        $pos:expr,
        on_enter = $on_enter:expr,
        on_hit = $on_hit:expr
    ) => {{
        let mut interval = $pos;
        let mut current = &mut $self.root;

        loop {
            let next_key = {
                let mut chosen = None;
                for (key, node) in current.children.iter() {
                    if interval < node.subtree_size {
                        chosen = Some(*key);
                        break;
                    } else {
                        interval -= node.subtree_size;
                    }
                }
                chosen
            };

            match next_key {
                Some(key) if interval == 0 => {
                    $on_hit(current, key);
                    break;
                }
                Some(key) => {
                    $on_enter(&mut current.subtree_size, key);
                    if current.kind != $crate::node::NodeKind::Empty {
                        interval -= 1;
                    }

                    current = current.children.get_mut(&key).unwrap();
                }
                None => panic!("Position out of bounds"),
            }
        }
    }};
}

impl Default for Tree {
    fn default() -> Self {
        let mut new = Self {
            root: Node {
                kind: NodeKind::Root,
                children: BTreeMap::new(),
                subtree_size: 2,
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
                subtree_size: 1,
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
                subtree_size: 1,
            }),
        );
        new
    }
}

impl Tree {
    pub(crate) fn bos_path(&self) -> Arc<[NodeKey]> {
        Arc::from([NodeKey {
            digit: MIN_POSITION_DIGIT,
            peer_id: 0,
            time: 0,
        }])
    }

    pub(crate) fn eos_path(&self) -> Arc<[NodeKey]> {
        Arc::from([NodeKey {
            digit: MAX_POSITION_DIGIT,
            peer_id: 0,
            time: 0,
        }])
    }

    // assumse path is valid and not exists yet
    pub(crate) fn insert_id(&mut self, path: &[NodeKey], data: char) {
        self.root.insert_id(path, data);
    }

    // assumse path is valid and exists yet
    pub(crate) fn remove_id(&mut self, path: &[NodeKey]) {
        self.root.remove_id(path);
    }

    pub(crate) fn collect_string(&self) -> String {
        self.root
            .children
            .values()
            .flat_map(|node| node.iter())
            .filter_map(|node| match node.kind {
                NodeKind::Char(ch) => Some(ch),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn id_from_absolute(&mut self, pos: usize) -> Arc<[NodeKey]> {
        let mut path = Vec::new();
        traverse_absolute!(
            self,
            pos,
            on_enter = |_, key: NodeKey| {
                path.push(key);
            },
            on_hit = |_, key: NodeKey| {
                path.push(key);
            }
        );

        path.into()
    }
}
