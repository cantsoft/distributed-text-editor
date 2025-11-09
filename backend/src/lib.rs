mod doc;
mod node_crdt;
mod side;
mod tree_crdt;
mod types;

pub use doc::Doc;
use node_crdt::Position;
use side::Side;
use tree_crdt::TreeCRDT;

use napi_derive::napi;
use tokio::time::{Duration, sleep};

#[napi]
pub async fn delayed_sum(a: i32, b: i32) -> i32 {
    sleep(Duration::from_secs(2)).await;
    a + b
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::sync::Arc;
    use types::IdType;

    // helper function to create default id from digits
    fn from_digits(digits: &[IdType]) -> Arc<[Position]> {
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
        let mut this_side = Side::new(123); // represensts user. need to be mocked during testing
        let mut doc = Doc::new();
        let id = doc.generate_id(
            &from_digits(&[0, std::u32::MAX]),
            &from_digits(&[1]),
            &mut this_side,
        ); // digits are close on purpose
        println!("{:?}", id);
    }

    #[test]
    pub fn insert_delete_collect_test() {
        let mut this_side = Side::new(123);
        let mut doc = Doc::new();
        let mut ids = vec![];
        let mut new_id = doc.tree().bos_path();
        let eos = doc.tree().eos_path();
        for ch in "abcdefghijklmnoprstuwxyz1234567890".chars() {
            println!("{:?} {:?} {}", new_id, eos, ch);
            new_id = doc.generate_id(&new_id, &eos, &mut this_side);
            println!("{:?}\n", new_id);
            doc.tree_mut().insert(&new_id, ch);
            ids.push(new_id.clone());
        }
        let doc_str = doc.tree().collect_string();
        println!("{}", doc_str);
        assert_eq!(doc_str, "abcdefghijklmnoprstuwxyz1234567890");
        for id in ids {
            doc.tree_mut().remove(&id);
        }
        let doc_str = doc.tree().collect_string();
        println!("{}", doc_str);
        assert_eq!(doc_str, "");
    }
}
