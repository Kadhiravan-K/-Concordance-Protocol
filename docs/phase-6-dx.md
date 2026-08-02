# Phase 6 — Developer Experience (DX)

Phase 6 is about making Concordance easier to adopt through better tools,
examples, and documentation.

## Built so far

- `concordance` CLI with `inspect`, `verify`, `summary`, and `interactive`
  commands.
- Developer-facing API reference in `docs/api-reference.md`.
- Python SDK crate skeleton in `sdk/python`.
- Standalone Rust sample application in `examples/standalone`.

## Getting started

1. Run the CLI against a JSON bundle:

   ```powershell
   cargo run -p concordance-cli -- inspect bundle.json
   cargo run -p concordance-cli -- verify bundle.json
   cargo run -p concordance-cli -- summary bundle.json
   cargo run -p concordance-cli -- interactive bundle.json
   ```

2. Build the Python SDK:

   ```powershell
   cd sdk/python
   cargo build
   ```

4. Run the Rust sample application:

   ```powershell
   cargo run --manifest-path examples/standalone/Cargo.toml
   ```

5. Run the certification suite:

   ```powershell
   cargo run --manifest-path certification/Cargo.toml -- --reports-dir registry/adapters
   ```

## Phase 8 Certification

The certification harness validates published adapter conformance reports against the public report schema in `schemas/adapter-conformance-report.schema.json`.
It ensures each report is:

- schema-valid
- published from external or live-derived fixtures
- fully covered for malformed, revoked, expired, and signature tamper cases
- all fixtures passing verification
- includes revocation-oriented evidence results

## Goal

A new developer should be able to inspect a Concordance bundle, validate its
integrity, and explore the envelope contents without writing additional code.
