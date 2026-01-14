use crate::protocol::{NodeEvent, PeerMessage};
use crate::state::PeerIdType;
use crate::{config, protocol, session, transport};
use futures::SinkExt;
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
        if let Err(e) = transport::run_tcp_server(tx_tcp, token_tcp, config.tcp_port, my_id).await {
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

    while let Some(event) = rx.recv().await {
        if token.is_cancelled() {
            break;
        }
        match event {
            NodeEvent::PeerDiscovered { id, addr } => {
                if peers.contains_key(&id) {
                    continue;
                }

                if my_id > id {
                    eprintln!("id ignored due to ordering: {}", id);
                    continue;
                }

                eprintln!("initializing id {} on addr {}", id, addr);

                let tx_clone = tx_loopback.clone();
                let token_clone = token.clone();
                tokio::spawn(transport::connect_to_peer(
                    addr,
                    tx_clone,
                    token_clone,
                    my_id,
                ));
            }
            NodeEvent::PeerConnected { id, sender } => {
                eprintln!("Peer connected: {}", id);
                peers.insert(id, sender);
            }
            NodeEvent::PeerDisconnected { id } => {
                eprintln!("Peer disconnected: {}", id);
                peers.remove(&id);
            }
            NodeEvent::User(op) => {
                session.handle_local_operation(op);
                let Ok(bytes) = transport::encode_protobuf(&op) else {
                    eprintln!("Protobuf encoding failed");
                    continue;
                };
                if let Err(e) = writer.send(bytes).await {
                    eprintln!("Failed to write to stdout: {}", e);
                }
            }
            NodeEvent::Network { from, payload } => {
                eprintln!("Network message from {}: {:?}", from, payload);
            }
        }
    }
}
