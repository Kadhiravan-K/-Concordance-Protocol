# Phase 3 adapter-pilot plan

## Built now

1. A signed adapter-announcement type in `concordance-core`.
2. A stable fixture-conformance API in `concordance-adapters`.
3. An ERC-8004 feedback normalizer with active/revoked, tag, fixed-point, and
   local-range validation.
4. An Ed25519 signed-capability-grant normalizer with trusted-issuer,
   exact-capability, and expiry validation.

## Selected external target

The consent target for Phase 3 is **Anumati**. The repository research names an
`ERC-8004 + Anumati` pair as the strongest next interoperability test because
it covers one reputation path and one durable-consent path without opening
multiple external protocols in parallel.

## Pilot harness boundary

The core never makes external calls. A dedicated `pilot-harness` crate owns
RPC or HTTP access, source authentication, canonical payload generation,
caching, finality policy, and any native revocation or policy-change watching.
The adapters remain pure normalizers over already-obtained native payloads.

## Pilot evidence still required

The ERC-8004 adapter must be exercised against independently maintained
fixtures or a verified deployment before it is called interoperable. The
placeholder signed-capability-grant path remains useful for deterministic tests
but does not satisfy the phase gate. Anumati fixtures or reproducible
live-derived canonical payloads must be added for the real consent path. Both
real adapters need published conformance reports, including malformed and
signature-tamper cases.

## Required conformance report contents

Each published Phase 3 report must identify:

- adapter identifier, scheme URI, and normalizer URI
- source classification: repository fixture, independently maintained fixture,
  or live-derived canonical fixture
- source identifier and verification policy
- fixture coverage, including malformed and tamper cases
- per-fixture pass or fail results
- reproducibility metadata for re-running the report

## Acceptance criteria

Phase 3 remains open until all of the following are true:

1. ERC-8004 fixtures come from an independently maintained or verified source.
2. Anumati is implemented as the real consent adapter target.
3. Both adapters pass offline conformance against their published fixture sets.
4. Published conformance reports exist for both adapters.
5. The pilot harness proves external acquisition and canonicalization without
   moving HTTP or chain logic into `concordance-core` or the adapter crate.
