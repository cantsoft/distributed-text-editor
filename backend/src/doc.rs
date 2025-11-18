use core::panic;
use napi_derive::napi;
use num_bigint::{BigInt, Sign};
use num_traits::One;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;

use crate::node::{Node, NodeKey};
use crate::side::Side;
use crate::traverse_absolute;
use crate::tree::Tree;
use crate::types::{DEFAULT_BOUNDARY, IdType};

// for reproducible results during testing
const SEED: [u8; 32] = [0; 32];

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
    pub(crate) fn tree(&self) -> &Tree {
        &self.tree
    }

    pub(crate) fn tree_mut(&mut self) -> &mut Tree {
        &mut self.tree
    }

    #[napi]
    pub fn insert_absolute_wrapper(&mut self, absolute_position: u32, data: String) {
        self.insert_absolute(
            absolute_position as usize,
            data.chars().next().unwrap(),
            &mut Side::new(123),
        );
    }

    #[napi]
    pub fn remove_absolute_wrapper(&mut self, absolute_position: u32) {
        self.remove_absolute(absolute_position as usize);
    }

    #[napi]
    pub fn collect_string(&self) -> String {
        self.tree.collect_string()
    }

    pub(crate) fn insert_absolute(
        &mut self,
        absolute_position: usize,
        data: char,
        side: &mut Side,
    ) {
        let before = self.tree.id_from_absolute(absolute_position);
        let after = self.tree.id_from_absolute(1 + absolute_position);
        let id = self.generate_id(&before, &after, side);
        self.tree.insert_id(&id, data);
    }

    pub(crate) fn remove_absolute(&mut self, pos: usize) {
        traverse_absolute!(
            self.tree,
            pos + 1,
            on_enter = |subtree_size: &mut usize, _| {
                *subtree_size -= 1;
            },
            on_hit = |parent: &mut Node, key: NodeKey| {
                parent.subtree_size -= 1;
                println!("removing {:?}", parent.children.get(&key));
                parent.save_remove_char(&key);
            }
        );
    }

    // use this function to generate new id between p and q
    pub(crate) fn generate_id(
        &mut self,
        p: &[NodeKey],
        q: &[NodeKey],
        side: &mut Side,
    ) -> Arc<[NodeKey]> {
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
        let mut id = Vec::new();
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
