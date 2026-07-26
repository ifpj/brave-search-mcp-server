use serde_json::{json, Value};

use crate::brave::client::{BraveClient, Endpoint};
use crate::brave::types::VideoSearchResponse;
use crate::tools::{ToolDefinition, ToolResult};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "brave_video_search".into(),
        title: "Brave Video Search".into(),
        description: "Searches for videos using Brave's Video Search API and returns structured video results with metadata.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "country": {"type": "string", "description": "Country code (default: US)"},
                "search_lang": {"type": "string", "description": "Search language (default: en)"},
                "count": {"type": "integer", "description": "Number of results", "minimum": 1, "maximum": 200},
                "safesearch": {"type": "string", "enum": ["off", "strict"]},
                "spellcheck": {"type": "boolean"}
            },
            "required": ["query"]
        }),
        output_schema: None,
    }
}

pub async fn execute(client: &BraveClient, params: Value) -> Result<ToolResult, String> {
    let response: VideoSearchResponse = client
        .request(Endpoint::Videos, params, None)
        .await
        .map_err(|e| e.to_string())?;

    let items: Vec<String> = response
        .results
        .iter()
        .map(|r| {
            json!({
                "url": r.url,
                "title": r.title,
                "description": r.description,
                "duration": r.duration,
                "thumbnail_url": r.thumbnail.as_ref().and_then(|t| t.src.as_ref())
            })
            .to_string()
        })
        .collect();

    Ok(ToolResult::text(items))
}
