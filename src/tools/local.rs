use serde_json::{json, Value};

use crate::brave::client::{BraveClient, Endpoint};
use crate::brave::types::{WebSearchResponse, LocalDescriptionsResponse};
use crate::tools::{ToolDefinition, ToolResult};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "brave_local_search".into(),
        title: "Brave Local Search".into(),
        description: "Brave Local Search API provides enrichments for location search results. Access to this API is available only through the Brave Search API Pro plans. Searches for local businesses and places using Brave's Local Search API. Best for queries related to physical locations, businesses, restaurants, services, etc.".into(),
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
    // Step 1: Add "web" and "locations" to result_filter
    let mut web_params = params.clone();
    let mut result_filter = params
        .get("result_filter")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    result_filter.push(json!("web"));
    result_filter.push(json!("locations"));
    web_params["result_filter"] = json!(result_filter);

    // Step 2: Call web endpoint to get locations
    let web_response: WebSearchResponse = client
        .request(Endpoint::Web, web_params, None)
        .await
        .map_err(|e| e.to_string())?;

    // Step 3: Extract location IDs (up to 20)
    let location_ids: Vec<String> = web_response
        .locations
        .as_ref()
        .map(|locs| {
            locs.results
                .iter()
                .filter_map(|loc| loc.id.clone())
                .take(20)
                .collect()
        })
        .unwrap_or_default();

    // Step 4: If no locations found, fall back to web results
    if location_ids.is_empty() {
        if let Some(web) = web_response.web {
            if !web.results.is_empty() {
                let mut items = vec![
                    "No location data was returned. Either the user's plan does not support local search, or the API was unable to find locations for the provided query. Falling back to general web search.".to_string()
                ];

                for r in &web.results {
                    items.push(json!({
                        "url": r.url,
                        "title": r.title,
                        "description": r.description,
                        "extra_snippets": r.extra_snippets
                    }).to_string());
                }

                return Ok(ToolResult::text(items));
            }
        }

        return Ok(ToolResult::text(vec![
            "No location data was returned. User's plan does not support local search, or the query may be unclear.".to_string()
        ]));
    }

    // Step 5: Call localDescriptions endpoint with location IDs
    let desc_params = json!({
        "ids": location_ids
    });

    let desc_response: LocalDescriptionsResponse = client
        .request(Endpoint::LocalDescriptions, desc_params, None)
        .await
        .map_err(|e| e.to_string())?;

    // Step 6: Format results by merging locations with descriptions
    let mut items = Vec::new();

    if let Some(locations) = web_response.locations {
        for loc in &locations.results {
            let description = desc_response
                .results
                .iter()
                .find(|desc| desc.id == loc.id)
                .and_then(|desc| desc.description.clone());

            let hours = loc.opening_hours.as_ref().map(format_opening_hours);

            items.push(json!({
                "name": loc.title,
                "price_range": loc.price_range,
                "phone": loc.contact.as_ref().and_then(|c| c.telephone.clone()),
                "rating": loc.rating.as_ref().and_then(|r| r.rating_value.clone()),
                "hours": hours,
                "rating_count": loc.rating.as_ref().and_then(|r| r.review_count.clone()),
                "description": description,
                "address": loc.postal_address.as_ref().and_then(|a| a.display_address.clone())
            }).to_string());
        }
    }

    Ok(ToolResult::text(items))
}

fn format_opening_hours(hours: &Value) -> Value {
    // Simplified: just return the raw hours data
    // Full implementation would format like the TypeScript version
    hours.clone()
}
