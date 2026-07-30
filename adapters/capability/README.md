# Signed capability-grant fixture adapter

This adapter verifies a small Ed25519-signed grant format with an allow-listed
issuer key, an exact required capability, and an expiration timestamp. It is
included to make the Phase-3 capability path fully testable without claiming
compatibility with a third-party authorization protocol.
