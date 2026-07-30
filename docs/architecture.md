# Reference architecture

```mermaid
flowchart TB
  Native["Synthetic native evidence"] --> Adapter["Pure adapter"]
  Adapter --> TOE["Signed TOE"]
  Manifest["Signed capability manifest"] --> Negotiation
  TOE --> Negotiation["Negotiate + present"]
  Negotiation --> Compose["Validate + compose"]
  Policy["Typed local policy"] --> Compose
  Compose --> Decision["ALLOW / DENY / ESCALATE / CONFLICT"]
  Revoke["Authorized REVOKE_ECHO"] --> Revocations["Revocation state"]
  Revocations --> Compose
```

The core crate owns the pure protocol operations. The CLI displays an existing
bundle. The simulator drives a deterministic two-agent lifecycle and synthetic
network scenarios. The registry and reference server are intentionally not part
of this MVP.
