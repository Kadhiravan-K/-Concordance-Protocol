use concordance_adapters::Erc8004Feedback;

use crate::HarnessError;

pub fn canonical_feedback(
    agent_id: u64,
    client_address: impl Into<String>,
    feedback_index: u64,
    value: i64,
    value_decimals: u8,
    tag1: impl Into<String>,
    is_revoked: bool,
) -> Result<Vec<u8>, HarnessError> {
    let client_address = client_address.into();
    let tag1 = tag1.into();
    if client_address.trim().is_empty() || tag1.trim().is_empty() {
        return Err(HarnessError::EmptySourceIdentifier);
    }
    Ok(serde_json::to_vec(&Erc8004Feedback {
        agent_id,
        client_address,
        feedback_index,
        value,
        value_decimals,
        tag1,
        is_revoked,
    })
    .expect("ERC-8004 canonical payload serialization is infallible for known struct"))
}
