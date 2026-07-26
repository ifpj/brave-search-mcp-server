use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
};
use serde_json::json;
use std::net::IpAddr;

/// Check if a hostname is a loopback address.
pub fn is_loopback_hostname(host: &str) -> bool {
    let host = host.trim().to_lowercase();

    // Remove IPv6 brackets
    let hostname = host.trim_start_matches('[').trim_end_matches(']');

    // Remove port (last :port for non-IPv6)
    let hostname = if let Some(idx) = hostname.rfind(':') {
        if hostname.matches(':').count() > 1 {
            hostname // IPv6 address
        } else {
            &hostname[..idx] // host:port
        }
    } else {
        hostname
    };

    if hostname == "localhost" || hostname == "::1" {
        return true;
    }

    // Check 127.0.0.0/8
    if let Ok(ip) = hostname.parse::<IpAddr>() {
        if let IpAddr::V4(ipv4) = ip {
            return ipv4.octets()[0] == 127;
        }
    }

    false
}

/// Build a 403 JSON-RPC error response.
pub fn forbidden_response(message: &str) -> Response {
    let body = json!({
        "jsonrpc": "2.0",
        "error": { "code": -32600, "message": message },
        "id": null
    });
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Check if an origin value is allowed.
fn is_origin_allowed(origin: &str, allowed_origins: &[String]) -> bool {
    let origin = origin.trim();
    if origin.is_empty() {
        return false;
    }

    // Try to extract hostname from origin URL
    if let Ok(url) = origin.parse::<url::Url>() {
        if let Some(host) = url.host_str() {
            if is_loopback_hostname(host) {
                return true;
            }
        }
    }

    // Check against explicit allowlist (compare canonical origins)
    allowed_origins.iter().any(|allowed| {
        let allowed = allowed.trim();
        // Simple string comparison on canonical origins
        origin == allowed
    })
}

/// Check if a host header value is allowed.
fn is_host_allowed(host: &str, allowed_hosts: &[String]) -> bool {
    let host = host.trim().to_lowercase();

    // Extract hostname (remove port)
    let hostname = host.trim_start_matches('[').trim_end_matches(']');
    let hostname = if let Some(idx) = hostname.rfind(':') {
        if hostname.matches(':').count() > 1 {
            hostname
        } else {
            &hostname[..idx]
        }
    } else {
        hostname
    };

    if is_loopback_hostname(hostname) {
        return true;
    }

    allowed_hosts.iter().any(|allowed| {
        let allowed = allowed.trim().to_lowercase();
        let allowed = allowed.trim_start_matches('[').trim_end_matches(']');
        allowed == hostname
    })
}

/// Validate request headers for DNS rebinding protection.
/// Returns Ok(()) if the request passes, or Err with a 403 response.
pub fn check_rebinding(
    headers: &axum::http::HeaderMap,
    allowed_origins: &[String],
    allowed_hosts: &[String],
) -> Result<(), Response> {
    // Validate Origin (if present)
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        if !is_origin_allowed(origin, allowed_origins) {
            return Err(forbidden_response("Forbidden: invalid Origin"));
        }
    }

    // Validate Host (only when allowed_hosts is configured)
    if !allowed_hosts.is_empty() {
        if let Some(host) = headers.get(header::HOST).and_then(|v| v.to_str().ok()) {
            if !is_host_allowed(host, allowed_hosts) {
                return Err(forbidden_response("Forbidden: invalid Host"));
            }
        } else {
            return Err(forbidden_response("Forbidden: missing Host"));
        }
    }

    Ok(())
}
