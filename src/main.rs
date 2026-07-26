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

#[derive(Parser, Debug)]
#[command(name = "brave-search-mcp-server")]
#[command(version = "2.1.0")]
#[command(about = "Brave Search MCP Server with multi-key load balancing", long_about = None)]
struct Cli {
    /// Brave Search API keys (comma-separated for load balancing)
    #[arg(short = 'k', long, env = "BRAVE_API_KEYS", value_delimiter = ',')]
    brave_api_keys: Vec<String>,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1", env = "MCP_HOST")]
    host: String,

    /// Port to listen on
    #[arg(short, long, default_value_t = 3000, env = "MCP_PORT")]
    port: u16,

    /// Allowed origins for CORS (comma-separated)
    #[arg(long, env = "MCP_ALLOWED_ORIGINS", value_delimiter = ',')]
    allowed_origins: Vec<String>,

    /// Allowed hosts for DNS rebinding protection (comma-separated)
    #[arg(long, env = "MCP_ALLOWED_HOSTS", value_delimiter = ',')]
    allowed_hosts: Vec<String>,

    /// Tools to enable (comma-separated, all enabled by default)
    #[arg(long, env = "MCP_ENABLED_TOOLS", value_delimiter = ',')]
    enabled_tools: Vec<String>,

    /// Tools to disable (comma-separated)
    #[arg(long, env = "MCP_DISABLED_TOOLS", value_delimiter = ',')]
    disabled_tools: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Validate API keys
    if cli.brave_api_keys.is_empty() {
        eprintln!("Error: At least one Brave API key is required. Use --brave-api-keys or set BRAVE_API_KEYS environment variable.");
        std::process::exit(1);
    }

    // Validate enabled/disabled tools (can't use both)
    if !cli.enabled_tools.is_empty() && !cli.disabled_tools.is_empty() {
        eprintln!("Error: Cannot specify both --enabled-tools and --disabled-tools");
        std::process::exit(1);
    }

    // Build config
    let config = Config {
        brave_api_keys: cli.brave_api_keys.clone(),
        host: cli.host.clone(),
        port: cli.port,
        allowed_origins: cli.allowed_origins.clone(),
        allowed_hosts: cli.allowed_hosts.clone(),
        enabled_tools: cli.enabled_tools.clone(),
        disabled_tools: cli.disabled_tools.clone(),
        log_level: "info".to_string(),
    };

    // Create Brave API client with key pool
    let brave_client = BraveClient::new(cli.brave_api_keys)?;

    // Create app state
    let state = AppState {
        config: Arc::new(config),
        brave_client: Arc::new(brave_client),
    };

    // Build router
    let app = create_router(state);

    // Bind to address
    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!("Brave Search MCP Server listening on {}", addr);
    println!("Endpoints:");
    println!("  POST /mcp    - MCP protocol endpoint");
    println!("  GET  /health - Health check");

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
