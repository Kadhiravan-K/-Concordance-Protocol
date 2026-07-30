# Typed policy model

V1 uses a typed, versioned internal model rather than CEL or Rego:

- `required_claims`: claim class → minimum normalized strength.
- `max_envelope_age_ms`: maximum evidence age.
- `escalation_floor`: insufficient evidence at or above this value escalates;
  lower evidence denies.
- `conflict_delta`: minimum opposing-strength difference that surfaces a
  conflict.

Policies are local. They never alter native evidence verification or the TOE
signature rules. The schema is [`../schemas/policy.schema.json`](../schemas/policy.schema.json).
