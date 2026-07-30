# Signed capability-grant fixture adapter

This adapter verifies a small Ed25519-signed grant format with an allow-listed
issuer key, an exact required capability, and an expiration timestamp. It is
included to make the Phase-3 capability path fully testable without claiming
compatibility with a third-party authorization protocol.

It remains a deterministic placeholder fixture for local tests and regression
coverage. It does **not** satisfy the real consent or capability phase gate by
itself.

Phase 3 now treats Anumati as the real consent target. This fixture adapter
stays in the repository to preserve deterministic tests while the real
interoperability evidence is gathered through the Anumati adapter and the
separate pilot harness.
