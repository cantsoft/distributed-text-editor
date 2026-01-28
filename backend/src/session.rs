use crate::protocol;
use crate::state::{Doc, NodeKey};
use crate::types::PeerId;
use std::sync::Arc;

pub struct Session {
    doc: Doc,
    local_id: PeerId,
}

impl Session {
    pub fn from(id: PeerId, path: &str) -> Self {
        let doc = match std::fs::read(path) {
            Ok(bytes) => Doc::load_bytes(&bytes).unwrap_or_else(|e| {
                eprintln!("Failed to parse {}: {}", path, e);
                Doc::new()
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Doc::new(),
            Err(e) => {
                eprintln!("Failed to read {}: {}", path, e);
                Doc::new()
            }
        };

        let doc = Doc::new(); // override
        Self { doc, local_id: id }
    }

    pub fn get_doc_snapshot(&self) -> Doc {
        self.doc.clone()
    }

    pub fn save_to_path(&self, path: &str) -> std::io::Result<()> {
        let bytes = self.doc.save_bytes()?;
        std::fs::write(path, bytes)
    }

    pub fn apply_local_op(&mut self, op: protocol::LocalOp) -> Option<protocol::PeerSyncOp> {
        let ret = match op.op_type {
            Some(protocol::local_op::OpType::In(insert)) => self.apply_insert(op.position, insert),
            Some(protocol::local_op::OpType::Rm(_)) => self.apply_remove(op.position),
            None => None,
        };
        eprintln!("Pos: {}", op.position);
        eprintln!("Doc: {:?}", self.doc.collect_string());
        ret
    }

    pub fn apply_network_message(&mut self, payload: protocol::PeerSyncOp) -> protocol::LocalOp {
        use protocol::PeerSyncOp;
        match payload {
            PeerSyncOp::RemoteInsert { id: key, value } => {
                let key: Arc<[NodeKey]> = key.into();
                if let Err(e) = self.doc.insert_id(key.clone(), value) {
                    eprintln!("Error while inserting character: {}", e);
                }
                let pos = self.doc.get_position(key.clone()).unwrap(); //TODO: unwrap
                protocol::LocalOp {
                    position: pos as u32,
                    op_type: Some(protocol::local_op::OpType::In(protocol::LocalInsert {
                        value: value as u32,
                    })),
                }
            }
            PeerSyncOp::RemoteRemove { char_id: id } => {
                let id: Arc<[NodeKey]> = id.into();
                let pos = self.doc.get_position(id.clone()).unwrap(); //TODO: unwrap
                self.doc.insert_cmentary(id.clone());
                if let Err(e) = self.doc.remove_id(id.clone()) {
                    eprintln!("Error while deleting character: {}", e);
                }
                protocol::LocalOp {
                    position: pos as u32,
                    op_type: Some(protocol::local_op::OpType::Rm(protocol::LocalRemove {})),
                }
            }
        }
    }

    fn apply_insert(
        &mut self,
        pos: u32,
        insert: protocol::LocalInsert,
    ) -> Option<protocol::PeerSyncOp> {
        let Ok(value) = u8::try_from(insert.value) else {
            eprintln!("Err: Invalid char code received: {}", insert.value);
            return None;
        };
        eprintln!("Insert: {} ({:?})", insert.value, value);

        match self.doc.insert_absolute(self.local_id, pos as usize, value) {
            Ok(id) => Some(protocol::PeerSyncOp::RemoteInsert {
                id: id.to_vec(),
                value,
            }),
            Err(e) => {
                eprintln!("Insert logic error: {}", e);
                return None;
            }
        }
    }

    fn apply_remove(&mut self, pos: u32) -> Option<protocol::PeerSyncOp> {
        eprintln!("Remove");

        match self.doc.remove_absolute(pos as usize) {
            Ok(id) => Some(protocol::PeerSyncOp::RemoteRemove {
                char_id: id.to_vec(),
            }),
            Err(e) => {
                eprintln!("Remove logic error: {}", e);
                None
            }
        }
    }
}
