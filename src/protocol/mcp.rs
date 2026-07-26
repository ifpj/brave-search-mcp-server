use serde_json::{json, Value};
use std::sync::Arc;

use crate::brave::client::BraveClient;
use crate::config::Config;
use crate::protocol::jsonrpc::{JsonRpcRequest, JsonRpcResponse};
use crate::tools::{self, ToolResult};

const SERVER_NAME: &str = "brave-search-mcp-server";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROTOCOL_VERSION: &str = "2025-03-26";
const INSTRUCTIONS: &str =
    "Use this server to search the Web for various types of data via the Brave Search API.";

/// Handle a JSON-RPC request, returning a JSON-RPC response.
pub async fn handle_request(
    req: JsonRpcRequest,
    config: &Config,
    brave: &BraveClient,
) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => handle_initialize(req.id),
        "initialized" => {
            // Notification — no response needed, but we return one for HTTP
            JsonRpcResponse::success(req.id, json!({}))
        }
        "ping" => JsonRpcResponse::success(req.id, json!({})),
        "tools/list" => handle_tools_list(req.id, config),
        "tools/call" => handle_tools_call(req.id, req.params, config, brave).await,
        _ => JsonRpcResponse::method_not_found(req.id, &req.method),
    }
}

fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    let result = json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "logging": {},
            "tools": { "listChanged": false }
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": SERVER_VERSION
        },
        "instructions": INSTRUCTIONS
    });
    JsonRpcResponse::success(id, result)
}

fn handle_tools_list(id: Option<Value>, config: &Config) -> JsonRpcResponse {
    let all_tools = tools::all_definitions();
    let filtered: Vec<_> = all_tools
        .into_iter()
        .filter(|t| config.is_tool_permitted(&t.name))
        .collect();

    let tools_json: Vec<Value> = filtered.iter().map(|t| t.to_json()).collect();

    JsonRpcResponse::success(id, json!({ "tools": tools_json }))
}

async fn handle_tools_call(
    id: Option<Value>,
    params: Option<Value>,
    config: &Config,
    brave: &BraveClient,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => return JsonRpcResponse::invalid_params(id, "Missing params"),
    };

    let tool_name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return JsonRpcResponse::invalid_params(id, "Missing tool name"),
    };

    if !config.is_tool_permitted(&tool_name) {
        return JsonRpcResponse::invalid_request(id, format!("Tool not found: {}", tool_name));
    }

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    match tools::execute(&tool_name, brave, arguments).await {
        Ok(result) => JsonRpcResponse::success(id, result.to_json()),
        Err(e) => JsonRpcResponse::internal_error(id, format!("Tool execution failed: {}", e)),
    }
}
