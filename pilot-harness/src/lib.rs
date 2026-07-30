//! Minimal pilot harness for Phase 3 external-validation work.
//!
//! This crate owns source metadata, canonical payload generation, and
//! conformance-report assembly. It deliberately avoids defining a service,
//! transport API, or persistent registry.

pub mod anumati;
pub mod erc8004;

use concordance_adapters::{
    generate_conformance_report, AdapterFixture, ConformanceCoverage, ConformanceReport,
    FixtureExpectation, FixtureSourceClass,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("source identifier must not be empty")]
    EmptySourceIdentifier,
    #[error("verification policy must not be empty")]
    EmptyVerificationPolicy,
}

#[derive(Debug, Clone)]
pub enum OwnedFixtureExpectation {
    Strength(f64),
    Reject,
}

#[derive(Debug, Clone)]
pub struct CanonicalFixture {
    pub name: String,
    pub payload: Vec<u8>,
    pub expectation: OwnedFixtureExpectation,
}

impl CanonicalFixture {
    pub fn borrowed(&self) -> AdapterFixture<'_> {
        AdapterFixture {
            name: &self.name,
            payload: &self.payload,
            expectation: match self.expectation {
                OwnedFixtureExpectation::Strength(value) => FixtureExpectation::Strength(value),
                OwnedFixtureExpectation::Reject => FixtureExpectation::Reject,
            },
        }
    }
}

pub fn report_from_canonical_fixtures(
    adapter_id: &str,
    adapter: &dyn concordance_core::TrustAdapter,
    fixtures: &[CanonicalFixture],
    source_class: FixtureSourceClass,
    source_identifier: &str,
    verification_policy: &str,
    reproducibility_notes: Vec<String>,
    coverage: ConformanceCoverage,
) -> Result<ConformanceReport, HarnessError> {
    if source_identifier.trim().is_empty() {
        return Err(HarnessError::EmptySourceIdentifier);
    }
    if verification_policy.trim().is_empty() {
        return Err(HarnessError::EmptyVerificationPolicy);
    }
    let borrowed: Vec<_> = fixtures.iter().map(CanonicalFixture::borrowed).collect();
    Ok(generate_conformance_report(
        adapter_id,
        adapter,
        &borrowed,
        source_class,
        source_identifier,
        verification_policy,
        reproducibility_notes,
        coverage,
    ))
}

#[cfg(test)]
mod tests {
    use concordance_adapters::{AnumatiAdapter, ConformanceCoverage, FixtureSourceClass};

    use super::{anumati::canonical_proof, report_from_canonical_fixtures, CanonicalFixture, OwnedFixtureExpectation};

    #[test]
    fn report_builder_requires_source_metadata() {
        let adapter = AnumatiAdapter::new("policy:write-orders:v1".into(), 0.6, 1_500).unwrap();
        let payload = canonical_proof("did:example:agent", "policy:write-orders:v1", 0.91, 1_000, 2_000, false).unwrap();
        let result = report_from_canonical_fixtures(
            "anumati-live",
            &adapter,
            &[CanonicalFixture {
                name: "valid".into(),
                payload,
                expectation: OwnedFixtureExpectation::Strength(0.91),
            }],
            FixtureSourceClass::LiveDerivedFixture,
            "",
            "policy hash checked against local allow-list",
            vec!["offline replay from captured source".into()],
            ConformanceCoverage { malformed: true, revoked: true, expired: true, signature_tamper: false },
        );
        assert!(result.is_err());
    }
}
