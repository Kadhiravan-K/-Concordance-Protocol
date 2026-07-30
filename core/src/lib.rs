//! Concordance/1.0 deterministic reference core.
//!
//! The crate deliberately contains no transport or storage implementation. It
//! provides deterministic protocol operations that a host agent may call.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use blake3::Hasher;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROTOCOL_VERSION: &str = "Concordance/1.0";

#[derive(Debug, Error)]
pub enum ConcordanceError {
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_cbor::Error),
    #[error("invalid hex encoding")]
    InvalidHex,
    #[error("invalid public key")]
    InvalidPublicKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("envelope ID does not match its canonical preimage")]
    InvalidEnvelopeId,
    #[error("payload commitment does not match native payload")]
    InvalidPayloadCommitment,
    #[error("normalized strength must be finite and in [0, 1]")]
    InvalidStrength,
    #[error("envelope is missing an issuer signature")]
    MissingSignature,
    #[error("binding proof does not bind the subject to this envelope")]
    InvalidBindingProof,
    #[error("the requested adapter is unavailable: {0}")]
    AdapterUnavailable(String),
    #[error("adapter result is invalid")]
    InvalidAdapterResult,
    #[error("revocation issuer is not authorized for this envelope")]
    UnauthorizedRevocation,
    #[error("revocation sequence was replayed or is out of order")]
    RevocationReplay,
}

pub type Result<T> = std::result::Result<T, ConcordanceError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Polarity {
    Support,
    Contradict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BindingProof {
    /// In v1 this is the same identifier as `TrustObjectEnvelope.subject`.
    pub presenter_id: String,
    pub session_id: String,
    /// Hex-encoded Ed25519 verification key.
    pub presenter_key: String,
    /// Ed25519 signature over the binding preimage described in the spec.
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrustObjectEnvelope {
    pub concordance_version: String,
    pub envelope_id: String,
    pub scheme_uri: String,
    pub claim_class: String,
    pub polarity: Polarity,
    pub subject: String,
    pub issuer: String,
    /// Hex-encoded Ed25519 verification key of the issuer.
    pub issuer_key: String,
    /// Omitted only when `redacted` is true.
    pub native_payload: Option<Vec<u8>>,
    /// BLAKE3 digest of the original native payload, whether or not redacted.
    pub payload_commitment: String,
    pub normalized_strength: f64,
    pub normalization_fn_uri: String,
    pub issued_at_ms: u64,
    pub expires_at_ms: Option<u64>,
    pub revocation_check_uri: Option<String>,
    pub independence_class: Option<String>,
    pub redacted: bool,
    /// Binding proof is deliberately excluded from issuer's hash/signature
    /// preimage, because it signs the resulting immutable envelope ID.
    pub binding_proof: BindingProof,
    /// Hex-encoded issuer Ed25519 signature over the envelope preimage.
    pub issuer_signature: Option<String>,
}

#[derive(Serialize)]
struct EnvelopePreimage<'a> {
    concordance_version: &'a str,
    scheme_uri: &'a str,
    claim_class: &'a str,
    polarity: &'a Polarity,
    subject: &'a str,
    issuer: &'a str,
    issuer_key: &'a str,
    payload_commitment: &'a str,
    normalized_strength: f64,
    normalization_fn_uri: &'a str,
    issued_at_ms: u64,
    expires_at_ms: Option<u64>,
    revocation_check_uri: &'a Option<String>,
    independence_class: &'a Option<String>,
}

impl TrustObjectEnvelope {
    #[allow(clippy::too_many_arguments)]
    pub fn sign(
        scheme_uri: String,
        claim_class: String,
        polarity: Polarity,
        subject: String,
        issuer: String,
        native_payload: Vec<u8>,
        normalized_strength: f64,
        normalization_fn_uri: String,
        issued_at_ms: u64,
        expires_at_ms: Option<u64>,
        revocation_check_uri: Option<String>,
        independence_class: Option<String>,
        issuer_key: &SigningKey,
        presenter_key: &SigningKey,
        session_id: String,
    ) -> Result<Self> {
        let issuer_key_hex = hex::encode(issuer_key.verifying_key().to_bytes());
        let payload_commitment = payload_commitment(&native_payload);
        let mut envelope = Self {
            concordance_version: PROTOCOL_VERSION.to_string(),
            envelope_id: String::new(),
            scheme_uri,
            claim_class,
            polarity,
            subject: subject.clone(),
            issuer,
            issuer_key: issuer_key_hex,
            native_payload: Some(native_payload),
            payload_commitment,
            normalized_strength,
            normalization_fn_uri,
            issued_at_ms,
            expires_at_ms,
            revocation_check_uri,
            independence_class,
            redacted: false,
            binding_proof: BindingProof {
                presenter_id: subject,
                session_id,
                presenter_key: hex::encode(presenter_key.verifying_key().to_bytes()),
                signature: String::new(),
            },
            issuer_signature: None,
        };
        envelope.validate_shape()?;
        envelope.envelope_id = envelope.compute_id()?;
        envelope.issuer_signature = Some(hex::encode(issuer_key.sign(&envelope.preimage_bytes()?).to_bytes()));
        envelope.binding_proof.signature = hex::encode(presenter_key.sign(&envelope.binding_preimage()).to_bytes());
        Ok(envelope)
    }

    pub fn redact(mut self) -> Self {
        self.native_payload = None;
        self.redacted = true;
        self
    }

    pub fn verify(&self) -> Result<()> {
        self.validate_shape()?;
        if self.envelope_id != self.compute_id()? {
            return Err(ConcordanceError::InvalidEnvelopeId);
        }
        if let Some(payload) = &self.native_payload {
            if self.payload_commitment != payload_commitment(payload) {
                return Err(ConcordanceError::InvalidPayloadCommitment);
            }
        } else if !self.redacted {
            return Err(ConcordanceError::InvalidPayloadCommitment);
        }
        let signature = self.issuer_signature.as_ref().ok_or(ConcordanceError::MissingSignature)?;
        verifying_key(&self.issuer_key)?.verify(&self.preimage_bytes()?, &signature_from_hex(signature)?)
            .map_err(|_| ConcordanceError::InvalidSignature)?;
        if self.binding_proof.presenter_id != self.subject {
            return Err(ConcordanceError::InvalidBindingProof);
        }
        verifying_key(&self.binding_proof.presenter_key)?.verify(
            &self.binding_preimage(),
            &signature_from_hex(&self.binding_proof.signature)?,
        ).map_err(|_| ConcordanceError::InvalidBindingProof)
    }

    pub fn is_stale(&self, now_ms: u64, max_age_ms: u64) -> bool {
        now_ms.saturating_sub(self.issued_at_ms) > max_age_ms
            || self.expires_at_ms.is_some_and(|expires| now_ms > expires)
    }

    fn validate_shape(&self) -> Result<()> {
        if !self.normalized_strength.is_finite() || !(0.0..=1.0).contains(&self.normalized_strength) {
            return Err(ConcordanceError::InvalidStrength);
        }
        Ok(())
    }

    fn preimage_bytes(&self) -> Result<Vec<u8>> {
        // Struct declaration order is the normative canonical CBOR map order.
        Ok(serde_cbor::to_vec(&EnvelopePreimage {
            concordance_version: &self.concordance_version,
            scheme_uri: &self.scheme_uri,
            claim_class: &self.claim_class,
            polarity: &self.polarity,
            subject: &self.subject,
            issuer: &self.issuer,
            issuer_key: &self.issuer_key,
            payload_commitment: &self.payload_commitment,
            normalized_strength: self.normalized_strength,
            normalization_fn_uri: &self.normalization_fn_uri,
            issued_at_ms: self.issued_at_ms,
            expires_at_ms: self.expires_at_ms,
            revocation_check_uri: &self.revocation_check_uri,
            independence_class: &self.independence_class,
        })?)
    }

    fn compute_id(&self) -> Result<String> {
        Ok(hash_bytes(&self.preimage_bytes()?))
    }

    fn binding_preimage(&self) -> Vec<u8> {
        format!("{PROTOCOL_VERSION}/binding/{}:{}:{}", self.envelope_id, self.subject, self.binding_proof.session_id).into_bytes()
    }
}

fn payload_commitment(payload: &[u8]) -> String { hash_bytes(payload) }
fn hash_bytes(bytes: &[u8]) -> String { hex::encode(Hasher::new().update(bytes).finalize().as_bytes()) }
fn verifying_key(key: &str) -> Result<VerifyingKey> {
    let bytes: [u8; 32] = hex::decode(key).map_err(|_| ConcordanceError::InvalidHex)?.try_into().map_err(|_| ConcordanceError::InvalidPublicKey)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| ConcordanceError::InvalidPublicKey)
}
fn signature_from_hex(value: &str) -> Result<Signature> {
    let bytes: [u8; 64] = hex::decode(value).map_err(|_| ConcordanceError::InvalidHex)?.try_into().map_err(|_| ConcordanceError::InvalidSignature)?;
    Ok(Signature::from_bytes(&bytes))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemeCapability { pub scheme_uri: String, pub claim_classes: BTreeSet<String> }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Requirement { pub min_strength: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InteractionPolicy {
    pub version: String,
    pub required_claims: BTreeMap<String, Requirement>,
    pub max_envelope_age_ms: u64,
    pub escalation_floor: f64,
    pub conflict_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemeManifest {
    pub concordance_version: String,
    pub agent_id: String,
    /// Hex-encoded Ed25519 verification key of the manifest publisher.
    pub agent_key: String,
    pub can_present: Vec<SchemeCapability>,
    pub can_verify: Vec<SchemeCapability>,
    pub policy_classes: BTreeMap<String, InteractionPolicy>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NegotiationResult { pub accepted: bool, pub missing_claim_classes: Vec<String> }

impl SchemeManifest {
    pub fn sign(&mut self, key: &SigningKey) -> Result<()> {
        self.agent_key = hex::encode(key.verifying_key().to_bytes());
        self.signature = Some(hex::encode(key.sign(&self.preimage_bytes()?).to_bytes()));
        Ok(())
    }
    pub fn verify(&self) -> Result<()> {
        let signature = self.signature.as_ref().ok_or(ConcordanceError::MissingSignature)?;
        verifying_key(&self.agent_key)?.verify(&self.preimage_bytes()?, &signature_from_hex(signature)?)
            .map_err(|_| ConcordanceError::InvalidSignature)
    }
    fn preimage_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_cbor::to_vec(&(&self.concordance_version, &self.agent_id, &self.agent_key, &self.can_present, &self.can_verify, &self.policy_classes))?)
    }
}

pub fn negotiate(verifier: &SchemeManifest, presenter: &SchemeManifest, policy_class: &str) -> Result<NegotiationResult> {
    verifier.verify()?;
    presenter.verify()?;
    let Some(policy) = verifier.policy_classes.get(policy_class) else {
        return Ok(NegotiationResult { accepted: false, missing_claim_classes: vec!["unknown-policy-class".into()] });
    };
    let presentable: BTreeSet<_> = presenter.can_present.iter().flat_map(|s| s.claim_classes.iter().cloned()).collect();
    let verifiable: BTreeSet<_> = verifier.can_verify.iter().flat_map(|s| s.claim_classes.iter().cloned()).collect();
    let mut missing = Vec::new();
    for claim in policy.required_claims.keys() {
        if !presentable.contains(claim) || !verifiable.contains(claim) { missing.push(claim.clone()); }
    }
    Ok(NegotiationResult { accepted: missing.is_empty(), missing_claim_classes: missing })
}

pub trait TrustAdapter: Send + Sync {
    fn scheme_uri(&self) -> &str;
    fn normalization_fn_uri(&self) -> &str;
    fn normalize(&self, native_payload: &[u8]) -> Result<f64>;
}

#[derive(Default)]
pub struct AdapterRegistry { adapters: HashMap<String, Box<dyn TrustAdapter>> }
impl AdapterRegistry {
    pub fn register(&mut self, adapter: Box<dyn TrustAdapter>) { self.adapters.insert(adapter.scheme_uri().to_string(), adapter); }
    pub fn normalize(&self, scheme_uri: &str, payload: &[u8]) -> Result<f64> {
        let strength = self.adapters.get(scheme_uri).ok_or_else(|| ConcordanceError::AdapterUnavailable(scheme_uri.into()))?.normalize(payload)?;
        if !strength.is_finite() || !(0.0..=1.0).contains(&strength) { return Err(ConcordanceError::InvalidAdapterResult); }
        Ok(strength)
    }
}

pub struct SyntheticReputationAdapter;
impl TrustAdapter for SyntheticReputationAdapter {
    fn scheme_uri(&self) -> &str { "urn:concordance:scheme:synthetic:reputation:v1" }
    fn normalization_fn_uri(&self) -> &str { "urn:concordance:adapter:synthetic:reputation:v1" }
    fn normalize(&self, payload: &[u8]) -> Result<f64> { std::str::from_utf8(payload).ok().and_then(|s| s.parse().ok()).ok_or(ConcordanceError::InvalidAdapterResult) }
}
pub struct SyntheticConsentAdapter;
impl TrustAdapter for SyntheticConsentAdapter {
    fn scheme_uri(&self) -> &str { "urn:concordance:scheme:synthetic:consent:v1" }
    fn normalization_fn_uri(&self) -> &str { "urn:concordance:adapter:synthetic:consent:v1" }
    fn normalize(&self, payload: &[u8]) -> Result<f64> {
        match payload { b"granted" => Ok(1.0), b"limited" => Ok(0.5), b"denied" => Ok(0.0), _ => Err(ConcordanceError::InvalidAdapterResult) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClaimStatus { Ok, Absent, Insufficient, Conflict }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaimResult { pub strength: f64, pub status: ClaimStatus, pub witnesses: Vec<String> }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompositionResult { pub claims: BTreeMap<String, ClaimResult>, pub derivation: Vec<String> }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Decision { Allow, Deny, Escalate, Conflict }

pub fn compose(bundle: &[TrustObjectEnvelope], policy: &InteractionPolicy, now_ms: u64, revoked: &HashSet<String>) -> Result<CompositionResult> {
    let mut by_class: BTreeMap<String, Vec<&TrustObjectEnvelope>> = BTreeMap::new();
    let mut derivation = Vec::new();
    for envelope in bundle {
        envelope.verify()?;
        if revoked.contains(&envelope.envelope_id) { derivation.push(format!("{} ignored: revoked", envelope.envelope_id)); continue; }
        if envelope.is_stale(now_ms, policy.max_envelope_age_ms) { derivation.push(format!("{} ignored: stale", envelope.envelope_id)); continue; }
        by_class.entry(envelope.claim_class.clone()).or_default().push(envelope);
    }
    let mut claims = BTreeMap::new();
    for (claim, requirement) in &policy.required_claims {
        let envelopes = by_class.get(claim).cloned().unwrap_or_default();
        if envelopes.is_empty() { claims.insert(claim.clone(), ClaimResult { strength: 0.0, status: ClaimStatus::Absent, witnesses: vec![] }); continue; }
        let support: Vec<_> = envelopes.iter().copied().filter(|e| e.polarity == Polarity::Support).collect();
        let contradict: Vec<_> = envelopes.iter().copied().filter(|e| e.polarity == Polarity::Contradict).collect();
        let has_conflict = support.iter().any(|s| contradict.iter().any(|c| (s.normalized_strength - c.normalized_strength).abs() >= policy.conflict_delta));
        let strength = noisy_or_capped(&support);
        let status = if has_conflict { ClaimStatus::Conflict } else if strength >= requirement.min_strength { ClaimStatus::Ok } else { ClaimStatus::Insufficient };
        let witnesses = envelopes.iter().map(|e| e.envelope_id.clone()).collect();
        derivation.push(format!("{claim}: strength={strength:.4}, status={status:?}"));
        claims.insert(claim.clone(), ClaimResult { strength, status, witnesses });
    }
    Ok(CompositionResult { claims, derivation })
}

pub fn decide(result: &CompositionResult, policy: &InteractionPolicy) -> Decision {
    if result.claims.values().any(|v| v.status == ClaimStatus::Conflict) { return Decision::Conflict; }
    if result.claims.values().any(|v| v.status == ClaimStatus::Absent) { return Decision::Escalate; }
    let minimum = result.claims.values().map(|v| v.strength).fold(1.0, f64::min);
    if result.claims.values().any(|v| v.status == ClaimStatus::Insufficient) {
        return if minimum >= policy.escalation_floor { Decision::Escalate } else { Decision::Deny };
    }
    Decision::Allow
}

pub fn noisy_or_capped(envelopes: &[&TrustObjectEnvelope]) -> f64 {
    let mut groups: BTreeMap<String, f64> = BTreeMap::new();
    for envelope in envelopes {
        let key = envelope.independence_class.clone().unwrap_or_else(|| envelope.envelope_id.clone());
        groups.entry(key).and_modify(|current| {
            if envelope.normalized_strength > *current { *current = envelope.normalized_strength; }
        }).or_insert(envelope.normalized_strength);
    }
    1.0 - groups.values().fold(1.0, |remaining, strength| remaining * (1.0 - strength))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevokeEcho { pub concordance_version: String, pub envelope_id: String, pub sequence: u64, pub revoked_at_ms: u64, pub reason: String, pub issuer: String, pub issuer_key: String, pub signature: String }
impl RevokeEcho {
    pub fn sign(envelope: &TrustObjectEnvelope, sequence: u64, revoked_at_ms: u64, reason: String, key: &SigningKey) -> Result<Self> {
        let mut echo = Self { concordance_version: PROTOCOL_VERSION.into(), envelope_id: envelope.envelope_id.clone(), sequence, revoked_at_ms, reason, issuer: envelope.issuer.clone(), issuer_key: hex::encode(key.verifying_key().to_bytes()), signature: String::new() };
        echo.signature = hex::encode(key.sign(&echo.preimage_bytes()?).to_bytes());
        Ok(echo)
    }
    fn preimage_bytes(&self) -> Result<Vec<u8>> { Ok(serde_cbor::to_vec(&( &self.concordance_version, &self.envelope_id, self.sequence, self.revoked_at_ms, &self.reason, &self.issuer, &self.issuer_key ))?) }
    pub fn verify_for(&self, envelope: &TrustObjectEnvelope) -> Result<()> {
        if self.issuer != envelope.issuer || self.issuer_key != envelope.issuer_key { return Err(ConcordanceError::UnauthorizedRevocation); }
        verifying_key(&self.issuer_key)?.verify(&self.preimage_bytes()?, &signature_from_hex(&self.signature)?).map_err(|_| ConcordanceError::InvalidSignature)
    }
}
#[derive(Default)]
pub struct RevocationState { latest_sequence: HashMap<String, u64>, revoked: HashSet<String> }
impl RevocationState {
    pub fn apply(&mut self, echo: &RevokeEcho, envelope: &TrustObjectEnvelope) -> Result<()> {
        echo.verify_for(envelope)?;
        if self.latest_sequence.get(&echo.envelope_id).is_some_and(|previous| *previous >= echo.sequence) { return Err(ConcordanceError::RevocationReplay); }
        self.latest_sequence.insert(echo.envelope_id.clone(), echo.sequence);
        self.revoked.insert(echo.envelope_id.clone());
        Ok(())
    }
    pub fn revoked_ids(&self) -> &HashSet<String> { &self.revoked }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(byte: u8) -> SigningKey { SigningKey::from_bytes(&[byte; 32]) }
    fn policy() -> InteractionPolicy { InteractionPolicy { version: "1".into(), required_claims: BTreeMap::from([("reputation".into(), Requirement { min_strength: 0.8 }), ("consent".into(), Requirement { min_strength: 0.9 })]), max_envelope_age_ms: 1_000, escalation_floor: 0.5, conflict_delta: 0.2 } }
    fn envelope(class: &str, strength: f64, group: Option<&str>) -> TrustObjectEnvelope { TrustObjectEnvelope::sign("urn:test".into(), class.into(), Polarity::Support, "did:test:alpha".into(), "did:test:issuer".into(), b"evidence".to_vec(), strength, "urn:adapter:test".into(), 1_000, None, None, group.map(str::to_string), &key(1), &key(2), "session-1".into()).unwrap() }
    #[test] fn signed_envelope_round_trips_and_redacts() { let e = envelope("reputation", 0.8, None); e.verify().unwrap(); e.redact().verify().unwrap(); }
    #[test] fn tampering_is_rejected() { let mut e = envelope("reputation", 0.8, None); e.normalized_strength = 0.9; assert!(matches!(e.verify(), Err(ConcordanceError::InvalidEnvelopeId))); }
    #[test] fn correlated_evidence_is_not_double_counted() { let a = envelope("reputation", 0.6, Some("vendor-x")); let b = envelope("reputation", 0.7, Some("vendor-x")); assert_eq!(noisy_or_capped(&[&a, &b]), 0.7); }
    #[test] fn composition_and_revocation_recompute() { let rep = envelope("reputation", 0.82, None); let consent = envelope("consent", 1.0, None); let p = policy(); let mut state = RevocationState::default(); let before = compose(&[rep.clone(), consent.clone()], &p, 1_001, state.revoked_ids()).unwrap(); assert_eq!(decide(&before, &p), Decision::Allow); let echo = RevokeEcho::sign(&rep, 1, 1_010, "slashed".into(), &key(1)).unwrap(); state.apply(&echo, &rep).unwrap(); let after = compose(&[rep, consent], &p, 1_011, state.revoked_ids()).unwrap(); assert_eq!(decide(&after, &p), Decision::Escalate); }
    #[test] fn replayed_revocation_is_rejected() { let e = envelope("reputation", 0.8, None); let echo = RevokeEcho::sign(&e, 1, 2_000, "x".into(), &key(1)).unwrap(); let mut state = RevocationState::default(); state.apply(&echo, &e).unwrap(); assert!(matches!(state.apply(&echo, &e), Err(ConcordanceError::RevocationReplay))); }
}
