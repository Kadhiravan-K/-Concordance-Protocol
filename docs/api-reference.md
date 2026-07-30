# Core API

`TrustObjectEnvelope::sign` creates a signed deterministic TOE and `verify`
validates its identifier, payload commitment, issuer signature, and presenter
binding. `AdapterRegistry::normalize` invokes trusted local adapters.
`negotiate` compares a verifier policy against advertised capabilities.
`compose` returns claim results and a derivation trace; `decide` maps that
result to `ALLOW`, `DENY`, `ESCALATE`, or `CONFLICT`.

`RevokeEcho::sign` creates an issuer-authorized event and
`RevocationState::apply` validates ordering and invalidates the referenced TOE.
