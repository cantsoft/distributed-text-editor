// mod doc;
// mod side;
// #[cfg(test)]
// mod tests;
// mod types;

// pub use doc::Doc;

// use std::io::{Write, stdin, stdout};

// fn main() {
//     let _ = stdout().write("hello form backend\n".as_bytes());
//     let mut str_buf = String::new();
//     let size_or_err = stdin().read_line(&mut str_buf);

//     println!("{:?}", size_or_err);
//     println!("{}", str_buf);
// }

use prost::Message;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/dte.rs"));
}

use proto::UserOperation;

fn main() {}
