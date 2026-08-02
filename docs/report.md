# Concordance Architecture and System Report

This document describes the Concordance repository architecture, file organization, and system design. It is intended to give developers, reviewers, and contributors a single reference for how the project is structured, what each component does, and how the pieces fit together.

## 1. Project Purpose

Concordance is a research reference implementation for composing signed trust evidence across independently evolving agent ecosystems. It focuses on:

- signed Trust Object Envelopes (TOEs)
- manifest negotiation and capability discovery
- typed policy evaluation
- evidence composition and revocation reasoning
- adapter conformance and certification
- simulation, benchmarks, and pilot integration

The repository is not a production trust authority. It is designed as an experimental and architectural foundation for interoperability, certification, and ecosystem governance.

## 2. High-Level Architecture

Concordance is organized as a multi-crate Rust workspace with supporting docs, schemas, examples, and research artifacts.

### 2.1 Core Protocol and Runtime

- `core/`
  - Contains the `concordance-core` crate.
  - Implements core protocol primitives, data models, and deterministic reference behavior.
  - Includes modules for composition, crypto, envelope processing, negotiation, policy, registry interaction, and revocation.

### 2.2 Adapter and Conformance Layer

- `adapters/`
  - Contains adapter definitions, fixture contracts, and conformance helper runtime.
  - Hosts adapter-specific directories such as `erc8004/`, `jwt/`, `oidc/`, `x509/`, `did/`, `oauth/`, and `verifiable_credentials/`.
  - Supports `adapter-sdk/` for shared adapter patterns and `custom/` for extensible implementations.

- `pilot-harness/`
  - Contains support for pilot testing, integration validation, and adapter-conformance experimentation.

### 2.3 Registry and Transport

- `http/`
  - Implements an HTTP transport shim for the Concordance protocol.
  - Provides reusable request/response serialization and wire-format helpers.

- `registry-service/`
  - Contains a reference Axum-based registry service and binary.
  - Exposes observability endpoints for metrics, audit logs, and decision reconstruction.

### 2.4 CLI and Developer Tools

- `cli/`
  - Implements the `concordance` command-line tool.
  - Supports bundle inspection, verification, summary reporting, and interactive analysis.

- `examples/standalone/`
  - Contains a standalone sample application demonstrating Concordance usage.

- `sdk/python/`
  - Contains Python bindings for Concordance via PyO3.

### 2.5 Certification and Verification

- `certification/`
  - Implements the Concordance Certification Suite.
  - Validates published adapter conformance reports against schema and coverage requirements.
  - Ensures reports are produced from external or live-derived fixtures, cover malformed/revoked/expired/signature-tamper cases, and include revocation evidence.

### 2.6 Simulation, Benchmarking, and Testing

- `simulator/`
  - Implements the Concordance simulator binary.
  - Supports synthetic agent scenarios, scheme mix configurations, adversarial conditions, and reproducible CSV output.

- `benchmarks/`
  - Implements benchmark tooling and integration effort measurement.

- `tests/`
  - Contains fixtures and integration/performance/security/unit test directories.

## 3. File and Folder Architecture

### 3.1 Root-level layout

At the repository root, the key files and folders are:

- `Cargo.toml` — workspace manifest for the main Rust workspace.
- `README.md` — project overview, quick start instructions, and roadmap pointers.
- `Dev_Phase.md` — development roadmap with clearly defined phases and exit gates.
- `CHANGELOG.md`, `CONTRIBUTING.md`, `LICENSE`, `CODE_OF_CONDUCT.md` — repository governance and contribution policies.
- `docs/` — documentation, specifications, research notes, and architecture references.
- `schemas/` — JSON and JSON Schema artifacts for the protocol contract.
- `core/`, `cli/`, `http/`, `registry-service/`, `simulator/`, `benchmarks/`, `adapters/`, `pilot-harness/`, `certification/` — Rust crates and tools.
- `examples/` — sample applications and integration templates.
- `sdk/` — language-specific SDKs and bindings.
- `tests/` — test fixtures and integration scenarios.

### 3.2 Documentation folder

- `docs/api-reference.md` — CLI, SDK, registry observability, and certification documentation.
- `docs/protocol-spec.md` — normative protocol specification.
- `docs/adapter-spec.md` — adapter contract, fixture, and conformance rules.
- `docs/phase-*.md` — phase-specific process documentation for phases 2, 3, 6, 9, 10, 11, and others.
- `docs/phase-9-rfc-template.md` — RFC proposal template for governance changes.
- `docs/phase-9-adapter-approval.md` — adapter approval and certification workflow.
- `docs/phase-9-registry-governance.md` — registry governance rules.
- `docs/phase-9-security-disclosure.md` — security disclosure process.
- `docs/phase-9-release-cadence.md` — release cadence policy.
- `docs/doc_core/` — research notes and methodology documents.
- `docs/diagrams/` — diagrams referenced by the README and research docs.

### 3.3 Core crate layout

- `core/Cargo.toml` — core crate manifest.
- `core/src/lib.rs` — core library entrypoint.
- `core/composition/` — evidence composition algorithm and derivation trace.
- `core/crypto/` — cryptographic primitives and envelope signing/verification.
- `core/envelope/` — Trust Object Envelope formats and helpers.
- `core/negotiation/` — policy negotiation and capability matching.
- `core/policy/` — typed policy evaluation and policy model.
- `core/registry/` — registry normalization and adapter interaction.
- `core/revocation/` — revocation state and replay handling.
- `core/validation/` — validation logic and conformance checks.
- `core/tests/` — core unit and integration tests.

### 3.4 Support modules

Concordance remains a transport-independent interoperability layer for heterogeneous trust evidence. The repository also benefits from reusable support modules that keep identity, authorization, and security concerns organized without changing the protocol.

- `identity/`
  - `verifier.rs`
  - `authenticator.rs`
  - `binding_proof.rs`
  - `credential.rs`
  - Verifies binding proofs and identity assertions from DID, X.509, JWT, FIDO2, TPM, and other schemes.
- `authorization/`
  - `capability.rs`
  - `permissions.rs`
  - `grants.rs`
  - `delegation.rs`
  - Provides reusable application-facing authorization helpers that wrap native capability and consent evidence.
- `security/`
  - `encryption/`
  - `signature/`
  - `hashing/`
  - `secure_storage/`
  - `key_rotation/`
  - `audit/`
  - Centralizes how Concordance uses existing cryptographic and audit services.
- `plugins/` (optional future module)
  - `reputation/`
  - `consent/`
  - `capability/`
  - `governance/`
  - `audit/`
  - Formalizes adapter and extension loading without changing the core protocol.

### 3.5 Adapter and SDK layout

- `adapters/Cargo.toml` — adapter crate manifest.
- `adapters/src/` — shared adapter helper code.
- Per-scheme directories such as `adapters/erc8004/`, `adapters/oauth/`, `adapters/oidc/`, `adapters/x509/`, `adapters/did/`, `adapters/jwt/`.
- `adapters/README.md` — adapter repository overview.
- `sdk/python/` — Python binding crate manifest and source.

## 4. System Design

### 4.1 Protocol model

Concordance uses a versioned protocol model centered on signed TOEs and evidence composition. The protocol is named `Concordance/1.0` and defines:

- deterministic envelope encoding and signature validation
- manifest versioning and field constraints
- evidence bundling and presentation semantics
- revocation events and invalidation rules

The protocol is implemented as a Rust reference core and exposed through an HTTP transport shim.

### 4.2 Trust composition

The system composes evidence from multiple independent adapters. Each adapter may represent a different trust scheme or credential source. Composition is based on:

- adapter capabilities and advertisement metadata
- verifier policy rules and consent requirements
- claim derivation traces and result aggregation
- conflict/delay escalation semantics

This design separates the core composition engine from adapter-specific logic.

### 4.3 Adapter conformance and certification

Concordance tracks adapter conformance through:

- adapter fixtures and published conformance reports
- certification validation rules in `certification/`
- explicit source class restrictions for live or external evidence
- coverage requirements for malformed, revoked, expired, and tampered cases

This model supports interoperability by requiring independent evidence of correct adapter behavior.

### 4.4 Registry and observability

The reference registry service implements federated discovery and runtime observability. Key capabilities include:

- durable audit logs for trust decisions
- metrics exposure for service and registry health
- decision history reconstruction through filtered audit views
- HTTP transport integration via `http/`

These services support Phase 7 observability goals.

### 4.5 Tooling and usability enhancements

Concordance's protocol scope remains focused on transport-independent trust evidence interoperability. The following tooling components improve usability, debugging, and credibility:

- **Trust Decision Explanation Engine**
  - Produces human- and machine-readable reasoning for allow/deny/escalate/conflict outcomes.
  - Exposes confidence, evidence count, contributing claim classes, and policy triggers.
- **Replay & Trace Recorder**
  - Records complete negotiation, presentation, composition, and revocation interactions.
  - Supports deterministic replay for debugging, research, and certification.
- **Visual Inspector**
  - Inspects envelopes, composition trees, and revocation chains.
  - Supports browser or CLI-based trust graph visualization.
- **Certification Suite**
  - Verifies independent implementations produce identical results from the same fixtures and policies.
  - Supports reproducible report validation and interop testing.

### 4.6 Development phases

The repository is guided by phase-based development documentation in `Dev_Phase.md`:

- Phase 0–1: protocol contract closure and deterministic MVP
- Phase 2: simulation and falsification benchmark
- Phase 3: adapter SDK and real-adapter pilot
- Phase 4: federated pilot and reference service
- Phase 5: hardening and standardization
- Phase 6: developer experience and CLI/docs improvements
- Phase 7: observability and operations
- Phase 8: certification program
- Phase 9: governance process
- Phase 10: ecosystem expansion
- Phase 11: advanced trust research

This phased approach ensures the project evolves with explicit evidence gates and implementation milestones.

## 5. Build and Workspace Organization

### 5.1 Cargo workspace

The root `Cargo.toml` is a workspace manifest with the following members:

- `core`
- `cli`
- `simulator`
- `benchmarks`
- `adapters`
- `pilot-harness`
- `http`
- `registry-service`
- `sdk/python`
- `certification`
- `examples/standalone`

This layout enables cross-crate dependency sharing and consistent workspace dependency management.

### 5.2 Shared workspace dependencies

Common dependencies are declared at the workspace level in the root manifest, including:

- `serde`, `serde_json`, `serde_cbor`
- `blake3`, `ed25519-dalek`, `hex`
- `thiserror`
- `axum`, `tokio`, `tower`, `tower-http`
- `tracing`, `tracing-subscriber`
- `clap`, `reqwest`, `tokio-stream`, `futures-util`
- `pyo3` for Python bindings

This avoids version drift and keeps the workspace coherent.

## 6. Recommended Usage Patterns

### 6.1 Local development

- Run all tests across the workspace:

```powershell
cargo test --workspace
```

- Run the CLI inspector:

```powershell
cargo run -p concordance-cli -- inspect <bundle.json>
```

- Run the simulator:

```powershell
cargo run -p concordance-simulate -- --agents 1000 --max-schemes 3 --adversarial-percent 10 --format csv
```

- Run the certification suite:

```powershell
cargo run --manifest-path certification/Cargo.toml -- --reports-dir registry/adapters
```

### 6.2 Documentation reference

- `docs/protocol-spec.md` for protocol semantics
- `docs/adapter-spec.md` for adapter conformance rules
- `docs/api-reference.md` for CLI/SDK/registry/certification usage
- `Dev_Phase.md` for roadmap and phase gates

## 7. Notes on Recent Cleanup

The repository has removed legacy or empty directories that were not part of the active workspace:

- `server/`
- `concordance/`

It also consolidated smaller documentation fragments into `docs/architecture.md`. The removed files include:

- `docs/policy-language.md`
- `docs/threat-model.md`
- `docs/trust-model.md`
- `docs/revocation.md`

## 8. Future Architecture Considerations

The current design anticipates several evolution paths:

- standardizing the protocol version and extension governance
- expanding adapter profiles to healthcare, finance, and government
- supporting optional transport extensions such as MCP, A2A, and ANP
- researching privacy-preserving composition and post-quantum cryptography
- publishing independent ecosystem adoption artifacts for Phase 10

## 9. Summary

Concordance is a research-driven multi-crate workspace that balances protocol design, runtime implementation, and certification. Its structure is intentionally modular:

- `core` for protocol semantics
- `adapters` for extension schemes
- `registry-service` and `http` for networking
- `certification` for conformance validation
- `simulator` and `benchmarks` for measurement
- `docs/` for specification, research, and roadmap guidance

This report should serve as the master architectural overview for contributors, maintainers, and collaborators working across the Concordance project.

## 7. Notes on Recent Cleanup

The repository has removed legacy or empty directories that were not part of the active workspace:

- `server/`
- `concordance/`

This cleanup reduces confusion and keeps the top-level layout focused on active crates, docs, and schemas.

## 8. Future Architecture Considerations

The current design anticipates several evolution paths:

- standardizing the protocol version and extension governance
- expanding adapter profiles to healthcare, finance, and government
- supporting optional transport extensions such as MCP, A2A, and ANP
- researching privacy-preserving composition and post-quantum cryptography
- publishing independent ecosystem adoption artifacts for Phase 10

## 9. Summary

Concordance is a research-driven multi-crate workspace that balances protocol design, runtime implementation, and certification. Its structure is intentionally modular:

- `core` for protocol semantics
- `adapters` for extension schemes
- `registry-service` and `http` for networking
- `certification` for conformance validation
- `simulator` and `benchmarks` for measurement
- `docs/` for specification, research, and roadmap guidance

This report should serve as the master architectural overview for contributors, maintainers, and collaborators working across the Concordance project.
