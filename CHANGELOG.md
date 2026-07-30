# Changelog

## 0.1.0 — Deterministic MVP

- Added the Concordance/1.0 protocol contract, schemas, threat/trust models,
  lifecycle diagram, and revised evidence-first development roadmap.
- Added Rust TOE signing/verification, signed manifests, typed policies,
  adapter registry, composition, decisioning, and replay-safe revocation.
- Added deterministic synthetic simulator, integration-count benchmark, CLI,
  tests, and GitHub Actions workflow.
- Deferred external adapters, remote registry/service, and production
  infrastructure to their evidence-gated phases.

## Unreleased — Phase 2 simulation harness

- Replaced the illustrative network loop with a deterministic, configurable
  10–1,000-agent simulator and CSV output contract.
- Added adversarial correlation, expiry, revocation, conflict, and 1–3-scheme
  scenarios plus repeatability tests.
- Expanded the integration benchmark to include conformance-suite counts and
  documented the measurement matrix required to validate the central claim.

## Unreleased — Phase 3 adapter pilots

- Added signed adapter announcements and local conformance-fixture execution.
- Added fixture-based ERC-8004 feedback and signed capability-grant adapters.
- Explicitly deferred live-chain and third-party capability-protocol claims
  until independently maintained validation is available.

## Unreleased — Phase 3 pilot closure

- Selected Anumati as the real consent target alongside ERC-8004 for the next
  interoperability phase.
- Added a pilot-harness crate boundary so source retrieval and canonicalization
  stay outside `concordance-core` and `concordance-adapters`.
- Added conformance-report metadata and schema validation so published reports
  can back interoperability claims.
- Added a Phase 2 publication artifact contract for measured adapter-effort
  evidence before service work begins.
