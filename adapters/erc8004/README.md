# ERC-8004 reputation fixture adapter

The adapter consumes the feedback fields used by ERC-8004’s reputation registry:
agent identifier, client address, feedback index, fixed-point value,
`value_decimals`, `tag1`, and revocation state. It rejects revoked feedback,
unknown tags, invalid fixed-point precision, and values outside the local
normalization range.

It is intentionally a parser/normalizer, not a chain client. A future live
pilot must obtain verified feedback from an ERC-8004 deployment, establish the
source/finality policy, and then pass the resulting canonical fixture payload
to this adapter.
