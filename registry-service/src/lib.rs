//! Axum reference registry node for the Phase 4 federated pilot.
//!
//! This service is intentionally *not* a production trust authority. It stores
//! signed protocol records (manifests, adapter announcements, revocation echoes)
//! and supports:
//! - Federated pull-based sync between nodes
//! - Durable (append-only) event logging for replay/catch-up
//! - Revocation delivery via SSE with polling fallback

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::{Path as AxumPath, Query, State},
    http::{HeaderMap, StatusCode},
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
}

impl AppState {
    pub fn store(&self) -> Arc<RegistryStore> {
        self.store.clone()
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
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/records", post(publish_record))
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

async fn publish_record(
    State(state): State<AppState>,
    AnyBody(req): AnyBody<PublishRecordRequest>,
    headers: HeaderMap,
) -> Result<axum::response::Response, (StatusCode, String)> {
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
    let store = Arc::new(RegistryStore::open(data_dir).await?);
    let (events_tx, _events_rx) = broadcast::channel(1_024);
    Ok(AppState {
        node_id,
        store,
        events_tx,
    })
}
