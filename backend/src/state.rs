use crate::types::{
    BOS_CHAR, DEFAULT_BOUNDARY, Digit, EOS_CHAR, MAX_POSITION_DIGIT, MIN_POSITION_DIGIT, PeerId,
    RESERVED_PEER, Timestamp,
};
use itertools::Itertools;
use num_bigint::{BigInt, Sign};
use num_traits::One;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::vec;

const SEED: [u8; 32] = [0; 32];

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Doc {
    id_list: im::Vector<(Arc<[NodeKey]>, u8)>,
    cmentary: HashSet<Arc<[NodeKey]>>,
}

impl Doc {
    pub fn new() -> Self {
        let mut id_list = im::Vector::new();
        id_list.push_back((
            Arc::from(vec![NodeKey::new(MIN_POSITION_DIGIT, RESERVED_PEER, 0)].into_boxed_slice()),
            BOS_CHAR,
        ));
        id_list.push_back((
            Arc::from(vec![NodeKey::new(MAX_POSITION_DIGIT, RESERVED_PEER, 0)].into_boxed_slice()),
            EOS_CHAR,
        ));
        Self {
            id_list: id_list,
            cmentary: HashSet::default(),
        }
    }

    pub fn load_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let doc = bincode::deserialize(bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(doc)
    }

    pub fn save_bytes(&self) -> std::io::Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    pub fn save_text(&self, path: &str) -> std::io::Result<()> {
        let content = self.collect_ascii();
        let mut file = File::create(path)?;
        file.write_all(content.as_slice())?;
        Ok(())
    }

    pub fn get_position(&self, id: Arc<[NodeKey]>) -> Option<usize> {
        self.id_list
            .binary_search_by(|(probe_id, _)| probe_id.cmp(&id))
            .ok()
    }

    pub fn collect_string(&self) -> String {
        let mut s = String::with_capacity(self.id_list.len());
        for (_, byte) in self.id_list.iter() {
            match *byte {
                BOS_CHAR => s.push_str("<BOS>"),
                EOS_CHAR => s.push_str("<EOS>"),
                b => s.push(b as char),
            }
        }
        s
    }

    pub fn collect_ascii(&self) -> Vec<u8> {
        self.id_list
            .iter()
            .skip(1)
            .take(self.id_list.len() - 2)
            .map(|(_, byte)| *byte)
            .collect()
    }

    pub fn insert_cmentary(&mut self, id: Arc<[NodeKey]>) {
        self.cmentary.insert(id);
    }

    pub fn insert_id(&mut self, id: Arc<[NodeKey]>, data: u8) -> Result<(), &'static str> {
        match self
            .id_list
            .binary_search_by(|(probe_id, _)| probe_id.cmp(&id))
        {
            Ok(_) => Err("Inserted ID already exists"),
            Err(idx) => {
                self.id_list.insert(idx, (id, data));
                Ok(())
            }
        }
    }

    pub fn remove_id(&mut self, id: Arc<[NodeKey]>) -> Result<(), &'static str> {
        match self
            .id_list
            .binary_search_by(|(probe_id, _)| probe_id.cmp(&id))
        {
            Ok(idx) => {
                self.cmentary.insert(id.clone());
                self.id_list.remove(idx);
                Ok(())
            }
            Err(_) => Err("Removed ID not found"),
        }
    }

    pub fn insert_absolute(
        &mut self,
        peer_id: PeerId,
        absolute_position: usize,
        data: u8,
    ) -> Result<Arc<[NodeKey]>, &'static str> {
        let before_key = self
            .id_list
            .get(absolute_position)
            .ok_or("missing key before position")?
            .0
            .clone();

        let after_key = self
            .id_list
            .get(absolute_position + 1)
            .ok_or("missing key after position")?
            .0
            .clone();

        let id = self.generate_id(&before_key, &after_key, peer_id);

        self.id_list
            .insert(absolute_position + 1, (id.clone(), data));

        Ok(id)
    }

    pub fn remove_absolute(
        &mut self,
        absolute_position: usize,
    ) -> Result<Arc<[NodeKey]>, &'static str> {
        if absolute_position == 0 {
            return Err("Can't remove BOS");
        }

        if absolute_position >= self.id_list.len() {
            return Err("missing position");
        }
        let (id, _) = self.id_list.remove(absolute_position);

        self.cmentary.insert(id.clone());
        Ok(id)
    }

    pub fn merge_state(&mut self, other: Self) {
        self.cmentary.extend(other.cmentary);

        let local_iter = self.id_list.iter().cloned();
        let remote_iter = other.id_list.into_iter();

        self.id_list = local_iter
            .merge(remote_iter)
            .dedup_by(|a, b| a.0 == b.0)
            .filter(|(id, _)| !self.cmentary.contains(id))
            .collect();
    }

    pub(crate) fn generate_id(
        &mut self,
        p: &[NodeKey],
        q: &[NodeKey],
        peer_id: PeerId,
    ) -> Arc<[NodeKey]> {
        let mut rng = StdRng::from_seed(SEED); // const seed
        // let mut rng = StdRng::from_os_rng();
        let (interval, p_pref, q_pref, depth) = Self::find_interval(p, q);
        let boundary = BigInt::new(Sign::Plus, vec![DEFAULT_BOUNDARY]);
        let step = min(boundary, interval)
            .to_u32_digits()
            .1
            .first()
            .copied()
            .unwrap_or_default();
        let val = 1 + rng.random_range(0..step);
        let digits = if depth % 2 == 1 {
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

    fn construct_id(r: &[Digit], p: &[NodeKey], q: &[NodeKey], peer_id: PeerId) -> Arc<[NodeKey]> {
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

    // #[cfg(test)]
    // pub(crate) fn bos_id(&self) -> Arc<[NodeKey]> {
    //     self.id_list
    //         .first_key_value()
    //         .expect("Error: BOS node missing")
    //         .0
    //         .clone()
    // }

    // #[cfg(test)]
    // pub(crate) fn eos_id(&self) -> Arc<[NodeKey]> {
    //     self.id_list
    //         .last_key_value()
    //         .expect("Error: EOS node missing")
    //         .0
    //         .clone()
    // }
}
