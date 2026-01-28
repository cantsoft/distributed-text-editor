use super::codec;
use crate::types::PeerId;
use crate::{config, protocol, select_loop, state};
use futures::{SinkExt, StreamExt};
use socket2::{Domain, Protocol, Socket, Type};
use std::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tokio_util::sync::CancellationToken;

type PacketSender = mpsc::Sender<protocol::NodeEvent>;

pub async fn run_stdin_listener(tx: PacketSender, token: CancellationToken) -> std::io::Result<()> {
    let stdin = tokio::io::stdin();
    let mut framed = FramedRead::new(stdin, LengthDelimitedCodec::new());

    select_loop! {
        _ = token.cancelled() => return Ok(()),

        maybe_frame = framed.next() => {
            match maybe_frame {
                Some(Ok(bytes)) => {
                    if let Some(cmd) = codec::try_decode_op(bytes) {
                        if let Err(e) = tx.send(protocol::NodeEvent::Local(cmd)).await {
                            return Err(std::io::Error::new(ErrorKind::BrokenPipe, e));
                        }
                    }
                }
                Some(Err(e)) => {
                    eprintln!("Stdin framing error: {}", e);
                }
                None => return Ok(()),
            }
        }
    }
}

fn bind_udp_shared(port: u16) -> std::io::Result<tokio::net::UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;
    let address =
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), port);
    socket.bind(&address.into())?;
    let std_socket: std::net::UdpSocket = socket.into();
    tokio::net::UdpSocket::from_std(std_socket)
}

pub async fn run_discovery(
    tx: PacketSender,
    token: CancellationToken,
    config: config::NodeConfig,
) -> std::io::Result<()> {
    let socket = bind_udp_shared(config.udp_discovery_port)?;

    let beacon = protocol::PeerBeacon {
        id: config.peer_id,
        tcp_port: config.tcp_port,
    };

    let msg_bytes =
        bincode::serialize(&beacon).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;

    let broadcast_target = format!("255.255.255.255:{}", config.udp_discovery_port);
    let mut buf = [0u8; 1024];

    select_loop! {
        _ = token.cancelled() => return Ok(()),

        _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
            if let Err(e) = socket.send_to(&msg_bytes, &broadcast_target).await {
                eprintln!("Discovery broadcast warn: {}", e);
            }
        }

        recv_result = socket.recv_from(&mut buf) => {
            match recv_result {
                Ok((len, remote_addr)) => {
                    if let Ok(remote_beacon) = bincode::deserialize::<protocol::PeerBeacon>(&buf[..len]) {

                        if remote_beacon.id == config.peer_id { continue; }

                        let peer_tcp_addr = std::net::SocketAddr::new(
                            remote_addr.ip(),
                            remote_beacon.tcp_port
                        );

                        if let Err(e) = tx.send(protocol::NodeEvent::Net(protocol::PeerEvent::Discovered {
                            id: remote_beacon.id,
                            addr: peer_tcp_addr
                        })).await {
                            eprintln!("Failed to send PeerDiscovered: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Discovery recv warn: {}", e);
                }
            }
        }
    }
}

pub async fn run_tcp_listener(
    tx: PacketSender,
    token: CancellationToken,
    tcp_port: u16,
) -> std::io::Result<()> {
    let addr_str = format!("0.0.0.0:{}", tcp_port);
    let listener = TcpListener::bind(&addr_str).await?;
    eprintln!("TCP listening on: {}", addr_str);

    select_loop! {
        _ = token.cancelled() => return Ok(()),

        accept_result = listener.accept() => {
            match accept_result {
                Ok((stream, addr)) => {
                    eprintln!("New incoming TCP connection from: {}", addr);
                    if let Err(e) = tx.send(protocol::NodeEvent::Net(protocol::PeerEvent::Connection { stream })).await
                    {
                        eprintln!("Failed to send PeerConnection: {}", e);
                    }
                }
                Err(e) => eprintln!("TCP accept error: {}", e),
            }
        }
    }
}

pub async fn connect_to_peer(
    addr: std::net::SocketAddr,
    tx: PacketSender,
    token: CancellationToken,
    doc_state: state::Doc,
    my_id: PeerId,
) {
    eprintln!("Connecting to peer at {}", addr);
    match TcpStream::connect(addr).await {
        Ok(stream) => {
            handle_connection(stream, tx, token, doc_state, my_id).await;
        }
        Err(e) => eprintln!("Failed to connect to {}: {}", addr, e),
    }
}

pub async fn handle_connection(
    mut stream: TcpStream,
    tx: PacketSender,
    token: CancellationToken,
    doc_state: state::Doc,
    my_id: PeerId,
) {
    let peer_id = match async {
        stream.write_all(&[my_id]).await?;
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;
        Ok::<_, std::io::Error>(buf[0])
    }
    .await
    {
        Ok(id) => {
            eprintln!("Handshake successful. Connected with peer {}", id);
            id
        }
        Err(e) => {
            eprintln!("Handshake failed: {}", e);
            return;
        }
    };

    let (read_half, write_half) = stream.into_split();
    let mut framed_read = FramedRead::new(read_half, codec::PeerSyncOpCodec::new());
    let mut framed_write = FramedWrite::new(write_half, codec::PeerSyncOpCodec::new());

    if let Err(e) = framed_write
        .send(protocol::PeerSyncOp::FullSync { state: doc_state })
        .await
    {
        eprintln!("Failed to send initial state to peer {}: {}", peer_id, e);
        return;
    }

    let (tx_peer, rx_peer) = mpsc::channel::<protocol::PeerSyncOp>(255);
    if let Err(e) = tx
        .send(protocol::NodeEvent::Net(protocol::PeerEvent::Connected {
            id: peer_id,
            sender: tx_peer,
        }))
        .await
    {
        eprintln!("Failed to send PeerConnected event: {}", e);
        return;
    }

    let write_token = token.clone();
    tokio::spawn(async move {
        run_writer_loop(framed_write, rx_peer, write_token, peer_id).await;
    });

    select_loop! {
        _ = token.cancelled() => break,

        frame = framed_read.next() => {
            match frame {
                Some(Ok(msg)) => {
                    if let Err(e) = tx.send(protocol::NodeEvent::Sync(msg)).await {
                        eprintln!("Failed to forward message from peer {}: {}", peer_id, e);
                        break;
                    }
                }
                Some(Err(e)) => {
                    eprintln!("Read error from peer {}: {}", peer_id, e);
                    break;
                }
                None => {
                    eprintln!("Connection closed by peer {}", peer_id);
                    break;
                }
            }
        }
    }

    eprintln!("Disconnected from peer {}", peer_id);
    let _ = tx
        .send(protocol::NodeEvent::Net(
            protocol::PeerEvent::Disconnected { id: peer_id },
        ))
        .await;
}

async fn run_writer_loop(
    mut framed_write: FramedWrite<tokio::net::tcp::OwnedWriteHalf, codec::PeerSyncOpCodec>,
    mut rx_peer: mpsc::Receiver<protocol::PeerSyncOp>,
    token: CancellationToken,
    peer_id: u8,
) {
    select_loop! {
        _ = token.cancelled() => return,

        msg = rx_peer.recv() => {
            match msg {
                Some(m) => {
                    if let Err(e) = framed_write.send(m).await {
                        eprintln!("Write error to peer {}: {}", peer_id, e);
                        return;
                    }
                }
                None => return,
            }
        }
    }
}

pub async fn send_server_event(
    event: &protocol::ServerEvent,
    writer: &mut FramedWrite<tokio::io::Stdout, LengthDelimitedCodec>,
) {
    let Ok(bytes) = codec::encode_protobuf(event) else {
        eprintln!("Protobuf encoding failed");
        return;
    };
    if let Err(e) = writer.send(bytes).await {
        eprintln!("Failed to write to stdout: {}", e);
    }
}
