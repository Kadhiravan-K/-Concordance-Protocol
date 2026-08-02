use std::{net::SocketAddr, time::Duration};

use concordance_core::{AdapterAnnouncement, Polarity, RevokeEcho, TrustObjectEnvelope};
use concordance_http::{PublishRecordRequest, SignedRecord};
use ed25519_dalek::SigningKey;
use tempfile::TempDir;

const TEST_API_KEY: &str = "test-key";

fn key(byte: u8) -> SigningKey {
    SigningKey::from_bytes(&[byte; 32])
}

fn default_publish_auth_config() -> concordance_registry_service::PublishAuthConfig {
    concordance_registry_service::PublishAuthConfig {
        api_key_header: "X-Api-Key".to_string(),
        allowed_api_keys: vec![TEST_API_KEY.to_string()].into_iter().collect(),
    }
}

fn default_rate_limit_config() -> concordance_registry_service::RateLimitConfig {
    concordance_registry_service::RateLimitConfig {
        per_ip_max_requests: 50,
        per_ip_window_secs: 60,
        per_key_max_requests: 200,
        per_key_window_secs: 60,
    }
}

async fn serve(
    node_id: &str,
    data_dir: &TempDir,
) -> (concordance_registry_service::AppState, String) {
    let state = concordance_registry_service::build_state_with_config(
        node_id.to_string(),
        data_dir.path().into(),
        default_publish_auth_config(),
        default_rate_limit_config(),
    )
    .await
    .expect("state");
    let app = concordance_registry_service::router(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr: SocketAddr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .expect("serve");
    });
    (state, format!("http://{}", addr))
}

#[tokio::test]
async fn two_nodes_sync_adapters_and_revocations() {
    let dir_a = TempDir::new().expect("tmp");
    let dir_b = TempDir::new().expect("tmp");

    let (_state_a, base_a) = serve("node-a", &dir_a).await;
    let (state_b, base_b) = serve("node-b", &dir_b).await;

    // B pulls from A.
    concordance_registry_service::spawn_peer_sync(state_b.clone(), base_a.clone(), Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Publish an adapter announcement to A.
    let ann = AdapterAnnouncement::sign(
        "urn:example:scheme:demo:v1".into(),
        "urn:example:adapter:demo:v1".into(),
        "1.0.0".into(),
        "did:example:publisher".into(),
        "fixtures://demo".into(),
        &key(7),
    )
    .unwrap();
    client
        .post(format!("{}/v1/records", base_a))
        .header("content-type", "application/json")
        .header("X-Api-Key", TEST_API_KEY)
        .json(&PublishRecordRequest {
            record: SignedRecord::AdapterAnnounce(ann.clone()),
        })
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Publish a revocation to A.
    let issuer_key = key(1);
    let presenter_key = key(2);
    let envelope = TrustObjectEnvelope::sign(
        "urn:example:scheme:demo:v1".into(),
        "demo".into(),
        Polarity::Support,
        "did:example:subject".into(),
        "did:example:issuer".into(),
        b"native".to_vec(),
        1.0,
        "urn:example:adapter:demo:v1".into(),
        1_000,
        None,
        None,
        None,
        &issuer_key,
        &presenter_key,
        "s1".into(),
    )
    .unwrap();
    let echo = RevokeEcho::sign(&envelope, 1, 2_000, "test".into(), &issuer_key).unwrap();
    client
        .post(format!("{}/v1/records", base_a))
        .header("content-type", "application/json")
        .header("X-Api-Key", TEST_API_KEY)
        .json(&PublishRecordRequest {
            record: SignedRecord::RevokeEcho(echo.clone()),
        })
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Wait briefly for sync.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Adapter is visible via B.
    let adapters: Vec<AdapterAnnouncement> = client
        .get(format!("{}/v1/adapters", base_b))
        .query(&[("scheme_uri", "urn:example:scheme:demo:v1")])
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(adapters.len(), 1);
    assert_eq!(adapters[0].version, "1.0.0");

    // Revocation is visible via B.
    let echoed: RevokeEcho = client
        .get(format!("{}/v1/revocations/{}", base_b, envelope.envelope_id))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(echoed.sequence, 1);
}

#[tokio::test]
async fn publish_requires_api_key() {
    let dir = TempDir::new().expect("tmp");
    let (_state, base) = serve("node-auth", &dir).await;

    let client = reqwest::Client::new();
    let ann = AdapterAnnouncement::sign(
        "urn:example:scheme:auth-demo:v1".into(),
        "urn:example:adapter:auth-demo:v1".into(),
        "1.0.0".into(),
        "did:example:publisher".into(),
        "fixtures://demo".into(),
        &key(7),
    )
    .unwrap();

    let resp = client
        .post(format!("{}/v1/records", base))
        .header("content-type", "application/json")
        .json(&PublishRecordRequest {
            record: SignedRecord::AdapterAnnounce(ann),
        })
        .send()
        .await
        .expect("send");

    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn publish_rate_limit_exceeded() {
    let dir = TempDir::new().expect("tmp");
    let state = concordance_registry_service::build_state_with_config(
        "node-rate".to_string(),
        dir.path().into(),
        default_publish_auth_config(),
        concordance_registry_service::RateLimitConfig {
            per_ip_max_requests: 1,
            per_ip_window_secs: 60,
            per_key_max_requests: 1,
            per_key_window_secs: 60,
        },
    )
    .await
    .expect("state");
    let app = concordance_registry_service::router(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr: SocketAddr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .expect("serve");
    });
    let base = format!("http://{}", addr);

    let client = reqwest::Client::new();
    let ann = AdapterAnnouncement::sign(
        "urn:example:scheme:limit-demo:v1".into(),
        "urn:example:adapter:limit-demo:v1".into(),
        "1.0.0".into(),
        "did:example:publisher".into(),
        "fixtures://demo".into(),
        &key(7),
    )
    .unwrap();

    let first = client
        .post(format!("{}/v1/records", base))
        .header("content-type", "application/json")
        .header("X-Api-Key", TEST_API_KEY)
        .json(&PublishRecordRequest {
            record: SignedRecord::AdapterAnnounce(ann.clone()),
        })
        .send()
        .await
        .expect("first send");
    assert!(first.status().is_success());

    let second = client
        .post(format!("{}/v1/records", base))
        .header("content-type", "application/json")
        .header("X-Api-Key", TEST_API_KEY)
        .json(&PublishRecordRequest {
            record: SignedRecord::AdapterAnnounce(ann),
        })
        .send()
        .await
        .expect("second send");

    assert_eq!(second.status(), reqwest::StatusCode::TOO_MANY_REQUESTS);
}

