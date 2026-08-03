//! HTTP transport shim for Concordance/1.0.
//!
//! The core protocol is transport-independent. This crate provides:
//! - A minimal CBOR request/response wrapper for Axum.
//! - Wire DTOs for a federated, signed-record registry pilot.

use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, Request},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

pub const CONTENT_TYPE_CBOR: &str = "application/cbor";
pub const CONTENT_TYPE_JSON: &str = "application/json";
pub const CONTENT_TYPE_CONCORDANCE_CBOR: &str = "application/concordance+cbor";
pub const CONTENT_TYPE_CONCORDANCE_JSON: &str = "application/concordance+json";

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("missing Content-Type")]
    MissingContentType,
    #[error("unsupported Content-Type: {0}")]
    UnsupportedContentType(String),
    #[error("failed to decode request body: {0}")]
    Decode(String),
    #[error("failed to encode response body: {0}")]
    Encode(String),
}

impl IntoResponse for TransportError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

/// Accept either JSON (`application/json`) or CBOR (`application/cbor`).
///
/// CBOR is preferred for production, but JSON remains useful for debugging and
/// tooling. Neither representation changes the signed canonical CBOR preimage
/// rules used by `concordance-core`.
pub struct AnyBody<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for AnyBody<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = TransportError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = req.headers();
        let content_type = headers
            .get(header::CONTENT_TYPE)
            .ok_or(TransportError::MissingContentType)?
            .to_str()
            .map_err(|_| TransportError::MissingContentType)?
            .to_ascii_lowercase();

        let bytes = Bytes::from_request(req, _state)
            .await
            .map_err(|e| TransportError::Decode(e.to_string()))?;

        if is_cbor_content_type(&content_type) {
            let value =
                serde_cbor::from_slice(&bytes).map_err(|e| TransportError::Decode(e.to_string()))?;
            Ok(Self(value))
        } else if is_json_content_type(&content_type) {
            let value = serde_json::from_slice(&bytes)
                .map_err(|e| TransportError::Decode(e.to_string()))?;
            Ok(Self(value))
        } else {
            Err(TransportError::UnsupportedContentType(content_type))
        }
    }
}

fn is_cbor_content_type(content_type: &str) -> bool {
    content_type.starts_with(CONTENT_TYPE_CBOR)
        || content_type.starts_with(CONTENT_TYPE_CONCORDANCE_CBOR)
}

fn is_json_content_type(content_type: &str) -> bool {
    content_type.starts_with(CONTENT_TYPE_JSON)
        || content_type.starts_with(CONTENT_TYPE_CONCORDANCE_JSON)
}

fn preferred_response_content_type(headers: Option<&HeaderMap>) -> &'static str {
    if let Some(headers) = headers {
        if let Some(accept) = accept_header(headers) {
            for token in accept.split(',') {
                let mime = token.split(';').next().unwrap_or("").trim();
                if is_cbor_content_type(mime) || mime.ends_with("+cbor") {
                    return CONTENT_TYPE_CONCORDANCE_CBOR;
                }
            }
        }
    }
    CONTENT_TYPE_JSON
}

/// Build a transport response using the caller's Accept header preference.
///
/// JSON is the default for debugging; CBOR is preferred when the caller
/// explicitly accepts it.
pub fn any_response<T>(value: T, headers: Option<&HeaderMap>) -> Response
where
    T: Serialize,
{
    let content_type = preferred_response_content_type(headers);

    let body: Result<Vec<u8>, String> = match content_type {
        CONTENT_TYPE_CONCORDANCE_CBOR => {
            serde_cbor::to_vec(&value).map_err(|e| e.to_string())
        }
        _ => serde_json::to_vec(&value).map_err(|e| e.to_string()),
    };

    match body {
        Ok(body) => {
            let mut res = Response::new(body.into());
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(content_type)
                    .unwrap_or_else(|_| HeaderValue::from_static(CONTENT_TYPE_JSON)),
            );
            res
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            e,
        )
            .into_response(),
    }
}

/// Respond with CBOR by default if the caller indicates support, otherwise JSON.
pub struct AnyResponse<T>(pub T);

impl<T> IntoResponse for AnyResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        // For now, default to JSON and allow CBOR opt-in via Accept header.
        // This keeps curl / browser debugging simple while still supporting
        // canonical CBOR payloads for production.
        let json = match serde_json::to_vec(&self.0) {
            Ok(body) => body,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };
        let mut res = Response::new(json.into());
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(CONTENT_TYPE_JSON),
        );
        res
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordKind {
    Manifest,
    AdapterAnnounce,
    RevokeEcho,
}

/// A single signed protocol record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "record")]
pub enum SignedRecord {
    Manifest(concordance_core::SchemeManifest),
    AdapterAnnounce(concordance_core::AdapterAnnouncement),
    RevokeEcho(concordance_core::RevokeEcho),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishRecordRequest {
    pub record: SignedRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishRecordResponse {
    pub cursor: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistryEvent {
    /// Local monotonic cursor for the *receiving* node's durable log.
    pub cursor: u64,
    /// The node that originally authored the event.
    pub origin_node_id: String,
    /// The original authoring node's cursor.
    pub origin_cursor: u64,
    pub record: SignedRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncEventsResponse {
    pub events: Vec<RegistryEvent>,
    pub next_cursor: u64,
}

pub fn accept_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::HeaderValue};
    use axum::http::HeaderMap;

    #[tokio::test]
    fn detect_cbor_content_type() {
        assert!(is_cbor_content_type("application/cbor"));
        assert!(is_cbor_content_type("application/concordance+cbor"));
        assert!(!is_cbor_content_type("application/json"));
    }

    #[test]
    fn detect_json_content_type() {
        assert!(is_json_content_type("application/json"));
        assert!(is_json_content_type("application/concordance+json"));
        assert!(!is_json_content_type("application/cbor"));
    }

    #[test]
    fn prefer_cbor_when_accepts_concordance_cbor() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/concordance+cbor, application/json;q=0.9"),
        );
        assert_eq!(preferred_response_content_type(Some(&headers)), CONTENT_TYPE_CONCORDANCE_CBOR);
    }

    #[test]
    fn default_to_json_when_accept_missing() {
        assert_eq!(preferred_response_content_type(None), CONTENT_TYPE_JSON);
    }

    #[tokio::test]
    async fn any_response_defaults_to_json() {
        let resp = any_response("hello", None);
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(header::CONTENT_TYPE).unwrap(), "application/json");
        let body = resp.into_body();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        assert_eq!(bytes, serde_json::to_vec(&"hello").unwrap());
    }

    #[tokio::test]
    async fn any_response_prefers_cbor_when_requested() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/concordance+cbor"),
        );
        let resp = any_response("hello", Some(&headers));
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(header::CONTENT_TYPE).unwrap(), "application/concordance+cbor");
        let body = resp.into_body();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        assert_eq!(bytes, serde_cbor::to_vec(&"hello").unwrap());
    
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/concordance+cbor"),
        );
        let resp = any_response("hello", Some(&headers));
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(header::CONTENT_TYPE).unwrap(), "application/concordance+cbor");
        let body = resp.into_body();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        assert_eq!(bytes, serde_cbor::to_vec(&"hello").unwrap());
    }
}
