use crate::Doc;
use crate::node::NodeKey;
use crate::side::Side;
use crate::types::IdType;
use std::iter::zip;
use std::sync::Arc;

// helper function to create default id from digits
fn from_digits(digits: &[IdType]) -> Arc<[NodeKey]> {
    digits
        .iter()
        .map(|digit| NodeKey {
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
pub fn insert_absolute_test() {
    let mut this_side = Side::new(123);
    let mut doc = Doc::new();
    doc.insert_absolute(0, 'a', &mut this_side);
    doc.insert_absolute(1, 'c', &mut this_side);
    doc.insert_absolute(1, 'b', &mut this_side);
    let doc_str = doc.tree().collect_string();
    assert_eq!("abc", doc_str)
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

#[test]
pub fn insert_remove_absolute_test() {
    let mut this_side = Side::new(123);
    let mut doc = Doc::new();
    doc.insert_absolute(0, 'a', &mut this_side);
    doc.insert_absolute(1, 'b', &mut this_side);
    doc.insert_absolute(2, 'c', &mut this_side);
    doc.insert_absolute(3, 'd', &mut this_side);
    doc.insert_absolute(4, 'e', &mut this_side);
    doc.insert_absolute(5, 'f', &mut this_side);
    doc.remove_absolute(1);
    doc.remove_absolute(4);
    doc.remove_absolute(1);
    doc.remove_absolute(3);
    doc.remove_absolute(1);
    doc.remove_absolute(2);
    let doc_str = doc.tree().collect_string();
    assert_eq!("c", doc_str)
}
