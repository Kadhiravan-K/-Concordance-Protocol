# Adapter contract

An adapter implements `TrustAdapter`: it declares one immutable `scheme_uri`,
one versioned `normalization_fn_uri`, and a pure `normalize(payload) -> [0,1]`
operation. It must not perform network I/O, mutate protocol state, or decide an
interaction outcome.

## Announcement and selection

`AdapterAnnouncement` signs the scheme URI, normalizer URI, semantic version,
publisher identity/key, and fixture URI with Ed25519. A host must verify this
announcement and locally allow-list the publisher key and exact adapter version
before registration. The registry intentionally does not fetch or execute an
announced adapter.

## Conformance fixtures

Every adapter must provide named native-payload fixtures with either an exact
expected normalized strength or an expected rejection. `run_conformance` runs
those fixtures without network access. A production candidate also requires
malformed, revoked, expired, and signature-tamper fixtures where the native
scheme supports those states.

The Phase-3 pilots are deliberately bounded:

- `erc8004` parses active fixed-point feedback and applies a local tag/range
  normalization policy. It does **not** claim to be an Ethereum RPC client or
  validate a chain event.
- `signed-capability-grant` verifies a Concordance-owned, Ed25519-signed grant
  fixture format with a local capability, time, and issuer-key policy. It is a
  testable capability adapter contract, not an implementation of OAuth, IBCT,
  Anumati, or another external standard.
