// mod doc;
// mod side;
// #[cfg(test)]
// mod tests;
// mod types;

// pub use doc::Doc;

// use prost::Message;
// use std::io::{self, Read, Write};
// pub mod proto {
//     include!(concat!(env!("OUT_DIR"), "/dte.rs"));
// }
// use crate::proto::{UserInsert, UserOperation, user_operation};

use bytes::Bytes;
use futures::StreamExt;
use std::error::Error;
use tokio::io::{AsyncRead, stdin};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
use tokio_util::sync::CancellationToken;

type ServiceResult = Result<(), Box<dyn Error>>;
type PacketSender = mpsc::Sender<IngressPacket>;

#[derive(Debug)]
enum IngressPacket {
    FromStdin(Bytes),
    FromTcp(Bytes),
}

async fn stream_reader<R, F>(stream: R, tx: PacketSender, token: CancellationToken, mapper: F)
where
    R: AsyncRead + Unpin,
    F: Fn(Bytes) -> IngressPacket,
{
    let mut framed = FramedRead::new(stream, LengthDelimitedCodec::new());

    loop {
        tokio::select! {
            _ = token.cancelled() => return,
            maybe_frame = framed.next() => {
                match maybe_frame {
                    Some(Ok(bytes)) => {
                        let packet = mapper(bytes.freeze());
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

async fn run_tcp_server(tx: PacketSender, token: CancellationToken) -> ServiceResult {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

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
                                IngressPacket::FromTcp
                            ).await;
                        });
                    }
                    Err(e) => eprintln!("TCP accept error: {}", e),
                }
            }
        }
    }
}

async fn process_packets(mut rx: mpsc::Receiver<IngressPacket>, token: CancellationToken) {
    let mut cnt = 0;
    while let Some(packet) = rx.recv().await {
        match packet {
            IngressPacket::FromStdin(bytes) => {
                eprintln!("{} IPC payload size: {}", cnt, bytes.len());
                cnt += 1;
                // If message == ShutdownCommand -> token.cancel(); break;
            }
            IngressPacket::FromTcp(bytes) => {
                eprintln!("Network payload size: {}", bytes.len());
            }
        }
    }
}

#[tokio::main]
async fn main() -> ServiceResult {
    let (tx, rx) = mpsc::channel(32);
    let token = CancellationToken::new();

    let tx_stdin = tx.clone();
    let token_stdin = token.clone();
    tokio::spawn(async move {
        stream_reader(stdin(), tx_stdin, token_stdin, IngressPacket::FromStdin).await;
    });

    let tx_tcp = tx.clone();
    let token_tcp = token.clone();
    tokio::spawn(async move {
        if let Err(e) = run_tcp_server(tx_tcp, token_tcp).await {
            eprintln!("TCP Server crashed: {}", e);
        }
    });

    drop(tx);

    process_packets(rx, token.clone()).await;

    Ok(())
}
