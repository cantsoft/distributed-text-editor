use crate::protocol::{NodeEvent, PeerMessage};
use crate::state::PeerIdType;
use crate::{config, protocol, session, transport};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use tokio_util::sync::CancellationToken;

pub async fn run(
    config: config::NodeConfig,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel(32);
    let token = CancellationToken::new();

    let tx_stdin = tx.clone();
    let token_stdin = token.clone();
    tokio::spawn(async move {
        use prost::Message;
        transport::stream_reader(tokio::io::stdin(), tx_stdin, token_stdin, |bytes| {
            protocol::LocalOperation::decode(bytes)
                .ok()
                .map(protocol::NodeEvent::User)
        })
        .await;
    });

    let tx_discovery = tx.clone();
    let token_discovery = token.clone();
    let config_discovery = config.clone();
    tokio::spawn(async move {
        if let Err(e) =
            transport::run_discovery(tx_discovery, token_discovery, config_discovery).await
        {
            eprintln!("Discovery crashed: {}", e);
        }
    });

    let tx_tcp = tx.clone();
    let token_tcp = token.clone();
    let my_id = config.peer_id;
    tokio::spawn(async move {
        if let Err(e) = transport::run_tcp_listener(tx_tcp, token_tcp, config.tcp_port, my_id).await
        {
            eprintln!("TCP Server crashed: {}", e);
        }
    });

    handle_events(rx, tx, token, config.peer_id).await;

    Ok(())
}

pub async fn handle_events(
    mut rx: tokio::sync::mpsc::Receiver<NodeEvent>,
    tx_loopback: mpsc::Sender<NodeEvent>,
    token: tokio_util::sync::CancellationToken,
    my_id: PeerIdType,
) {
    let mut session = session::Session::new(my_id);
    let mut writer = FramedWrite::new(tokio::io::stdout(), LengthDelimitedCodec::new());
    let mut peers: HashMap<PeerIdType, mpsc::Sender<PeerMessage>> = HashMap::new();

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                eprintln!("Shutting down event loop");
                break;
            }
            Some(event) = rx.recv() => {
                match event {
                    NodeEvent::PeerDiscovered { id, addr } => {
                        if !peers.contains_key(&id) && my_id < id {
                             let tx = tx_loopback.clone();
                             let tok = token.clone();
                             tokio::spawn(transport::connect_to_peer(addr, tx, tok, my_id));
                        }
                    }
                    NodeEvent::PeerConnected { id, sender } => {
                        peers.insert(id, sender);
                    }
                    NodeEvent::PeerDisconnected { id } => {
                        peers.remove(&id);
                    }
                    NodeEvent::User(op) => {
                        if let Some(remote_op) = session.handle_local_operation(op) {
                            transport::send_local_op(&op, &mut writer).await;
                            for (peer_id, tx) in peers.iter() {
                                let tx = tx.clone();
                                let msg = PeerMessage::SyncOp(remote_op.clone());
                                eprintln!("Sending peer message: {:?}", msg);
                                let peer_id = *peer_id;
                                tokio::spawn(async move {
                                    if let Err(_) = tx.send(msg).await {
                                        eprintln!("Failed to send to peer {}, channel closed", peer_id);
                                    }
                                });
                            }
                        }
                    }
                    NodeEvent::Network(msg) => {
                         let local_op = session.handle_network(msg);
                         transport::send_local_op(&local_op, &mut writer).await;
                    }
                }
            }
            else => break,
        }
    }
}
