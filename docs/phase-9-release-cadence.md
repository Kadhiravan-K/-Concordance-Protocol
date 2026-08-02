# Phase 9 — Release Cadence Policy

This policy defines Concordance release timing, version semantics, and the relationship between governance review and technical publication.
It is intended to keep protocol evolution predictable, auditable, and compatible.

## Release Types

- Major release
  - Introduces incompatible protocol or schema changes
  - Requires governance approval and at least two independent implementations
  - Includes a published migration plan
- Minor release
  - Adds compatible features, new optional fields, or new adapter/registry metadata
  - Requires governance review and conformance test updates
- Patch release
  - Fixes security vulnerabilities, correctness bugs, or governance tooling defects
  - May be issued on an accelerated timeline
- Emergency advisory
  - Issued for critical security or registry incidents outside the normal cadence

## Version Semantics

Adopt a semver-inspired scheme for:

- protocol version
- schema version
- transport version

Compatibility guarantees:

- Major version changes may break compatibility and require explicit migration.
- Minor changes are backward-compatible for existing clients.
- Patch releases preserve semantics and are safe for deployment without protocol migration.

## Release Schedule

- Quarterly cadence for minor and patch releases.
- Annual cadence for major releases, aligned with governance review cycles.
- Release candidates and pre-release versions should be available at least 30 days before a major release.
- Security advisories and emergency patches may be issued at any time.

## Release Workflow

1. Proposal accepted via RFC template.
2. Implementation and interoperability testing performed.
3. Conformance reports and audit artifacts are generated.
4. Governance body approves the release candidate.
5. Publish release notes, advisory summaries, and registry metadata updates.
6. Monitor adoption and collect stakeholder feedback.

## Governance Review and Approval

- Major and minor releases require formal governance review.
- The governance process must record reviewers, dissenting opinions, and decision rationale.
- Changes to adapter approval, registry governance, or security disclosure policies must be synchronized with the release.

## Change Management

- Document deprecation schedules and sunset timelines for removed features.
- Provide migration guidance for adapters, registries, and clients.
- Coordinate release timing with registry operators and ecosystem participants.

## Emergency and Security Releases

- Critical security fixes may bypass the normal cadence when necessary.
- Emergency releases must still document the change, impacted versions, and mitigation steps.
- After issuance, the governance body must review the event and recommend any policy updates.
