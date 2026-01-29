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
        Self { doc, local_id: id }
    }

    pub fn get_doc_snapshot(&self) -> Doc {
        self.doc.clone()
    }

    pub fn get_doc_ascii(&self) -> Vec<u8> {
        self.doc.collect_ascii()
    }

    pub fn save_bytes(&self, path: &str) -> std::io::Result<()> {
        let bytes = self.doc.save_bytes()?;
        std::fs::write(path, bytes)
    }

    pub fn save_text(&self, path: &str) -> std::io::Result<()> {
        self.doc.save_text(path)
    }

    pub fn apply_local_op(&mut self, local_op: protocol::LocalOp) -> Option<protocol::PeerSyncOp> {
        let ret = match local_op.op_type.unwrap() {
            protocol::local_op::OpType::Insert(insert) => {
                self.apply_local_insert(local_op.position, insert)
            }
            protocol::local_op::OpType::Remove(_) => self.apply_local_remove(local_op.position),
        };
        ret
    }

    pub fn apply_peer_sync_op(
        &mut self,
        sync_op: protocol::PeerSyncOp,
    ) -> Option<protocol::ServerEvent> {
        use protocol::{PeerSyncOp, server_event};

        let event_variant = match sync_op {
            PeerSyncOp::Insert { char_id, value } => {
                if let Some(event) = self.apply_remote_insert(char_id, value) {
                    event
                } else {
                    return None;
                }
            }
            PeerSyncOp::Remove { char_id } => {
                if let Some(event) = self.apply_remote_remove(char_id) {
                    event
                } else {
                    return None;
                }
            }
            PeerSyncOp::FullSync { state } => {
                self.doc.merge_state(state);
                server_event::Variant::State(protocol::FullState {
                    content: self.doc.collect_ascii(),
                })
            }
        };
        Some(protocol::ServerEvent {
            variant: Some(event_variant),
        })
    }

    fn apply_local_insert(
        &mut self,
        pos: u32,
        insert: protocol::LocalInsert,
    ) -> Option<protocol::PeerSyncOp> {
        let Ok(value) = u8::try_from(insert.value) else {
            eprintln!("Err: Invalid char code received: {}", insert.value);
            return None;
        };
        eprintln!("Insert: {} ({:?})", value, char::from(value));

        match self.doc.insert_absolute(self.local_id, pos as usize, value) {
            Ok(id) => {
                eprintln!("Doc: {}", self.doc.collect_string());
                Some(protocol::PeerSyncOp::Insert {
                    char_id: id.to_vec(),
                    value,
                })
            }
            Err(e) => {
                eprintln!("Insert logic error: {}", e);
                return None;
            }
        }
    }

    fn apply_local_remove(&mut self, pos: u32) -> Option<protocol::PeerSyncOp> {
        eprintln!("Remove at position: {}", pos);
        match self.doc.remove_absolute(pos as usize) {
            Ok(id) => {
                eprintln!("Doc: {}", self.doc.collect_string());
                Some(protocol::PeerSyncOp::Remove {
                    char_id: id.to_vec(),
                })
            }
            Err(e) => {
                eprintln!("Remove logic error: {}", e);
                None
            }
        }
    }

    fn apply_remote_insert(
        &mut self,
        key: Vec<NodeKey>,
        value: u8,
    ) -> Option<protocol::server_event::Variant> {
        let key: Arc<[NodeKey]> = key.into();

        if let Err(e) = self.doc.insert_id(key.clone(), value) {
            eprintln!("Error while inserting character: {}", e);
            return None;
        }
        let raw_pos = self.doc.get_position(key)?;

        // Bezpieczne odejmowanie
        let ui_pos = raw_pos.checked_sub(1).unwrap_or(0);

        Some(protocol::server_event::Variant::Op(protocol::LocalOp {
            position: ui_pos as u32,
            remote: true,
            op_type: Some(protocol::local_op::OpType::Insert(protocol::LocalInsert {
                value: value as u32,
            })),
        }))
    }

    fn apply_remote_remove(&mut self, id: Vec<NodeKey>) -> Option<protocol::server_event::Variant> {
        let id: Arc<[NodeKey]> = id.into();

        self.doc.insert_cmentary(id.clone());
        let pos = self.doc.get_position(id.clone())?;

        if let Err(e) = self.doc.remove_id(id) {
            eprintln!("Error while deleting character: {}", e);
            return None;
        }

        Some(protocol::server_event::Variant::Op(protocol::LocalOp {
            position: pos as u32,
            remote: true,
            op_type: Some(protocol::local_op::OpType::Remove(protocol::LocalRemove {})),
        }))
    }
}
