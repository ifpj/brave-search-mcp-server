use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{json, Value};

use crate::brave::client::{BraveClient, Endpoint};
use crate::tools::{ToolDefinition, ToolResult};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "brave_place_search".into(),
        title: "Brave Place Search".into(),
        description: "Searches Brave's Place Search API. A single call may populate any combination of 'results' (POIs), 'cities', 'addresses', 'streets', and 'location' (the resolved search area), depending on the query's shape.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "radius": {"type": "number", "description": "Search radius in meters", "minimum": 0},
                "count": {"type": "integer", "description": "Number of results", "minimum": 1, "maximum": 50},
                "latitude": {"type": "number", "minimum": -90, "maximum": 90},
                "longitude": {"type": "number", "minimum": -180, "maximum": 180},
                "location": {"type": "string", "description": "Location string (e.g. 'san francisco ca united states')"},
                "country": {"type": "string", "description": "Country code"},
                "search_lang": {"type": "string", "description": "Search language"},
                "ui_lang": {"type": "string", "description": "UI language"},
                "units": {"type": "string", "enum": ["metric", "imperial"]},
                "safesearch": {"type": "string", "enum": ["off", "moderate", "strict"]},
                "spellcheck": {"type": "boolean"},
                "geoloc": {"type": "string"},
                "api-version": {"type": "string"},
                "accept": {"type": "string"},
                "cache-control": {"type": "string"},
                "user-agent": {"type": "string"}
            }
        }),
        output_schema: None,
    }
}

const HEADER_FIELDS: &[&str] = &["api-version", "accept", "cache-control", "user-agent"];

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
        .request(Endpoint::PlaceSearch, Value::Object(query_params), Some(headers))
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
