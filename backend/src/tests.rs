use crate::Doc;
use crate::side::Side;
use crate::types::{DigitType, NodeKey};
use std::iter::zip;
use std::sync::Arc;
use std::fs;
use serde::Deserialize;


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
    let data = fs::read_to_string("../data/relative_insert_remove.json").unwrap();
    let data_wrapper: DataWrapper = serde_json::from_str(&data.as_str()).unwrap();
    println!("Parsed data: {:?}", data_wrapper);
    let mut doc = Doc::new();
    let mut ids = Vec::new();
    let mut new_id = doc.bos_id();
    let eos = doc.eos_id();
    let bos = doc.bos_id();
    for op in data_wrapper.data.iter() {
        ids.sort_by_key(|id: &Arc<[NodeKey]>| id.as_ref()[0].digit);
        let left_node: Arc<[NodeKey]> = Arc::from([NodeKey {
            digit: op.left_op.unwrap_or(-1).abs() as u32,
            peer_id: op.peer_id,
            time: op.timestamp,
        }]);
        println!("Left node: {:?}", left_node);

        let right_node: Arc<[NodeKey]> = Arc::from([NodeKey {
            digit: op.right_op.unwrap_or(0).abs() as u32,
            peer_id: op.peer_id,
            time: op.timestamp,
        }]);
        println!("Right node: {:?}", right_node);

        if op.op_type == "insert" {
            if op.left_op.unwrap_or(-1) == -1 && op.right_op.unwrap_or(-1) == -1{
                println!("Both none case1");
                new_id = doc.generate_id(&bos, &eos, &mut this_side);
                println!("Both none case2");
            }
            else if op.right_op.unwrap_or(-1) == -1{
                println!("Right none case1");
                let left_id = &ids[op.left_op.unwrap_or(0) as usize];
                new_id = doc.generate_id(left_id, &eos, &mut this_side);
                println!("Right none case2");
            }
            else if op.left_op.unwrap_or(-1) == -1 {
                println!("Left none case1");
                let right_id = &ids[op.right_op.unwrap_or(0) as usize];
                new_id = doc.generate_id(&bos, right_id, &mut this_side);
                println!("Left none case2");
            }
            else {
                println!("Normal case1");
                let left_id = &ids[op.left_op.unwrap_or(0) as usize];
                let right_id = &ids[op.right_op.unwrap_or(0) as usize];
                new_id = doc.generate_id(left_id, right_id, &mut this_side);
                println!("Normal case2");
            }
            ids.push(new_id.clone());


            let pos: Arc<[NodeKey]> = Arc::from([NodeKey {
                digit: new_id.as_ref()[0].digit,
                peer_id: op.peer_id,
                time: op.timestamp,
            }]);
            println!("Inserting at position: {:?} char: {:?}", pos, op.char);
                
            doc.insert_id(pos, op.char.unwrap_or(' '));
            println!("After insert: {:?}", doc.collect_string());


        } else if op.op_type == "remove" {
            println!("Remove operation detected");
            if op.to_remove_op.unwrap_or(-1) != -1{
                let pos_to_remove: Arc<[NodeKey]> = Arc::from([NodeKey {
                    digit: op.to_remove_op.unwrap_or(-1) as u32,
                    peer_id: op.peer_id,
                    time: op.timestamp,
                }]);
                println!("Removing at position: {:?}", pos_to_remove);
                let remove_id = &ids[op.to_remove_op.unwrap_or(0) as usize];
                doc.remove_id(remove_id);
                ids.remove(op.to_remove_op.unwrap_or(0) as usize);
                println!("After remove: {:?}", doc.collect_string());
            }
        }
        let text = doc.collect_string();

        //let ground = std::fs::read_to_string("../data/test_dataset_adding.json_ground_truth.txt").unwrap();
        //let ground = ground.replace("\r\n", "\n");

        println!("Reconstructed text: {}", text);

        //assert_eq!(text, ground, "\nReconstructed text does not match ground truth");
    }
}
