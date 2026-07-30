# Concordance adapter SDK

`concordance-adapters` contains local, deterministic normalizers built on the
core `TrustAdapter` trait. It also provides `run_conformance`, which runs named
fixtures without network access.

```powershell
cargo test -p concordance-adapters
```

The Phase-3 adapters are fixture-based pilots. A real-adapter exit gate requires
independently maintained fixtures or a live integration and is not met merely
by these tests.
