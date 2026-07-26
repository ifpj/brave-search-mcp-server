use clap::Parser;
use std::fs;
use std::path::PathBuf;

/// Brave Search MCP Server configuration.
#[derive(Parser, Debug, Clone)]
#[command(name = "brave-search-mcp-server", version, about)]
pub struct Config {
    /// Brave API keys (comma-separated for round-robin load balancing).
    #[arg(
        long,
        env = "BRAVE_API_KEYS",
        value_delimiter = ',',
        alias = "brave-api-key"
    )]
    pub brave_api_keys: Vec<String>,

    /// Path to file containing Brave API keys (one per line, supports # comments).
    #[arg(long, env = "BRAVE_API_KEYS_FILE", alias = "brave-api-keys-file")]
    pub brave_api_keys_file: Option<PathBuf>,

    /// Host to bind to.
    #[arg(long, env = "BRAVE_MCP_HOST", default_value = "127.0.0.1")]
    pub host: String,

    /// Port to listen on.
    #[arg(long, env = "BRAVE_MCP_PORT", default_value_t = 8080)]
    pub port: u16,

    /// Allowed Origin header values (DNS rebinding protection).
    #[arg(long, env = "BRAVE_MCP_ALLOWED_ORIGINS", value_delimiter = ',')]
    pub allowed_origins: Vec<String>,

    /// Allowed Host header values (opt-in DNS rebinding protection).
    #[arg(long, env = "BRAVE_MCP_ALLOWED_HOSTS", value_delimiter = ',')]
    pub allowed_hosts: Vec<String>,

    /// Tools to enable (space or comma separated). If set, only these tools are available.
    #[arg(long, env = "BRAVE_MCP_ENABLED_TOOLS", value_delimiter = ',')]
    pub enabled_tools: Vec<String>,

    /// Tools to disable (space or comma separated). Cannot be used with --enabled-tools.
    #[arg(long, env = "BRAVE_MCP_DISABLED_TOOLS", value_delimiter = ',')]
    pub disabled_tools: Vec<String>,

    /// Logging level.
    #[arg(long, env = "BRAVE_MCP_LOG_LEVEL", default_value = "info")]
    pub log_level: String,
}

impl Config {
    /// Loads API keys from file if specified, merging with command-line keys.
    /// File format: one key per line, lines starting with # are comments, empty lines are ignored.
    pub fn load_keys(&mut self) -> Result<(), String> {
        if let Some(ref path) = self.brave_api_keys_file {
            let content = fs::read_to_string(path).map_err(|e| {
                format!("Failed to read keys file '{}': {}", path.display(), e)
            })?;

            let file_keys: Vec<String> = content
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .collect();

            if file_keys.is_empty() {
                return Err(format!(
                    "No valid keys found in file '{}'",
                    path.display()
                ));
            }

            // Merge file keys with command-line keys (file keys first)
            let mut all_keys = file_keys;
            all_keys.extend(self.brave_api_keys.drain(..));
            self.brave_api_keys = all_keys;
        }
        Ok(())
    }

    /// Returns true if the given tool name is permitted by the user's enable/disable lists.
    pub fn is_tool_permitted(&self, tool_name: &str) -> bool {
        if !self.enabled_tools.is_empty() {
            self.enabled_tools.iter().any(|t| t == tool_name)
        } else {
            !self.disabled_tools.iter().any(|t| t == tool_name)
        }
    }

    /// Validates the configuration, returning an error message if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.brave_api_keys.is_empty() {
            return Err(
                "At least one Brave API key is required via --brave-api-keys, --brave-api-keys-file, or BRAVE_API_KEYS"
                    .into(),
            );
        }
        if !self.enabled_tools.is_empty() && !self.disabled_tools.is_empty() {
            return Err("--enabled-tools and --disabled-tools cannot be used together".into());
        }
        Ok(())
    }
}
