use serde_json::{json, Value};

use crate::brave::client::{BraveClient, Endpoint};
use crate::brave::types::ImageSearchResponse;
use crate::tools::{ToolDefinition, ToolResult};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "brave_image_search".into(),
        title: "Brave Image Search".into(),
        description: "Performs an image search using the Brave Search API. Helpful for when you need pictures of people, places, things, graphic design ideas, art inspiration, and more.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "country": {"type": "string", "description": "Country code (default: US)"},
                "search_lang": {"type": "string", "description": "Search language (default: en)"},
                "ui_lang": {"type": "string", "description": "UI language (default: en-US)"},
                "count": {"type": "integer", "description": "Number of results", "minimum": 1, "maximum": 50},
                "offset": {"type": "integer", "description": "Result offset", "minimum": 0, "maximum": 9},
                "spellcheck": {"type": "boolean"},
                "safesearch": {"type": "string", "enum": ["off", "moderate", "strict"]},
                "freshness": {"type": "string", "description": "Time filter"},
                "extra_snippets": {"type": "boolean"},
                "goggles": {"description": "Goggle(s) for custom ranking"}
            },
            "required": ["query"]
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "type": {"type": "string"},
                "items": {"type": "array"},
                "count": {"type": "integer"},
                "might_be_offensive": {"type": "boolean"}
            }
        })),
    }
}

pub async fn execute(client: &BraveClient, params: Value) -> Result<ToolResult, String> {
    let response: ImageSearchResponse = client
        .request(Endpoint::Images, params, None)
        .await
        .map_err(|e| e.to_string())?;

    let items: Vec<Value> = response
        .results
        .iter()
        .map(|r| {
            json!({
                "title": r.title,
                "url": r.url,
                "page_fetched": r.page_fetched,
                "confidence": r.source,
                "properties": {
                    "url": r.properties.as_ref().and_then(|p| p.url.as_ref()),
                    "width": r.properties.as_ref().and_then(|p| p.width.as_ref()),
                    "height": r.properties.as_ref().and_then(|p| p.height.as_ref())
                }
            })
        })
        .collect();

    let structured = json!({
        "type": "object",
        "items": items,
        "count": items.len(),
        "might_be_offensive": response.might_be_offensive
    });

    Ok(ToolResult::structured(
        vec![serde_json::to_string(&structured).unwrap()],
        structured,
    ))
}
