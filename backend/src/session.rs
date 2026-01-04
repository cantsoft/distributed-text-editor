use crate::{
    doc::{Doc, Side},
    proto::{UserInsert, UserOperation, user_operation::OperationType},
};
use bytes::Bytes;
use prost::Message;

pub struct EditorSession {
    doc: Doc,
    identity: Side,
}

impl EditorSession {
    pub fn new(id: u8) -> Self {
        Self {
            doc: Doc::new(),
            identity: Side::new(id),
        }
    }

    pub fn handle_ipc_packet(&mut self, frame_bytes: Bytes) {
        let op = match UserOperation::decode(frame_bytes) {
            Ok(op) => op,
            Err(e) => {
                eprintln!("Protobuf decode error: {}", e);
                return;
            }
        };

        match op.operation_type {
            Some(OperationType::Insert(insert)) => self.apply_insert(op.position, insert),
            Some(OperationType::Remove(_)) => self.apply_remove(op.position),
            None => eprintln!("Warn: Received operation with no type"),
        }
        eprintln!("Pos: {}", op.position);
        eprintln!("Doc: {:?}", self.doc.collect_string());
    }

    fn apply_insert(&mut self, pos: u32, insert: UserInsert) {
        let char_code = insert.char;
        let Some(character) = char::from_u32(char_code) else {
            eprintln!("Err: Invalid char code received: {}", char_code);
            return;
        };
        eprintln!("Insert: {} ('{}')", char_code, character);

        if let Err(e) = self.doc.insert_absolute(&mut self.identity, pos, character) {
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
