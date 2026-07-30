use std::collections::{BTreeMap, HashSet};
use std::env;

use concordance_core::{compose, decide, InteractionPolicy, Polarity, Requirement, RevokeEcho, RevocationState, TrustObjectEnvelope};
use ed25519_dalek::SigningKey;

fn key(byte: u8) -> SigningKey { SigningKey::from_bytes(&[byte; 32]) }

fn policy() -> InteractionPolicy {
    InteractionPolicy {
        version: "1".into(),
        required_claims: BTreeMap::from([
            ("reputation".into(), Requirement { min_strength: 0.8 }),
            ("consent".into(), Requirement { min_strength: 0.9 }),
        ]),
        max_envelope_age_ms: 60_000,
        escalation_floor: 0.5,
        conflict_delta: 0.2,
    }
}

fn envelope(class: &str, strength: f64, independence_class: Option<String>) -> TrustObjectEnvelope {
    TrustObjectEnvelope::sign(
        format!("urn:concordance:scheme:synthetic:{class}:v1"), class.into(), Polarity::Support,
        "did:example:alpha".into(), "did:example:issuer".into(), strength.to_string().into_bytes(), strength,
        format!("urn:concordance:adapter:synthetic:{class}:v1"), 1_000, Some(61_000), None,
        independence_class, &key(1), &key(2), "deterministic-session".into(),
    ).expect("synthetic envelope is valid")
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let agents = args.windows(2).find(|w| w[0] == "--agents").and_then(|w| w[1].parse::<usize>().ok()).unwrap_or(10);
    let adversarial_percent = args.windows(2).find(|w| w[0] == "--adversarial-percent").and_then(|w| w[1].parse::<u64>().ok()).unwrap_or(10);
    if agents > 10 { run_network(agents, adversarial_percent); } else { run_mvp(); }
}

fn run_mvp() {
    let reputation = envelope("reputation", 0.82, None);
    let consent = envelope("consent", 1.0, None);
    let policy = policy();
    let mut revocations = RevocationState::default();
    let before = compose(&[reputation.clone(), consent.clone()], &policy, 1_001, revocations.revoked_ids()).unwrap();
    println!("MVP before revocation: {:?}", decide(&before, &policy));
    for line in &before.derivation { println!("  {line}"); }
    let echo = RevokeEcho::sign(&reputation, 1, 2_000, "synthetic reputation slashed".into(), &key(1)).unwrap();
    revocations.apply(&echo, &reputation).unwrap();
    let after = compose(&[reputation, consent], &policy, 2_001, revocations.revoked_ids()).unwrap();
    println!("MVP after revocation: {:?}", decide(&after, &policy));
    for line in &after.derivation { println!("  {line}"); }
}

fn run_network(agents: usize, adversarial_percent: u64) {
    let mut naive_allows = 0usize;
    let mut capped_allows = 0usize;
    for agent in 0..agents {
        let adversarial = (agent as u64 * 37 % 100) < adversarial_percent;
        let strengths = if adversarial { [0.55, 0.55] } else { [0.45 + (agent % 45) as f64 / 100.0, 0.0] };
        let naive = 1.0 - (1.0 - strengths[0]) * (1.0 - strengths[1]);
        let capped = if adversarial { strengths[0].max(strengths[1]) } else { naive };
        if naive >= 0.8 { naive_allows += 1; }
        if capped >= 0.8 { capped_allows += 1; }
    }
    let memory_bytes = agents * 2 * 96;
    println!("Synthetic network: agents={agents}, adversarial={adversarial_percent}%");
    println!("Naive allows: {naive_allows}; independence-capped allows: {capped_allows}");
    println!("Estimated active-envelope metadata: {memory_bytes} bytes");
    println!("This is a deterministic scenario harness, not a claim of real-world security performance.");
}
