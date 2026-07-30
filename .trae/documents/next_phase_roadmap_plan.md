# Next phase roadmap plan

## Summary

The next build phase should focus on closing the repository's open evidence gates before starting Phase 4 service infrastructure. The strongest path is to make **Phase 3 closure** the primary engineering stream, with **Phase 2 evidence publication** completed in parallel, because the repository already implements the deterministic MVP, simulator, and fixture-based pilot adapters but does not yet prove real interoperability or publish measured adapter-effort evidence.

This plan keeps `concordance-core` pure, adds a minimal pilot harness outside the core for external validation work, selects **Anumati** as the real consent target alongside ERC-8004, and updates the roadmap and adapter contracts so the repository has one unambiguous direction.

## Current State Analysis

`Dev_Phase.md` defines an evidence-gated roadmap. Phase 2 remains open even though the simulator, scenario matrix, CSV contract, and integration-count model are implemented, because the repository still lacks published measured adapter-effort data. Phase 3 also remains open because the ERC-8004 adapter has not been validated against an independent source, and the current capability path is still a Concordance-owned fixture rather than a third-party protocol integration.

`README.md` confirms the current repo status as a deterministic, synthetic reference implementation with simulation and an integration-cost benchmark, not a production trust authority. `docs/phase-2-evaluation.md` defines the scenario matrix and explicitly says Phase 2 is incomplete until the count-growth model is replaced or supplemented by measured LOC, engineering time, and conformance time. `docs/phase-3-pilot.md`, `adapters/README.md`, `adapters/erc8004/README.md`, and `adapters/capability/README.md` all reinforce the same boundary: adapters are pure local normalizers, real interoperability still needs independently maintained fixtures or a verified deployment, and the current signed capability grant is only a deterministic placeholder.

`Tech-Stack.md` places the Axum service, persistence, fan-out, and other service infrastructure in the federated pilot stage, not the current stage. That means Phase 4 work should remain deferred until the repository proves real adapter interoperability first.

## Proposed Changes

### Roadmap direction

Define the next build phase as:

1. Phase 3 closure through a pilot harness, ERC-8004 external validation, and an Anumati consent adapter.
2. Phase 2 closure through publication of measured adapter-effort evidence.
3. Explicit deferral of Axum service, registry, durable revocation delivery, and multi-organization pilot work until the above evidence exists.

### File-by-file changes

#### `Cargo.toml`

- What: add a new workspace member for a pilot harness crate.
- Why: external evidence acquisition, source authentication, canonicalization, and conformance-report generation do not belong in `concordance-core` or the pure adapter crates.
- How: extend the workspace member list with a new crate dedicated to pilot-only external integration logic.

#### `README.md`

- What: update the status and roadmap pointers.
- Why: the current README accurately describes the synthetic MVP but does not state that the next engineering focus is Phase 3 closure with real validation and published reports.
- How: revise the status paragraph and quick-start references so readers see the active roadmap focus, the remaining open gates, and the documents that define them.

#### `Dev_Phase.md`

- What: refine the wording of Phase 3 and the transition into Phase 4.
- Why: the roadmap already says not to start service infrastructure early, but the next build plan should make that deferral explicit so future work does not mix pilot validation with reference-service construction.
- How: add a clear sub-gate for external validation, published conformance reports, and the selected Anumati consent target before any Phase 4 service work is started.

#### `docs/phase-3-pilot.md`

- What: expand this from a status note into the authoritative implementation spec for Phase 3 closure.
- Why: it already names the real gap, but it does not yet specify the selected Anumati target, report requirements, acceptance criteria, or boundaries of the pilot harness.
- How: define the external validation flow, canonical payload requirements, malformed and tamper coverage, report format, and the separation between pure adapters and the new harness.

#### `docs/phase-2-evaluation.md`

- What: extend the document with a publication contract for measured adapter-effort evidence.
- Why: the current document defines the simulation matrix, but not the exact artifact needed to close the phase.
- How: add required fields for measured LOC, implementation effort, conformance effort, methodology notes, and storage location for the published results.

#### `docs/adapter-spec.md`

- What: extend the adapter conformance contract.
- Why: interoperability claims need auditable report metadata, not just passing local fixtures.
- How: add source classification, verification policy, reproducibility metadata, malformed-case expectations, and conformance-report output requirements.

#### `adapters/README.md`

- What: reframe the adapters crate as pure deterministic normalizers plus conformance tooling.
- Why: the current README states this implicitly, but the next phase depends on a sharper line between pure adapter code and the external pilot harness.
- How: document that live-source retrieval belongs outside the crate and that Phase 3 evidence comes from published conformance reports that consume canonical fixtures.

#### `adapters/erc8004/README.md`

- What: add an explicit canonical payload and validation contract.
- Why: the README already says the adapter is not a chain client and needs an external pilot, but the next phase needs a concrete target for what the harness must produce and how it will be validated.
- How: define the canonical input shape, source and finality assumptions, rejected cases, and the evidence needed to call the adapter externally validated.

#### `adapters/capability/README.md`

- What: downgrade the current signed capability grant language to placeholder-fixture status.
- Why: the repository should not imply third-party interoperability where none exists.
- How: document that this adapter remains only for deterministic tests while Anumati becomes the real external consent path for Phase 3.

#### `adapters/src/lib.rs`

- What: add conformance-report types and the new Anumati adapter entry point.
- Why: Phase 3 closure requires machine-checkable reports and a real external target in addition to the existing fixture-only adapters.
- How: keep the `TrustAdapter` boundary intact, add report data structures and runner support, and expose the Anumati adapter implementation alongside the existing ones.

#### `.github/workflows/ci.yml`

- What: extend CI with offline conformance-report validation.
- Why: the repository should keep deterministic CI while still checking the new report schema and fixture coverage rules.
- How: validate reports and schemas offline in CI, while leaving live-source integration checks opt-in or separate from required CI.

#### `CHANGELOG.md`

- What: record the move from fixture-only pilots to external-validation work.
- Why: the repository's phase status should stay auditable over time.
- How: add an entry that captures the new pilot harness, report contract, and Phase 2 evidence publication work.

### New files and directories

#### `pilot-harness/Cargo.toml`

- What: new Rust crate manifest for the external validation harness.
- Why: provides a clean boundary for pilot-only integration logic without contaminating core crates.
- How: define a small crate that depends on the existing core and adapter crates plus only the external dependencies needed for canonicalization and report generation.

#### `pilot-harness/src/lib.rs`

- What: shared harness API.
- Why: keeps common ingestion, canonicalization, and reporting logic in one place.
- How: expose pure orchestration functions that fetch or accept native records, canonicalize them, invoke the adapter normalizers, and emit structured reports.

#### `pilot-harness/src/erc8004.rs`

- What: ERC-8004-specific ingestion and canonicalization module.
- Why: the existing adapter intentionally avoids external calls, so the new harness must own source retrieval and canonical fixture generation.
- How: implement source access, validation-policy handling, canonical payload generation, and error mapping for ERC-8004 inputs.

#### `pilot-harness/src/anumati.rs`

- What: Anumati-specific ingestion and canonicalization module.
- Why: the repository research names Anumati as the strongest consent target for real interoperability testing alongside ERC-8004, and the checked-in roadmap says not to add multiple external protocols in parallel.
- How: implement Anumati-specific ingestion, canonicalization, local validation rules, and report generation flow in parallel to the ERC-8004 module.

#### `adapters/anumati/README.md`

- What: protocol-specific adapter contract.
- Why: the real external target needs the same clarity already present in the ERC-8004 README.
- How: document native fields, canonical payload mapping, rejected cases, and conformance evidence requirements.

#### `adapters/anumati/fixtures/`

- What: independently sourced or reproducibly generated fixtures for the selected real protocol.
- Why: Phase 3 cannot close on internal-only synthetic fixtures.
- How: store canonicalized examples, malformed cases, tampered cases, and any source metadata allowed by the repository's reproducibility rules.

#### `docs/phase-2-results.md`

- What: publication artifact for measured adapter-effort evidence.
- Why: Phase 2 needs a concrete published result, not just benchmark code and a note saying more evidence is needed.
- How: record methodology, comparison basis, LOC or effort measurements, conformance-time measurements, and the interpretation of the results, including the possibility of a negative outcome.

#### `docs/phase-3-conformance-report.md`

- What: report template or reference example for published conformance outcomes.
- Why: a stable report shape helps make interoperability claims auditable and repeatable.
- How: provide the required sections and fields that each protocol validation report must include.

#### `schemas/adapter-conformance-report.schema.json`

- What: JSON schema for conformance reports.
- Why: report files should be machine-checkable in CI and reproducible by outside readers.
- How: encode required metadata, result items, source classification, verification policy, and coverage declarations.

## Assumptions & Decisions

### Decisions

1. The next build phase should not start Phase 4 service infrastructure yet.
2. Phase 3 closure is the primary engineering stream because it resolves the repository's strongest open interoperability gap.
3. Phase 2 closure runs in parallel as a publication and evidence task, not as a new simulator-feature stream.
4. `concordance-core` remains transport- and network-free.
5. External-source retrieval, source policy, canonicalization, and report generation belong in a new harness outside the core and adapter crates.
6. Anumati is the selected real consent protocol for the next phase because the repository research explicitly identifies an `ERC-8004 + Anumati` starting pair for real interoperability testing.

### Assumptions

1. The current protocol and core adapter boundary are stable enough that the next phase is mainly about validation and documentation, not redesign.
2. Independently maintained fixtures or reproducible live-derived canonical fixtures can be obtained for ERC-8004 and Anumati.
3. The pilot harness can stay smaller in scope than a full reference service and still provide enough evidence to close Phase 3.
4. Published conformance reports and measured-effort results are acceptable evidence for closing the current open phases.

## Verification steps

### Existing verification to keep

1. Run `cargo test --workspace`.
2. Run `cargo run -p concordance-simulator -- --agents 1000 --max-schemes 3 --adversarial-percent 10 --revoked-percent 10 --expired-percent 10 --conflict-percent 10 --seed 7 --format csv`.
3. Run `cargo run -p concordance-benchmarks -- --format csv`.

### New verification to add

1. Validate offline fixture conformance for ERC-8004 and the Anumati adapter.
2. Validate malformed, revoked, expired, mismatched-policy, and signature-tamper cases for each real adapter where applicable.
3. Validate that conformance reports match `schemas/adapter-conformance-report.schema.json`.
4. Test that the pilot harness produces reproducible canonical payloads for identical native inputs.
5. Verify that untrusted or incomplete source inputs are rejected by harness policy checks.
6. Confirm that the repository contains at least one published ERC-8004 conformance report, one published Anumati conformance report, and one published Phase 2 measured-effort artifact before claiming the open phases are closed.
