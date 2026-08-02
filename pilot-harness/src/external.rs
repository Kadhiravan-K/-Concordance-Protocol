use std::fs;
use std::path::Path;
use std::time::Duration;

use concordance_adapters::{AdapterFixture, ConformanceCoverage, ConformanceReport, FixtureSourceClass};
use concordance_core::TrustAdapter;
use ed25519_dalek::{Signature, VerifyingKey};
use hex;
use reqwest::blocking::Client;
use reqwest::redirect::Policy as RedirectPolicy;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use serde_cbor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExternalManifestError {
    #[error("manifest URI must not be empty")]
    EmptyManifestUri,
    #[error("failed to fetch manifest from {0}: {1}")]
    FetchManifest(String, String),
    #[error("unsupported URI scheme: {0}")]
    UnsupportedUriScheme(String),
    #[error("invalid manifest payload: {0}")]
    InvalidManifest(String),
    #[error("fixture payload fetch failed for {0}: {1}")]
    FixtureFetch(String, String),
    #[error("fixture {fixture_name} is missing payload_uri and payload_base64")]
    MissingPayload { fixture_name: String },
    #[error("manifest content-type not allowed: {0:?}")]
    ContentTypeNotAllowed(Option<String>),
    #[error("uri host not in allowlist: {0}")]
    HostNotAllowed(String),
    #[error("resource too large: {0} bytes (limit {1})")]
    ResourceTooLarge(u64, u64),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExternalFixtureExpectation {
    Strength(f64),
    Reject,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalFixtureEntry {
    pub name: String,
    pub payload_uri: Option<String>,
    pub payload_base64: Option<String>,
    pub expectation: ExternalFixtureExpectation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalFixtureManifest {
    pub version: String,
    pub source_class: FixtureSourceClass,
    pub source_identifier: String,
    pub verification_policy: String,
    pub reproducibility_notes: Vec<String>,
    pub coverage: ConformanceCoverage,
    pub fixtures: Vec<ExternalFixtureEntry>,
}

#[derive(Debug, Clone)]
pub struct ExternalCanonicalFixture {
    pub name: String,
    pub payload: Vec<u8>,
    pub expectation: ExternalFixtureExpectation,
}

impl ExternalCanonicalFixture {
    pub fn borrowed(&self) -> AdapterFixture<'_> {
        let expectation = match self.expectation.clone() {
            ExternalFixtureExpectation::Strength(value) => concordance_adapters::FixtureExpectation::Strength(value),
            ExternalFixtureExpectation::Reject => concordance_adapters::FixtureExpectation::Reject,
        };
        AdapterFixture {
            name: &self.name,
            payload: &self.payload,
            expectation,
        }
    }
}

pub fn load_external_fixture_manifest(uri: &str) -> Result<ExternalFixtureManifest, ExternalManifestError> {
    if uri.trim().is_empty() {
        return Err(ExternalManifestError::EmptyManifestUri);
    }
    let (raw, content_type) = fetch_uri_contents(uri).map_err(|err| ExternalManifestError::FetchManifest(uri.to_string(), err.to_string()))?;
    // Validate content-type for network-fetched manifests. For local files, content_type is None.
    if uri.starts_with("https://") {
        // Accept application/json or any +json subtype
        let ok = content_type.as_deref().map(|ct| ct.contains("application/json") || ct.ends_with("+json")).unwrap_or(false);
        if !ok {
            return Err(ExternalManifestError::ContentTypeNotAllowed(content_type));
        }
    }
    // Attempt to parse JSON first so we can optionally verify manifest signatures
    let mut json: JsonValue = serde_json::from_slice(&raw)
        .map_err(|err| ExternalManifestError::InvalidManifest(err.to_string()))?;

    // Require manifest authentication: `publisher_key`, `signature`, and `signature_alg` must be present.
    let signature = json.get("signature").and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| ExternalManifestError::InvalidManifest("manifest missing signature".into()))?;
    let publisher_key = json.get("publisher_key").and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| ExternalManifestError::InvalidManifest("manifest missing publisher_key".into()))?;
    let signature_alg = json.get("signature_alg").and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| ExternalManifestError::InvalidManifest("manifest missing signature_alg".into()))?;

    // Only ed25519 supported for now
    if signature_alg.to_lowercase() != "ed25519" {
        return Err(ExternalManifestError::InvalidManifest(format!("unsupported signature_alg: {}", signature_alg)));
    }

    // Remove signature field to compute preimage; keep publisher_key in preimage
    let mut json_no_sig = json.clone();
    json_no_sig.as_object_mut().map(|m| m.remove("signature"));
    // Canonical CBOR preimage
    let preimage = serde_cbor::to_vec(&json_no_sig).map_err(|e| ExternalManifestError::InvalidManifest(e.to_string()))?;
    // Verify signature
    let sig_bytes = hex::decode(&signature).map_err(|e| ExternalManifestError::InvalidManifest(e.to_string()))?;
    let sig: Signature = Signature::from_bytes(&sig_bytes).map_err(|_| ExternalManifestError::InvalidManifest("bad signature encoding".into()))?;
    let pk_bytes = hex::decode(&publisher_key).map_err(|e| ExternalManifestError::InvalidManifest(e.to_string()))?;
    let vk = VerifyingKey::from_bytes(&pk_bytes).map_err(|_| ExternalManifestError::InvalidManifest("bad publisher_key encoding".into()))?;
    vk.verify(&preimage, &sig).map_err(|_| ExternalManifestError::InvalidManifest("manifest signature verification failed".into()))?;

    let manifest: ExternalFixtureManifest = serde_json::from_value(json)
        .map_err(|err| ExternalManifestError::InvalidManifest(err.to_string()))?;
    if manifest.version != "concordance-external-fixture-manifest/v1" {
        return Err(ExternalManifestError::InvalidManifest(format!(
            "unsupported manifest version: {}",
            manifest.version
        )));
    }
    Ok(manifest)
}

pub fn report_from_external_manifest_uri(
    adapter_id: &str,
    adapter: &dyn TrustAdapter,
    manifest_uri: &str,
) -> Result<ConformanceReport, ExternalManifestError> {
    let manifest = load_external_fixture_manifest(manifest_uri)?;
    let fixtures = fetch_external_fixtures(&manifest)?;
    let borrowed: Vec<_> = fixtures.iter().map(|f| f.borrowed()).collect();
    Ok(concordance_adapters::generate_conformance_report(
        adapter_id,
        adapter,
        &borrowed,
        manifest.source_class,
        manifest.source_identifier,
        manifest.verification_policy,
        manifest.reproducibility_notes,
        manifest.coverage,
    ))
}

pub fn fetch_external_fixtures(
    manifest: &ExternalFixtureManifest,
) -> Result<Vec<ExternalCanonicalFixture>, ExternalManifestError> {
    let mut result = Vec::with_capacity(manifest.fixtures.len());
    for fixture in &manifest.fixtures {
        let payload = if let Some(uri) = fixture.payload_uri.as_deref() {
            fetch_uri_contents(uri)
                .map_err(|err| ExternalManifestError::FixtureFetch(fixture.name.clone(), err.to_string()))?
                .0
        } else if let Some(base64_text) = fixture.payload_base64.as_deref() {
            base64::decode(base64_text)
                .map_err(|err| ExternalManifestError::FixtureFetch(fixture.name.clone(), err.to_string()))?
        } else {
            return Err(ExternalManifestError::MissingPayload {
                fixture_name: fixture.name.clone(),
            });
        };
        result.push(ExternalCanonicalFixture {
            name: fixture.name.clone(),
            payload,
            expectation: fixture.expectation.clone(),
        });
    }
    Ok(result)
}

fn fetch_uri_contents(uri: &str) -> Result<(Vec<u8>, Option<String>), Box<dyn std::error::Error>> {
    // Security policy defaults:
    // - Only allow https:// for remote fetches
    // - file:// and raw paths are allowed only when PILOT_HARNESS_ALLOW_LOCAL=1 and
    //   PILOT_HARNESS_LOCAL_BASE_DIR is set and the requested path is inside that dir.
    // - Enforce HTTP timeouts and optional max content length via PILOT_HARNESS_MAX_BYTES (default 1MB).

    const DEFAULT_MAX_BYTES: u64 = 1_048_576; // 1 MiB
    let max_bytes: u64 = std::env::var("PILOT_HARNESS_MAX_BYTES").ok().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_MAX_BYTES);

    if uri.starts_with("https://") {
        // Timeouts and redirect policy
        let connect_secs: u64 = std::env::var("PILOT_HARNESS_HTTP_CONNECT_TIMEOUT_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
        let read_secs: u64 = std::env::var("PILOT_HARNESS_HTTP_READ_TIMEOUT_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(10);
        let max_redirects: usize = std::env::var("PILOT_HARNESS_MAX_REDIRECTS").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(connect_secs))
            .timeout(Duration::from_secs(read_secs))
            .redirect(RedirectPolicy::limited(max_redirects))
            .build()?;

        // Optional allowlist of hosts (comma-separated)
        if let Ok(list) = std::env::var("PILOT_HARNESS_ALLOWLIST_HOSTS") {
            if !list.trim().is_empty() {
                let url = reqwest::Url::parse(uri)?;
                let host = url.host_str().ok_or_else(|| format!("no host in url: {}", uri))?;
                let allowed: Vec<&str> = list.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                if !allowed.iter().any(|a| *a == host) {
                    return Err(Box::new(ExternalManifestError::HostNotAllowed(host.to_string())));
                }
            }
        }

        let response = client.get(uri).send()?;
        if !response.status().is_success() {
            return Err(format!("http status {}", response.status()).into());
        }
        if let Some(len_val) = response.headers().get(reqwest::header::CONTENT_LENGTH) {
            if let Ok(len_str) = len_val.to_str() {
                if let Ok(len) = len_str.parse::<u64>() {
                    if len > max_bytes {
                        return Err(format!("content-length {} exceeds limit {}", len, max_bytes).into());
                    }
                }
            }
        }
        // Capture content-type header if present
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        // Read body (no streaming cap available in blocking client without more code)
        let bytes = response.bytes()?;
        if (bytes.len() as u64) > max_bytes {
            return Err(format!("response size {} exceeds limit {}", bytes.len(), max_bytes).into());
        }
        Ok((bytes.to_vec(), content_type))
    } else if uri.starts_with("file://") {
        // Only allow local file reads when explicitly enabled and scoped to base dir
        if std::env::var("PILOT_HARNESS_ALLOW_LOCAL").as_deref().unwrap_or("") != "1" {
            return Err(format!("local file access disallowed: {}", uri).into());
        }
        let base = std::env::var("PILOT_HARNESS_LOCAL_BASE_DIR").map_err(|_| format!("local base dir not configured"))?;
        let path = uri.trim_start_matches("file://");
        // On Windows URIs may start with a leading slash before drive letter
        let normalized_path = path.trim_start_matches('/');
        let canonical = std::path::Path::new(normalized_path).canonicalize()?;
        let base_canonical = std::path::Path::new(&base).canonicalize()?;
        if !canonical.starts_with(&base_canonical) {
            return Err(format!("local path not allowed: {}", canonical.display()).into());
        }
        let metadata = fs::metadata(&canonical)?;
        if metadata.len() > max_bytes {
            return Err(format!("local file {} too large: {} bytes", canonical.display(), metadata.len()).into());
        }
        let data = fs::read(canonical)?;
        Ok((data, None))
    } else {
        // Plain filesystem path
        if std::env::var("PILOT_HARNESS_ALLOW_LOCAL").as_deref().unwrap_or("") != "1" {
            return Err(format!("local path access disallowed: {}", uri).into());
        }
        let base = std::env::var("PILOT_HARNESS_LOCAL_BASE_DIR").map_err(|_| format!("local base dir not configured"))?;
        let path = Path::new(uri);
        if path.exists() {
            let canonical = path.canonicalize()?;
            let base_canonical = std::path::Path::new(&base).canonicalize()?;
            if !canonical.starts_with(&base_canonical) {
                return Err(format!("local path not allowed: {}", canonical.display()).into());
            }
            let metadata = fs::metadata(&canonical)?;
            if metadata.len() > max_bytes {
                return Err(format!("local file {} too large: {} bytes", canonical.display(), metadata.len()).into());
            }
            let data = fs::read(canonical)?;
            Ok((data, None))
        } else {
            Err(format!("unsupported URI or missing file: {}", uri).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn local_file_access_blocked_by_default() {
        let td = TempDir::new().unwrap();
        let path = td.path().join("test.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "hello").unwrap();
        let uri = format!("file://{}", path.display());
        let res = fetch_uri_contents(&uri);
        assert!(res.is_err());
    }

    #[test]
    fn local_file_access_allowed_when_configured_and_within_base() {
        let td = TempDir::new().unwrap();
        let base = td.path().to_path_buf();
        std::env::set_var("PILOT_HARNESS_ALLOW_LOCAL", "1");
        std::env::set_var("PILOT_HARNESS_LOCAL_BASE_DIR", base.to_string_lossy().to_string());
        let path = base.join("fixture.json");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "{{\"a\":1}}\n").unwrap();
        let uri = format!("file://{}", path.display());
        let (res, _ct) = fetch_uri_contents(&uri).expect("should read local file");
        assert!(res.starts_with(b"{"));
    }

    #[test]
    fn manifest_signature_verify_roundtrip() {
        // Prepare manifest JSON without signature
        let manifest = json!({
            "version": "concordance-external-fixture-manifest/v1",
            "source_class": "ExternalFixture",
            "source_identifier": "test",
            "verification_policy": "none",
            "reproducibility_notes": [],
            "coverage": {"malformed": false, "revoked": false, "expired": false, "signature_tamper": false},
            "fixtures": []
        });
        // Prepare publisher key and include it in the signed preimage
        let sk = SigningKey::from_bytes(&[5u8; 32]).unwrap();
        let pubhex = hex::encode(sk.verifying_key().to_bytes());
        let mut manifest_with_key = manifest.clone();
        manifest_with_key.as_object_mut().unwrap().insert("publisher_key".into(), json!(pubhex));
        // Compute CBOR preimage and sign it
        let preimage = serde_cbor::to_vec(&manifest_with_key).unwrap();
        let sig = sk.sign(&preimage);
        let sighex = hex::encode(sig.to_bytes());

        // Compose signed manifest with signature field
        let mut signed = manifest_with_key.clone();
        signed.as_object_mut().unwrap().insert("signature".into(), json!(sighex));

        // Write to temp file and read using load_external_fixture_manifest
        let td = TempDir::new().unwrap();
        let path = td.path().join("manifest.json");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "{}", serde_json::to_string(&signed).unwrap()).unwrap();

        // Allow local reads for test
        std::env::set_var("PILOT_HARNESS_ALLOW_LOCAL", "1");
        std::env::set_var("PILOT_HARNESS_LOCAL_BASE_DIR", td.path().to_string_lossy().to_string());

        let uri = format!("file://{}", path.display());
            // The signed manifest includes publisher_key and signature fields; loader requires they be present.
            let got = load_external_fixture_manifest(&uri).expect("manifest should verify");
        assert_eq!(got.version, "concordance-external-fixture-manifest/v1");
    }

        #[test]
        fn unsigned_manifest_is_rejected() {
            let manifest = json!({
                "version": "concordance-external-fixture-manifest/v1",
                "source_class": "ExternalFixture",
                "source_identifier": "test",
                "verification_policy": "none",
                "reproducibility_notes": [],
                "coverage": {"malformed": false, "revoked": false, "expired": false, "signature_tamper": false},
                "fixtures": []
            });
            let td = TempDir::new().unwrap();
            let path = td.path().join("manifest.json");
            let mut f = File::create(&path).unwrap();
            writeln!(f, "{}", serde_json::to_string(&manifest).unwrap()).unwrap();
            std::env::set_var("PILOT_HARNESS_ALLOW_LOCAL", "1");
            std::env::set_var("PILOT_HARNESS_LOCAL_BASE_DIR", td.path().to_string_lossy().to_string());
            let uri = format!("file://{}", path.display());
            let err = load_external_fixture_manifest(&uri).expect_err("unsigned manifest should be rejected");
            match err {
                ExternalManifestError::InvalidManifest(_) => {}
                _ => panic!("expected InvalidManifest for unsigned manifest"),
            }
        }

        #[test]
        fn invalid_signature_is_rejected() {
            let manifest = json!({
                "version": "concordance-external-fixture-manifest/v1",
                "source_class": "ExternalFixture",
                "source_identifier": "test",
                "verification_policy": "none",
                "reproducibility_notes": [],
                "coverage": {"malformed": false, "revoked": false, "expired": false, "signature_tamper": false},
                "fixtures": [],
                "publisher_key": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "signature_alg": "ed25519",
                "signature": "00"
            });
            let td = TempDir::new().unwrap();
            let path = td.path().join("manifest.json");
            let mut f = File::create(&path).unwrap();
            writeln!(f, "{}", serde_json::to_string(&manifest).unwrap()).unwrap();
            std::env::set_var("PILOT_HARNESS_ALLOW_LOCAL", "1");
            std::env::set_var("PILOT_HARNESS_LOCAL_BASE_DIR", td.path().to_string_lossy().to_string());
            let uri = format!("file://{}", path.display());
            let err = load_external_fixture_manifest(&uri).expect_err("invalid signature should be rejected");
            match err {
                ExternalManifestError::InvalidManifest(_) => {}
                _ => panic!("expected InvalidManifest for invalid signature"),
            }
        }
}
