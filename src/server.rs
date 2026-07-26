use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};

use crate::brave::client::BraveClient;
use crate::config::Config;
use crate::tools;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub brave_client: Arc<BraveClient>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp))
        .route("/health", get(health_check))
        .route("/keys", get(handle_key_stats))
        .route("/keys/summary", get(handle_key_summary))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

async fn health_check() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

async fn handle_key_stats(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "keys": state.brave_client.key_stats()
    }))
}

async fn handle_key_summary(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.brave_client.key_summary())
}

async fn handle_mcp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // DNS rebinding protection
    if !state.config.allowed_origins.is_empty() || !state.config.allowed_hosts.is_empty() {
        validate_origin_and_host(&headers, &state.config)?;
    }

    // Parse JSON-RPC request
    let method = body
        .get("method")
        .and_then(|m| m.as_str())
        .ok_or((StatusCode::BAD_REQUEST, "Missing 'method' field".to_string()))?;

    let id = body.get("id").cloned();
    let params = body.get("params").cloned();

    // Route to appropriate handler
    let result = match method {
        "initialize" => handle_initialize(params),
        "tools/list" => handle_tools_list(&state.config),
        "tools/call" => handle_tools_call(params, &state).await,
        _ => Err((
            StatusCode::BAD_REQUEST,
            format!("Unknown method: {}", method),
        )),
    }?;

    // Wrap in JSON-RPC response
    Ok(Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })))
}

fn handle_initialize(_params: Option<Value>) -> Result<Value, (StatusCode, String)> {
    Ok(json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "brave-search-mcp-server",
            "version": "2.1.0"
        }
    }))
}

fn handle_tools_list(config: &Config) -> Result<Value, (StatusCode, String)> {
    let all_tools = tools::all_definitions();
    let tools: Vec<Value> = all_tools
        .into_iter()
        .filter(|tool| config.is_tool_permitted(&tool.name))
        .map(|tool| tool.to_json())
        .collect();

    Ok(json!({ "tools": tools }))
}

async fn handle_tools_call(
    params: Option<Value>,
    state: &AppState,
) -> Result<Value, (StatusCode, String)> {
    let params = params.ok_or((StatusCode::BAD_REQUEST, "Missing params".to_string()))?;

    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or((StatusCode::BAD_REQUEST, "Missing tool name".to_string()))?;

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    // Check if tool is permitted
    if !state.config.is_tool_permitted(name) {
        return Err((
            StatusCode::FORBIDDEN,
            format!("Tool '{}' is not permitted", name),
        ));
    }

    // Execute tool
    let result = tools::execute(name, &state.brave_client, arguments)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(result.to_json())
}

fn validate_origin_and_host(
    headers: &HeaderMap,
    config: &Config,
) -> Result<(), (StatusCode, String)> {
    // Check Origin header
    if !config.allowed_origins.is_empty() {
        if let Some(origin) = headers.get("origin").and_then(|o| o.to_str().ok()) {
            let origin_allowed = config.allowed_origins.iter().any(|allowed| {
                origin == allowed || origin.starts_with(&format!("{}:", allowed))
            });
            if !origin_allowed {
                return Err((
                    StatusCode::FORBIDDEN,
                    format!("Origin '{}' not allowed", origin),
                ));
            }
        }
    }

    // Check Host header
    if !config.allowed_hosts.is_empty() {
        if let Some(host) = headers.get("host").and_then(|h| h.to_str().ok()) {
            let host_allowed = config.allowed_hosts.iter().any(|allowed| {
                host == allowed || host.starts_with(&format!("{}:", allowed))
            });
            if !host_allowed {
                return Err((
                    StatusCode::FORBIDDEN,
                    format!("Host '{}' not allowed", host),
                ));
            }
        } else {
            return Err((StatusCode::BAD_REQUEST, "Missing Host header".to_string()));
        }
    }

    Ok(())
}
