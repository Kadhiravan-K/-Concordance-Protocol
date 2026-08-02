use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use concordance_core::{ConcordanceError, Polarity, TrustObjectEnvelope};

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyBindingProof {
    #[pyo3(get, set)]
    pub presenter_id: String,
    #[pyo3(get, set)]
    pub session_id: String,
    #[pyo3(get, set)]
    pub presenter_key: String,
    #[pyo3(get, set)]
    pub signature: String,
}

impl From<concordance_core::BindingProof> for PyBindingProof {
    fn from(p: concordance_core::BindingProof) -> Self {
        Self {
            presenter_id: p.presenter_id,
            session_id: p.session_id,
            presenter_key: p.presenter_key,
            signature: p.signature,
        }
    }
}

impl From<PyBindingProof> for concordance_core::BindingProof {
    fn from(p: PyBindingProof) -> Self {
        Self {
            presenter_id: p.presenter_id,
            session_id: p.session_id,
            presenter_key: p.presenter_key,
            signature: p.signature,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyTrustObjectEnvelope {
    #[pyo3(get, set)]
    pub concordance_version: String,
    #[pyo3(get, set)]
    pub envelope_id: String,
    #[pyo3(get, set)]
    pub scheme_uri: String,
    #[pyo3(get, set)]
    pub claim_class: String,
    #[pyo3(get, set)]
    pub polarity: String,
    #[pyo3(get, set)]
    pub subject: String,
    #[pyo3(get, set)]
    pub issuer: String,
    #[pyo3(get, set)]
    pub issuer_key: String,
    #[pyo3(get, set)]
    pub native_payload: Option<Vec<u8>>,
    #[pyo3(get, set)]
    pub payload_commitment: String,
    #[pyo3(get, set)]
    pub normalized_strength: f64,
    #[pyo3(get, set)]
    pub normalization_fn_uri: String,
    #[pyo3(get, set)]
    pub issued_at_ms: u64,
    #[pyo3(get, set)]
    pub expires_at_ms: Option<u64>,
    #[pyo3(get, set)]
    pub revocation_check_uri: Option<String>,
    #[pyo3(get, set)]
    pub independence_class: Option<String>,
    #[pyo3(get, set)]
    pub redacted: bool,
    #[pyo3(get, set)]
    pub binding_proof: PyBindingProof,
    #[pyo3(get, set)]
    pub issuer_signature: Option<String>,
}

impl From<TrustObjectEnvelope> for PyTrustObjectEnvelope {
    fn from(env: TrustObjectEnvelope) -> Self {
        Self {
            concordance_version: env.concordance_version,
            envelope_id: env.envelope_id,
            scheme_uri: env.scheme_uri,
            claim_class: env.claim_class,
            polarity: match env.polarity {
                Polarity::Support => "Support".into(),
                Polarity::Contradict => "Contradict".into(),
            },
            subject: env.subject,
            issuer: env.issuer,
            issuer_key: env.issuer_key,
            native_payload: env.native_payload,
            payload_commitment: env.payload_commitment,
            normalized_strength: env.normalized_strength,
            normalization_fn_uri: env.normalization_fn_uri,
            issued_at_ms: env.issued_at_ms,
            expires_at_ms: env.expires_at_ms,
            revocation_check_uri: env.revocation_check_uri,
            independence_class: env.independence_class,
            redacted: env.redacted,
            binding_proof: env.binding_proof.into(),
            issuer_signature: env.issuer_signature,
        }
    }
}

impl TryFrom<PyTrustObjectEnvelope> for TrustObjectEnvelope {
    type Error = ConcordanceError;

    fn try_from(py: PyTrustObjectEnvelope) -> Result<Self, Self::Error> {
        let polarity = match py.polarity.as_str() {
            "Support" => Polarity::Support,
            "Contradict" => Polarity::Contradict,
            _ => return Err(ConcordanceError::InvalidSignature),
        };
        Ok(TrustObjectEnvelope {
            concordance_version: py.concordance_version,
            envelope_id: py.envelope_id,
            scheme_uri: py.scheme_uri,
            claim_class: py.claim_class,
            polarity,
            subject: py.subject,
            issuer: py.issuer,
            issuer_key: py.issuer_key,
            native_payload: py.native_payload,
            payload_commitment: py.payload_commitment,
            normalized_strength: py.normalized_strength,
            normalization_fn_uri: py.normalization_fn_uri,
            issued_at_ms: py.issued_at_ms,
            expires_at_ms: py.expires_at_ms,
            revocation_check_uri: py.revocation_check_uri,
            independence_class: py.independence_class,
            redacted: py.redacted,
            binding_proof: py.binding_proof.into(),
            issuer_signature: py.issuer_signature,
        })
    }
}

#[pyfunction]
pub fn sign_envelope(
    scheme_uri: String,
    claim_class: String,
    polarity: String,
    subject: String,
    issuer: String,
    native_payload: Vec<u8>,
    normalized_strength: f64,
    normalization_fn_uri: String,
    issued_at_ms: u64,
    expires_at_ms: Option<u64>,
    revocation_check_uri: Option<String>,
    independence_class: Option<String>,
    issuer_key_bytes: Vec<u8>,
    presenter_key_bytes: Vec<u8>,
    session_id: String,
) -> PyResult<PyTrustObjectEnvelope> {
    let polarity = match polarity.as_str() {
        "Support" => Polarity::Support,
        "Contradict" => Polarity::Contradict,
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("invalid polarity")),
    };
    let issuer_key = ed25519_dalek::SigningKey::from_bytes(&issuer_key_bytes)
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>("invalid issuer key"))?;
    let presenter_key = ed25519_dalek::SigningKey::from_bytes(&presenter_key_bytes)
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>("invalid presenter key"))?;

    let env = TrustObjectEnvelope::sign(
        scheme_uri,
        claim_class,
        polarity,
        subject,
        issuer,
        native_payload,
        normalized_strength,
        normalization_fn_uri,
        issued_at_ms,
        expires_at_ms,
        revocation_check_uri,
        independence_class,
        &issuer_key,
        &presenter_key,
        session_id,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(PyTrustObjectEnvelope::from(env))
}

#[pyfunction]
pub fn verify_envelope(envelope: PyTrustObjectEnvelope) -> PyResult<bool> {
    let env: TrustObjectEnvelope = envelope
        .try_into()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(env.verify().is_ok())
}

#[pymodule]
fn concordance_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTrustObjectEnvelope>()?;
    m.add_function(wrap_pyfunction!(sign_envelope, m)?)?;
    m.add_function(wrap_pyfunction!(verify_envelope, m)?)?;
    Ok(())
}
