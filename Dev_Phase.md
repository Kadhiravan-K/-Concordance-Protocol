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
maintained fixtures or integrations, published conformance reports exist for
both paths, and the selected consent target is exercised through the pilot
harness rather than a Concordance-owned placeholder fixture.

**Implementation status:** signed adapter announcements, a fixture-conformance
API, an ERC-8004 feedback normalizer, and a signed capability-grant normalizer
are implemented. This phase remains open: the ERC-8004 path has not yet been
validated against an independent deployment, and the capability grant is a
Concordance fixture format rather than a third-party protocol integration. The
next build step is a pilot harness that keeps network and source-policy logic
outside `concordance-core`, validates ERC-8004 against external evidence, and
adds Anumati as the selected consent target. Phase 4 work stays deferred until
that evidence exists.

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

## Phase 6 — Developer Experience (DX)

Goal: Make Concordance easy to adopt.

Build:

Better CLI
Interactive inspector
Documentation website
Tutorials
Sample applications
Example repositories
API reference
SDK examples

Exit gate

A new developer can build a working adapter in less than one day using only the published documentation.

## Phase 7 — Observability & Operations

Goal: Make trust decisions explainable in production.

Build:

Audit logs
Trust decision history
Metrics
Event tracing
Health endpoints
Decision visualization

Example:
~~~

Agent A

↓

Decision

↓

Explain WHY

↓

Confidence

↓

Evidence

↓

Policy

Exit gate
~~~



Every trust decision can be fully reconstructed from logs without re-running the interaction.

## Phase 8 — Certification Program

Goal: Ensure interoperability.

Create

Concordance Certification Suite

Tests

Message compatibility
Schema validation
Adapter compliance
Revocation behavior
Composition correctness

Exit gate

Independent implementations pass the certification suite.

## Phase 9 — Governance

Goal: Build a sustainable protocol ecosystem.

Build:

- Version policy
  - Define compatibility semantics for protocol, schema, and transport changes.
  - Document upgrade guidance and compatibility guarantees.
- RFC process
  - Create a public proposal template and review workflow.
  - Track proposal status, comment, and approval history.
- Deprecation policy
  - Define sunset timelines for schema fields, transport features, and adapter capabilities.
  - Publish migration and compatibility guidance for consumers.
- Adapter approval
  - Establish review criteria for adapter announcements, certification, and conformance.
  - Document independent validation, release criteria, and permitted adapter behaviors.
- Registry governance
  - Define operating roles, trust boundaries, and audit transparency.
  - Document how the reference registry collaborates with independent registries.
- Security disclosure process
  - Define reporting channels, response timelines, and coordinator roles.
  - Publish severity classification and disclosure expectations.
- Release cadence
  - Specify cadence for protocol releases, advisory updates, and governance decisions.
  - Tie releases to independent implementation and certification checkpoints.

Exit gate

External contributors can propose and standardize protocol extensions through a documented process.

**Implementation status:**

The Phase 9 governance landing page exists in `docs/phase-9-governance.md`. The next step is to formalize the RFC and review process in a repository-maintained governance document.

## Phase 10 — Ecosystem Expansion

Goal: Grow adoption.

Build:

- MCP extension
  - Define an optional interoperability path with the Model Context Protocol ecosystem.
  - Publish extension guidance and compatibility examples.
- A2A extension
  - Add optional agent-to-agent transport or discovery integration.
  - Keep core Concordance behavior unchanged by default.
- ANP integration
  - Define how Concordance can interoperate with adapter normalization or registry coordination layers.
  - Document compatibility boundaries and integration patterns.
- Enterprise examples
  - Publish reference deployments for secure registries, adapter approval workflows, and audit-ready operations.
- Healthcare profile
  - Build a healthcare-specific example that exercises consent, credential exchange, and revocation semantics.
- Finance profile
  - Build a finance-specific example that exercises regulatory compliance, risk review, and transaction authorization.
- Government profile
  - Build a government-specific example that exercises identity, authorization, and audit requirements.
- Academic collaborations
  - Partner with research groups to validate Concordance against new use cases, simulations, or governance models.

Exit gate

At least three independent ecosystems use Concordance without protocol modifications.

**Implementation status:**

The Phase 10 expansion landing page exists in `docs/phase-10-ecosystem-expansion.md`. The next step is to publish independent ecosystem artifacts and evidence for three adoption scenarios.

## Phase 11 — Advanced Trust Research

Once the core protocol is stable, explore new research directions rather than adding more infrastructure.

Potential topics:

Adaptive trust weighting
Context-aware policies
Privacy-preserving evidence composition
Zero-knowledge proof integration
Post-quantum cryptography
Federated trust analytics
AI-assisted policy recommendations

Exit gate

At least one experimental extension is published and shown to be backward-compatible with Concordance v1.x.