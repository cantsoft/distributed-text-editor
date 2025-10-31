use napi_derive::napi;
use num_bigint::{BigInt, Sign};
use num_traits::One;
use rand::{self, Rng, SeedableRng, rngs::StdRng};
use std::{
    cmp::min,
    collections::{BTreeMap, HashMap},
    iter::zip,
    vec,
};
use tokio::time::{Duration, sleep};

const SEED: [u8; 32] = [0; 32];

#[napi]
pub async fn delayed_sum(a: i32, b: i32) -> i32 {
    sleep(Duration::from_secs(2)).await;
    a + b
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Side {
    peer_id: u8,
    time: u64,
}

impl Side {
    pub fn new(peer_id: u8) -> Self {
        Self { peer_id, time: 0 }
    }

    pub fn time_inc(&mut self) -> u64 {
        let ret = self.time;
        self.time += 1;
        ret
    }
}

type IdBase = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    digit: IdBase,
    peer_id: u8,
    time: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeCRDT {
    depth: u8,
    data: char,
    childrens: BTreeMap<Position, Box<NodeCRDT>>,
}

impl NodeCRDT {
    pub fn max_digit(depth: u8) -> IdBase {
        1 << (4 + depth)
    }
    pub fn collect_string(&self) -> String {
        let mut ret = String::new();
        ret.push(self.data);
        for (_, node) in &self.childrens {
            ret += &node.collect_string();
        }
        ret
    }
}

// pub struct NodeCRDTIter<'a> {
//     children: &'a BTreeMap<IdBase, Box<NodeCRDT>>,
//     parent: Option<Box<NodeCRDTIter<'a>>>,
// }
// impl<'a> Iterator for NodeCRDTIter<'a> {
//     fn next(&mut self) -> Option<Position> {
//         if self.children.is_empty() {
//             match self.parent.take() {
//                 Some(parent) => {
//                     // continue with the parent node
//                     *self = *parent;
//                     self.next()
//                 }
//                 None => None,
//             }
//         } else {
//             let next_child
//         }
//         match self.children.first_key_value() {
//             Some((digit, child)) => {
//                 self.children = &self.children[1..];
//                 // start iterating the child trees
//                 *self = NodeIter {
//                     children: children.as_slice(),
//                     parent: Some(Box::new(mem::take(self))),
//                 };
//                 self.next()
//             }
//         }
//     }
// }

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

#[derive(Debug)]
pub struct Doc {
    tree: TreeCRDT,
    strategy: HashMap<usize, bool>,
    boundry: IdBase,
}

impl Doc {
    pub fn new() -> Self {
        Self {
            tree: TreeCRDT::default(),
            strategy: HashMap::new(),
            boundry: 16,
        }
    }

    fn prefix(id: &Vec<Position>, depth: usize) -> Vec<Position> {
        id.iter()
            .take(depth)
            .cloned()
            .chain(std::iter::repeat_n(
                Position {
                    digit: 0,
                    peer_id: 0,
                    time: 0,
                },
                depth.saturating_sub(id.len()),
            ))
            .rev()
            .collect()
    }
    fn construct_id(
        r: Vec<u32>,
        p: &Vec<Position>,
        q: &Vec<Position>,
        side: &mut Side,
    ) -> Vec<Position> {
        println!("{:?}", r);
        let mut id = vec![];
        for i in 0..r.len() - 1 {
            println!("{:?}; {:?}; {:?}", r[i], p[i], q[i]);
            id.push(if r[i] == p[i].digit { p[i] } else { q[i] }); //to optimize
        }
        id.push(Position {
            digit: *r.last().unwrap(),
            peer_id: side.peer_id,
            time: side.time_inc(),
        });
        id
    }

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
        println!("pq: {:?}", (&q_pref).to_u32_digits().1);
        if self.strategy[&depth] {
            // println!("p_pref {:?}", p_pref.to_u32_digits()); //this is reversed
            println!("digits: {:?}", (&p_pref + val).to_u32_digits().1);
            Self::construct_id((p_pref + val).to_u32_digits().1, p, q, side)
        } else {
            // println!("q_pref {:?}", q_pref.to_u32_digits()); //this is reversed
            println!("digits: {:?}", (&q_pref - val).to_u32_digits().1);
            Self::construct_id((q_pref - val).to_u32_digits().1, p, q, side)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_digits(digits: &Vec<u32>) -> Vec<Position> {
        digits
            .iter()
            .map(|digit| Position {
                digit: *digit,
                peer_id: 0,
                time: 0,
            })
            .collect()
    }

    #[test]
    pub fn id_test() {
        let mut this_side = Side::new(123);
        let mut doc = Doc::new();
        let id = doc.generate_id(
            &from_digits(&vec![0, std::u32::MAX]),
            &from_digits(&vec![1]),
            &mut this_side,
        );
        println!("{:?}", id.iter().map(|id| id.digit).collect::<Vec<u32>>());
    }

    #[test]
    pub fn tree_test() {
        let mut this_side = Side::new(123);
        let mut doc = Doc::new();
        let mut new_id = from_digits(&vec![0]);
        let eof = from_digits(&vec![std::u32::MAX]);
        for ch in "abcdefghijklmnoprstuwxyz1234567890".chars() {
            println!("{:?} {:?} {}", new_id, eof, ch);
            new_id = doc.generate_id(&new_id, &eof, &mut this_side);
            new_id.reverse(); // niestety na razie trzeba obracac
            println!("{:?}\n", new_id);
            doc.tree.insert(&new_id, ch);
        }
        let doc_str = doc.tree.collect_string();
        println!("{}", doc_str);
    }
}
