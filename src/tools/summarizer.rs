use serde_json::{json, Value};
use tokio::time::{sleep, Duration};

use crate::brave::client::{BraveClient, Endpoint};
use crate::brave::types::SummarizerResponse;
use crate::tools::{ToolDefinition, ToolResult};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "brave_summarizer".into(),
        title: "Brave Summarizer".into(),
        description: "Retrieves AI-generated summaries of web search results using Brave's Summarizer API. This tool processes search results to create concise, coherent summaries of information gathered from multiple sources. Must first perform a web search using brave_web_search with summary=true parameter. Requires a Pro AI subscription.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "key": {"type": "string", "description": "Summarizer key from web search results"},
                "entity_info": {"type": "boolean", "description": "Include entity information"},
                "inline_references": {"type": "boolean", "description": "Include inline references"}
            },
            "required": ["key"]
        }),
        output_schema: None,
    }
}

pub async fn execute(client: &BraveClient, params: Value) -> Result<ToolResult, String> {
    // Poll for summary with 50ms interval, up to 20 attempts
    let mut attempts = 20;

    while attempts > 0 {
        match client.request::<SummarizerResponse>(Endpoint::Summarizer, params.clone(), None).await {
            Ok(response) => {
                if response.status.as_deref() == Some("complete") {
                    // Process summary parts
                    let summary_text = response
                        .summary
                        .iter()
                        .map(|part| {
                            match part.part_type.as_deref() {
                                Some("token") => part.data.clone().unwrap_or_default(),
                                Some("inline_reference") => {
                                    if let Some(data) = &part.data {
                                        if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                                            if let Some(url) = parsed.get("url").and_then(|v| v.as_str()) {
                                                return format!(" ({})", url);
                                            }
                                        }
                                    }
                                    String::new()
                                }
                                _ => String::new()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("");

                    if summary_text.is_empty() {
                        return Ok(ToolResult::error("Unable to retrieve a Summarizer summary.".into()));
                    }

                    return Ok(ToolResult::text(vec![summary_text]));
                }
            }
            Err(_) => {
                // Sleep and retry
                sleep(Duration::from_millis(50)).await;
            }
        }

        attempts -= 1;
    }

    Ok(ToolResult::error("Unable to retrieve a Summarizer summary.".into()))
}
