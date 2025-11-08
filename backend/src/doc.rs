use num_bigint::{BigInt, Sign};
use num_traits::One;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::min;
use std::collections::HashMap;

use crate::types::DEFAULT_BOUNDARY;
use crate::{Position, Side, TreeCRDT, types::IdType};

// for reproducible results during testing
const SEED: [u8; 32] = [0; 32];

// use this as main structure
#[derive(Debug)]
pub struct Doc {
    tree: TreeCRDT,
    strategy: HashMap<usize, bool>,
    boundry: IdType,
}

impl Doc {
    pub fn new() -> Self {
        Self {
            tree: TreeCRDT::default(),
            strategy: HashMap::new(),
            boundry: DEFAULT_BOUNDARY,
        }
    }

    pub fn tree(&self) -> &TreeCRDT {
        &self.tree
    }
    pub fn tree_mut(&mut self) -> &mut TreeCRDT {
        &mut self.tree
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
        let (interval, p_pref, q_pref) = Self::find_interval(p, q);
        let boundary = BigInt::new(Sign::Plus, vec![self.boundry]);
        let depth = p_pref.to_u32_digits().1.len();
        let step = min(boundary, interval)
            .to_u32_digits()
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

    fn find_interval(p: &Vec<Position>, q: &Vec<Position>) -> (BigInt, BigInt, BigInt) {
        let (mut p_it, mut q_it) = (p.iter(), q.iter());
        let (mut interval, mut p_pref, mut q_pref) = (BigInt::ZERO, BigInt::ZERO, BigInt::ZERO);
        while interval < BigInt::one() {
            p_pref = (p_pref << 32) + p_it.next().map_or(0, |pos| pos.digit);
            q_pref = (q_pref << 32) + q_it.next().map_or(0, |pos| pos.digit);
            interval = &q_pref - &p_pref - 1;
        }
        (interval, p_pref, q_pref)
    }

    fn construct_id(
        r: Vec<IdType>,
        p: &Vec<Position>,
        q: &Vec<Position>,
        side: &mut Side,
    ) -> Vec<Position> {
        println!("{:?}", r);
        let mut once = true;
        let time = side.time_inc();
        let (mut p_it, mut q_it) = (p.iter(), q.iter());
        let mut id = vec![];
        for digit in r {
            let (p_opt, q_opt) = (p_it.next(), q_it.next());
            let pos = match (p_opt, q_opt) {
                (Some(p), _) if digit == p.digit => p.clone(),
                (_, Some(q)) if digit == q.digit => q.clone(),
                _ => {
                    once = if once { false } else { unreachable!() }; // temporary safeguard
                    Position {
                        digit,
                        peer_id: side.peer_id,
                        time: time,
                    }
                }
            };
            id.push(pos);
        }
        id
    }
}
