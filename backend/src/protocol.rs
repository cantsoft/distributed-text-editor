pub use crate::doc::Doc;
use crate::{
    proto::{UserOperation, user_operation},
    side::Side,
    transport,
};
use bytes::Bytes;
use prost::Message;

pub async fn process_packets(
    mut rx: tokio::sync::mpsc::Receiver<transport::IngressPacket>,
    token: tokio_util::sync::CancellationToken,
) {
    let mut doc = Doc::new();
    let mut this_side = Side::new(123);
    while let Some(packet) = rx.recv().await {
        match packet {
            transport::IngressPacket::FromStdin(bytes) => {
                eprintln!("IPC payload size: {}", bytes.len());
                on_user_input(&mut doc, &mut this_side, bytes);
                // If message == ShutdownCommand -> token.cancel(); break;
            }
            transport::IngressPacket::FromTcp(bytes) => {
                eprintln!("Network payload size: {}", bytes.len());
            }
        }
    }
}

fn on_user_input(doc: &mut Doc, this_side: &mut Side, frame_bytes: Bytes) {
    let op = UserOperation::decode(frame_bytes).expect("decode");
    match op.operation_type {
        Some(user_operation::OperationType::Insert(user_insert)) => {
            doc.insert_absolute(
                this_side,
                op.position,
                char::from_u32(user_insert.char).expect("non valid char"),
            )
            .unwrap_or_else(|e| eprintln!("Err: {}", e));

            eprintln!(
                "Insert: {} {}",
                user_insert.char,
                char::from_u32(user_insert.char).expect("non valid char")
            );
        }
        Some(user_operation::OperationType::Remove(_)) => {
            eprintln!("Remove");
            doc.remove_absolute(op.position as usize)
                .unwrap_or_else(|e| eprintln!("Err: {}", e))
        }
        None => {
            panic!("operation type is None")
        }
    }
    eprintln!("Pos: {}", op.position);
    eprintln!("Doc: {:?}", doc.collect_string());
}
