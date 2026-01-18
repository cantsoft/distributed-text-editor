use crate::{protocol, state};
use state::{Doc, NodeKey, Side};
use std::rc::Rc;

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

    pub fn handle_local_operation(
        &mut self,
        op: protocol::LocalOperation,
    ) -> Option<protocol::RemoteOp> {
        let ret = match op.operation_type {
            Some(protocol::local_operation::OperationType::Insert(insert)) => {
                self.apply_insert(op.position, insert)
            }
            Some(protocol::local_operation::OperationType::Remove(_)) => {
                self.apply_remove(op.position)
            }
            None => None,
        };
        eprintln!("Pos: {}", op.position);
        eprintln!("Doc: {:?}", self.doc.collect_string());
        ret
    }

    pub fn handle_network(&mut self, payload: protocol::PeerMessage) -> protocol::LocalOperation {
        match payload {
            protocol::PeerMessage::SyncOp(remote_op) => match remote_op {
                protocol::RemoteOp::RemoteInsert { id: key, value } => {
                    let key: Rc<[NodeKey]> = key.into();
                    if let Err(e) = self.doc.insert_id(key.clone(), value) {
                        eprintln!("Error while inserting character: {}", e);
                    }
                    let pos = self.doc.get_position(key.clone());
                    protocol::LocalOperation {
                        position: pos as u32,
                        operation_type: Some(protocol::local_operation::OperationType::Insert(
                            protocol::LocalInsert {
                                value: value as u32,
                            },
                        )),
                    }
                }
                protocol::RemoteOp::RemoteRemove { char_id: key } => {
                    let key: Rc<[NodeKey]> = key.into();
                    let pos = self.doc.get_position(key.clone());
                    if let Err(e) = self.doc.remove_id(key.clone()) {
                        eprintln!("Error while deleting character: {}", e);
                    }
                    protocol::LocalOperation {
                        position: pos as u32,
                        operation_type: Some(protocol::local_operation::OperationType::Remove(
                            protocol::LocalRemove {},
                        )),
                    }
                }
            },
        }
    }

    fn apply_insert(
        &mut self,
        pos: u32,
        insert: protocol::LocalInsert,
    ) -> Option<protocol::RemoteOp> {
        let Some(value) = char::from_u32(insert.value) else {
            eprintln!("Err: Invalid char code received: {}", insert.value);
            return None;
        };
        eprintln!("Insert: {} ({:?})", insert.value, value);

        match self
            .doc
            .insert_absolute(&mut self.this_side, pos as usize, value)
        {
            Ok(id) => Some(protocol::RemoteOp::RemoteInsert {
                id: id.to_vec(),
                value,
            }),
            Err(e) => {
                eprintln!("Insert logic error: {}", e);
                return None;
            }
        }
    }

    fn apply_remove(&mut self, pos: u32) -> Option<protocol::RemoteOp> {
        eprintln!("Remove");

        match self.doc.remove_absolute(pos as usize) {
            Ok(id) => Some(protocol::RemoteOp::RemoteRemove {
                char_id: id.to_vec(),
            }),
            Err(e) => {
                eprintln!("Remove logic error: {}", e);
                None
            }
        }
    }
}
