use reqwest::{header, Client, StatusCode};
use serde::de::DeserializeOwned;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, warn, error, info};

const BASE_URL: &str = "https://api.search.brave.com";
const COOLDOWN_BASE: Duration = Duration::from_secs(60); // Base cooldown: 1 minute
const COOLDOWN_MAX: Duration = Duration::from_secs(3600); // Max cooldown: 1 hour
const MAX_RETRIES: usize = 3; // Max retries on rate limit

#[derive(Error, Debug)]
pub enum BraveApiError {
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("All keys are rate-limited or unavailable")]
    AllKeysUnavailable,
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

/// Key state tracking for intelligent load balancing
struct KeyState {
    key: String,
    /// Number of consecutive failures
    failure_count: AtomicU64,
    /// Number of successful requests
    success_count: AtomicU64,
    /// Cooldown expiry time (0 = not in cooldown)
    cooldown_until: std::sync::atomic::AtomicI64,
    /// Last error message
    last_error: std::sync::RwLock<Option<String>>,
}

impl KeyState {
    fn new(key: String) -> Self {
        Self {
            key,
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            cooldown_until: std::sync::atomic::AtomicI64::new(0),
            last_error: std::sync::RwLock::new(None),
        }
    }

    /// Check if this key is currently available (not in cooldown)
    fn is_available(&self, now: i64) -> bool {
        let cooldown = self.cooldown_until.load(Ordering::Relaxed);
        now >= cooldown
    }

    /// Mark this key as rate-limited with exponential backoff
    fn mark_rate_limited(&self, now: i64) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed);
        let exponent = std::cmp::min(failures, 6) as u32;
        let backoff_secs = std::cmp::min(
            COOLDOWN_BASE.as_secs() * 2u64.pow(exponent),
            COOLDOWN_MAX.as_secs(),
        );
        let cooldown_until = now + backoff_secs as i64;
        self.cooldown_until.store(cooldown_until, Ordering::Relaxed);
        warn!(
            key = %self.key_masked(),
            failures,
            backoff_secs,
            "Key rate-limited, entering cooldown"
        );
    }

    /// Mark this key as failed (auth error, etc.)
    fn mark_failed(&self, now: i64, error: String) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut last_error) = self.last_error.write() {
            *last_error = Some(error.clone());
        }
        // Disable key permanently on auth errors
        let cooldown_until = now + COOLDOWN_MAX.as_secs() as i64 * 24;
        self.cooldown_until.store(cooldown_until, Ordering::Relaxed);
        error!(
            key = %self.key_masked(),
            error,
            "Key disabled due to authentication error"
        );
    }

    /// Mark this key as successful
    fn mark_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        // Reset failure count on success
        self.failure_count.store(0, Ordering::Relaxed);
        // Clear cooldown if we had one
        self.cooldown_until.store(0, Ordering::Relaxed);
    }

    /// Get masked key for logging (show only first 4 and last 4 chars)
    fn key_masked(&self) -> String {
        if self.key.len() <= 10 {
            "***".to_string()
        } else {
            format!("{}...{}", &self.key[..4], &self.key[self.key.len()-4..])
        }
    }

    /// Get statistics for this key
    fn stats(&self, now: i64) -> KeyStats {
        KeyStats {
            key_masked: self.key_masked(),
            success_count: self.success_count.load(Ordering::Relaxed),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            is_available: self.is_available(now),
            last_error: self.last_error.read().ok().and_then(|e| e.clone()),
        }
    }
}

/// Key statistics for monitoring
#[derive(Debug, Clone, serde::Serialize)]
pub struct KeyStats {
    pub key_masked: String,
    pub success_count: u64,
    pub failure_count: u64,
    pub is_available: bool,
    pub last_error: Option<String>,
}

/// Multi-key pool with intelligent load balancing and health tracking
pub struct KeyPool {
    keys: Vec<KeyState>,
    counter: AtomicUsize,
    epoch: Instant,
}

impl KeyPool {
    pub fn new(keys: Vec<String>) -> Self {
        let epoch = Instant::now();
        let key_states = keys.into_iter().map(KeyState::new).collect();
        Self {
            keys: key_states,
            counter: AtomicUsize::new(0),
            epoch,
        }
    }

    fn now_secs(&self) -> i64 {
        self.epoch.elapsed().as_secs() as i64
    }

    /// Get the next available key, skipping rate-limited ones
    pub fn next_key(&self) -> Option<&KeyState> {
        let len = self.keys.len();
        let start = self.counter.fetch_add(1, Ordering::Relaxed) % len;
        let now = self.now_secs();

        // Try up to len times to find an available key
        for i in 0..len {
            let idx = (start + i) % len;
            let key_state = &self.keys[idx];
            if key_state.is_available(now) {
                debug!(
                    key = %key_state.key_masked(),
                    index = idx,
                    "Selected key for request"
                );
                return Some(key_state);
            }
        }

        warn!("All {} keys are currently rate-limited or unavailable", len);
        None
    }

    /// Get statistics for all keys
    pub fn stats(&self) -> Vec<KeyStats> {
        let now = self.now_secs();
        self.keys.iter().map(|k| k.stats(now)).collect()
    }

    /// Get summary statistics
    pub fn summary(&self) -> KeyPoolSummary {
        let total = self.keys.len();
        let now = self.now_secs();
        let available = self.keys.iter().filter(|k| k.is_available(now)).count();
        let total_success: u64 = self.keys.iter().map(|k| k.stats(now).success_count).sum();
        let total_failure: u64 = self.keys.iter().map(|k| k.stats(now).failure_count).sum();

        KeyPoolSummary {
            total_keys: total,
            available_keys: available,
            unavailable_keys: total - available,
            total_requests: total_success + total_failure,
            success_rate: if total_success + total_failure > 0 {
                (total_success as f64 / (total_success + total_failure) as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Summary statistics for the key pool
#[derive(Debug, Clone, serde::Serialize)]
pub struct KeyPoolSummary {
    pub total_keys: usize,
    pub available_keys: usize,
    pub unavailable_keys: usize,
    pub total_requests: u64,
    pub success_rate: f64,
}

/// Brave Search API client with intelligent multi-key load balancing
pub struct BraveClient {
    http: Client,
    key_pool: KeyPool,
}

impl BraveClient {
    pub fn new(keys: Vec<String>) -> Result<Self, reqwest::Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, "application/json".parse().unwrap());
        headers.insert(header::ACCEPT_ENCODING, "gzip".parse().unwrap());

        let http = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;

        info!("Initialized BraveClient with {} API keys", keys.len());

        Ok(Self {
            http,
            key_pool: KeyPool::new(keys),
        })
    }

    /// Make a request to the Brave API with intelligent retry logic.
    ///
    /// Features:
    /// - Automatic key rotation on rate limits (429)
    /// - Exponential backoff for rate-limited keys
    /// - Automatic disabling of keys with auth errors (401/403)
    /// - Up to MAX_RETRIES attempts with different keys
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
        let url = format!("{}{}", BASE_URL, endpoint.path());

        // Build query string once (it's the same for all retries)
        let query_pairs = self.build_query_pairs(&params);

        // Retry loop with key rotation
        for attempt in 0..MAX_RETRIES {
            let now = self.key_pool.now_secs();

            // Get next available key
            let key_state = match self.key_pool.next_key() {
                Some(key) => key,
                None => {
                    error!("No available keys after {} attempts", attempt);
                    return Err(BraveApiError::AllKeysUnavailable);
                }
            };

            debug!(
                attempt = attempt + 1,
                key = %key_state.key_masked(),
                "Making API request"
            );

            let mut request = self
                .http
                .get(&url)
                .header("X-Subscription-Token", &key_state.key)
                .query(&query_pairs);

            // Add extra headers if provided
            if let Some(ref headers) = extra_headers {
                request = request.headers(headers.clone());
            }

            let response = match request.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    warn!(
                        key = %key_state.key_masked(),
                        error = %e,
                        "Network error, rotating key"
                    );
                    key_state.mark_rate_limited(now);
                    continue;
                }
            };

            let status = response.status();

            // Success
            if status.is_success() {
                key_state.mark_success();
                debug!(
                    key = %key_state.key_masked(),
                    status = %status,
                    "Request successful"
                );
                return Ok(response.json().await?);
            }

            // Rate limited (429) - mark key and retry with next key
            if status == StatusCode::TOO_MANY_REQUESTS {
                key_state.mark_rate_limited(now);
                debug!(
                    attempt = attempt + 1,
                    key = %key_state.key_masked(),
                    "Rate limited, trying next key"
                );
                continue;
            }

            // Auth errors (401/403) - disable this key permanently
            if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
                let body = response.text().await.unwrap_or_default();
                key_state.mark_failed(now, format!("Auth error {}: {}", status, body));
                error!(
                    key = %key_state.key_masked(),
                    status = %status,
                    "Key disabled due to authentication error"
                );
                continue; // Try next key
            }

            // Other HTTP errors - return immediately (don't retry)
            let body = response.text().await.unwrap_or_default();
            key_state.mark_failed(now, format!("HTTP {}: {}", status, body));
            return Err(BraveApiError::Http {
                status: status.as_u16(),
                body,
            });
        }

        // All retries exhausted
        Err(BraveApiError::AllKeysUnavailable)
    }

    /// Get key pool statistics for monitoring
    pub fn key_stats(&self) -> Vec<KeyStats> {
        self.key_pool.stats()
    }

    /// Get key pool summary
    pub fn key_summary(&self) -> KeyPoolSummary {
        self.key_pool.summary()
    }

    /// Build query pairs from params JSON
    fn build_query_pairs(&self, params: &serde_json::Value) -> Vec<(String, String)> {
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

        query_pairs
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
