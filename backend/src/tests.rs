use crate::Doc;
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
    doc.insert_absolute(0, 'a', &mut this_side)?;
    doc.insert_absolute(1, 'c', &mut this_side)?;
    doc.insert_absolute(1, 'b', &mut this_side)?;
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
    doc.insert_absolute(0, 'a', &mut this_side)?;
    doc.insert_absolute(1, 'b', &mut this_side)?;
    doc.insert_absolute(2, 'c', &mut this_side)?;
    doc.insert_absolute(3, 'd', &mut this_side)?;
    doc.insert_absolute(4, 'e', &mut this_side)?;
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
pub struct DataWrapper {
    pub data: Vec<OperationData>,
}

#[derive(Debug, Deserialize)]
pub struct OperationData {
    pub op_type: String, // "insert" or "remove"
    pub peer_id: u8,
    pub timestamp: u64,
    pub left_op: Option<i32>, // use i32 to allow -1
    pub right_op: Option<i32>,
    pub char: Option<char>,
    pub to_remove_op: Option<i32>,
}

#[test]
pub fn relative_insert_remove_test() {
    let mut this_side = Side::new(123);
    let data = fs::read_to_string("../data/relative_insert_remove.json")
        .expect("Failed to read data file");
    let data_wrapper: DataWrapper = serde_json::from_str(&data).expect("Failed to parse JSON");
    println!("Parsed data with {} operations", data_wrapper.data.len());

    let mut doc = Doc::new();
    // ids maps op_index -> NodeKey. Use Option because Remove ops don't produce a NodeKey.
    let mut ids: Vec<Option<Arc<[NodeKey]>>> = Vec::new();
    let eos = doc.eos_id();
    let bos = doc.bos_id();

    for (_, op) in data_wrapper.data.iter().enumerate() {
        if op.op_type == "insert" {
            let left_id = match op.left_op {
                Some(idx) if idx != -1 => ids[idx as usize].as_ref().expect("Left ID should exist"),
                _ => &bos,
            };
            let right_id = match op.right_op {
                Some(idx) if idx != -1 => {
                    ids[idx as usize].as_ref().expect("Right ID should exist")
                }
                _ => &eos,
            };

            let new_id = doc.generate_id(left_id, right_id, &mut this_side);
            ids.push(Some(new_id.clone()));

            doc.insert_id(new_id, op.char.unwrap_or(' '))
                .expect("Insert failed");
        } else if op.op_type == "remove" {
            if let Some(idx) = op.to_remove_op {
                if idx != -1 {
                    let remove_id = ids[idx as usize]
                        .as_ref()
                        .expect("ID to remove should exist");
                    doc.remove_id(remove_id).expect("Remove failed");
                }
            }
            // Push None to maintain index alignment with the operations list
            ids.push(None);
        }
    }

    let text = doc.collect_string();
    println!("Final text: {}", text);
    assert_eq!(text, "abcdefghijklmnoprstuxyz");
}
