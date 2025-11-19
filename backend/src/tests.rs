use crate::Doc;
use crate::node::NodeKey;
use crate::side::Side;
use crate::types::IdType;
use std::iter::zip;
use std::sync::Arc;

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
        println!("removeing: {}", ch);
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
    // println!("{}", doc.collect_string());
    doc.remove_absolute(0)?; // cd
    doc.remove_absolute(1)?; // c
    // doc.remove_absolute(0); // EOS could be removed
    let doc_str = doc.collect_string();
    assert_eq!("c", doc_str);
    Ok(())
}
