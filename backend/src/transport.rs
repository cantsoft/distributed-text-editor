use crate::protocol;
use bytes::Bytes;
use futures::StreamExt;
use tokio::io::AsyncRead;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
use tokio_util::sync::CancellationToken;

type PacketSender = mpsc::Sender<protocol::IngressPacket>;

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

async fn run_tcp_server(
    tx: PacketSender,
    token: CancellationToken,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234").await?;
    loop {
        tokio::select! {
            _ = token.cancelled() => return Ok(()),
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((socket, addr)) => {
                        eprintln!("New TCP connection: {}", addr);
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

pub async fn run_service() -> Result<(), Box<dyn std::error::Error>> {
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

    let tx_tcp = tx.clone();
    let token_tcp = token.clone();
    tokio::spawn(async move {
        if let Err(e) = run_tcp_server(tx_tcp, token_tcp).await {
            eprintln!("TCP Server crashed: {}", e);
        }
    });

    drop(tx);

    protocol::process_packets(rx, token.clone()).await;

    Ok(())
}
