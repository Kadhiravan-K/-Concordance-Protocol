//! Deterministic Phase-2 network scenarios.

use std::collections::{BTreeMap, HashSet};
use std::time::Instant;

use concordance_core::{
    compose, decide, Decision, InteractionPolicy, Polarity, Requirement,
    RevokeEcho, RevocationState, TrustObjectEnvelope,
};
use ed25519_dalek::SigningKey;

const NOW_MS: u64 = 10_000;

#[derive(Debug, Clone, Copy)]
pub struct ScenarioConfig {
    pub agents: usize,
    pub max_schemes_per_agent: usize,
    pub adversarial_percent: u8,
    pub revoked_percent: u8,
    pub expired_percent: u8,
    pub conflict_percent: u8,
    pub seed: u64,
}

impl Default for ScenarioConfig {
    fn default() -> Self {
        Self {
            agents: 100,
            max_schemes_per_agent: 3,
            adversarial_percent: 10,
            revoked_percent: 5,
            expired_percent: 5,
            conflict_percent: 0,
            seed: 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimulationResult {
    pub agents: usize,
    pub envelopes: usize,
    pub adversarial_agents: usize,
    pub revoked_agents: usize,
    pub expired_agents: usize,
    pub conflict_agents: usize,
    pub naive_allows: usize,
    pub capped_allows: usize,
    pub naive_adversarial_allows: usize,
    pub capped_adversarial_allows: usize,
    pub capped_conflicts: usize,
    pub estimated_state_bytes: usize,
    pub elapsed_micros: u128,
}

impl SimulationResult {
    pub fn csv_header() -> &'static str {
        "agents,envelopes,adversarial_agents,revoked_agents,expired_agents,conflict_agents,naive_allows,capped_allows,naive_adversarial_allows,capped_adversarial_allows,capped_conflicts,estimated_state_bytes,elapsed_micros"
    }

    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.agents,
            self.envelopes,
            self.adversarial_agents,
            self.revoked_agents,
            self.expired_agents,
            self.conflict_agents,
            self.naive_allows,
            self.capped_allows,
            self.naive_adversarial_allows,
            self.capped_adversarial_allows,
            self.capped_conflicts,
            self.estimated_state_bytes,
            self.elapsed_micros
        )
    }
}

pub fn run(config: ScenarioConfig) -> Result<SimulationResult, String> {
    validate_config(config)?;
    let started = Instant::now();
    let policy = reputation_policy();
    let issuer_key = key(1);
    let presenter_key = key(2);
    let mut state = config.seed;
    let mut result = SimulationResult {
        agents: config.agents,
        envelopes: 0,
        adversarial_agents: 0,
        revoked_agents: 0,
        expired_agents: 0,
        conflict_agents: 0,
        naive_allows: 0,
        capped_allows: 0,
        naive_adversarial_allows: 0,
        capped_adversarial_allows: 0,
        capped_conflicts: 0,
        estimated_state_bytes: 0,
        elapsed_micros: 0,
    };

    for agent in 0..config.agents {
        let adversarial = draw_percent(&mut state, config.adversarial_percent);
        let revoke = draw_percent(&mut state, config.revoked_percent);
        let expire = draw_percent(&mut state, config.expired_percent);
        let conflict = draw_percent(&mut state, config.conflict_percent);
        let scheme_count = 1 + (next(&mut state) as usize % config.max_schemes_per_agent);
        let issued_at_ms = if expire { NOW_MS - policy.max_envelope_age_ms - 1 } else { NOW_MS - 100 };
        let subject = format!("did:scenario:{agent}");
        let mut bundle = Vec::new();

        if adversarial {
            // Two Sybil witnesses look strong to naïve noisy-OR (0.84), but
            // collapse to 0.60 because they share one declared source.
            bundle.push(envelope(&subject, "reputation", "sybil-witness-a", Polarity::Support, 0.60, issued_at_ms, Some(format!("sybil-cluster-{}", agent / 4)), &issuer_key, &presenter_key));
            bundle.push(envelope(&subject, "reputation", "sybil-witness-b", Polarity::Support, 0.60, issued_at_ms, Some(format!("sybil-cluster-{}", agent / 4)), &issuer_key, &presenter_key));
            result.adversarial_agents += 1;
        } else {
            bundle.push(envelope(&subject, "reputation", "primary", Polarity::Support, 0.85, issued_at_ms, Some(format!("independent-{agent}")), &issuer_key, &presenter_key));
        }
        if conflict {
            bundle.push(envelope(&subject, "reputation", "contradiction", Polarity::Contradict, 0.20, issued_at_ms, Some(format!("auditor-{agent}")), &issuer_key, &presenter_key));
            result.conflict_agents += 1;
        }
        for scheme in 1..scheme_count {
            bundle.push(envelope(&subject, "intent-sensitivity", &format!("scheme-{scheme}"), Polarity::Support, 0.75, issued_at_ms, Some(format!("scheme-{scheme}-agent-{agent}")), &issuer_key, &presenter_key));
        }

        let mut revocations = RevocationState::default();
        if revoke {
            let echo = RevokeEcho::sign(&bundle[0], 1, NOW_MS, "deterministic scenario revocation".into(), &issuer_key).map_err(|error| error.to_string())?;
            revocations.apply(&echo, &bundle[0]).map_err(|error| error.to_string())?;
            result.revoked_agents += 1;
        }
        if expire { result.expired_agents += 1; }

        let composition = compose(&bundle, &policy, NOW_MS, revocations.revoked_ids()).map_err(|error| error.to_string())?;
        let capped = decide(&composition, &policy);
        let naive = naive_decision(&bundle, &policy, revocations.revoked_ids());
        if capped == Decision::Allow { result.capped_allows += 1; }
        if naive == Decision::Allow { result.naive_allows += 1; }
        if capped == Decision::Conflict { result.capped_conflicts += 1; }
        if adversarial && capped == Decision::Allow { result.capped_adversarial_allows += 1; }
        if adversarial && naive == Decision::Allow { result.naive_adversarial_allows += 1; }
        result.envelopes += bundle.len();
    }

    // TOE index plus revocation records: a transparent, deliberately coarse
    // metadata-only estimate rather than a measured allocator result.
    result.estimated_state_bytes = result.envelopes * 96 + result.revoked_agents * 64;
    result.elapsed_micros = started.elapsed().as_micros();
    Ok(result)
}

fn validate_config(config: ScenarioConfig) -> Result<(), String> {
    if !(10..=1_000).contains(&config.agents) { return Err("agents must be in 10..=1000".into()); }
    if !(1..=3).contains(&config.max_schemes_per_agent) { return Err("max schemes per agent must be in 1..=3".into()); }
    for (name, value) in [("adversarial", config.adversarial_percent), ("revoked", config.revoked_percent), ("expired", config.expired_percent), ("conflict", config.conflict_percent)] {
        if value > 100 { return Err(format!("{name} percent must be in 0..=100")); }
    }
    Ok(())
}

fn reputation_policy() -> InteractionPolicy {
    InteractionPolicy {
        version: "phase-2".into(),
        required_claims: BTreeMap::from([("reputation".into(), Requirement { min_strength: 0.8 })]),
        max_envelope_age_ms: 500,
        escalation_floor: 0.5,
        conflict_delta: 0.2,
    }
}

fn naive_decision(bundle: &[TrustObjectEnvelope], policy: &InteractionPolicy, revoked: &HashSet<String>) -> Decision {
    let strengths: Vec<f64> = bundle.iter().filter(|e| e.claim_class == "reputation" && e.polarity == Polarity::Support && !revoked.contains(&e.envelope_id) && !e.is_stale(NOW_MS, policy.max_envelope_age_ms)).map(|e| e.normalized_strength).collect();
    if strengths.is_empty() { return Decision::Escalate; }
    let score = 1.0 - strengths.iter().fold(1.0, |remaining, strength| remaining * (1.0 - strength));
    if score >= 0.8 { Decision::Allow } else if score >= policy.escalation_floor { Decision::Escalate } else { Decision::Deny }
}

fn envelope(subject: &str, claim_class: &str, evidence_label: &str, polarity: Polarity, strength: f64, issued_at_ms: u64, independence_class: Option<String>, issuer_key: &SigningKey, presenter_key: &SigningKey) -> TrustObjectEnvelope {
    TrustObjectEnvelope::sign(
        format!("urn:concordance:scheme:synthetic:{claim_class}:v1"),
        claim_class.into(), polarity, subject.into(), "did:scenario:issuer".into(),
        format!("{subject}:{claim_class}:{evidence_label}:{strength}").into_bytes(), strength,
        format!("urn:concordance:adapter:synthetic:{claim_class}:v1"), issued_at_ms,
        Some(NOW_MS + 1_000), None, independence_class, issuer_key, presenter_key,
        format!("session:{subject}"),
    ).expect("deterministic scenario TOE must be valid")
}

fn key(seed: u8) -> SigningKey { SigningKey::from_bytes(&[seed; 32]) }
fn next(state: &mut u64) -> u64 { *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); *state }
fn draw_percent(state: &mut u64, percent: u8) -> bool { next(state) % 100 < u64::from(percent) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_capping_eliminates_the_synthetic_sybil_allow_path() {
        let result = run(ScenarioConfig { agents: 100, adversarial_percent: 100, revoked_percent: 0, expired_percent: 0, conflict_percent: 0, ..ScenarioConfig::default() }).unwrap();
        assert_eq!(result.naive_adversarial_allows, 100);
        assert_eq!(result.capped_adversarial_allows, 0);
    }

    #[test]
    fn the_scenario_is_repeatable_for_a_seed() {
        let config = ScenarioConfig { agents: 10, seed: 42, ..ScenarioConfig::default() };
        let first = run(config).unwrap();
        let second = run(config).unwrap();
        assert_eq!(first.agents, second.agents);
        assert_eq!(first.envelopes, second.envelopes);
        assert_eq!(first.naive_adversarial_allows, second.naive_adversarial_allows);
        assert_eq!(first.capped_adversarial_allows, second.capped_adversarial_allows);
    }

    #[test]
    fn invalid_scale_is_rejected() {
        assert!(run(ScenarioConfig { agents: 1_001, ..ScenarioConfig::default() }).is_err());
    }
}
