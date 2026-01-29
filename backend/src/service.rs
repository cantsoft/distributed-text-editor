use crate::session::Session;
use crate::types::PeerId;
use crate::{config, protocol, select_loop, transport};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use tokio_util::sync::CancellationToken;

pub async fn run(config: config::NodeConfig) -> Result<(), ()> {
    let (tx, rx) = mpsc::channel(255);
    let token = CancellationToken::new();

    let tx_stdin = tx.clone();
    let token_stdin = token.clone();
    tokio::spawn(async move {
        if let Err(e) = transport::run_stdin_listener(tx_stdin, token_stdin.clone()).await {
            eprintln!("Stdin listener crashed: {}", e);
            token_stdin.cancel();
        }
    });

    let tx_discovery = tx.clone();
    let token_discovery = token.clone();
    let config_discovery = config.clone();
    tokio::spawn(async move {
        if let Err(e) =
            transport::run_discovery(tx_discovery, token_discovery.clone(), config_discovery).await
        {
            eprintln!("Discovery crashed: {}", e);
            token_discovery.cancel();
        }
    });

    let tx_tcp = tx.clone();
    let token_tcp = token.clone();
    tokio::spawn(async move {
        if let Err(e) =
            transport::run_tcp_listener(tx_tcp, token_tcp.clone(), config.tcp_port).await
        {
            eprintln!("TCP Server crashed: {}", e);
            token_tcp.cancel();
        }
    });

    handle_events(rx, tx, token, config.peer_id).await
}

async fn handle_events(
    mut rx: tokio::sync::mpsc::Receiver<protocol::NodeEvent>,
    tx_loopback: mpsc::Sender<protocol::NodeEvent>,
    token: tokio_util::sync::CancellationToken,
    my_id: PeerId,
) -> Result<(), ()> {
    let save_path = "./native/doc.bin";
    let mut session = Session::from(my_id, save_path);
    let mut writer = FramedWrite::new(tokio::io::stdout(), LengthDelimitedCodec::new());
    let mut peers: HashMap<PeerId, mpsc::Sender<protocol::PeerSyncOp>> = HashMap::new();

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let init_sync = protocol::ServerEvent {
        variant: Some(protocol::server_event::Variant::State(
            protocol::FullState {
                content: session.get_doc_ascii(),
            },
        )),
    };
    transport::send_server_event(&init_sync, &mut writer).await;

    select_loop! {
        'main_loop:

        _ = token.cancelled() => {
            return Err(());
        }

        event = rx.recv() => {
            let Some(event) = event else {
                eprintln!("Event channel closed");
                break;
            };
            use protocol::NodeEvent;
            match event {
                NodeEvent::Net(event) => {
                    handle_peer_event(event, &mut peers, &session, &tx_loopback, &token, my_id);
                },
                NodeEvent::Local(protocol::ClientCommand{variant}) => {
                    match variant.unwrap() {
                        protocol::client_command::Variant::Edit(local_op) => handle_local_op(&mut session, local_op, &peers, &mut writer).await,
                        protocol::client_command::Variant::Save(protocol::SaveDocument{ filename }) => {
                            eprintln!("{}", filename);
                            if let Err(e)  = session.save_text(format!("./native/{}", filename).as_str()) {
                                eprintln!("Failed to save file: {}", e)
                            };
                        },
                        protocol::client_command::Variant::Close(_) => {
                            token.cancel();
                            break 'main_loop;
                        },
                    }
                }
                NodeEvent::Sync(sync_op) => {
                    let server_event = session.apply_peer_sync_op(sync_op);
                    transport::send_server_event(&server_event, &mut writer).await;
                }
            }
        }
    }

    if let Err(e) = session.save_bytes(save_path) {
        eprintln!("Failed to write {}: {}", save_path, e);
    }

    Ok(())
}

fn handle_peer_event(
    event: protocol::PeerEvent,
    peers: &mut HashMap<PeerId, mpsc::Sender<protocol::PeerSyncOp>>,
    session: &Session,
    tx_loopback: &mpsc::Sender<protocol::NodeEvent>,
    token: &tokio_util::sync::CancellationToken,
    my_id: PeerId,
) {
    use protocol::PeerEvent;

    match event {
        PeerEvent::Discovered { id, addr } => {
            if !peers.contains_key(&id) && my_id < id {
                let tx = tx_loopback.clone();
                let tok = token.clone();
                let doc_snapshot = session.get_doc_snapshot();

                tokio::spawn(transport::connect_to_peer(
                    addr,
                    tx,
                    tok,
                    doc_snapshot,
                    my_id,
                ));
            }
        }
        PeerEvent::Connection { stream } => {
            let tx = tx_loopback.clone();
            let tok = token.clone();
            let doc_snapshot = session.get_doc_snapshot();

            tokio::spawn(async move {
                transport::handle_connection(stream, tx, tok, doc_snapshot, my_id).await;
            });
        }
        PeerEvent::Connected { id, sender } => {
            peers.insert(id, sender);
        }
        PeerEvent::Disconnected { id } => {
            peers.remove(&id);
        }
    }
}

async fn handle_local_op(
    session: &mut Session,
    local_op: protocol::LocalOp,
    peers: &HashMap<PeerId, mpsc::Sender<protocol::PeerSyncOp>>,
    writer: &mut FramedWrite<tokio::io::Stdout, LengthDelimitedCodec>,
) {
    match session.apply_local_op(local_op.clone()) {
        Some(remote_op) => {
            let server_event = protocol::ServerEvent {
                variant: Some(protocol::server_event::Variant::Op(local_op)),
            };
            transport::send_server_event(&server_event, writer).await;

            for (peer_id, tx) in peers.iter() {
                let tx = tx.clone();
                let msg = remote_op.clone();
                let peer_id = *peer_id;

                tokio::spawn(async move {
                    if let Err(_) = tx.send(msg).await {
                        eprintln!("Failed to send to peer {}, channel closed", peer_id);
                    }
                });
            }
        }
        None => {
            panic!("Failed to apply operation");
        }
    }
}
