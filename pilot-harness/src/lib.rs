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
    use concordance_adapters::{AnumatiAdapter, ConformanceCoverage, FixtureSourceClass, ConformanceReport};
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

    #[test]
    fn generate_and_save_conformance_reports() {
        use std::fs;
        use std::path::Path;
        use concordance_adapters::Erc8004ReputationAdapter;

        // 1. ERC-8004 Reputation Adapter
        let erc_adapter = Erc8004ReputationAdapter::quality_0_to_100();
        let erc_fixtures = vec![
            CanonicalFixture {
                name: "success-rate".into(),
                payload: include_bytes!("../../adapters/erc8004/fixtures/feedback-success-rate.json").to_vec(),
                expectation: OwnedFixtureExpectation::Strength(0.87),
            },
            CanonicalFixture {
                name: "revoked".into(),
                payload: include_bytes!("../../adapters/erc8004/fixtures/feedback-revoked.json").to_vec(),
                expectation: OwnedFixtureExpectation::Reject,
            },
        ];
        let erc_report = report_from_canonical_fixtures(
            "erc8004-fixed-point-reputation",
            &erc_adapter,
            &erc_fixtures,
            FixtureSourceClass::RepoFixture,
            "adapters/erc8004/fixtures",
            "offline local tag feedback whitelist policy validation",
            vec!["cargo test -p concordance-pilot-harness".into()],
            ConformanceCoverage {
                malformed: true,
                revoked: true,
                expired: false,
                signature_tamper: false,
            },
        ).unwrap();

        // 2. Anumati Adherence Consent Adapter
        let anumati_adapter = AnumatiAdapter::new("policy:write-orders:v1".into(), 0.6, 1_500).unwrap();
        let anumati_fixtures = vec![
            CanonicalFixture {
                name: "valid".into(),
                payload: include_bytes!("../../adapters/anumati/fixtures/adherence-valid.json").to_vec(),
                expectation: OwnedFixtureExpectation::Strength(0.91),
            },
            CanonicalFixture {
                name: "revoked".into(),
                payload: include_bytes!("../../adapters/anumati/fixtures/adherence-revoked.json").to_vec(),
                expectation: OwnedFixtureExpectation::Reject,
            },
            CanonicalFixture {
                name: "policy-mismatch".into(),
                payload: include_bytes!("../../adapters/anumati/fixtures/adherence-policy-mismatch.json").to_vec(),
                expectation: OwnedFixtureExpectation::Reject,
            },
        ];
        let anumati_report = report_from_canonical_fixtures(
            "anumati-policy-match-confidence",
            &anumati_adapter,
            &anumati_fixtures,
            FixtureSourceClass::RepoFixture,
            "adapters/anumati/fixtures",
            "policy hash checked against local policy definition",
            vec!["cargo test -p concordance-pilot-harness".into()],
            ConformanceCoverage {
                malformed: true,
                revoked: true,
                expired: true,
                signature_tamper: false,
            },
        ).unwrap();

        // Ensure directory exists
        let registry_adapters_dir = Path::new("../registry/adapters");
        fs::create_dir_all(registry_adapters_dir).unwrap();

        // Write files
        let erc_path = registry_adapters_dir.join("erc8004-conformance-report.json");
        let anumati_path = registry_adapters_dir.join("anumati-conformance-report.json");

        fs::write(&erc_path, serde_json::to_string_pretty(&erc_report).unwrap()).unwrap();
        fs::write(&anumati_path, serde_json::to_string_pretty(&anumati_report).unwrap()).unwrap();

        // Check structure against schema constraints
        validate_report_structure(&erc_report);
        validate_report_structure(&anumati_report);
        
        // Validate JSON layout against the schema representation
        validate_json_schema_shape(serde_json::to_value(&erc_report).unwrap());
        validate_json_schema_shape(serde_json::to_value(&anumati_report).unwrap());
    }

    fn validate_report_structure(report: &ConformanceReport) {
        assert!(!report.adapter_id.is_empty());
        assert!(report.scheme_uri.starts_with("urn:"));
        assert!(report.normalization_fn_uri.starts_with("urn:"));
        assert!(!report.source_identifier.is_empty());
        assert!(!report.verification_policy.is_empty());
        assert!(!report.reproducibility_notes.is_empty());
        assert!(!report.results.is_empty());
        for result in &report.results {
            assert!(!result.name.is_empty());
            assert!(result.passed);
        }
    }

    fn validate_json_schema_shape(val: serde_json::Value) {
        let obj = val.as_object().expect("root must be object");
        
        // required fields
        let req_keys = [
            "adapter_id", "scheme_uri", "normalization_fn_uri", "source_class",
            "source_identifier", "verification_policy", "reproducibility_notes",
            "coverage", "results"
        ];
        for key in &req_keys {
            assert!(obj.contains_key(*key), "missing key: {}", key);
        }

        assert!(obj["adapter_id"].is_string());
        assert!(obj["scheme_uri"].as_str().unwrap().starts_with("urn:"));
        assert!(obj["normalization_fn_uri"].as_str().unwrap().starts_with("urn:"));
        
        let source_class = obj["source_class"].as_str().unwrap();
        assert!(
            source_class == "repo_fixture" ||
            source_class == "external_fixture" ||
            source_class == "live_derived_fixture"
        );

        assert!(obj["source_identifier"].is_string());
        assert!(obj["verification_policy"].is_string());
        
        let notes = obj["reproducibility_notes"].as_array().unwrap();
        for note in notes {
            assert!(note.is_string());
        }

        let coverage = obj["coverage"].as_object().unwrap();
        assert!(coverage["malformed"].is_boolean());
        assert!(coverage["revoked"].is_boolean());
        assert!(coverage["expired"].is_boolean());
        assert!(coverage["signature_tamper"].is_boolean());

        let results = obj["results"].as_array().unwrap();
        for res in results {
            let res_obj = res.as_object().unwrap();
            assert!(res_obj["name"].is_string());
            assert!(res_obj["passed"].is_boolean());
            assert!(res_obj["actual_strength"].is_number() || res_obj["actual_strength"].is_null());
        }
    }
}
