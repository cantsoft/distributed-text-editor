use num_bigint::{BigInt, Sign};
use num_traits::One;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::collections::HashMap;

use super::node_crdt::{IdBase, Position};
use super::side::Side;
use super::tree_crdt::TreeCRDT;

// for reproducible results during testing
const SEED: [u8; 32] = [0; 32];

// use this as main structure
#[derive(Debug)]
pub struct Doc {
    pub tree: TreeCRDT,
    pub strategy: HashMap<usize, bool>,
    pub boundry: IdBase,
}

impl Doc {
    pub fn new() -> Self {
        Self {
            tree: TreeCRDT::default(),
            strategy: HashMap::new(),
            boundry: 16,
        }
    }

    fn construct_id(
        r: Vec<u32>,
        p: &Vec<Position>,
        q: &Vec<Position>,
        side: &mut Side,
    ) -> Vec<Position> {
        println!("{:?}", r);
        let mut p_it = p.iter();
        let mut q_it = q.iter();
        let mut id = vec![];
        for digit in r {
            let p_opt = p_it.next();
            let q_opt = q_it.next();
            let pos = match (p_opt, q_opt) {
                (Some(p), _) if digit == p.digit => p.clone(),
                (_, Some(q)) if digit == q.digit => q.clone(),
                _ => Position {
                    digit,
                    peer_id: side.peer_id,
                    time: side.time_inc(),
                },
            };
            id.push(pos);
        }
        id
    }

    // use this function to generate new id between p and q
    pub fn generate_id(
        &mut self,
        p: &Vec<Position>,
        q: &Vec<Position>,
        side: &mut Side,
    ) -> Vec<Position> {
        let mut rng = StdRng::from_seed(SEED); // const seed
        // let mut rng = StdRng::from_os_rng();
        let boundry = BigInt::new(Sign::Plus, vec![self.boundry]);
        let mut depth = 0;
        let mut p_it = p.iter();
        let mut q_it = q.iter();
        let mut interval = BigInt::ZERO;
        let mut p_pref = BigInt::ZERO;
        let mut q_pref = BigInt::ZERO;
        while interval < BigInt::one() {
            depth += 1;
            p_pref = (p_pref << 32) + p_it.next().map_or(0, |pos| pos.digit);
            q_pref = (q_pref << 32) + q_it.next().map_or(0, |pos| pos.digit);
            interval = &q_pref - &p_pref - 1;
            println!("{}: {:?} {:?} {:?}", depth, p_pref, q_pref, interval);
        }
        let step = std::cmp::min(boundry, interval)
            .to_u64_digits()
            .1
            .first()
            .copied()
            .unwrap_or_default();
        let val = 1 + rng.random_range(0..step);
        println!("step {:?} rand {:?}", step, val);
        if !self.strategy.contains_key(&depth) {
            self.strategy.insert(depth, rng.random_bool(0.5));
        }
        println!("pq: {:?}", (&p_pref).to_u32_digits().1);
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
            .collect();
        println!("digits: {:?}", digits);
        Self::construct_id(digits, p, q, side)
    }
}
