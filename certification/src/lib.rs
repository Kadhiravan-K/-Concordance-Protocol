use std::{fs, path::PathBuf};

use concordance_adapters::{ConformanceReport, FixtureSourceClass};
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;

#[derive(Debug)]
pub struct CertificationResult {
    pub passed: bool,
    pub failures: Vec<String>,
}

pub fn run_certification_suite(reports_dir: &PathBuf) -> Result<CertificationResult, String> {
    let mut failures = Vec::new();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_path = manifest_dir.join("../schemas/adapter-conformance-report.schema.json");
    let schema_text = fs::read_to_string(&schema_path)
        .map_err(|e| format!("failed to read schema file {}: {e}", schema_path.display()))?;
    let schema_json: Value = serde_json::from_str(&schema_text).map_err(|e| format!("failed to parse schema JSON: {e}"))?;
    let schema = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema_json)
        .map_err(|e| format!("failed to compile schema: {e}"))?;

    if !reports_dir.is_dir() {
        return Err(format!("reports_dir is not a directory: {}", reports_dir.display()));
    }

    for entry in fs::read_dir(reports_dir).map_err(|e| format!("failed to read reports dir: {e}"))? {
        let entry = entry.map_err(|e| format!("failed to read report entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let report_text = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read report file {}: {e}", path.display()))?;
        let report_json: Value = serde_json::from_str(&report_text)
            .map_err(|e| format!("failed to parse report JSON {}: {e}", path.display()))?;

        if let Err(errors) = schema.validate(&report_json) {
            failures.push(format!("Schema validation failed for {}", path.display()));
            for error in errors {
                failures.push(format!("  {}", error));
            }
            continue;
        }

        let report: ConformanceReport = serde_json::from_value(report_json)
            .map_err(|e| format!("failed to deserialize report {}: {e}", path.display()))?;

        if report.source_class == FixtureSourceClass::RepoFixture {
            failures.push(format!("Report {} uses repo_fixture source_class; certification requires external_fixture or live_derived_fixture", path.display()));
        }

        if !report.coverage.malformed
            || !report.coverage.revoked
            || !report.coverage.expired
            || !report.coverage.signature_tamper
        {
            failures.push(format!("Report {} does not include required coverage flags", path.display()));
        }

        if report.results.iter().any(|result| !result.passed) {
            failures.push(format!("Report {} contains failed fixture results", path.display()));
        }

        let has_revocation_result = report.results.iter().any(|result| {
            result.name.to_lowercase().contains("revoked") || result.actual_strength.is_none()
        });
        if !has_revocation_result {
            failures.push(format!("Report {} does not include a revocation-oriented fixture result", path.display()));
        }
    }

    Ok(CertificationResult {
        passed: failures.is_empty(),
        failures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn write_report(dir: &std::path::Path, name: &str, report: &serde_json::Value) {
        let path = dir.join(name);
        fs::write(&path, serde_json::to_string_pretty(report).unwrap()).unwrap();
    }

    #[test]
    fn valid_external_fixture_report_passes() {
        let temp = tempdir().unwrap();
        let report = json!({
            "adapter_id": "erc8004-fixed-point-reputation",
            "scheme_uri": "urn:example:scheme:demo:v1",
            "normalization_fn_uri": "urn:example:normalizer:demo:v1",
            "source_class": "external_fixture",
            "source_identifier": "external-erc8004-collection",
            "verification_policy": "offline source validation",
            "reproducibility_notes": ["externally published fixture set"],
            "coverage": {"malformed": true, "revoked": true, "expired": true, "signature_tamper": true},
            "results": [
                {"name": "success-rate", "passed": true, "actual_strength": 0.87},
                {"name": "revoked", "passed": true, "actual_strength": null}
            ]
        });
        write_report(temp.path(), "good-report.json", &report);

        let result = run_certification_suite(&temp.path().to_path_buf()).unwrap();
        assert!(result.passed, "expected certification to pass, got failures: {:?}", result.failures);
    }

    #[test]
    fn repo_fixture_report_fails() {
        let temp = tempdir().unwrap();
        let report = json!({
            "adapter_id": "erc8004-fixed-point-reputation",
            "scheme_uri": "urn:example:scheme:demo:v1",
            "normalization_fn_uri": "urn:example:normalizer:demo:v1",
            "source_class": "repo_fixture",
            "source_identifier": "adapters/erc8004/fixtures",
            "verification_policy": "offline source validation",
            "reproducibility_notes": ["repo fixture"],
            "coverage": {"malformed": true, "revoked": true, "expired": true, "signature_tamper": true},
            "results": [
                {"name": "success-rate", "passed": true, "actual_strength": 0.87}
            ]
        });
        write_report(temp.path(), "repo-report.json", &report);

        let result = run_certification_suite(&temp.path().to_path_buf()).unwrap();
        assert!(!result.passed);
        assert!(result.failures.iter().any(|f| f.contains("repo_fixture")));
    }

    #[test]
    fn invalid_report_schema_fails() {
        let temp = tempdir().unwrap();
        let report = json!({
            "adapter_id": "bad",
            "scheme_uri": "urn:example:scheme:demo:v1"
        });
        write_report(temp.path(), "bad-report.json", &report);

        let result = run_certification_suite(&temp.path().to_path_buf()).unwrap();
        assert!(!result.passed);
        assert!(result.failures.iter().any(|f| f.contains("Schema validation failed")));
    }
}
