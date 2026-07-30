use concordance_adapters::AnumatiAdherenceProof;

use crate::HarnessError;

pub fn canonical_proof(
    subject: impl Into<String>,
    policy_hash: impl Into<String>,
    confidence: f64,
    issued_at_ms: u64,
    expires_at_ms: u64,
    is_revoked: bool,
) -> Result<Vec<u8>, HarnessError> {
    let subject = subject.into();
    let policy_hash = policy_hash.into();
    if subject.trim().is_empty() || policy_hash.trim().is_empty() {
        return Err(HarnessError::EmptySourceIdentifier);
    }
    Ok(serde_json::to_vec(&AnumatiAdherenceProof {
        version: "anumati-adherence/v1".into(),
        subject,
        policy_hash,
        confidence,
        issued_at_ms,
        expires_at_ms,
        is_revoked,
    })
    .expect("Anumati canonical payload serialization is infallible for known struct"))
}
