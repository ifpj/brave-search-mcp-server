use serde_json::{json, Value};

use crate::brave::client::{BraveClient, Endpoint};
use crate::brave::types::WebSearchResponse;
use crate::tools::{ToolDefinition, ToolResult};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "brave_web_search".into(),
        title: "Brave Web Search".into(),
        description: "Performs web searches using the Brave Search API and returns comprehensive search results with rich metadata.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "country": {"type": "string", "description": "Country code (default: US)"},
                "search_lang": {"type": "string", "description": "Search language (default: en)"},
                "ui_lang": {"type": "string", "description": "UI language (default: en-US)"},
                "count": {"type": "integer", "description": "Number of results", "minimum": 1, "maximum": 20},
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
    let response: WebSearchResponse = client
        .request(Endpoint::Web, params, None)
        .await
        .map_err(|e| e.to_string())?;

    let mut items: Vec<String> = Vec::new();

    // Summarizer key
    if let Some(ref summarizer) = response.summarizer {
        if let Some(key) = summarizer.get("key").and_then(|v| v.as_str()) {
            items.push(format!("Summarizer key: {}", key));
        }
    }

    // Web results
    if let Some(ref web) = response.web {
        if web.results.is_empty() {
            return Ok(ToolResult::error("No web results found".into()));
        }
        for r in &web.results {
            items.push(json!({
                "url": r.url,
                "title": r.title,
                "description": r.description,
                "extra_snippets": r.extra_snippets
            })
            .to_string());
        }
    } else {
        return Ok(ToolResult::error("No web results found".into()));
    }

    // FAQ results
    if let Some(ref faq) = response.faq {
        for r in &faq.results {
            items.push(json!({
                "question": r.question,
                "answer": r.answer,
                "title": r.title,
                "url": r.url
            })
            .to_string());
        }
    }

    // Discussion results
    if let Some(ref discussions) = response.discussions {
        for r in &discussions.results {
            items.push(json!({
                "mutated_by_goggles": discussions.mutated_by_goggles,
                "url": r.url,
                "data": r.data
            })
            .to_string());
        }
    }

    // News results
    if let Some(ref news) = response.news {
        for r in &news.results {
            items.push(json!({
                "mutated_by_goggles": news.mutated_by_goggles,
                "source": r.source,
                "breaking": r.breaking,
                "is_live": r.is_live,
                "age": r.age,
                "url": r.url,
                "title": r.title,
                "description": r.description,
                "extra_snippets": r.extra_snippets
            })
            .to_string());
        }
    }

    // Video results
    if let Some(ref videos) = response.videos {
        for r in &videos.results {
            items.push(json!({
                "mutated_by_goggles": videos.mutated_by_goggles,
                "url": r.url,
                "title": r.title,
                "description": r.description,
                "age": r.age,
                "thumbnail_url": r.thumbnail.as_ref().and_then(|t| t.src.as_ref()),
                "duration": r.duration,
                "view_count": r.view_count,
                "creator": r.creator,
                "publisher": r.publisher,
                "tags": r.tags
            })
            .to_string());
        }
    }

    Ok(ToolResult::text(items))
}
