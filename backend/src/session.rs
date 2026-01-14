use crate::{protocol, state};
use state::{Doc, Side};

pub struct Session {
    doc: Doc,
    this_side: Side,
}

impl Session {
    pub fn new(id: u8) -> Self {
        Self {
            doc: Doc::new(),
            this_side: Side::new(id),
        }
    }

    pub fn handle_local_operation(&mut self, op: protocol::LocalOperation) {
        match op.operation_type {
            Some(protocol::local_operation::OperationType::Insert(insert)) => {
                self.apply_insert(op.position, insert)
            }
            Some(protocol::local_operation::OperationType::Remove(_)) => {
                self.apply_remove(op.position)
            }
            None => eprintln!("Warn: Received operation with no type"),
        }
        eprintln!("Pos: {}", op.position);
        eprintln!("Doc: {:?}", self.doc.collect_string());
    }

    fn apply_insert(&mut self, pos: u32, insert: protocol::LocalInsert) {
        let char_code = insert.char;
        let Some(character) = char::from_u32(char_code) else {
            eprintln!("Err: Invalid char code received: {}", char_code);
            return;
        };
        eprintln!("Insert: {} ({:?})", char_code, character);

        if let Err(e) = self
            .doc
            .insert_absolute(&mut self.this_side, pos as usize, character)
        {
            eprintln!("Insert logic error: {}", e);
        }
    }

    fn apply_remove(&mut self, pos: u32) {
        eprintln!("Remove");
        if let Err(e) = self.doc.remove_absolute(pos as usize) {
            eprintln!("Remove logic error: {}", e);
        }
    }
}
