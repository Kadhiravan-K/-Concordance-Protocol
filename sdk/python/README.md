# Concordance Python SDK

This crate exposes a minimal Python binding layer for Concordance core envelope
operations.

## Build

```bash
cd sdk/python
cargo build
```

## Usage

From Python, install the crate using a PyO3-compatible workflow such as
`maturin` or `pip`.

Example usage:

```python
from concordance_python import sign_envelope, verify_envelope

issuer_key = bytes([1] * 32)
presenter_key = bytes([2] * 32)

envelope = sign_envelope(
    "urn:example:scheme:demo:v1",
    "capability",
    "Support",
    "did:example:subject",
    "did:example:issuer",
    b"native payload",
    1.0,
    "urn:example:adapter:demo:v1",
    1_700_000_000_000,
    None,
    None,
    None,
    issuer_key,
    presenter_key,
    "session-1",
)

print(verify_envelope(envelope))
```
