pub mod web;
pub mod images;
pub mod videos;
pub mod news;
pub mod local;
pub mod summarizer;
pub mod llm_context;
pub mod place_search;

use serde_json::{json, Value};
use std::sync::OnceLock;

use crate::brave::client::BraveClient;

/// MCP tool definition.
pub struct ToolDefinition {
    pub name: String,
    pub title: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
}

impl ToolDefinition {
    /// Convert to JSON for tools/list response.
    pub fn to_json(&self) -> Value {
        let mut obj = json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.input_schema,
            "annotations": {
                "title": self.title,
                "openWorldHint": true
            }
        });
        if let Some(ref schema) = self.output_schema {
            obj["outputSchema"] = schema.clone();
        }
        obj
    }
}

/// Result from tool execution.
pub struct ToolResult {
    pub content: Vec<Value>,
    pub is_error: bool,
    pub structured_content: Option<Value>,
}

impl ToolResult {
    pub fn text(items: Vec<String>) -> Self {
        let content = items
            .into_iter()
            .map(|t| json!({ "type": "text", "text": t }))
            .collect();
        Self {
            content,
            is_error: false,
            structured_content: None,
        }
    }

    pub fn structured(items: Vec<String>, structured: Value) -> Self {
        let content = items
            .into_iter()
            .map(|t| json!({ "type": "text", "text": t }))
            .collect();
        Self {
            content,
            is_error: false,
            structured_content: Some(structured),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            content: vec![json!({ "type": "text", "text": message })],
            is_error: true,
            structured_content: None,
        }
    }

    pub fn to_json(&self) -> Value {
        let mut obj = json!({
            "content": self.content,
            "isError": self.is_error
        });
        if let Some(ref sc) = self.structured_content {
            obj["structuredContent"] = sc.clone();
        }
        obj
    }
}

type ToolHandler = fn(
    &BraveClient,
    Value,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolResult, String>> + Send + '_>>;

struct ToolEntry {
    definition: ToolDefinition,
    handler: ToolHandler,
}

static TOOLS: OnceLock<Vec<ToolEntry>> = OnceLock::new();

fn get_tools() -> &'static Vec<ToolEntry> {
    TOOLS.get_or_init(|| {
        vec![
            ToolEntry {
                definition: web::definition(),
                handler: |c, p| Box::pin(web::execute(c, p)),
            },
            ToolEntry {
                definition: images::definition(),
                handler: |c, p| Box::pin(images::execute(c, p)),
            },
            ToolEntry {
                definition: videos::definition(),
                handler: |c, p| Box::pin(videos::execute(c, p)),
            },
            ToolEntry {
                definition: news::definition(),
                handler: |c, p| Box::pin(news::execute(c, p)),
            },
            ToolEntry {
                definition: local::definition(),
                handler: |c, p| Box::pin(local::execute(c, p)),
            },
            ToolEntry {
                definition: summarizer::definition(),
                handler: |c, p| Box::pin(summarizer::execute(c, p)),
            },
            ToolEntry {
                definition: llm_context::definition(),
                handler: |c, p| Box::pin(llm_context::execute(c, p)),
            },
            ToolEntry {
                definition: place_search::definition(),
                handler: |c, p| Box::pin(place_search::execute(c, p)),
            },
        ]
    })
}

/// All tool definitions.
pub fn all_definitions() -> Vec<&'static ToolDefinition> {
    get_tools().iter().map(|t| &t.definition).collect()
}

/// Execute a tool by name.
pub async fn execute(
    name: &str,
    client: &BraveClient,
    params: Value,
) -> Result<ToolResult, String> {
    let tools = get_tools();
    let tool = tools.iter().find(|t| t.definition.name == name);
    match tool {
        Some(tool) => (tool.handler)(client, params).await,
        None => Err(format!("Unknown tool: {}", name)),
    }
}
