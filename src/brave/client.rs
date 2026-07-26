use reqwest::{Client, header};
use serde::de::DeserializeOwned;
use std::sync::atomic::{AtomicUsize, Ordering};
use thiserror::Error;

const BASE_URL: &str = "https://api.search.brave.com";

#[derive(Error, Debug)]
pub enum BraveApiError {
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// API endpoint paths.
pub enum Endpoint {
    Web,
    Images,
    Videos,
    News,
    Summarizer,
    LlmContext,
    LocalPois,
    LocalDescriptions,
    PlaceSearch,
}

impl Endpoint {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Web => "/res/v1/web/search",
            Self::Images => "/res/v1/images/search",
            Self::Videos => "/res/v1/videos/search",
            Self::News => "/res/v1/news/search",
            Self::Summarizer => "/res/v1/summarizer/search",
            Self::LlmContext => "/res/v1/llm/context",
            Self::LocalPois => "/res/v1/local/pois",
            Self::LocalDescriptions => "/res/v1/local/descriptions",
            Self::PlaceSearch => "/res/v1/local/place_search",
        }
    }
}

/// Multi-key pool with atomic round-robin rotation.
pub struct KeyPool {
    keys: Vec<String>,
    counter: AtomicUsize,
}

impl KeyPool {
    pub fn new(keys: Vec<String>) -> Self {
        Self {
            keys,
            counter: AtomicUsize::new(0),
        }
    }

    /// Get the next key in round-robin order.
    pub fn next_key(&self) -> &str {
        let idx = self.counter.fetch_add(1, Ordering::Relaxed) % self.keys.len();
        &self.keys[idx]
    }
}

/// Brave Search API client with multi-key load balancing.
pub struct BraveClient {
    http: Client,
    key_pool: KeyPool,
}

impl BraveClient {
    pub fn new(keys: Vec<String>) -> Result<Self, reqwest::Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, "application/json".parse().unwrap());
        headers.insert(header::ACCEPT_ENCODING, "gzip".parse().unwrap());

        let http = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            http,
            key_pool: KeyPool::new(keys),
        })
    }

    /// Make a request to the Brave API.
    ///
    /// `params` is a JSON object that will be converted to query parameters.
    /// Special cases:
    /// - `query` is renamed to `q`
    /// - `result_filter` is comma-joined
    /// - `ids` is repeated
    /// - `goggles` is repeated with HTTPS validation
    pub async fn request<T: DeserializeOwned>(
        &self,
        endpoint: Endpoint,
        params: serde_json::Value,
        extra_headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<T, BraveApiError> {
        let key = self.key_pool.next_key();
        let url = format!("{}{}", BASE_URL, endpoint.path());

        // Build query string
        let mut query_pairs: Vec<(String, String)> = Vec::new();

        if let Some(obj) = params.as_object() {
            for (key_name, value) in obj {
                let param_key = if key_name == "query" { "q" } else { key_name };

                match key_name.as_str() {
                    "result_filter" => {
                        // Special case: skip if summary=true
                        if obj.get("summary").and_then(|v| v.as_bool()).unwrap_or(false) {
                            continue;
                        }
                        if let Some(arr) = value.as_array() {
                            let joined: Vec<&str> =
                                arr.iter().filter_map(|v| v.as_str()).collect();
                            if !joined.is_empty() {
                                query_pairs.push((param_key.to_string(), joined.join(",")));
                            }
                        }
                    }
                    "ids" => {
                        // Repeated parameter
                        if let Some(arr) = value.as_array() {
                            for id in arr.iter().filter_map(|v| v.as_str()) {
                                query_pairs.push((param_key.to_string(), id.to_string()));
                            }
                        }
                    }
                    "goggles" => {
                        // Repeated with HTTPS validation
                        let goggles: Vec<&str> = if let Some(arr) = value.as_array() {
                            arr.iter().filter_map(|v| v.as_str()).collect()
                        } else if let Some(s) = value.as_str() {
                            vec![s]
                        } else {
                            vec![]
                        };

                        for goggle in goggles {
                            if let Some(normalized) = normalize_goggle(goggle) {
                                query_pairs.push((param_key.to_string(), normalized));
                            }
                        }
                    }
                    _ => {
                        // Skip null/undefined
                        if value.is_null() {
                            continue;
                        }
                        // Convert to string
                        let param_value = if let Some(s) = value.as_str() {
                            s.to_string()
                        } else if let Some(b) = value.as_bool() {
                            b.to_string()
                        } else if let Some(n) = value.as_i64() {
                            n.to_string()
                        } else if let Some(n) = value.as_f64() {
                            n.to_string()
                        } else {
                            continue;
                        };
                        query_pairs.push((param_key.to_string(), param_value));
                    }
                }
            }
        }

        let mut request = self
            .http
            .get(&url)
            .header("X-Subscription-Token", key)
            .query(&query_pairs);

        // Add extra headers if provided
        if let Some(headers) = extra_headers {
            request = request.headers(headers);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(BraveApiError::Http { status, body });
        }

        Ok(response.json().await?)
    }
}

/// Normalize a goggle value:
/// - HTTPS URLs pass through
/// - Bare strings pass through
/// - HTTP URLs are rejected (return None)
/// - Empty strings are skipped (return None)
fn normalize_goggle(goggle: &str) -> Option<String> {
    let trimmed = goggle.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check if it's a URL
    if trimmed.starts_with("http://") {
        return None; // Reject HTTP URLs
    }

    if trimmed.starts_with("https://") {
        return Some(trimmed.to_string()); // HTTPS URLs pass
    }

    // Bare strings pass through
    Some(trimmed.to_string())
}
