# Deterministic test vectors

The canonical golden vectors are generated from fixed Ed25519 seed keys in
`concordance_core` unit tests. Before publishing a release, run
`cargo test --workspace` and export the resulting `TrustObjectEnvelope` JSON
from the deterministic simulator into this directory. The repository does not
check in a manually created signature vector because an unverifiable hand-made
cryptographic fixture would be worse than the executable vectors in the test
suite.

The fixed vectors cover: a valid signed TOE, a redacted TOE preserving the same
ID/signature, a tampered TOE, a correlation-capped pair, an authorized
revocation, and a replayed revocation.
