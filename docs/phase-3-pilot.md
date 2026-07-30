# Phase 3 adapter-pilot plan

## Built now

1. A signed adapter-announcement type in `concordance-core`.
2. A stable fixture-conformance API in `concordance-adapters`.
3. An ERC-8004 feedback normalizer with active/revoked, tag, fixed-point, and
   local-range validation.
4. An Ed25519 signed-capability-grant normalizer with trusted-issuer,
   exact-capability, and expiry validation.

## Pilot evidence still required

The ERC-8004 adapter must be exercised against independently maintained
fixtures or a verified deployment before it is called interoperable. The
capability path must be replaced or supplemented by a selected external
capability/consent protocol and its fixtures. Both adapters need published
conformance reports, including malformed and signature-tamper cases.

The core never makes external calls; a later integration host owns RPC access,
chain finality, source authentication, caching, and native revocation event
watching.
