use std::fs;
use std::path::Path;

use concordance_adapters::{AdapterFixture, ConformanceCoverage, ConformanceReport, FixtureSourceClass};
use concordance_core::TrustAdapter;
use reqwest::blocking::Client;
use serde::Deserialize;
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
    let raw = fetch_uri_contents(uri).map_err(|err| ExternalManifestError::FetchManifest(uri.to_string(), err.to_string()))?;
    let manifest: ExternalFixtureManifest = serde_json::from_slice(&raw)
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

fn fetch_uri_contents(uri: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if uri.starts_with("http://") || uri.starts_with("https://") {
        let client = Client::builder().build()?;
        let response = client.get(uri).send()?;
        if !response.status().is_success() {
            return Err(format!("http status {}", response.status()).into());
        }
        Ok(response.bytes()?.to_vec())
    } else if uri.starts_with("file://") {
        let path = uri.trim_start_matches("file://");
        let normalized_path = path.trim_start_matches('/');
        Ok(fs::read(normalized_path)?)
    } else {
        let path = Path::new(uri);
        if path.exists() {
            Ok(fs::read(path)?)
        } else {
            Err(format!("unsupported URI or missing file: {}", uri).into())
        }
    }
}
