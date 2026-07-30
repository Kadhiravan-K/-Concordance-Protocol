# Concordance Development Roadmap

This roadmap replaces the previous subsystem-by-subsystem ordering. A phase is
not complete merely because code exists: its stated evidence gate must pass.

## Phase 0 — Protocol contract closure

Produce the versioned TOE, manifest, policy, evidence-bundle, adapter, and
revocation schemas; normative encoding/signature rules; threat model; diagrams;
and executable golden vectors. Resolve payload redaction, identifier binding,
conflict, adapter-trust, and revocation-replay semantics.

**Exit gate:** an independent implementer can reproduce the MVP behavior from
the specification and vectors without a semantic clarification.

## Phase 1 — Deterministic vertical-slice MVP

Build the Rust core, typed local policy, synthetic reputation and consent
adapters, two-agent in-process simulation, inspector CLI, composition trace,
and authorized revocation/recomposition flow.

**Exit gate:** CI demonstrates negotiate → present → compose → decide → revoke
without network, time, or external-chain dependencies.

## Phase 2 — Simulation and falsification benchmark

Run deterministic 10–1,000-agent synthetic scenarios with configurable scheme
mix, expiry, revocation, correlation, and adversarial evidence. Compare capped
noisy-OR with naïve composition. Measure the one-adapter-per-scheme model
against bespoke pairwise integrations for 1, 2, 4, 8, and 16 schemes.

**Exit gate:** publish the data and explicitly report whether the O(n) claim
holds under measured adapter effort; a negative result is valid evidence.

**Implementation status:** the deterministic simulator, CSV result contract,
scenario matrix, and integration-count model are implemented. The exit gate is
intentionally still open until measured adapter-effort data is collected and
published.

## Phase 3 — Adapter SDK and real-adapter pilot

Stabilize the adapter trait, announcement metadata, fixture contract, and
conformance suite. Add one ERC-8004-style reputation adapter and one stable
consent/capability adapter. Do not add DID, VC, OAuth/OIDC, JWT, X.509, or IBCT
in parallel.

**Exit gate:** both real adapters pass conformance tests against independently
maintained fixtures or integrations.

## Phase 4 — Federated pilot and reference service

Add the Axum reference service, HTTP transport shim, signed-record registry,
durable revocation delivery, and MCP/A2A examples. Operate two independent
registry nodes in a non-production multi-organization pilot. Introduce
PostgreSQL, Redis, or NATS only where measurement justifies them.

**Exit gate:** the pilot documents reduced bespoke integration work and safe
behavior when a registry, adapter, or revocation delivery path fails.

## Phase 5 — Hardening and standardization

Add codec/schema fuzzing, malformed-signature and replay testing, adapter
security review, service benchmarks, Docker deployment, Python bindings, and
two independent implementations. Pursue standardization after real pilot
traffic; TypeScript/Go SDKs, Kubernetes, ANP, OpenClaw, Hermes, and SwarmPher
remain demand-driven follow-on work.

**Exit gate:** independent implementations and a documented governance process
exist before a public trust-network or standards proposal is claimed.
