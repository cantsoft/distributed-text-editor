mod config;
mod protocol;
mod service;
mod session;
mod state;
mod transport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load_or_create("./native/config.toml").expect("failed to load config");
    service::run(config).await
}
