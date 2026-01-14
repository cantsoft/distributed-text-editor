use crate::config;
use crate::protocol::{self, NodeEvent, PeerMessage};
use crate::state::PeerIdType;
use crate::transport::codec::PeerMessageCodec;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tokio_util::sync::CancellationToken;

type PacketSender = mpsc::Sender<protocol::NodeEvent>;

fn bind_udp_shared(port: u16) -> std::io::Result<tokio::net::UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;
    let address = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
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

    let broadcast_target = format!("255.255.255.255:{}", config.udp_discovery_port);
    let msg_bytes = serde_json::to_vec(&beacon).unwrap();
    let mut buf = [0u8; 1024];

    loop {
        tokio::select! {
            _ = token.cancelled() => return Ok(()),

            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                if let Err(e) = socket.send_to(&msg_bytes, &broadcast_target).await {
                    eprintln!("Discovery broadcast error: {}", e);
                }
            }

            recv_result = socket.recv_from(&mut buf) => {
                match recv_result {
                    Ok((len, remote_addr)) => {
                        if let Ok(beacon) = serde_json::from_slice::<protocol::PeerBeacon>(&buf[..len]) {
                            if beacon.id == config.peer_id { continue; }

                            let peer_tcp_addr = std::net::SocketAddr::new(
                                remote_addr.ip(),
                                beacon.tcp_port
                            );

                            let packet = protocol::NodeEvent::PeerDiscovered { id: beacon.id, addr: peer_tcp_addr };

                            if let Err(e) = tx.send(packet).await {
                                eprintln!("Discover package send error: {}", e);
                            }
                        }
                    }
                    Err(e) => eprintln!("Discovery recv error: {}", e),
                }
            }
        }
    }
}

pub async fn connect_to_peer(
    addr: SocketAddr,
    tx: PacketSender,
    token: CancellationToken,
    my_id: PeerIdType,
) {
    eprintln!("Connecting to peer at {}", addr);
    match TcpStream::connect(addr).await {
        Ok(stream) => {
            handle_connection(stream, tx, token, my_id).await;
        }
        Err(e) => eprintln!("Failed to connect to {}: {}", addr, e),
    }
}

pub async fn run_tcp_server(
    tx: PacketSender,
    token: CancellationToken,
    tcp_port: u16,
    my_id: PeerIdType,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr_str = format!("0.0.0.0:{}", tcp_port);
    let listener = TcpListener::bind(&addr_str).await?;
    eprintln!("TCP Server listening on: {}", addr_str);

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                eprintln!("Stopping TCP server...");
                return Ok(());
            },
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, addr)) => {
                        eprintln!("New incoming TCP connection from: {}", addr);
                        let tx_inner = tx.clone();
                        let token_inner = token.clone();
                        tokio::spawn(async move {
                            handle_connection(stream, tx_inner, token_inner, my_id).await;
                        });
                    }
                    Err(e) => eprintln!("TCP accept error: {}", e),
                }
            }
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    tx: PacketSender,
    token: CancellationToken,
    my_id: PeerIdType,
) {
    if let Err(e) = stream.write_all(&[my_id]).await {
        eprintln!("Handshake write error: {}", e);
        return;
    }

    let mut buf = [0u8; 1];
    if let Err(e) = stream.read_exact(&mut buf).await {
        eprintln!("Handshake read error: {}", e);
        return;
    }
    let peer_id = buf[0];
    eprintln!("Handshake successful. Connected with peer {}", peer_id);

    let (read_half, write_half) = stream.into_split();
    let mut framed_read = FramedRead::new(read_half, PeerMessageCodec::new());
    let mut framed_write = FramedWrite::new(write_half, PeerMessageCodec::new());

    let (tx_peer, mut rx_peer) = mpsc::channel::<PeerMessage>(32);

    if let Err(e) = tx
        .send(NodeEvent::PeerConnected {
            id: peer_id,
            sender: tx_peer,
        })
        .await
    {
        eprintln!("Failed to send PeerConnected event: {}", e);
        return;
    }

    let write_token = token.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = write_token.cancelled() => return,
                msg = rx_peer.recv() => {
                    match msg {
                        Some(m) => {
                            if let Err(e) = framed_write.send(m).await {
                                eprintln!("Write error to peer {}: {}", peer_id, e);
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    loop {
        tokio::select! {
            _ = token.cancelled() => break,
            frame = framed_read.next() => {
                match frame {
                    Some(Ok(msg)) => {
                        if let Err(e) = tx.send(NodeEvent::Network {
                            from: peer_id,
                            payload: msg,
                        }).await {
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
    }

    let _ = tx.send(NodeEvent::PeerDisconnected { id: peer_id }).await;
    eprintln!("Disconnected from peer {}", peer_id);
}

pub async fn stream_reader<R, F>(
    stream: R,
    tx: PacketSender,
    token: CancellationToken,
    packet_constructor: F,
) where
    R: AsyncRead + Unpin,
    F: Fn(Bytes) -> Option<protocol::NodeEvent>,
{
    let mut framed = FramedRead::new(stream, LengthDelimitedCodec::new());

    loop {
        tokio::select! {
            _ = token.cancelled() => return,

            maybe_frame = framed.next() => {
                match maybe_frame {
                    Some(Ok(bytes)) => {
                        if let Some(packet) = packet_constructor(bytes.freeze()) {
                            if tx.send(packet).await.is_err() {
                                return;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Framing error: {}", e);
                    }
                    None => return,
                }
            }
        }
    }
}
