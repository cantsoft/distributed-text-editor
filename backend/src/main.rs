mod config;
mod macros;
mod protocol;
mod service;
mod session;
mod state;
#[cfg(test)]
mod tests;
mod transport;
mod types;

#[tokio::main]
async fn main() {
    let config = match config::load_or_create("./native/config.toml") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("CRITICAL: Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = service::run(config).await {
        eprintln!("CRITICAL: Service crashed unexpectedly: {}", e);
        std::process::exit(2);
    }

    println!("Service stopped gracefully.");
    std::process::exit(0);
}
