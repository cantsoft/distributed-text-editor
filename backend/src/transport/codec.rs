use crate::protocol::{ClientCommand, PeerSyncOp, ServerEvent};
use bytes::{Bytes, BytesMut};
use prost::Message;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

pub struct PeerSyncOpCodec {
    delegate: LengthDelimitedCodec,
}

impl PeerSyncOpCodec {
    pub fn new() -> Self {
        Self {
            delegate: LengthDelimitedCodec::new(),
        }
    }
}

impl Encoder<PeerSyncOp> for PeerSyncOpCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: PeerSyncOp, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data = bincode::serialize(&item)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.delegate.encode(Bytes::from(data), dst)
    }
}

impl Decoder for PeerSyncOpCodec {
    type Item = PeerSyncOp;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.delegate.decode(src)? {
            Some(frame) => {
                let msg = bincode::deserialize(&frame)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                Ok(Some(msg))
            }
            None => Ok(None),
        }
    }
}

pub fn try_decode_op(bytes: bytes::BytesMut) -> Option<ClientCommand> {
    match ClientCommand::decode(bytes) {
        Ok(cmd) => Some(cmd),
        Err(e) => {
            eprintln!("Invalid protobuf on stdin: {}", e);
            None
        }
    }
}

pub fn encode_protobuf(msg: &ServerEvent) -> Result<Bytes, prost::EncodeError> {
    let mut buf = BytesMut::with_capacity(msg.encoded_len());
    msg.encode(&mut buf)?;
    Ok(buf.freeze())
}
