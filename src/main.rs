use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

mod brave;
mod config;
mod server;
mod tools;

use brave::client::BraveClient;
use config::Config;
use server::{create_router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments and environment variables
    let mut config = Config::parse();

    // Load keys from file if specified
    config.load_keys()?;

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    println!("Loaded {} API keys", config.brave_api_keys.len());

    // Create Brave API client with key pool
    let brave_client = BraveClient::new(config.brave_api_keys.clone())?;

    // Create app state
    let state = AppState {
        config: Arc::new(config.clone()),
        brave_client: Arc::new(brave_client),
    };

    // Build router
    let app = create_router(state);

    // Bind to address
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!("Brave Search MCP Server listening on {}", addr);
    println!("Endpoints:");
    println!("  POST /mcp         - MCP protocol endpoint");
    println!("  GET  /health      - Health check");
    println!("  GET  /keys        - Key pool statistics");
    println!("  GET  /keys/summary - Key pool summary");

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
