use crate::state::{Doc, NodeKey};
use crate::types::{Digit, PeerId};
use serde::Deserialize;
use std::iter;
use std::sync::Arc;

fn from_digits(digits: &[Digit]) -> Arc<[NodeKey]> {
    digits
        .iter()
        .map(|digit| NodeKey::new(*digit, 0, 0))
        .collect()
}

#[test]
pub fn id_test() {
    let peer_id: PeerId = 123;
    let mut doc = Doc::new();
    let id = doc.generate_id(
        &from_digits(&[0, std::u32::MAX]),
        &from_digits(&[1]),
        peer_id,
    ); // digits are close on purpose
    println!("{:?}", id);
}

#[test]
pub fn insert_delete_collect_test() -> Result<(), &'static str> {
    let test_str = "abcdefghijklmnoprstuwxyz1234567890";
    let peer_id: PeerId = 123;
    let mut doc = Doc::new();
    let mut ids = Vec::new();
    let mut new_id = doc.bos_id();
    let eos = doc.eos_id();
    for ch in test_str.chars() {
        println!("ch: {:?}", ch);
        println!("after: {:?}", new_id);
        new_id = doc.generate_id(&new_id, &eos, peer_id);
        println!("new_id: {:?}\n", new_id);
        doc.insert_id(new_id.clone(), ch as u8)?;
        ids.push(new_id.clone());
    }
    let doc_str = String::from_utf8(doc.collect_ascii()).unwrap();
    assert_eq!(test_str, doc_str);
    for (id, ch) in iter::zip(ids, test_str.chars()) {
        println!("removing: {}", ch);
        doc.remove_id(id)?;
    }
    let doc_str = String::from_utf8(doc.collect_ascii()).unwrap();
    assert_eq!("", doc_str);
    Ok(())
}

#[test]
pub fn insert_absolute_test() -> Result<(), &'static str> {
    let peer_id: PeerId = 123;
    let mut doc = Doc::new();
    doc.insert_absolute(peer_id, 0, b'a')?;
    doc.insert_absolute(peer_id, 1, b'c')?;
    doc.insert_absolute(peer_id, 1, b'b')?;
    let doc_str = String::from_utf8(doc.collect_ascii()).unwrap();
    assert_eq!("abc", doc_str);
    Ok(())
}

#[test]
pub fn remove_absolute_test() -> Result<(), &'static str> {
    let test_str = "aabbccddeeffgg";
    let peer_id: PeerId = 123;
    let mut doc = Doc::new();
    let mut ids = Vec::new();
    let mut new_id = doc.bos_id();
    let eos = doc.eos_id();
    for ch in test_str.chars() {
        println!("ch: {}", ch);
        new_id = doc.generate_id(&new_id, &eos, peer_id);
        println!("new_id: {:?}\n", &new_id);
        doc.insert_id(new_id.clone(), ch as u8)?;
        ids.push(new_id.clone());
    }
    (0..=test_str.len())
        .rev()
        .filter(|i| i % 2 == 1)
        .try_for_each(|i| doc.remove_absolute(i).map(drop))?;
    let doc_str = String::from_utf8(doc.collect_ascii()).unwrap();
    assert_eq!("abcdefg", doc_str);
    Ok(())
}

#[test]
pub fn insert_remove_absolute_test() -> Result<(), &'static str> {
    let peer_id: PeerId = 123;
    let mut doc = Doc::new();
    doc.insert_absolute(peer_id, 0, b'a')?;
    doc.insert_absolute(peer_id, 1, b'b')?;
    doc.insert_absolute(peer_id, 2, b'c')?;
    doc.insert_absolute(peer_id, 3, b'd')?;
    doc.insert_absolute(peer_id, 4, b'e')?;
    doc.remove_absolute(1)?; // bcde
    doc.remove_absolute(4)?; // bcd
    doc.remove_absolute(1)?; // cd
    doc.remove_absolute(2)?; // c
    // doc.remove_absolute(0); // EOS could be removed
    let doc_str = String::from_utf8(doc.collect_ascii()).unwrap();
    assert_eq!("c", doc_str);
    Ok(())
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InsertOp {
    peer_id: u32,
    timestamp: u32,
    left_op: i32,
    right_op: i32,
    char: char,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RemoveOp {
    peer_id: u32,
    timestamp: u32,
    to_remove_op: u32,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op_type")]
enum Operation {
    #[serde(rename = "insert")]
    Insert(InsertOp),
    #[serde(rename = "remove")]
    Remove(RemoveOp),
}

#[derive(Debug, Deserialize)]
struct DataWrapper {
    result: String,
    operations: Vec<Operation>,
}

#[test]
pub fn relative_insert_remove_test() {
    let data = std::fs::read_to_string("../data/relative_insert_remove.json")
        .expect("Failed to read data file");
    let data_wrapper: DataWrapper = serde_json::from_str(&data).expect("Failed to parse JSON");
    println!(
        "Parsed data with {} operations",
        data_wrapper.operations.len()
    );

    let mut doc = Doc::new();
    // ids maps op_index -> NodeKey. Use Option because Remove ops don't produce a NodeKey.
    let mut ids: Vec<Option<Arc<[NodeKey]>>> = Vec::new();
    let eos = doc.eos_id();
    let bos = doc.bos_id();

    for op in data_wrapper.operations.iter() {
        match op {
            Operation::Insert(insert_op) => {
                let left_id = match insert_op.left_op {
                    -1 => bos.clone(),
                    idx => ids[idx as usize]
                        .as_ref()
                        .expect("Left ID should exist")
                        .clone(),
                };
                let right_id = match insert_op.right_op {
                    -1 => eos.clone(),
                    idx => ids[idx as usize]
                        .as_ref()
                        .expect("Right ID should exist")
                        .clone(),
                };
                let peer_id = insert_op.peer_id as PeerId;
                let new_id = doc.generate_id(&left_id, &right_id, peer_id);
                ids.push(Some(new_id.clone()));
                doc.insert_id(new_id, insert_op.char as u8)
                    .expect("Insert failed");
            }
            Operation::Remove(remove_op) => {
                let remove_id = ids[remove_op.to_remove_op as usize]
                    .as_ref()
                    .expect("ID to remove should exist")
                    .clone();
                doc.remove_id(remove_id).expect("Remove failed");
                ids.push(None);
            }
        }
    }
    let text = String::from_utf8(doc.collect_ascii()).unwrap();
    println!("Final text: {}", text);
    assert_eq!(text, data_wrapper.result);
}