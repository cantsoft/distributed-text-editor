use crate::protocol::{LocalOperation, PeerMessage};
use bytes::{Bytes, BytesMut};
use prost::Message;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

pub struct PeerMessageCodec {
    delegate: LengthDelimitedCodec,
}

impl PeerMessageCodec {
    pub fn new() -> Self {
        Self {
            delegate: LengthDelimitedCodec::new(),
        }
    }
}

impl Encoder<PeerMessage> for PeerMessageCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: PeerMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data = bincode::serialize(&item)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.delegate.encode(Bytes::from(data), dst)
    }
}

impl Decoder for PeerMessageCodec {
    type Item = PeerMessage;
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

pub fn encode_protobuf(msg: &LocalOperation) -> Result<Bytes, prost::EncodeError> {
    let mut buf = BytesMut::with_capacity(msg.encoded_len());
    msg.encode(&mut buf)?;
    Ok(buf.freeze())
}
