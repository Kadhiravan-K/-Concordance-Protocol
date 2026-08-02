# Core API

`TrustObjectEnvelope::sign` creates a signed deterministic TOE and `verify`
validates its identifier, payload commitment, issuer signature, and presenter
binding. `AdapterRegistry::normalize` invokes trusted local adapters.
`negotiate` compares a verifier policy against advertised capabilities.
`compose` returns claim results and a derivation trace; `decide` maps that
result to `ALLOW`, `DENY`, `ESCALATE`, or `CONFLICT`.

`RevokeEcho::sign` creates an issuer-authorized event and
`RevocationState::apply` validates ordering and invalidates the referenced TOE.

## CLI Reference

The `concordance` command-line tool supports the following commands:

- `inspect <bundle.json>` — print each envelope summary.
- `verify <bundle.json>` — validate signatures and binding proofs.
- `summary <bundle.json>` — report totals and counts by claim class and issuer.
- `interactive <bundle.json>` — open a prompt to inspect individual envelopes.

## Registry observability

The reference registry service exposes observability endpoints for Phase 7:

- `GET /v1/observability/metrics` — returns service metrics and registry counts.
- `GET /v1/observability/audit-log` — returns durable audit events; filter by `kind`.
- `GET /v1/observability/decision-history` — alias for filtered audit log, supporting trust decision reconstruction.

## SDK Examples

A minimal Python SDK is available in `sdk/python`. It exposes bindings for
creating and verifying Concordance envelopes from Python.

A standalone Rust sample application is provided in `examples/standalone`.
Use `cargo run --manifest-path examples/standalone/Cargo.toml` to build and run it.

For local CLI development, use `cargo run -p concordance-cli -- <command> <bundle.json>`.

## Certification Suite

A certification harness is available in `certification`. It validates published adapter conformance reports against the Concordance report schema and minimum coverage requirements.

## Governance

The Phase 9 governance landing page is available at `docs/phase-9-governance.md`. It describes version policy, RFC process, deprecation rules, adapter approval, registry governance, security disclosure, and release cadence.

## Ecosystem Expansion

The Phase 10 ecosystem expansion landing page is available at `docs/phase-10-ecosystem-expansion.md`. It describes MCP/A2A/ANP integration paths, enterprise examples, industry profiles, and academic collaboration goals.

## Advanced Trust Research

The Phase 11 research landing page is available at `docs/phase-11-advanced-trust-research.md`. It describes advanced experiments in adaptive trust weighting, context-aware policies, privacy-preserving composition, zero-knowledge proofs, post-quantum cryptography, federated analytics, and AI-assisted policy tooling.
