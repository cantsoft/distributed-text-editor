use crate::types::{
    DEFAULT_BOUNDARY, Digit, MAX_POSITION_DIGIT, MIN_POSITION_DIGIT, PeerId, RESERVED_PEER,
    Timestamp,
};
use bincode;
use num_bigint::{BigInt, Sign};
use num_traits::One;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

const SEED: [u8; 32] = [0; 32];

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeKey {
    digit: Digit,
    peer_id: PeerId,
    time: Timestamp,
}

impl NodeKey {
    pub fn new(digit: Digit, peer_id: PeerId, time: Timestamp) -> Self {
        Self {
            digit: digit,
            peer_id: peer_id,
            time: time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Doc {
    id_list: BTreeMap<Rc<[NodeKey]>, Option<char>>,
    strategy: HashMap<usize, bool>,
    boundary: Digit,
}

impl Doc {
    pub fn new() -> Self {
        let mut id_list = BTreeMap::default();
        id_list.insert(
            Rc::from(vec![NodeKey::new(MIN_POSITION_DIGIT, RESERVED_PEER, 0)].into_boxed_slice()),
            None,
        );
        id_list.insert(
            Rc::from(vec![NodeKey::new(MAX_POSITION_DIGIT, RESERVED_PEER, 0)].into_boxed_slice()),
            None,
        );
        Self {
            id_list: id_list,
            strategy: HashMap::new(),
            boundary: DEFAULT_BOUNDARY,
        }
    }

    pub fn bos_id(&self) -> Rc<[NodeKey]> {
        self.id_list
            .first_key_value()
            .expect("Error: BOS node missing")
            .0
            .clone()
    }

    pub fn eos_id(&self) -> Rc<[NodeKey]> {
        self.id_list
            .last_key_value()
            .expect("Error: EOS node missing")
            .0
            .clone()
    }

    pub fn insert_absolute(
        &mut self,
        peer_id: PeerId,
        absolute_position: usize,
        data: char,
    ) -> Result<Rc<[NodeKey]>, &'static str> {
        let mut keys = self.id_list.keys();
        let before_key = keys
            .nth(absolute_position) // because of bos
            .cloned()
            .ok_or("missing key before position")?;
        let after_key = keys.next().cloned().ok_or("missing key after position")?;
        let id = self.generate_id(&before_key, &after_key, peer_id);
        self.id_list.insert(id.clone(), Some(data));
        Ok(id)
    }

    pub fn remove_absolute(
        &mut self,
        absolute_position: usize,
    ) -> Result<Rc<[NodeKey]>, &'static str> {
        if absolute_position == 0 {
            return Err("Can't remove at position 0");
        }
        let id = self
            .id_list
            .keys()
            .nth(absolute_position)
            .cloned()
            .ok_or("missing position")?;
        self.id_list.remove(&id);
        Ok(id)
    }

    pub fn insert_id(&mut self, id: Rc<[NodeKey]>, data: char) -> Result<(), &'static str> {
        (!self.id_list.contains_key(&id))
            .then(|| self.id_list.insert(id, Some(data)))
            .ok_or("ID already exists")
            .map(drop)
    }

    pub fn remove_id(&mut self, id: Rc<[NodeKey]>) -> Result<(), &'static str> {
        self.id_list.remove(&id).ok_or("id not found").map(drop)
    }

    pub fn collect_string(&self) -> String {
        self.id_list.values().filter_map(|ch| *ch).collect()
    }

    pub(super) fn generate_id(
        &mut self,
        p: &[NodeKey],
        q: &[NodeKey],
        peer_id: PeerId,
    ) -> Rc<[NodeKey]> {
        let mut rng = StdRng::from_seed(SEED); // const seed
        // let mut rng = StdRng::from_os_rng();
        let (interval, p_pref, q_pref, depth) = Self::find_interval(p, q);
        let boundary = BigInt::new(Sign::Plus, vec![self.boundary]);
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
            .collect::<Vec<Digit>>();
        Self::construct_id(&digits, p, q, peer_id)
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

    fn construct_id(r: &[Digit], p: &[NodeKey], q: &[NodeKey], peer_id: PeerId) -> Rc<[NodeKey]> {
        let mut once = true;
        let time = now_millis();
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
                        peer_id,
                        time,
                    }
                }
            };
            id.push(pos);
        }
        id.into()
    }

    pub fn save_text(&self, path: &str) -> std::io::Result<()> {
        let content = self.collect_string();
        let mut file = File::create(path)?;
        // Add UTF-8 BOM
        file.write_all(b"\xEF\xBB\xBF")?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn save_binary(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        bincode::serialize_into(file, self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    pub fn load_binary(path: &str) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let doc = bincode::deserialize_from(file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(doc)
    }

    pub fn get_position(&self, key: Rc<[NodeKey]>) -> usize {
        self.id_list.range(..key).count()
    }
}
