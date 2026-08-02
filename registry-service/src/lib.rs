//! Axum reference registry node for the Phase 4 federated pilot.
//!
//! This service is intentionally *not* a production trust authority. It stores
//! signed protocol records (manifests, adapter announcements, revocation echoes)
//! and supports:
//! - Federated pull-based sync between nodes
//! - Durable (append-only) event logging for replay/catch-up
//! - Revocation delivery via SSE with polling fallback

use std::{
    collections::{HashMap, HashSet, VecDeque},
    env,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::{ConnectInfo, Extension, Path as AxumPath, Query, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Request, Response, StatusCode},
    middleware::{from_fn_with_state, Next},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Router,
};
use concordance_http::{
    any_response, AnyBody, PublishRecordRequest, PublishRecordResponse, RecordKind, RegistryEvent, SignedRecord,
    SyncEventsResponse,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, OpenOptions},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::{broadcast, RwLock},
};
use tokio_stream::{wrappers::BroadcastStream, Stream};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

#[derive(Clone)]
pub struct AppState {
    pub node_id: String,
    store: Arc<RegistryStore>,
    events_tx: broadcast::Sender<RegistryEvent>,
    publish_auth_config: PublishAuthConfig,
    rate_limit_config: RateLimitConfig,
    rate_limit_store: Arc<RateLimitStore>,
}

impl AppState {
    pub fn store(&self) -> Arc<RegistryStore> {
        self.store.clone()
    }
}

#[derive(Debug, Clone)]
pub struct PublisherAuthorization {
    pub allowed_subjects: HashSet<String>,
    pub allowed_public_keys: HashSet<String>,
}

impl PublisherAuthorization {
    pub fn authorizes_record(&self, record: &SignedRecord) -> bool {
        match record {
            SignedRecord::Manifest(manifest) => {
                self.allowed_public_keys.contains(&manifest.agent_key)
                    || self.allowed_subjects.contains(&manifest.agent_id)
            }
            SignedRecord::AdapterAnnounce(announcement) => {
                self.allowed_public_keys.contains(&announcement.publisher_key)
                    || self.allowed_subjects.contains(&announcement.publisher)
            }
            SignedRecord::RevokeEcho(echo) => {
                self.allowed_public_keys.contains(&echo.issuer_key)
                    || self.allowed_subjects.contains(&echo.issuer)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PublishAuthConfig {
    pub api_key_header: String,
    pub api_keys: HashMap<String, PublisherAuthorization>,
}

#[derive(Debug, Clone, Deserialize)]
struct PublishKeyStoreEntry {
    pub allowed_subjects: Option<HashSet<String>>,
    pub allowed_public_keys: Option<HashSet<String>>,
}

#[derive(Debug, Clone, Deserialize)]
struct PublishKeyStore {
    pub api_key_header: Option<String>,
    pub api_keys: HashMap<String, PublishKeyStoreEntry>,
}

impl PublishAuthConfig {
    pub fn from_env() -> Result<Self, String> {
        let key_store_json = env::var("CONCORDANCE_REGISTRY_PUBLISH_KEYSTORE").ok();
        let key_store_path = env::var("CONCORDANCE_REGISTRY_PUBLISH_KEYSTORE_PATH").ok();

        let raw_store = if let Some(path) = key_store_path {
            fs::read_to_string(&path)
                .map_err(|e| format!("failed to read publish key store path {path}: {e}"))?
        } else if let Some(json) = key_store_json {
            json
        } else {
            return Err("publish authentication requires CONCORDANCE_REGISTRY_PUBLISH_KEYSTORE or CONCORDANCE_REGISTRY_PUBLISH_KEYSTORE_PATH".into());
        };

        let store: PublishKeyStore = serde_json::from_str(&raw_store)
            .map_err(|e| format!("failed to parse publish key store: {e}"))?;

        if store.api_keys.is_empty() {
            return Err("publish key store contains no api_keys".into());
        }

        let api_key_header = store
            .api_key_header
            .unwrap_or_else(|| "X-Api-Key".to_string());

        let api_keys = store
            .api_keys
            .into_iter()
            .map(|(key, entry)| {
                let allowed_subjects = entry.allowed_subjects.unwrap_or_default();
                let allowed_public_keys = entry.allowed_public_keys.unwrap_or_default();
                (
                    key,
                    PublisherAuthorization {
                        allowed_subjects,
                        allowed_public_keys,
                    },
                )
            })
            .collect();

        Ok(Self {
            api_key_header,
            api_keys,
        })
    }

    pub fn authenticate(&self, api_key: Option<&str>) -> Option<&PublisherAuthorization> {
        api_key
            .and_then(|value| if !value.is_empty() { Some(value) } else { None })
            .and_then(|value| self.api_keys.get(value))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RateLimitInfo {
    pub per_ip_max_requests: usize,
    pub per_ip_window_secs: u64,
    pub per_key_max_requests: usize,
    pub per_key_window_secs: u64,
    pub api_key_header: String,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub per_ip_max_requests: usize,
    pub per_ip_window_secs: u64,
    pub per_key_max_requests: usize,
    pub per_key_window_secs: u64,
}

#[derive(Debug)]
pub struct RateLimitStore {
    ip_buckets: tokio::sync::Mutex<HashMap<String, VecDeque<Instant>>>,
    key_buckets: tokio::sync::Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl RateLimitStore {
    pub fn new() -> Self {
        Self {
            ip_buckets: tokio::sync::Mutex::new(HashMap::new()),
            key_buckets: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    async fn check_and_record(
        &self,
        bucket: &tokio::sync::Mutex<HashMap<String, VecDeque<Instant>>>,
        identifier: &str,
        max_requests: usize,
        window: Duration,
    ) -> Option<Duration> {
        if max_requests == 0 {
            return None;
        }
        let mut store = bucket.lock().await;
        let deque = store.entry(identifier.to_string()).or_default();
        let now = Instant::now();
        let threshold = now - window;
        while deque.front().map_or(false, |ts| *ts <= threshold) {
            deque.pop_front();
        }
        if deque.len() >= max_requests {
            if let Some(oldest) = deque.front() {
                let retry_after = window.saturating_sub(now.saturating_duration_since(*oldest));
                return Some(retry_after);
            }
            return Some(window);
        }
        deque.push_back(now);
        None
    }

    pub async fn record_ip(&self, ip: &str, max_requests: usize, window: Duration) -> Option<Duration> {
        self.check_and_record(&self.ip_buckets, ip, max_requests, window).await
    }

    pub async fn record_key(&self, key: &str, max_requests: usize, window: Duration) -> Option<Duration> {
        self.check_and_record(&self.key_buckets, key, max_requests, window).await
    }
}

impl RateLimitConfig {
    pub fn from_env() -> Self {
        let per_ip_max_requests = env::var("CONCORDANCE_REGISTRY_RATE_LIMIT_PER_IP_MAX")
            .ok()
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(50);
        let per_ip_window_secs = env::var("CONCORDANCE_REGISTRY_RATE_LIMIT_PER_IP_WINDOW_SECS")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(60);
        let per_key_max_requests = env::var("CONCORDANCE_REGISTRY_RATE_LIMIT_PER_KEY_MAX")
            .ok()
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(200);
        let per_key_window_secs = env::var("CONCORDANCE_REGISTRY_RATE_LIMIT_PER_KEY_WINDOW_SECS")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(60);

        Self {
            per_ip_max_requests,
            per_ip_window_secs,
            per_key_max_requests,
            per_key_window_secs,
        }
    }

    pub fn info(&self, api_key_header: String) -> RateLimitInfo {
        RateLimitInfo {
            per_ip_max_requests: self.per_ip_max_requests,
            per_ip_window_secs: self.per_ip_window_secs,
            per_key_max_requests: self.per_key_max_requests,
            per_key_window_secs: self.per_key_window_secs,
            api_key_header,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventsQuery {
    pub after: Option<u64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdapterQuery {
    pub scheme_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamQuery {
    pub after: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityQuery {
    pub after: Option<u64>,
    pub limit: Option<usize>,
    pub kind: Option<RecordKind>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub node_id: String,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub node_id: String,
    pub event_count: u64,
    pub next_cursor: u64,
    pub manifest_count: usize,
    pub adapter_count: usize,
    pub revocation_count: usize,
    pub rate_limit: RateLimitInfo,
}

pub fn router(state: AppState) -> Router {
    let publish_middleware = from_fn_with_state(state.clone(), publish_rate_limit_middleware);
    Router::new()
        .route("/health", get(health))
        .route("/v1/records", post(publish_record).layer(publish_middleware))
        .route("/v1/manifests/:agent_id", get(get_manifest))
        .route("/v1/adapters", get(get_adapters))
        .route("/v1/revocations/:envelope_id", get(get_revocation))
        .route("/v1/revocations/stream", get(revocation_stream))
        .route("/v1/sync/events", get(sync_events))
        .route("/v1/observability/metrics", get(observability_metrics))
        .route("/v1/observability/audit-log", get(observability_audit_log))
        .route("/v1/observability/decision-history", get(observability_decision_history))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health(State(state): State<AppState>, headers: HeaderMap) -> axum::response::Response {
    any_response(
        HealthResponse {
            node_id: state.node_id,
        },
        Some(&headers),
    )
}

fn parse_header_name(name: &str) -> Option<HeaderName> {
    HeaderName::from_bytes(name.as_bytes()).ok()
}

fn client_ip_from_request(connect_info: Option<ConnectInfo<SocketAddr>>, req: &Request<Body>) -> Option<String> {
    if let Some(connect_info) = connect_info {
        return Some(connect_info.0.ip().to_string());
    }

    req.headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|raw| raw.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
}

fn unauthorized_response(api_key_header: &str) -> Response {
    let mut response = Response::new("missing or invalid API key".into());
    *response.status_mut() = StatusCode::UNAUTHORIZED;
    if let Ok(value) = HeaderValue::from_str(&format!("ApiKey realm=\"concordance\", header=\"{}\"", api_key_header)) {
        response.headers_mut().insert(header::WWW_AUTHENTICATE, value);
    }
    response
}

fn too_many_requests_response(retry_after: Duration) -> Response {
    let mut response = Response::new(format!("rate limit exceeded; retry after {}s", retry_after.as_secs()).into());
    *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
    if let Ok(value) = HeaderValue::from_str(&retry_after.as_secs().to_string()) {
        response.headers_mut().insert(header::RETRY_AFTER, value);
    }
    response
}

async fn publish_rate_limit_middleware(
    State(state): State<AppState>,
    connect_info: ConnectInfo<SocketAddr>,
    mut req: Request<Body>,
    next: Next<Body>,
) -> Response {
    let api_key_header = &state.publish_auth_config.api_key_header;
    let header_name = match parse_header_name(api_key_header) {
        Some(name) => name,
        None => return unauthorized_response(api_key_header),
    };

    let api_key = req
        .headers()
        .get(header_name)
        .and_then(|value| value.to_str().ok());

    let auth = match state.publish_auth_config.authenticate(api_key) {
        Some(auth) => auth,
        None => return unauthorized_response(api_key_header),
    };

    let rate_limit_key = api_key.unwrap_or_default();
    let key_retry = state
        .rate_limit_store
        .record_key(
            rate_limit_key,
            state.rate_limit_config.per_key_max_requests,
            Duration::from_secs(state.rate_limit_config.per_key_window_secs),
        )
        .await;

    let ip_retry = if let Some(ip) = client_ip_from_request(Some(connect_info), &req) {
        state
            .rate_limit_store
            .record_ip(
                &ip,
                state.rate_limit_config.per_ip_max_requests,
                Duration::from_secs(state.rate_limit_config.per_ip_window_secs),
            )
            .await
    } else {
        None
    };

    let retry_after = match (ip_retry, key_retry) {
        (Some(ip_retry), Some(key_retry)) => Some(ip_retry.max(key_retry)),
        (Some(ip_retry), None) => Some(ip_retry),
        (None, Some(key_retry)) => Some(key_retry),
        _ => None,
    };

    if let Some(retry_after) = retry_after {
        return too_many_requests_response(retry_after);
    }

    let auth = auth.clone();
    req.extensions_mut().insert(auth);
    next.run(req).await
}

async fn publish_record(
    State(state): State<AppState>,
    AnyBody(req): AnyBody<PublishRecordRequest>,
    headers: HeaderMap,
    Extension(auth): Extension<PublisherAuthorization>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    if !auth.authorizes_record(&req.record) {
        return Err((StatusCode::FORBIDDEN, "publisher is not authorized for this record".into()));
    }

    let cursor = state
        .store
        .publish_local(&state.node_id, req.record.clone())
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Broadcast for SSE subscribers. (Subscribers can filter on kind.)
    let _ = state.events_tx.send(RegistryEvent {
        cursor,
        origin_node_id: state.node_id.clone(),
        origin_cursor: cursor,
        record: req.record,
    });

    Ok(any_response(
        PublishRecordResponse { cursor },
        Some(&headers),
    ))
}

async fn get_manifest(
    State(state): State<AppState>,
    AxumPath(agent_id): AxumPath<String>,
    headers: HeaderMap,
) -> Result<axum::response::Response, StatusCode> {
    state
        .store
        .get_manifest(&agent_id)
        .await
        .map(|manifest| any_response(manifest, Some(&headers)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn get_adapters(
    State(state): State<AppState>,
    Query(q): Query<AdapterQuery>,
    headers: HeaderMap,
) -> Result<axum::response::Response, StatusCode> {
    state
        .store
        .get_adapters(&q.scheme_uri)
        .await
        .map(|adapters| any_response(adapters, Some(&headers)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn get_revocation(
    State(state): State<AppState>,
    AxumPath(envelope_id): AxumPath<String>,
    headers: HeaderMap,
) -> Result<axum::response::Response, StatusCode> {
    state
        .store
        .get_revocation(&envelope_id)
        .await
        .map(|revocation| any_response(revocation, Some(&headers)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn sync_events(
    State(state): State<AppState>,
    Query(q): Query<EventsQuery>,
    headers: HeaderMap,
) -> axum::response::Response {
    let after = q.after.unwrap_or(0);
    let limit = q.limit.unwrap_or(250).min(2_000);
    any_response(state.store.events_since(after, limit).await, Some(&headers))
}

async fn observability_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> axum::response::Response {
    let metrics = state.store.metrics().await;
    any_response(
        MetricsResponse {
            node_id: state.node_id.clone(),
            event_count: metrics.event_count,
            next_cursor: metrics.next_cursor,
            manifest_count: metrics.manifest_count,
            adapter_count: metrics.adapter_count,
            revocation_count: metrics.revocation_count,
            rate_limit: state
                .rate_limit_config
                .info(state.publish_auth_config.api_key_header.clone()),
        },
        Some(&headers),
    )
}

async fn observability_audit_log(
    State(state): State<AppState>,
    Query(q): Query<ObservabilityQuery>,
    headers: HeaderMap,
) -> axum::response::Response {
    let after = q.after.unwrap_or(0);
    let limit = q.limit.unwrap_or(250).min(2_000);
    any_response(
        state.store.audit_log_since(after, limit, q.kind).await,
        Some(&headers),
    )
}

async fn observability_decision_history(
    State(state): State<AppState>,
    Query(q): Query<ObservabilityQuery>,
    headers: HeaderMap,
) -> axum::response::Response {
    let after = q.after.unwrap_or(0);
    let limit = q.limit.unwrap_or(250).min(2_000);
    any_response(
        state.store.audit_log_since(after, limit, q.kind).await,
        Some(&headers),
    )
}

async fn revocation_stream(
    State(state): State<AppState>,
    Query(q): Query<StreamQuery>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let after = q.after.unwrap_or(0);

    // Catch-up first, then switch to live stream.
    let initial = state
        .store
        .revocation_events_since(after, 5_000)
        .await
        .into_iter()
        .map(|ev| Ok(event_to_sse(ev)));

    let rx = state.events_tx.subscribe();
    let live = BroadcastStream::new(rx).filter_map(|item| async move {
        match item {
            Ok(ev) => match &ev.record {
                SignedRecord::RevokeEcho(_) => Some(Ok(event_to_sse(ev))),
                _ => None,
            },
            Err(_) => None,
        }
    });

    let stream = tokio_stream::iter(initial).chain(live);

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

fn event_to_sse(ev: RegistryEvent) -> Event {
    let data = serde_json::to_string(&ev).unwrap_or_else(|_| "{}".into());
    Event::default().id(ev.cursor.to_string()).data(data)
}

#[derive(Debug)]
pub struct RegistryStore {
    data_dir: PathBuf,
    events_path: PathBuf,
    inner: RwLock<StoreInner>,
}

#[derive(Debug, Default)]
struct StoreInner {
    next_cursor: u64,
    events: Vec<RegistryEvent>,
    manifests: HashMap<String, concordance_core::SchemeManifest>,
    adapters: HashMap<String, Vec<concordance_core::AdapterAnnouncement>>,
    revocations: HashMap<String, concordance_core::RevokeEcho>,
}

impl RegistryStore {
    pub async fn open(data_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&data_dir)
            .await
            .map_err(|e| format!("failed to create data dir: {e}"))?;

        let events_path = data_dir.join("events.jsonl");
        if fs::metadata(&events_path).await.is_err() {
            // Ensure the file exists so `BufReader::new(File)` works.
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&events_path)
                .await
                .map_err(|e| format!("failed to create events log: {e}"))?;
        }

        let store = Self {
            data_dir,
            events_path,
            inner: RwLock::new(StoreInner::default()),
        };

        store.load_from_disk().await?;
        Ok(store)
    }

    async fn load_from_disk(&self) -> Result<(), String> {
        let file = OpenOptions::new()
            .read(true)
            .open(&self.events_path)
            .await
            .map_err(|e| format!("failed to open events log: {e}"))?;
        let mut reader = BufReader::new(file).lines();
        let mut inner = StoreInner::default();

        while let Some(line) = reader
            .next_line()
            .await
            .map_err(|e| format!("failed to read events log: {e}"))?
        {
            if line.trim().is_empty() {
                continue;
            }
            let ev: RegistryEvent =
                serde_json::from_str(&line).map_err(|e| format!("bad event log line: {e}"))?;
            inner.next_cursor = inner.next_cursor.max(ev.cursor + 1);
            apply_event_to_inner(&mut inner, ev.clone())?;
            inner.events.push(ev);
        }

        *self.inner.write().await = inner;
        Ok(())
    }

    pub async fn publish_local(&self, node_id: &str, record: SignedRecord) -> Result<u64, String> {
        verify_record(&record)?;

        let (cursor, event_json) = {
            let mut inner = self.inner.write().await;
            let cursor = inner.next_cursor.max(1);
            inner.next_cursor = cursor + 1;
            let ev = RegistryEvent {
                cursor,
                origin_node_id: node_id.to_string(),
                origin_cursor: cursor,
                record,
            };
            apply_event_to_inner(&mut inner, ev.clone())?;
            inner.events.push(ev.clone());
            let json = serde_json::to_string(&ev).map_err(|e| e.to_string())?;
            (cursor, json)
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)
            .await
            .map_err(|e| format!("failed to append events log: {e}"))?;
        file.write_all(event_json.as_bytes())
            .await
            .map_err(|e| format!("failed to write events log: {e}"))?;
        file.write_all(b"\n")
            .await
            .map_err(|e| format!("failed to write events log newline: {e}"))?;

        Ok(cursor)
    }

    /// Apply an event received from a peer. This mutates local state but does
    /// not change the event's origin metadata.
    pub async fn apply_remote(&self, ev: RegistryEvent) -> Result<Option<RegistryEvent>, String> {
        verify_record(&ev.record)?;

        let (event_json, event_for_broadcast) = {
            let mut inner = self.inner.write().await;
            // Deduplicate by (origin_node_id, origin_cursor).
            if inner.events.iter().any(|existing| {
                existing.origin_node_id == ev.origin_node_id && existing.origin_cursor == ev.origin_cursor
            }) {
                return Ok(None);
            }
            let cursor = inner.next_cursor.max(1);
            inner.next_cursor = cursor + 1;
            let local_ev = RegistryEvent {
                cursor,
                origin_node_id: ev.origin_node_id,
                origin_cursor: ev.origin_cursor,
                record: ev.record,
            };
            apply_event_to_inner(&mut inner, local_ev.clone())?;
            inner.events.push(local_ev.clone());
            let json = serde_json::to_string(&local_ev).map_err(|e| e.to_string())?;
            (json, local_ev)
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)
            .await
            .map_err(|e| format!("failed to append events log: {e}"))?;
        file.write_all(event_json.as_bytes())
            .await
            .map_err(|e| format!("failed to write events log: {e}"))?;
        file.write_all(b"\n")
            .await
            .map_err(|e| format!("failed to write events log newline: {e}"))?;

        Ok(Some(event_for_broadcast))
    }

    pub async fn events_since(&self, after: u64, limit: usize) -> SyncEventsResponse {
        let inner = self.inner.read().await;
        let mut events: Vec<_> = inner
            .events
            .iter()
            .filter(|ev| ev.cursor > after)
            .take(limit)
            .cloned()
            .collect();
        // Ensure stable sort (cursor is monotonic for locally-originated events).
        events.sort_by_key(|ev| ev.cursor);
        let next_cursor = events.last().map(|e| e.cursor).unwrap_or(after);
        SyncEventsResponse {
            events,
            next_cursor,
        }
    }

    pub async fn revocation_events_since(&self, after: u64, limit: usize) -> Vec<RegistryEvent> {
        let inner = self.inner.read().await;
        inner
            .events
            .iter()
            .filter(|ev| ev.cursor > after)
            .filter(|ev| matches!(ev.record, SignedRecord::RevokeEcho(_)))
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn audit_log_since(
        &self,
        after: u64,
        limit: usize,
        kind: Option<RecordKind>,
    ) -> Vec<RegistryEvent> {
        let inner = self.inner.read().await;
        let mut events: Vec<_> = inner
            .events
            .iter()
            .filter(|ev| ev.cursor > after)
            .filter(|ev| match &kind {
                Some(kind) => record_matches_kind(&ev.record, kind),
                None => true,
            })
            .take(limit)
            .cloned()
            .collect();
        events.sort_by_key(|ev| ev.cursor);
        events
    }

    pub async fn metrics(&self) -> MetricsResponseData {
        let inner = self.inner.read().await;
        MetricsResponseData {
            event_count: inner.events.len() as u64,
            next_cursor: inner.next_cursor,
            manifest_count: inner.manifests.len(),
            adapter_count: inner.adapters.len(),
            revocation_count: inner.revocations.len(),
        }
    }

    pub async fn get_manifest(&self, agent_id: &str) -> Option<concordance_core::SchemeManifest> {
        let inner = self.inner.read().await;
        inner.manifests.get(agent_id).cloned()
    }

    pub async fn get_adapters(
        &self,
        scheme_uri: &str,
    ) -> Option<Vec<concordance_core::AdapterAnnouncement>> {
        let inner = self.inner.read().await;
        inner.adapters.get(scheme_uri).cloned()
    }

    pub async fn get_revocation(&self, envelope_id: &str) -> Option<concordance_core::RevokeEcho> {
        let inner = self.inner.read().await;
        inner.revocations.get(envelope_id).cloned()
    }
}

fn apply_event_to_inner(inner: &mut StoreInner, ev: RegistryEvent) -> Result<(), String> {
    match ev.record {
        SignedRecord::Manifest(manifest) => {
            inner.manifests.insert(manifest.agent_id.clone(), manifest);
        }
        SignedRecord::AdapterAnnounce(announcement) => {
            inner
                .adapters
                .entry(announcement.scheme_uri.clone())
                .or_default()
                .retain(|a| a.version != announcement.version);
            inner
                .adapters
                .entry(announcement.scheme_uri.clone())
                .or_default()
                .push(announcement);
        }
        SignedRecord::RevokeEcho(echo) => {
            inner.revocations.insert(echo.envelope_id.clone(), echo);
        }
    }
    Ok(())
}

fn record_matches_kind(record: &SignedRecord, kind: &RecordKind) -> bool {
    match (record, kind) {
        (SignedRecord::Manifest(_), RecordKind::Manifest) => true,
        (SignedRecord::AdapterAnnounce(_), RecordKind::AdapterAnnounce) => true,
        (SignedRecord::RevokeEcho(_), RecordKind::RevokeEcho) => true,
        _ => false,
    }
}

#[derive(Debug, Serialize)]
pub struct MetricsResponseData {
    pub event_count: u64,
    pub next_cursor: u64,
    pub manifest_count: usize,
    pub adapter_count: usize,
    pub revocation_count: usize,
}

fn verify_record(record: &SignedRecord) -> Result<(), String> {
    match record {
        SignedRecord::Manifest(m) => m.verify().map_err(|e| e.to_string()),
        SignedRecord::AdapterAnnounce(a) => a.verify().map_err(|e| e.to_string()),
        SignedRecord::RevokeEcho(r) => r.verify_signature().map_err(|e| e.to_string()),
    }
}

pub async fn spawn_peer_sync(
    state: AppState,
    peer_base_url: String,
    interval: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let mut cursor: u64 = 0;
        loop {
            let url = format!("{}/v1/sync/events?after={}", peer_base_url.trim_end_matches('/'), cursor);
            match client.get(&url).send().await {
                Ok(resp) => match resp.error_for_status() {
                    Ok(ok) => match ok.json::<SyncEventsResponse>().await {
                        Ok(body) => {
                            if !body.events.is_empty() {
                                info!(
                                    peer = %peer_base_url,
                                    count = body.events.len(),
                                    "synced events"
                                );
                            }
                            for ev in body.events {
                                match state.store.apply_remote(ev).await {
                                    Ok(Some(stored)) => {
                                        let _ = state.events_tx.send(stored);
                                    }
                                    Ok(None) => {}
                                    Err(e) => warn!(peer = %peer_base_url, error = %e, "failed to apply remote event"),
                                }
                            }
                            cursor = cursor.max(body.next_cursor);
                        }
                        Err(e) => warn!(peer = %peer_base_url, error = %e, "bad sync response json"),
                    },
                    Err(e) => warn!(peer = %peer_base_url, error = %e, "sync response error"),
                },
                Err(e) => warn!(peer = %peer_base_url, error = %e, "peer unreachable"),
            }
            tokio::time::sleep(interval).await;
        }
    })
}

pub async fn build_state(node_id: String, data_dir: PathBuf) -> Result<AppState, String> {
    let publish_auth_config = PublishAuthConfig::from_env()?;
    let rate_limit_config = RateLimitConfig::from_env();
    build_state_with_config(node_id, data_dir, publish_auth_config, rate_limit_config).await
}

pub async fn build_state_with_publish_config(
    node_id: String,
    data_dir: PathBuf,
    publish_auth_config: PublishAuthConfig,
) -> Result<AppState, String> {
    let rate_limit_config = RateLimitConfig::from_env();
    build_state_with_config(node_id, data_dir, publish_auth_config, rate_limit_config).await
}

pub async fn build_state_with_config(
    node_id: String,
    data_dir: PathBuf,
    publish_auth_config: PublishAuthConfig,
    rate_limit_config: RateLimitConfig,
) -> Result<AppState, String> {
    let store = Arc::new(RegistryStore::open(data_dir).await?);
    let (events_tx, _events_rx) = broadcast::channel(1_024);
    Ok(AppState {
        node_id,
        store,
        events_tx,
        publish_auth_config,
        rate_limit_config,
        rate_limit_store: Arc::new(RateLimitStore::new()),
    })
}
