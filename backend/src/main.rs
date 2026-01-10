mod config;
mod doc;
mod protocol;
mod session;
mod transport;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/dte.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    transport::run_service().await
}
