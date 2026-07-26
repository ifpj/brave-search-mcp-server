use serde::Deserialize;
use serde_json::Value;

// ─── Web Search Response ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WebSearchResponse {
    pub web: Option<WebSearch>,
    pub faq: Option<Faq>,
    pub discussions: Option<Discussions>,
    pub news: Option<News>,
    pub videos: Option<Videos>,
    pub locations: Option<Locations>,
    pub summarizer: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct WebSearch {
    #[serde(default)]
    pub results: Vec<WebResult>,
}

#[derive(Debug, Deserialize)]
pub struct WebResult {
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub extra_snippets: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Faq {
    #[serde(default)]
    pub results: Vec<FaqResult>,
}

#[derive(Debug, Deserialize)]
pub struct FaqResult {
    pub question: Option<String>,
    pub answer: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Discussions {
    pub mutated_by_goggles: Option<bool>,
    #[serde(default)]
    pub results: Vec<DiscussionResult>,
}

#[derive(Debug, Deserialize)]
pub struct DiscussionResult {
    pub mutated_by_goggles: Option<bool>,
    pub url: Option<String>,
    pub data: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct News {
    pub mutated_by_goggles: Option<bool>,
    #[serde(default)]
    pub results: Vec<NewsResult>,
}

#[derive(Debug, Deserialize)]
pub struct NewsResult {
    pub mutated_by_goggles: Option<bool>,
    pub source: Option<String>,
    pub breaking: Option<bool>,
    pub is_live: Option<bool>,
    pub age: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub extra_snippets: Option<Vec<String>>,
    pub thumbnail: Option<Thumbnail>,
    pub page_age: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Videos {
    pub mutated_by_goggles: Option<bool>,
    #[serde(default)]
    pub results: Vec<VideoResult>,
}

#[derive(Debug, Deserialize)]
pub struct VideoResult {
    pub mutated_by_goggles: Option<bool>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub age: Option<String>,
    pub thumbnail: Option<Thumbnail>,
    pub duration: Option<String>,
    pub view_count: Option<i64>,
    pub creator: Option<String>,
    pub publisher: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Thumbnail {
    pub src: Option<String>,
}

// ─── Locations ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Locations {
    #[serde(default)]
    pub results: Vec<LocationResult>,
}

#[derive(Debug, Deserialize)]
pub struct LocationResult {
    pub id: Option<String>,
    pub title: Option<String>,
    pub price_range: Option<String>,
    pub contact: Option<Contact>,
    pub rating: Option<Rating>,
    pub opening_hours: Option<Value>,
    pub postal_address: Option<PostalAddress>,
}

#[derive(Debug, Deserialize)]
pub struct Contact {
    pub telephone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Rating {
    #[serde(rename = "ratingValue")]
    pub rating_value: Option<Value>,
    #[serde(rename = "reviewCount")]
    pub review_count: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct PostalAddress {
    #[serde(rename = "displayAddress")]
    pub display_address: Option<String>,
}

// ─── Image Search Response ──────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ImageSearchResponse {
    #[serde(default)]
    pub results: Vec<ImageResult>,
    pub might_be_offensive: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ImageResult {
    pub title: Option<String>,
    pub url: Option<String>,
    pub page_fetched: Option<String>,
    pub properties: Option<ImageProperties>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageProperties {
    pub url: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
}

// ─── Video Search Response ──────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VideoSearchResponse {
    #[serde(default)]
    pub results: Vec<VideoSearchResult>,
}

#[derive(Debug, Deserialize)]
pub struct VideoSearchResult {
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub duration: Option<String>,
    pub thumbnail: Option<Thumbnail>,
    pub age: Option<String>,
}

// ─── News Search Response ───────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct NewsSearchResponse {
    #[serde(default)]
    pub results: Vec<NewsSearchResult>,
}

#[derive(Debug, Deserialize)]
pub struct NewsSearchResult {
    pub url: Option<String>,
    pub title: Option<String>,
    pub age: Option<String>,
    pub page_age: Option<String>,
    pub breaking: Option<bool>,
    pub description: Option<String>,
    pub extra_snippets: Option<Vec<String>>,
    pub thumbnail: Option<Thumbnail>,
}

// ─── Summarizer Response ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SummarizerResponse {
    pub status: Option<String>,
    #[serde(default)]
    pub summary: Vec<SummaryPart>,
}

#[derive(Debug, Deserialize)]
pub struct SummaryPart {
    #[serde(rename = "type")]
    pub part_type: Option<String>,
    pub data: Option<String>,
}

// ─── Local Descriptions Response ────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LocalDescriptionsResponse {
    #[serde(default)]
    pub results: Vec<LocalDescription>,
}

#[derive(Debug, Deserialize)]
pub struct LocalDescription {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<Value>,
    pub rating: Option<Value>,
    pub reviews: Option<Value>,
    pub price_range: Option<String>,
    pub phone: Option<String>,
    pub postal_address: Option<Value>,
    pub opening_hours: Option<Value>,
    pub rating_count: Option<i64>,
}

// ─── LLM Context Response ───────────────────────────────────────

/// Pass-through as raw JSON value.
pub type LlmContextResponse = Value;

/// Pass-through as raw JSON value.
pub type PlaceSearchResponse = Value;
