use crate::doc::Doc;
use crate::side::Side;
use crate::types::{DigitType, NodeKey};
use serde::Deserialize;
use std::fs;
use std::iter::zip;
use std::sync::Arc;

fn from_digits(digits: &[DigitType]) -> Arc<[NodeKey]> {
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
    let mut this_side = Side::new(123);
    let mut doc = Doc::new();
    let id = doc.generate_id(
        &from_digits(&[0, std::u32::MAX]),
        &from_digits(&[1]),
        &mut this_side,
    ); // digits are close on purpose
    println!("{:?}", id);
}

#[test]
pub fn insert_delete_collect_test() -> Result<(), &'static str> {
    let test_str = "abcdefghijklmnoprstuwxyz1234567890";
    let mut this_side = Side::new(123);
    let mut doc = Doc::new();
    let mut ids = Vec::new();
    let mut new_id = doc.bos_id();
    let eos = doc.eos_id();
    for ch in test_str.chars() {
        println!("ch: {:?}", ch);
        println!("after: {:?}", new_id);
        new_id = doc.generate_id(&new_id, &eos, &mut this_side);
        println!("new_id: {:?}\n", new_id);
        doc.insert_id(new_id.clone(), ch)?;
        ids.push(new_id.clone());
    }
    let doc_str = doc.collect_string();
    assert_eq!(test_str, doc_str);
    for (id, ch) in zip(ids, test_str.chars()) {
        println!("removing: {}", ch);
        doc.remove_id(&id)?;
    }
    let doc_str = doc.collect_string();
    assert_eq!("", doc_str);
    Ok(())
}

#[test]
pub fn insert_absolute_test() -> Result<(), &'static str> {
    let mut this_side = Side::new(123);
    let mut doc = Doc::new();
    doc.insert_absolute(&mut this_side, 0, 'a')?;
    doc.insert_absolute(&mut this_side, 1, 'c')?;
    doc.insert_absolute(&mut this_side, 1, 'b')?;
    let doc_str = doc.collect_string();
    assert_eq!("abc", doc_str);
    Ok(())
}

#[test]
pub fn remove_absolute_test() -> Result<(), &'static str> {
    let test_str = "aabbccddeeffgg";
    let mut this_side = Side::new(123);
    let mut doc = Doc::new();
    let mut ids = Vec::new();
    let mut new_id = doc.bos_id();
    let eos = doc.eos_id();
    for ch in test_str.chars() {
        println!("ch: {}", ch);
        new_id = doc.generate_id(&new_id, &eos, &mut this_side);
        println!("new_id: {:?}\n", &new_id);
        doc.insert_id(new_id.clone(), ch)?;
        ids.push(new_id.clone());
    }
    (0..=test_str.len())
        .rev()
        .filter(|i| i % 2 == 0)
        .try_for_each(|i| doc.remove_absolute(i))?;
    let doc_str = doc.collect_string();
    assert_eq!("abcdefg", doc_str);
    Ok(())
}

#[test]
pub fn insert_remove_absolute_test() -> Result<(), &'static str> {
    let mut this_side = Side::new(123);
    let mut doc = Doc::new();
    doc.insert_absolute(&mut this_side, 0, 'a')?;
    doc.insert_absolute(&mut this_side, 1, 'b')?;
    doc.insert_absolute(&mut this_side, 2, 'c')?;
    doc.insert_absolute(&mut this_side, 3, 'd')?;
    doc.insert_absolute(&mut this_side, 4, 'e')?;
    doc.remove_absolute(0)?; // bcde
    doc.remove_absolute(3)?; // bcd
    doc.remove_absolute(0)?; // cd
    doc.remove_absolute(1)?; // c
    // doc.remove_absolute(0); // EOS could be removed
    let doc_str = doc.collect_string();
    assert_eq!("c", doc_str);
    Ok(())
}

#[derive(Debug, Deserialize)]
struct InsertOp {
    peer_id: u32,
    timestamp: u32,
    left_op: i32,
    right_op: i32,
    char: char,
}

#[derive(Debug, Deserialize)]
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
    let data = fs::read_to_string("../data/relative_insert_remove.json")
        .expect("Failed to read data file");
    let data_wrapper: DataWrapper = serde_json::from_str(&data).expect("Failed to parse JSON");
    println!(
        "Parsed data with {} operations",
        data_wrapper.operations.len()
    );

    let mut this_side = Side::new(123);
    let mut doc = Doc::new();
    // ids maps op_index -> NodeKey. Use Option because Remove ops don't produce a NodeKey.
    let mut ids: Vec<Option<Arc<[NodeKey]>>> = Vec::new();
    let eos = doc.eos_id();
    let bos = doc.bos_id();

    for op in data_wrapper.operations.iter() {
        match op {
            Operation::Insert(insert_op) => {
                let left_id = match insert_op.left_op {
                    -1 => &bos,
                    idx => ids[idx as usize].as_ref().expect("Left ID should exist"),
                };
                let right_id = match insert_op.right_op {
                    -1 => &eos,
                    idx => ids[idx as usize].as_ref().expect("Right ID should exist"),
                };
                let new_id = doc.generate_id(left_id, right_id, &mut this_side);
                ids.push(Some(new_id.clone()));
                doc.insert_id(new_id, insert_op.char)
                    .expect("Insert failed");
            }
            Operation::Remove(remove_op) => {
                let remove_id = ids[remove_op.to_remove_op as usize]
                    .as_ref()
                    .expect("ID to remove should exist");
                doc.remove_id(remove_id).expect("Remove failed");
                ids.push(None);
            }
        }
    }
    let text = doc.collect_string();
    println!("Final text: {}", text);
    assert_eq!(text, data_wrapper.result);
}
