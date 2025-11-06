// use napi_derive::napi;
// use tokio::time::sleep;

mod side;
pub use side::Side;

mod node_crdt;
pub use node_crdt::{NodeCRDT, Position};

mod tree_crdt;
pub use tree_crdt::TreeCRDT;

mod doc;
pub use doc::Doc;

#[cfg(test)]
mod tests {
    use super::*;

    // helper function to create default id from digits
    fn from_digits(digits: &Vec<u32>) -> Vec<Position> {
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
            &from_digits(&vec![0, std::u32::MAX]),
            &from_digits(&vec![1]),
            &mut this_side,
        ); // digits are close on purpose
        println!("{:?}", id);
    }

    // forward insertion test example
    #[test]
    pub fn tree_test() {
        let mut this_side = Side::new(123);
        let mut doc = Doc::new();
        let mut new_id = from_digits(&vec![0]);
        let eof = from_digits(&vec![std::u32::MAX]);
        for ch in "abcdefghijklmnoprstuwxyz1234567890".chars() {
            println!("{:?} {:?} {}", new_id, eof, ch);
            new_id = doc.generate_id(&new_id, &eof, &mut this_side);
            println!("{:?}\n", new_id);
            doc.tree.insert(&new_id, ch);
        }
        let doc_str = doc.tree.collect_string();
        println!("{}", doc_str);
        assert_eq!(doc_str, "abcdefghijklmnoprstuwxyz1234567890");
    }
}
