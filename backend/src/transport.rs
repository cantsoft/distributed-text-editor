use crate::config;
use crate::protocol;
use bytes::Bytes;
use futures::StreamExt;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
use tokio_util::sync::CancellationToken;

type PacketSender = mpsc::Sender<protocol::IngressPacket>;

pub async fn run_service() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = config::load_or_create("./native/config.toml").expect("failed to load config");

    let (tx, rx) = mpsc::channel(32);
    let token = CancellationToken::new();

    let tx_stdin = tx.clone();
    let token_stdin = token.clone();
    tokio::spawn(async move {
        stream_reader(
            tokio::io::stdin(),
            tx_stdin,
            token_stdin,
            protocol::IngressPacket::FromStdin,
        )
        .await;
    });

    let tx_discovery = tx.clone();
    let token_discovery = token.clone();
    let config_discovery = config.clone();
    tokio::spawn(async move {
        if let Err(e) = run_discovery(tx_discovery, token_discovery, config_discovery).await {
            eprintln!("Discovery crashed: {}", e);
        }
    });

    let tx_tcp = tx.clone();
    let token_tcp = token.clone();
    tokio::spawn(async move {
        if let Err(e) = run_tcp_server(tx_tcp, token_tcp, config.tcp_port).await {
            eprintln!("TCP Server crashed: {}", e);
        }
    });

    let tx_loopback = tx.clone();
    protocol::process_packets(rx, tx_loopback, token.clone(), config.peer_id).await;

    drop(tx);

    Ok(())
}

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

async fn run_discovery(
    tx: PacketSender,
    token: CancellationToken,
    config: config::AppConfig,
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

                            let packet = protocol::IngressPacket::FromDiscovered {
                                id: beacon.id,
                                addr: peer_tcp_addr,
                            };

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

async fn run_tcp_server(
    tx: PacketSender,
    token: CancellationToken,
    tcp_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", tcp_port);
    let listener = TcpListener::bind(&addr).await?;
    eprintln!("TCP Server listening on: {}", addr);
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                eprintln!("Stopping TCP server...");
                return Ok(());
            },

            accept_result = listener.accept() => {
                match accept_result {
                    Ok((socket, addr)) => {
                        eprintln!("New incoming TCP connection from: {}", addr);

                        let tx_inner = tx.clone();
                        let token_inner = token.clone();

                        tokio::spawn(async move {
                            stream_reader(
                                socket,
                                tx_inner,
                                token_inner,
                                protocol::IngressPacket::FromTcp
                            ).await;
                        });
                    }
                    Err(e) => eprintln!("TCP accept error: {}", e),
                }
            }
        }
    }
}

pub async fn handle_peer_connection(
    addr: SocketAddr,
    tx: mpsc::Sender<protocol::IngressPacket>,
    token: CancellationToken,
) {
    match TcpStream::connect(addr).await {
        Ok(socket) => {
            eprintln!("Connected with {}", addr);
            stream_reader(socket, tx, token, protocol::IngressPacket::FromTcp).await;
        }
        Err(e) => eprintln!("Failed to connect with {}: {}", addr, e),
    }
}

async fn stream_reader<R, F>(
    stream: R,
    tx: PacketSender,
    token: CancellationToken,
    packet_constructor: F,
) where
    R: AsyncRead + Unpin,
    F: Fn(Bytes) -> protocol::IngressPacket,
{
    let mut framed = FramedRead::new(stream, LengthDelimitedCodec::new());

    loop {
        tokio::select! {
            _ = token.cancelled() => return,

            maybe_frame = framed.next() => {
                match maybe_frame {
                    Some(Ok(bytes)) => {
                        let packet = packet_constructor(bytes.freeze());
                        if tx.send(packet).await.is_err() {
                            return;
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
