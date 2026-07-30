# Concordance/1.0 Protocol Specification

Status: normative for the deterministic reference MVP. This specification is
transport-independent and does not define discovery of counterparties or a
global trust authority.

## 1. Canonical objects and cryptography

Concordance messages use CBOR for production and the JSON schemas in
[`../schemas`](../schemas) as their human-readable contract. Implementations
MUST preserve the field order defined by the Rust public structs when producing
the signed CBOR preimages. JSON is a debug representation and MUST NOT be used
to calculate a signature or identifier.

A TOE identifier is the lowercase hexadecimal BLAKE3 digest of its canonical
CBOR **envelope preimage**. The preimage includes every TOE field except
`envelope_id`, `issuer_signature`, `binding_proof`, `native_payload`, and
`redacted`. The native payload is committed by `payload_commitment`, its BLAKE3
digest. This permits a signed TOE to be redacted without changing its ID or
issuer signature.

The issuer MUST sign the envelope preimage with Ed25519. A presenter MUST sign
the UTF-8 binding preimage
`Concordance/1.0/binding/<envelope_id>:<subject>:<session_id>` with Ed25519.
In v1, `binding_proof.presenter_id` MUST equal `subject`. Cross-identifier
binding is deferred until a verified native binding adapter exists.

`native_payload` MAY be absent only when `redacted` is true. Receivers MUST
verify the payload commitment when the payload is present; redacted payloads
remain cryptographically attributable but are not independently re-verifiable
until disclosed through an `ENVELOPE_CHALLENGE`.

## 2. Messages and negotiation

The protocol messages are `MANIFEST_REQUEST`, `MANIFEST_OFFER`,
`ENVELOPE_PRESENT`, `ENVELOPE_CHALLENGE`, `DECISION`, `REVOKE_ECHO`,
`ADAPTER_ANNOUNCE`, and `ADAPTER_QUERY`. The MVP implements the message data
types through in-process calls; wire transport is a later shim.

Every manifest is Ed25519-signed by its `agent_key` over its canonical CBOR
preimage (all fields except `signature`) and MUST be verified before
negotiation. For a policy class, the verifier accepts negotiation only when the presenter
advertises every required claim class and the verifier advertises that it can
verify every required class. Otherwise it returns the complete list of missing
classes. A manifest capability advertises both a scheme URI and its claim
classes, avoiding an unsafe inference from a scheme name alone.

## 3. Composition and decisions

Receivers first reject invalid signatures, revoked TOEs, stale TOEs, and TOEs
outside the policy's expiry window. They group remaining supporting evidence by
claim class, collapse every shared `independence_class` to its maximum strength,
then combine distinct groups with `1 - product(1 - strength)`.

Evidence with opposite `polarity` values is a conflict when their strengths
differ by at least `policy.conflict_delta`. A conflict results in `CONFLICT`;
it is never automatically allowed. A missing required class results in
`ESCALATE`. An insufficient class results in `ESCALATE` when its weakest value
is at least `escalation_floor`, otherwise `DENY`. All sufficient, non-conflict
classes result in `ALLOW`.

The independence class is an assertion supplied by the evidence issuer, not a
solution to correlation discovery. Implementations MUST surface it in their
derivation trace and MUST NOT claim automated Sybil detection.

## 4. Revocation

Only the same `(issuer, issuer_key)` that signed a TOE may revoke it in v1.
`REVOKE_ECHO` is Ed25519-signed over its canonical CBOR tuple and contains a
strictly increasing, per-envelope sequence number. A receiver MUST reject an
invalid signer and any sequence not greater than the last accepted sequence.
Accepted revocations are idempotent and force recomposition of active decisions.

## 5. Adapter trust

An adapter is a pure normalizer from a native payload to `[0,1]`. An announced
adapter identifies its version, publisher key, conformance-fixture URI, and
publisher signature. Local policy selects trusted publisher keys and exact
adapter versions; an unknown adapter MUST cause `ESCALATE`, never `ALLOW`.
The MVP has only built-in synthetic adapters and no remote adapter execution.

## 6. Compatibility

`Concordance/1.0` recipients MUST reject an unknown major protocol version.
Unknown JSON fields may be ignored only outside the signed preimage. Any new
signed field requires a new major version or a separately versioned message
type. Implementations MUST retain accepted revocations until all decisions that
reference the TOE have expired.
