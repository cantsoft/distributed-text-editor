mod doc;
mod protocol;
mod side;
#[cfg(test)]
mod tests;
mod transport;
mod types;

// use prost::Message;
// use std::io::{self, Read, Write};
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/dte.rs"));
}

#[tokio::main]
async fn main() -> transport::ServiceResult {
    transport::run_service().await
}
