use std::{net::SocketAddr, time::Duration};

use concordance_core::{AdapterAnnouncement, Polarity, RevokeEcho, TrustObjectEnvelope};
use concordance_http::{PublishRecordRequest, SignedRecord};
use ed25519_dalek::SigningKey;
use tempfile::TempDir;

fn key(byte: u8) -> SigningKey {
    SigningKey::from_bytes(&[byte; 32])
}

async fn serve(
    node_id: &str,
    data_dir: &TempDir,
) -> (concordance_registry_service::AppState, String) {
    let state = concordance_registry_service::build_state(node_id.to_string(), data_dir.path().into())
        .await
        .expect("state");
    let app = concordance_registry_service::router(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr: SocketAddr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
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

