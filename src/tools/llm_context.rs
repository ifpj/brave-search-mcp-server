use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{json, Value};

use crate::brave::client::{BraveClient, Endpoint};
use crate::tools::{ToolDefinition, ToolResult};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "brave_llm_context".into(),
        title: "Brave LLM Context".into(),
        description: "Retrieves pre-extracted, relevance-ranked web content using Brave's LLM Context API, optimized for AI agents, LLM grounding, and RAG pipelines. Returns the actual substance of matching pages — text chunks, tables, code blocks, and structured data — so the model can reason over it directly.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "country": {"type": "string", "description": "Country code"},
                "search_lang": {"type": "string", "description": "Search language"},
                "count": {"type": "integer", "description": "Number of results", "minimum": 1, "maximum": 50},
                "spellcheck": {"type": "boolean"},
                "maximum_number_of_urls": {"type": "integer", "minimum": 1, "maximum": 50},
                "maximum_number_of_tokens": {"type": "integer", "minimum": 1024, "maximum": 32768},
                "maximum_number_of_snippets": {"type": "integer", "minimum": 1, "maximum": 256},
                "context_threshold_mode": {"type": "string", "enum": ["disabled", "strict", "lenient", "balanced"]},
                "maximum_number_of_tokens_per_url": {"type": "integer", "minimum": 512, "maximum": 8192},
                "maximum_number_of_snippets_per_url": {"type": "integer", "minimum": 1, "maximum": 100},
                "goggles": {"description": "Goggle(s) for custom ranking"},
                "freshness": {"type": "string", "description": "Time filter"},
                "enable_local": {"type": "boolean"},
                "enable_source_metadata": {"type": "boolean"},
                "x-loc-lat": {"type": "number", "minimum": -90, "maximum": 90},
                "x-loc-long": {"type": "number", "minimum": -180, "maximum": 180},
                "x-loc-city": {"type": "string"},
                "x-loc-state": {"type": "string", "maxLength": 3},
                "x-loc-state-name": {"type": "string"},
                "x-loc-country": {"type": "string", "minLength": 2, "maxLength": 2},
                "x-loc-postal-code": {"type": "string"},
                "api-version": {"type": "string"},
                "accept": {"type": "string"},
                "cache-control": {"type": "string"},
                "user-agent": {"type": "string"}
            },
            "required": ["query"]
        }),
        output_schema: None,
    }
}

const HEADER_FIELDS: &[&str] = &[
    "x-loc-lat",
    "x-loc-long",
    "x-loc-city",
    "x-loc-state",
    "x-loc-state-name",
    "x-loc-country",
    "x-loc-postal-code",
    "api-version",
    "accept",
    "cache-control",
    "user-agent",
];

pub async fn execute(client: &BraveClient, params: Value) -> Result<ToolResult, String> {
    let obj = params.as_object().ok_or("Params must be an object")?;

    // Split into query params and headers
    let mut query_params = serde_json::Map::new();
    let mut headers = HeaderMap::new();

    for (key, value) in obj {
        if HEADER_FIELDS.contains(&key.as_str()) {
            if let Some(s) = value_to_header_string(value) {
                if let (Ok(name), Ok(val)) = (
                    HeaderName::try_from(key.as_str()),
                    HeaderValue::try_from(&s),
                ) {
                    headers.insert(name, val);
                }
            }
        } else {
            query_params.insert(key.clone(), value.clone());
        }
    }

    let response: Value = client
        .request(Endpoint::LlmContext, Value::Object(query_params), Some(headers))
        .await
        .map_err(|e| e.to_string())?;

    let text = serde_json::to_string_pretty(&response).unwrap_or_default();

    Ok(ToolResult::structured(vec![text.clone()], response))
}

fn value_to_header_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}
