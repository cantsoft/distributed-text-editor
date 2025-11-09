mod doc;
mod node_crdt;
mod side;
mod tree_crdt;
mod types;

pub use doc::Doc;
use node_crdt::Position;
use side::Side;
use tree_crdt::TreeCRDT;

#[cfg(test)]
mod tests {

    use super::*;
    use std::{iter::zip, sync::Arc};
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
        let test_str = "abcdefghijklmnoprstuwxyz1234567890";
        let mut this_side = Side::new(123);
        let mut doc = Doc::new();
        let mut ids = vec![];
        let mut new_id = doc.tree().bos_path();
        let eos = doc.tree().eos_path();
        for ch in test_str.chars() {
            println!("ch: {}", ch);
            println!("between: {:?}", new_id);
            new_id = doc.generate_id(&new_id, &eos, &mut this_side);
            println!("new_id: {:?}\n", new_id);
            doc.tree_mut().insert(&new_id, ch);
            ids.push(new_id.clone());
        }
        let doc_str = doc.tree().collect_string();
        assert_eq!(test_str, doc_str);
        for (id, ch) in zip(ids, test_str.chars()) {
            println!("ch: {}", ch);
            doc.tree_mut().remove(&id);
        }
        let doc_str = doc.tree().collect_string();
        assert_eq!("", doc_str);
    }

    #[test]
    pub fn remove_absolute_test() {
        let test_str = "aabbccddeeffgg";
        let mut this_side = Side::new(123);
        let mut doc = Doc::new();
        let mut ids = vec![];
        let mut new_id = doc.tree().bos_path();
        let eos = doc.tree().eos_path();
        for ch in test_str.chars() {
            println!("ch: {}", ch);
            println!("between: {:?}", new_id);
            new_id = doc.generate_id(&new_id, &eos, &mut this_side);
            println!("new_id: {:?}\n", new_id);
            doc.tree_mut().insert(&new_id, ch);
            ids.push(new_id.clone());
        }
        println!("tree size: {}", doc.tree().root.subtree_size);
        for i in (1..=test_str.len()).rev() {
            if i % 2 == 0 {
                doc.remove_absolute(i as u32);
            }
        }
        let doc_str = doc.tree().collect_string();
        assert_eq!("abcdefg", doc_str);
    }
}
