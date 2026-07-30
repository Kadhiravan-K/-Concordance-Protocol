# Concordance technology decisions

| Layer | Decision | Phase |
|---|---|---|
| Core | Rust workspace | MVP |
| Prototype adapter bindings | Python after the Rust API stabilizes | Pilot |
| Serialization | CBOR signed preimages; JSON schemas/debug output | MVP |
| Cryptography | Ed25519 signatures and BLAKE3 content addressing | MVP |
| Policy | Typed, versioned Rust model | MVP |
| Reference service | Axum, with a transport-neutral core | Federated pilot |
| Registry persistence | PostgreSQL only when shared durability is required | Federated pilot |
| Fan-out/cache | NATS/Redis only after measured need | Federated pilot |
| Tests | `cargo test`, golden vectors, integration, simulation, fuzzing later | MVP onward |
| CI | GitHub Actions | MVP |
| Deployment | Docker after the service; Kubernetes post-pilot | Later |

Protocol Buffers, FastAPI, X25519, SHA-256, CEL, Rego, Kafka, all non-Rust
SDKs, Playwright, and Kubernetes are deliberately excluded from v1 MVP scope.
