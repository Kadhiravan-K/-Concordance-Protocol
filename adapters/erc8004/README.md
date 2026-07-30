# ERC-8004 reputation fixture adapter

The adapter consumes the feedback fields used by ERC-8004’s reputation registry:
agent identifier, client address, feedback index, fixed-point value,
`value_decimals`, `tag1`, and revocation state. It rejects revoked feedback,
unknown tags, invalid fixed-point precision, and values outside the local
normalization range.

## Canonical payload contract

The pilot harness must present a canonical JSON payload containing:

- `agent_id`
- `client_address`
- `feedback_index`
- `value`
- `value_decimals`
- `tag1`
- `is_revoked`

The adapter treats that payload as already obtained native evidence. It does
not verify RPC responses, chain finality, or event authenticity itself.

It is intentionally a parser/normalizer, not a chain client. A future live
pilot must obtain verified feedback from an ERC-8004 deployment, establish the
source/finality policy, and then pass the resulting canonical fixture payload
to this adapter.

To claim external validation, the repository must publish a conformance report
showing independently maintained or live-derived canonical fixtures, the source
policy used to obtain them, and malformed or tamper coverage for the payloads
that the harness passes to this adapter.
