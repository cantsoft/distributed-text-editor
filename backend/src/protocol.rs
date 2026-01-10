use crate::doc::PeerIdType;
use crate::{doc, transport};
use crate::{proto::LocalOperation, session};
use bytes::Bytes;
use futures::SinkExt;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

#[derive(Serialize, Deserialize, Debug)]
pub struct PeerBeacon {
    pub id: doc::PeerIdType,
    pub tcp_port: u16,
}

#[derive(Debug)]
pub enum IngressPacket {
    FromStdin(Bytes),
    FromTcp(Bytes),
    FromDiscovered {
        id: doc::PeerIdType,
        addr: SocketAddr,
    },
}

pub async fn process_packets(
    mut rx: tokio::sync::mpsc::Receiver<IngressPacket>,
    tx_loopback: tokio::sync::mpsc::Sender<IngressPacket>,
    token: tokio_util::sync::CancellationToken,
    my_id: PeerIdType,
) {
    let mut session = session::EditorSession::new(my_id);
    let mut writer = FramedWrite::new(tokio::io::stdout(), LengthDelimitedCodec::new());
    let mut connected_peers = HashSet::new();

    while let Some(packet) = rx.recv().await {
        if token.is_cancelled() {
            break;
        }

        match packet {
            IngressPacket::FromStdin(bytes) => {
                eprintln!("IPC payload size: {}", bytes.len());
                let bytes_copy = bytes.clone();
                let op = match LocalOperation::decode(bytes) {
                    Ok(op) => op,
                    Err(e) => {
                        eprintln!("Protobuf decode error: {}", e);
                        return;
                    }
                };
                session.handle_local_operation(op);
                if let Err(e) = writer.send(bytes_copy).await {
                    eprintln!("Failed to write to stdout: {}", e);
                    break;
                }
                // If message == ShutdownCommand -> token.cancel();
            }
            IngressPacket::FromTcp(bytes) => {
                eprintln!("Network payload size: {}", bytes.len());
                // session.handle_network_packet(bytes);
            }
            IngressPacket::FromDiscovered { id, addr } => {
                if connected_peers.contains(&id) {
                    continue;
                }

                if my_id > id {
                    eprintln!("id ignored: {}", id);
                    continue;
                }

                eprintln!("initializing id {} on addr {}", id, addr);
                connected_peers.insert(id);

                let tx_clone = tx_loopback.clone();
                let token_clone = token.clone();
                tokio::spawn(transport::handle_peer_connection(
                    addr,
                    tx_clone,
                    token_clone,
                ));
            }
        }
    }
}
