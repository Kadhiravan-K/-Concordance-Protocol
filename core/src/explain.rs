use serde::{Deserialize, Serialize};

/// Human- and machine-readable explanation for trust decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explanation {
    pub decision: String,
    pub rationale: Vec<String>,
    pub confidence: f64,
    pub evidence_count: usize,
    pub policy: String,
}

impl Explanation {
    pub fn new(decision: impl Into<String>, rationale: Vec<String>, confidence: f64, evidence_count: usize, policy: impl Into<String>) -> Self {
        Self {
            decision: decision.into(),
            rationale,
            confidence,
            evidence_count,
            policy: policy.into(),
        }
    }
}
