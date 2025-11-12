use core::panic;
use napi_derive::napi;
use num_bigint::{BigInt, Sign};
use num_traits::One;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;

use crate::node::{NodeKey, NodeKind};
use crate::side::Side;
use crate::tree::Tree;
use crate::types::{DEFAULT_BOUNDARY, IdType};

// for reproducible results during testing
const SEED: [u8; 32] = [0; 32];

// use this as main structure
#[napi]
#[derive(Debug)]
pub struct Doc {
    tree: Tree,
    strategy: HashMap<usize, bool>,
    boundry: IdType,
}

#[napi]
impl Doc {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            tree: Tree::default(),
            strategy: HashMap::new(),
            boundry: DEFAULT_BOUNDARY,
        }
    }
    pub fn tree(&self) -> &Tree {
        &self.tree
    }
    pub fn tree_mut(&mut self) -> &mut Tree {
        &mut self.tree
    }

    #[napi]
    pub fn remove_absolute(&mut self, absolute_position: u32) {
        let mut interval = absolute_position;
        let mut current = &mut self.tree.root;
        loop {
            current.subtree_size -= 1;
            let next_key = current.children.iter().find_map(|(key, node)| {
                println!("interval: {}", interval);
                println!("size: {}", node.subtree_size);
                println!("ch: {:?} pos: {:?}\n", node.kind, key);
                if (interval as usize) <= node.subtree_size {
                    Some(*key)
                } else {
                    interval -= node.subtree_size as u32;
                    None
                }
            });
            match next_key {
                Some(key) if interval == 1 => {
                    let to_remove = current.children.get_mut(&key).unwrap();
                    println!("removeing: {:?}", to_remove);
                    if to_remove.children.is_empty() {
                        //should be function
                        current.children.remove(&key);
                    } else {
                        to_remove.kind = crate::node::NodeKind::Empty;
                    }
                    break;
                }
                Some(key) => {
                    interval -= 1;
                    current = current.children.get_mut(&key).unwrap();
                }
                None => panic!("Position out of bounds"),
            }
        }
    }

    pub fn insert_absolute(&mut self, absolute_position: u32, data: char, side: &mut Side) {
        let before = self.id_from_absolute(absolute_position);
        let after = self.id_from_absolute(absolute_position);
        let id = self.generate_id(&before, &after, side);
        self.tree.insert(&id, data);
    }

    fn id_from_absolute(&self, absolute_position: u32) -> Arc<[NodeKey]> {
        let mut id = vec![];
        let mut interval = absolute_position;
        let mut current = &self.tree.root;
        loop {
            let next_key = current.children.iter().find_map(|(key, node)| {
                if (interval as usize) <= node.subtree_size {
                    println!("interval: {}", interval);
                    println!("size: {}", node.subtree_size);
                    println!("ch: {:?} pos: {:?}\n", node.kind, key);
                    Some(*key)
                } else {
                    interval -= node.subtree_size as u32;
                    None
                }
            });
            match next_key {
                Some(key) if interval == 0 => {
                    id.push(key);
                    break;
                }
                Some(key) => {
                    id.push(key);
                    if current.kind != NodeKind::Empty {
                        interval -= 1;
                    }
                    current = current.children.get(&key).unwrap();
                }
                None => panic!("Position out of bounds"),
            }
        }
        id.into()
    }

    // use this function to generate new id between p and q
    pub fn generate_id(&mut self, p: &[NodeKey], q: &[NodeKey], side: &mut Side) -> Arc<[NodeKey]> {
        let mut rng = StdRng::from_seed(SEED); // const seed
        // let mut rng = StdRng::from_os_rng();
        let (interval, p_pref, q_pref, depth) = Self::find_interval(p, q);
        let boundary = BigInt::new(Sign::Plus, vec![self.boundry]);
        let step = min(boundary, interval)
            .to_u32_digits()
            .1
            .first()
            .copied()
            .unwrap_or_default();
        let val = 1 + rng.random_range(0..step);
        if !self.strategy.contains_key(&depth) {
            self.strategy.insert(depth, false);
        }
        let digits = if self.strategy[&depth] {
            (&p_pref + val).to_u32_digits().1
        } else {
            (&q_pref - val).to_u32_digits().1
        };
        let len = digits.len();
        let digits = digits
            .into_iter()
            .chain(std::iter::repeat_n(0, depth.saturating_sub(len)))
            .rev()
            .collect::<Vec<IdType>>();
        Self::construct_id(&digits, p, q, side)
    }

    fn find_interval(p: &[NodeKey], q: &[NodeKey]) -> (BigInt, BigInt, BigInt, usize) {
        let (mut p_it, mut q_it) = (p.iter(), q.iter());
        let (mut interval, mut p_pref, mut q_pref) = (BigInt::ZERO, BigInt::ZERO, BigInt::ZERO);
        let mut depth = 0;
        while interval < BigInt::one() {
            depth += 1;
            p_pref = (p_pref << 32) + p_it.next().map_or(0, |pos| pos.digit);
            q_pref = (q_pref << 32) + q_it.next().map_or(0, |pos| pos.digit);
            interval = &q_pref - &p_pref - 1;
        }
        (interval, p_pref, q_pref, depth)
    }

    fn construct_id(r: &[IdType], p: &[NodeKey], q: &[NodeKey], side: &mut Side) -> Arc<[NodeKey]> {
        let mut once = true;
        let time = side.time_inc();
        let (mut p_it, mut q_it) = (p.iter(), q.iter());
        let mut id = vec![];
        for digit in r {
            let (p_opt, q_opt) = (p_it.next(), q_it.next());
            let pos = match (p_opt, q_opt) {
                (Some(p), _) if *digit == p.digit => p.clone(),
                (_, Some(q)) if *digit == q.digit => q.clone(),
                _ => {
                    once = if once {
                        false
                    } else {
                        panic!("More than one new position generated")
                    }; // temporary safeguard
                    NodeKey {
                        digit: *digit,
                        peer_id: side.peer_id,
                        time: time,
                    }
                }
            };
            id.push(pos);
        }
        id.into()
    }
}
