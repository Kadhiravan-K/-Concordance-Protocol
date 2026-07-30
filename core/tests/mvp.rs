use std::collections::{BTreeMap, BTreeSet};

use concordance_core::{negotiate, AdapterRegistry, InteractionPolicy, Requirement, SchemeCapability, SchemeManifest, SyntheticConsentAdapter, SyntheticReputationAdapter};
use ed25519_dalek::SigningKey;

fn key(seed: u8) -> SigningKey { SigningKey::from_bytes(&[seed; 32]) }

fn manifest(agent: &str, capabilities: &[(&str, &[&str])], policy: BTreeMap<String, InteractionPolicy>, signing_key: &SigningKey) -> SchemeManifest {
    let caps = capabilities.iter().map(|(scheme, classes)| SchemeCapability {
        scheme_uri: (*scheme).into(), claim_classes: classes.iter().map(|class| (*class).into()).collect::<BTreeSet<_>>(),
    }).collect();
    let mut result = SchemeManifest { concordance_version: "Concordance/1.0".into(), agent_id: agent.into(), agent_key: String::new(), can_present: caps, can_verify: vec![], policy_classes: policy, signature: None };
    result.sign(signing_key).unwrap();
    result
}

#[test]
fn signed_manifests_negotiate_required_claims() {
    let policy = InteractionPolicy { version: "1".into(), required_claims: BTreeMap::from([("reputation".into(), Requirement { min_strength: 0.8 })]), max_envelope_age_ms: 1_000, escalation_floor: 0.5, conflict_delta: 0.2 };
    let mut verifier = manifest("did:example:beta", &[("urn:test:rep", &["reputation"])], BTreeMap::from([("write".into(), policy)]), &key(1));
    verifier.can_verify = verifier.can_present.clone();
    verifier.sign(&key(1)).unwrap();
    let presenter = manifest("did:example:alpha", &[("urn:test:rep", &["reputation"])], BTreeMap::new(), &key(2));
    assert!(negotiate(&verifier, &presenter, "write").unwrap().accepted);
}

#[test]
fn synthetic_adapters_are_deterministic_and_bounded() {
    let mut registry = AdapterRegistry::default();
    registry.register(Box::new(SyntheticReputationAdapter));
    registry.register(Box::new(SyntheticConsentAdapter));
    assert_eq!(registry.normalize("urn:concordance:scheme:synthetic:reputation:v1", b"0.82").unwrap(), 0.82);
    assert_eq!(registry.normalize("urn:concordance:scheme:synthetic:consent:v1", b"granted").unwrap(), 1.0);
}
