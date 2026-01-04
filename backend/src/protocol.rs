use crate::{proto::LocalOperation, session};
use bytes::Bytes;
use prost::Message;

#[derive(Debug)]
pub enum IngressPacket {
    FromStdin(Bytes),
    FromTcp(Bytes),
}

pub async fn process_packets(
    mut rx: tokio::sync::mpsc::Receiver<IngressPacket>,
    token: tokio_util::sync::CancellationToken,
) {
    let mut session = session::EditorSession::new(123);

    while let Some(packet) = rx.recv().await {
        if token.is_cancelled() {
            break;
        }

        match packet {
            IngressPacket::FromStdin(bytes) => {
                eprintln!("IPC payload size: {}", bytes.len());
                let op = match LocalOperation::decode(bytes) {
                    Ok(op) => op,
                    Err(e) => {
                        eprintln!("Protobuf decode error: {}", e);
                        return;
                    }
                };
                session.handle_local_operation(op);
                // If message == ShutdownCommand -> token.cancel();
            }
            IngressPacket::FromTcp(bytes) => {
                eprintln!("Network payload size: {}", bytes.len());
                // session.handle_network_packet(bytes);
            }
        }
    }
}
