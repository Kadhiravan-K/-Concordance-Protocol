Migration notes: manifest authentication

The external manifest loader now requires manifest authentication.

Changes:
- Manifests must include the following fields:
  - `publisher_key`: hex-encoded Ed25519 public key (32 bytes -> 64 hex chars)
  - `signature_alg`: must be `ed25519`
  - `signature`: hex-encoded Ed25519 signature over the CBOR canonical preimage of the manifest JSON with the `signature` field omitted. The `publisher_key` must be included in the preimage.

Behavior:
- Unsigned manifests are now rejected with `InvalidManifest`.
- Invalid signatures are rejected.
- Operators can still allow local file reads with `PILOT_HARNESS_ALLOW_LOCAL=1` and `PILOT_HARNESS_LOCAL_BASE_DIR`.

How to sign a manifest (example):
1. Create manifest JSON including `publisher_key` (hex public key) and WITHOUT `signature`.
2. Serialize the JSON to CBOR canonically (e.g., `serde_cbor::to_vec(&json_value_without_signature)` in Rust).
3. Sign the CBOR bytes with the Ed25519 private key to produce a 64-byte signature.
4. Hex-encode the signature and add it to the manifest as the `signature` field.
5. Publish the manifest.

Note: The signing preimage must include the `publisher_key` field; do not include the `signature` field when computing the preimage.
