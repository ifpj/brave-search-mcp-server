use serde_json::{json, Value};

use crate::brave::client::{BraveClient, Endpoint};
use crate::brave::types::NewsSearchResponse;
use crate::tools::{ToolDefinition, ToolResult};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "brave_news_search".into(),
        title: "Brave News Search".into(),
        description: "Searches for news articles using Brave's News Search API based on the user's query. Use it when you need current news information, breaking news updates, or articles about specific topics, events, or entities.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "country": {"type": "string", "description": "Country code (default: US)"},
                "search_lang": {"type": "string", "description": "Search language (default: en)"},
                "ui_lang": {"type": "string", "description": "UI language (default: en-US)"},
                "count": {"type": "integer", "description": "Number of results", "minimum": 1, "maximum": 200},
                "offset": {"type": "integer", "description": "Result offset", "minimum": 0, "maximum": 9},
                "safesearch": {"type": "string", "enum": ["off", "moderate", "strict"]},
                "freshness": {"type": "string", "description": "Time filter (pd|pw|pm|py or date range)"},
                "text_decorations": {"type": "boolean"},
                "spellcheck": {"type": "boolean"},
                "result_filter": {"type": "array", "items": {"type": "string"}, "description": "Result types"},
                "goggles": {"description": "Goggle(s) for custom ranking"},
                "units": {"type": "string", "enum": ["metric", "imperial"]},
                "extra_snippets": {"type": "boolean"},
                "summary": {"type": "boolean"}
            },
            "required": ["query"]
        }),
        output_schema: None,
    }
}

pub async fn execute(client: &BraveClient, params: Value) -> Result<ToolResult, String> {
    let response: NewsSearchResponse = client
        .request(Endpoint::News, params, None)
        .await
        .map_err(|e| e.to_string())?;

    let items: Vec<String> = response
        .results
        .iter()
        .map(|r| {
            json!({
                "url": r.url,
                "title": r.title,
                "age": r.age,
                "page_age": r.page_age,
                "breaking": r.breaking.unwrap_or(false),
                "description": r.description,
                "extra_snippets": r.extra_snippets,
                "thumbnail": r.thumbnail.as_ref().and_then(|t| t.src.as_ref())
            })
            .to_string()
        })
        .collect();

    Ok(ToolResult::text(items))
}
