# Threat model

The MVP defends against altered TOEs, altered native payloads, stale evidence,
unauthorized revocations, replayed revocations, and declared-source double
counting. It verifies Ed25519 signatures, BLAKE3 commitments, expiry, binding
proofs, issuer equality for revocation, and increasing revocation sequences.

It does not solve malicious native issuers, undisclosed correlation, adapter
publisher compromise, aggregation inference, prompt injection, or network
availability. These remain explicit pilot and hardening work; no current code
claims otherwise.
