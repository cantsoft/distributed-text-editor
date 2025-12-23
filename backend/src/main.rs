// mod doc;
// mod side;
// #[cfg(test)]
// mod tests;
// mod types;

// pub use doc::Doc;

use prost::Message;
use std::io::{self, Read, Write, stdin, stdout};
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/dte.rs"));
}
use crate::proto::{UserInsert, UserOperation, user_operation};

fn main() -> io::Result<()> {
    let usr_op = UserOperation {
        position: 2,
        operation_type: Some(user_operation::OperationType::Insert(UserInsert {
            char: 97,
        })),
    };
    let binary = usr_op.encode_to_vec();
    let length = binary.len() as u32;
    let mut stdout = io::stdout().lock();
    stdout.write_all(&length.to_be_bytes()).expect("write len");
    stdout.write_all(&binary).expect("write body");
    stdout.flush().expect("flush");

    let mut stdin = io::stdin().lock();
    loop {
        let mut len_buf = [0u8; 4];
        if let Err(_) = stdin.read_exact(&mut len_buf) {
            break;
        }
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut payload_buf = vec![0u8; len];
        stdin.read_exact(&mut payload_buf)?;

        let op = UserOperation::decode(&payload_buf[..])?;
        println!("Received op at pos: {}", op.position);
    }

    Ok(())
}
