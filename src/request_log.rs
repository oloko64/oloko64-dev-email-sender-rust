use std::time::{Duration, Instant};

use axum::{
    extract::Request,
    http::{HeaderMap, Method, StatusCode, Uri, header},
    middleware::Next,
    response::Response,
};
use jiff::Timestamp;
use tracing::warn;

use crate::telegram::Telegram;

const MAX_USER_AGENT_CHARS: usize = 200;

/// Sends a Telegram message for every request, so the API can be monitored.
///
/// The notification is awaited (not spawned) because on Lambda background tasks are frozen
/// once the response is returned. A failed notification never affects the response.
pub async fn request_logger(request: Request, next: Next) -> Response {
    let started_at = Instant::now();
    let timestamp = Timestamp::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    let response = next.run(request).await;

    let text = format_log(
        &method,
        &uri,
        response.status(),
        timestamp,
        started_at.elapsed(),
        &headers,
    );
    if let Err(err) = Telegram::send_text(text).await {
        warn!(error = %err, "Error while sending request log to Telegram");
    }

    response
}

fn format_log(
    method: &Method,
    uri: &Uri,
    status: StatusCode,
    timestamp: Timestamp,
    duration: Duration,
    headers: &HeaderMap,
) -> String {
    let path = uri.path_and_query().map_or(uri.path(), |pq| pq.as_str());
    let ip = header_value(headers, "x-forwarded-for")
        // The first entry is the original client, the rest are proxies.
        .and_then(|value| value.split(',').next())
        .map_or("unknown", str::trim);
    let user_agent = header_value(headers, header::USER_AGENT.as_str())
        .map(|ua| ua.chars().take(MAX_USER_AGENT_CHARS).collect::<String>())
        .unwrap_or_else(|| String::from("unknown"));

    format!(
        "{method} {path} → {status}\n\
         Time: {timestamp:.0}\n\
         Duration: {} ms\n\
         IP: {ip}\n\
         User-Agent: {user_agent}",
        duration.as_millis()
    )
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn formats_log_message() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.7, 10.0.0.1"),
        );
        headers.insert(header::USER_AGENT, HeaderValue::from_static("curl/8.0"));

        let text = format_log(
            &Method::POST,
            &Uri::from_static("/v1/send-message?x=1"),
            StatusCode::SERVICE_UNAVAILABLE,
            "2026-11-08T14:03:22.123Z".parse().unwrap(),
            Duration::from_millis(412),
            &headers,
        );

        assert_eq!(
            text,
            "POST /v1/send-message?x=1 → 503 Service Unavailable\n\
             Time: 2026-11-08T14:03:22Z\n\
             Duration: 412 ms\n\
             IP: 203.0.113.7\n\
             User-Agent: curl/8.0"
        );
    }

    #[test]
    fn uses_unknown_when_headers_are_missing() {
        let text = format_log(
            &Method::GET,
            &Uri::from_static("/health"),
            StatusCode::OK,
            Timestamp::UNIX_EPOCH,
            Duration::ZERO,
            &HeaderMap::new(),
        );

        assert!(text.contains("IP: unknown"));
        assert!(text.contains("User-Agent: unknown"));
    }
}
