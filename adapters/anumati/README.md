# Anumati adherence adapter

This adapter parses a canonicalized Anumati-style adherence proof and returns
the proof's policy-match confidence when the proof satisfies local policy,
expiry, and revocation checks.

## Canonical payload contract

The adapter expects a JSON payload with:

- `version`
- `subject`
- `policy_hash`
- `confidence`
- `issued_at_ms`
- `expires_at_ms`
- `is_revoked`

The pilot harness owns source retrieval, source authentication, policy-change
watching, and any native proof canonicalization. The adapter only validates the
already-obtained canonical payload.

To claim interoperability for Phase 3, the repository must publish a
conformance report showing independently maintained or reproducibly
live-derived fixtures, the verification policy applied by the harness, and the
malformed or tamper coverage for the fixtures used.
