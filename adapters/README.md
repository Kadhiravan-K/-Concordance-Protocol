# Concordance adapter SDK

`concordance-adapters` contains local, deterministic normalizers built on the
core `TrustAdapter` trait. It also provides `run_conformance`, which runs named
fixtures without network access and emits machine-checkable conformance reports
for Phase 3 pilot evidence.

```powershell
cargo test -p concordance-adapters
```

The Phase-3 adapters are fixture-based pilots. A real-adapter exit gate requires
independently maintained fixtures or a live integration and is not met merely
by these tests. Live-source retrieval, source authentication, canonicalization,
and finality policy belong in the separate `pilot-harness` crate rather than in
this adapter crate.
