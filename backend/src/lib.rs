mod doc;
mod node_crdt;
mod side;
mod tree_crdt;
mod types;

pub use doc::Doc;
use node_crdt::Position;
use side::Side;
use tree_crdt::TreeCRDT;
use serde::Deserialize;

// use napi_derive::napi;
// use tokio::time::sleep;
#[derive(Debug, Deserialize)]
pub struct OperationData {
    pub char: char, 
    pub position: u32,
    pub id: u32,
    pub user_id: u8,
    pub timestamp: u64,
    pub type_of_operation: char // 'i' = insert, 'd' = delete
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::Arc};
    use super::*;

    #[test]
    pub fn id_test() {
        let mut this_side = Side::new(123); // represensts user. need to be mocked during testing
        let mut doc = Doc::new();
        let id = doc.generate_id(
            &doc.tree().bos_path(),
            &doc.tree().eos_path(),
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

    #[test]
    pub fn dataset_test() {
        let data = fs::read_to_string("../data/test_dataset_adding.json").unwrap();
        let ops: Vec<OperationData> = serde_json::from_str(&data.as_str()).unwrap();
        let mut doc = Doc::new();
            for op in ops {
                let pos: Arc<[Position]> = Arc::from([Position {
                    digit: op.position,
                    peer_id: op.user_id,
                    time: op.timestamp,
                }]);
                if op.type_of_operation == 'i' {
                    doc.tree_mut().insert(&pos, op.char);
                } else if op.type_of_operation == 'd' {
                    doc.tree_mut().remove(&pos);
                }
            }
        let text = doc.tree().collect_string();
        let ground = std::fs::read_to_string("../data/test_dataset_adding.json_ground_truth.txt").unwrap();
        let ground = ground.replace("\r\n", "\n");

        println!("Reconstructed text: {}", text);
        assert_eq!(text, ground, "\nReconstructed text does not match ground truth");

    }

}
