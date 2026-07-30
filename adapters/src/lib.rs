//! Phase-3 local adapter SDK and fixture-based pilot adapters.
//!
//! These adapters normalize already-obtained native evidence. They intentionally
//! contain no HTTP/RPC client, chain indexer, or remote-code execution path.

use std::collections::BTreeSet;

use concordance_core::{ConcordanceError, Result, TrustAdapter};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

pub const ERC8004_SCHEME_URI: &str = "urn:concordance:scheme:erc8004:reputation:v1";
pub const ERC8004_NORMALIZER_URI: &str = "urn:concordance:adapter:erc8004:fixed-point-reputation:v1";
pub const CAPABILITY_GRANT_SCHEME_URI: &str = "urn:concordance:scheme:signed-capability-grant:v1";
pub const CAPABILITY_GRANT_NORMALIZER_URI: &str = "urn:concordance:adapter:signed-capability-grant:v1";

#[derive(Debug, Clone, Deserialize)]
pub struct Erc8004Feedback {
    pub agent_id: u64,
    pub client_address: String,
    pub feedback_index: u64,
    pub value: i64,
    pub value_decimals: u8,
    pub tag1: String,
    pub is_revoked: bool,
}

/// Local mapping policy for an ERC-8004 feedback category. ERC-8004 permits
/// arbitrary tag semantics, so a score cannot be safely normalized without an
/// explicit tag whitelist and numeric range.
pub struct Erc8004ReputationAdapter {
    accepted_tags: BTreeSet<String>,
    minimum_value: f64,
    maximum_value: f64,
}

impl Erc8004ReputationAdapter {
    pub fn new(accepted_tags: impl IntoIterator<Item = String>, minimum_value: f64, maximum_value: f64) -> std::result::Result<Self, String> {
        if !minimum_value.is_finite() || !maximum_value.is_finite() || minimum_value >= maximum_value {
            return Err("normalization range must be finite and increasing".into());
        }
        let accepted_tags: BTreeSet<_> = accepted_tags.into_iter().collect();
        if accepted_tags.is_empty() { return Err("at least one ERC-8004 feedback tag is required".into()); }
        Ok(Self { accepted_tags, minimum_value, maximum_value })
    }

    pub fn quality_0_to_100() -> Self {
        Self::new(["successRate".to_string(), "starred".to_string()], 0.0, 100.0).expect("static configuration is valid")
    }

    pub fn parse_feedback(payload: &[u8]) -> Result<Erc8004Feedback> {
        serde_json::from_slice(payload).map_err(|_| ConcordanceError::InvalidAdapterResult)
    }
}

impl TrustAdapter for Erc8004ReputationAdapter {
    fn scheme_uri(&self) -> &str { ERC8004_SCHEME_URI }
    fn normalization_fn_uri(&self) -> &str { ERC8004_NORMALIZER_URI }

    fn normalize(&self, native_payload: &[u8]) -> Result<f64> {
        let feedback = Self::parse_feedback(native_payload)?;
        if feedback.agent_id == 0 || feedback.client_address.is_empty() || feedback.feedback_index == 0 || feedback.is_revoked || feedback.value_decimals > 18 || !self.accepted_tags.contains(&feedback.tag1) {
            return Err(ConcordanceError::InvalidAdapterResult);
        }
        let fixed_point = feedback.value as f64 / 10_f64.powi(i32::from(feedback.value_decimals));
        if !fixed_point.is_finite() || fixed_point < self.minimum_value || fixed_point > self.maximum_value {
            return Err(ConcordanceError::InvalidAdapterResult);
        }
        Ok((fixed_point - self.minimum_value) / (self.maximum_value - self.minimum_value))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedCapabilityGrant {
    pub version: String,
    pub subject: String,
    pub issuer: String,
    pub issuer_key: String,
    pub capabilities: BTreeSet<String>,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
    pub granted: bool,
    pub signature: String,
}

impl SignedCapabilityGrant {
    pub fn sign(
        subject: String,
        issuer: String,
        capabilities: BTreeSet<String>,
        issued_at_ms: u64,
        expires_at_ms: u64,
        granted: bool,
        key: &SigningKey,
    ) -> Result<Self> {
        let mut grant = Self {
            version: "signed-capability-grant/v1".into(), subject, issuer,
            issuer_key: hex::encode(key.verifying_key().to_bytes()), capabilities,
            issued_at_ms, expires_at_ms, granted, signature: String::new(),
        };
        grant.signature = hex::encode(key.sign(&grant.preimage()?).to_bytes());
        Ok(grant)
    }

    pub fn verify(&self) -> Result<()> {
        let key = public_key(&self.issuer_key)?;
        key.verify(&self.preimage()?, &signature(&self.signature)?).map_err(|_| ConcordanceError::InvalidSignature)
    }

    fn preimage(&self) -> Result<Vec<u8>> {
        Ok(serde_cbor::to_vec(&(
            &self.version, &self.subject, &self.issuer, &self.issuer_key,
            &self.capabilities, self.issued_at_ms, self.expires_at_ms, self.granted,
        ))?)
    }
}

pub struct SignedCapabilityGrantAdapter {
    required_capability: String,
    trusted_issuer_keys: BTreeSet<String>,
    now_ms: u64,
}

impl SignedCapabilityGrantAdapter {
    pub fn new(required_capability: String, trusted_issuer_keys: impl IntoIterator<Item = String>, now_ms: u64) -> Self {
        Self { required_capability, trusted_issuer_keys: trusted_issuer_keys.into_iter().collect(), now_ms }
    }
}

impl TrustAdapter for SignedCapabilityGrantAdapter {
    fn scheme_uri(&self) -> &str { CAPABILITY_GRANT_SCHEME_URI }
    fn normalization_fn_uri(&self) -> &str { CAPABILITY_GRANT_NORMALIZER_URI }

    fn normalize(&self, native_payload: &[u8]) -> Result<f64> {
        let grant: SignedCapabilityGrant = serde_json::from_slice(native_payload).map_err(|_| ConcordanceError::InvalidAdapterResult)?;
        if grant.version != "signed-capability-grant/v1" || grant.subject.is_empty() || grant.issuer.is_empty() || grant.issued_at_ms > grant.expires_at_ms || self.now_ms > grant.expires_at_ms || !grant.granted || !grant.capabilities.contains(&self.required_capability) || !self.trusted_issuer_keys.contains(&grant.issuer_key) {
            return Err(ConcordanceError::InvalidAdapterResult);
        }
        grant.verify()?;
        Ok(1.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureExpectation { Strength(f64), Reject }

#[derive(Debug, Clone)]
pub struct AdapterFixture<'a> { pub name: &'a str, pub payload: &'a [u8], pub expectation: FixtureExpectation }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult { pub name: String, pub passed: bool }

pub fn run_conformance(adapter: &dyn TrustAdapter, fixtures: &[AdapterFixture<'_>]) -> Vec<FixtureResult> {
    fixtures.iter().map(|fixture| {
        let passed = match (&fixture.expectation, adapter.normalize(fixture.payload)) {
            (FixtureExpectation::Strength(expected), Ok(actual)) => (actual - expected).abs() < 1e-12,
            (FixtureExpectation::Reject, Err(_)) => true,
            _ => false,
        };
        FixtureResult { name: fixture.name.into(), passed }
    }).collect()
}

fn public_key(value: &str) -> Result<VerifyingKey> {
    let bytes: [u8; 32] = hex::decode(value).map_err(|_| ConcordanceError::InvalidHex)?.try_into().map_err(|_| ConcordanceError::InvalidPublicKey)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| ConcordanceError::InvalidPublicKey)
}
fn signature(value: &str) -> Result<Signature> {
    let bytes: [u8; 64] = hex::decode(value).map_err(|_| ConcordanceError::InvalidHex)?.try_into().map_err(|_| ConcordanceError::InvalidSignature)?;
    Ok(Signature::from_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ERC_OK: &[u8] = include_bytes!("../erc8004/fixtures/feedback-success-rate.json");
    const ERC_REVOKED: &[u8] = include_bytes!("../erc8004/fixtures/feedback-revoked.json");

    fn key(seed: u8) -> SigningKey { SigningKey::from_bytes(&[seed; 32]) }

    #[test]
    fn erc8004_fixture_contract_accepts_only_configured_active_feedback() {
        let adapter = Erc8004ReputationAdapter::quality_0_to_100();
        let results = run_conformance(&adapter, &[
            AdapterFixture { name: "success-rate", payload: ERC_OK, expectation: FixtureExpectation::Strength(0.87) },
            AdapterFixture { name: "revoked", payload: ERC_REVOKED, expectation: FixtureExpectation::Reject },
        ]);
        assert!(results.iter().all(|result| result.passed));
    }

    #[test]
    fn signed_capability_fixture_verifies_trust_expiry_and_capability() {
        let issuer = key(9);
        let trusted = hex::encode(issuer.verifying_key().to_bytes());
        let grant = SignedCapabilityGrant::sign("did:example:agent".into(), "did:example:issuer".into(), BTreeSet::from(["write:orders".into()]), 1_000, 2_000, true, &issuer).unwrap();
        let payload = serde_json::to_vec(&grant).unwrap();
        let adapter = SignedCapabilityGrantAdapter::new("write:orders".into(), [trusted], 1_500);
        assert_eq!(adapter.normalize(&payload).unwrap(), 1.0);
        let expired = SignedCapabilityGrantAdapter::new("write:orders".into(), [grant.issuer_key.clone()], 2_001);
        assert!(expired.normalize(&payload).is_err());
    }

    #[test]
    fn modified_capability_grant_is_rejected() {
        let issuer = key(10);
        let grant = SignedCapabilityGrant::sign("did:example:agent".into(), "did:example:issuer".into(), BTreeSet::from(["read:orders".into()]), 1_000, 2_000, true, &issuer).unwrap();
        let mut changed: serde_json::Value = serde_json::to_value(grant).unwrap();
        changed["granted"] = serde_json::Value::Bool(false);
        let payload = serde_json::to_vec(&changed).unwrap();
        let adapter = SignedCapabilityGrantAdapter::new("read:orders".into(), [hex::encode(issuer.verifying_key().to_bytes())], 1_500);
        assert!(adapter.normalize(&payload).is_err());
    }
}
