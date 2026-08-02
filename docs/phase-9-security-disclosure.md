# Phase 9 — Security Disclosure Process

This policy defines how Concordance receives, triages, and responds to vulnerability reports, protocol issues, and governance risks.
It applies to the protocol, registry service, adapter metadata, and documentation artifacts.

## Scope

This process covers:

- Concordance protocol specifications and schema definitions
- Reference and peer registry implementations
- Adapter approvals, announcements, and certification metadata
- Documentation, governance artifacts, and release procedures

## Reporting Channels

Preferred reporting channels:

- Email: security@concordance.example (replace with a project-managed alias)
- Issue template: `security-report` in the repository
- Secure messaging: PGP/GPG fingerprint published in governance metadata

Reports should include:

- Affected component(s)
- Description of the issue
- Steps to reproduce
- Impact assessment
- Suggested mitigation, if available

## Severity Classification

- Critical: remote compromise, unauthenticated full control, or serious protocol breakage.
- High: significant data integrity/privacy risk, or trust model violation.
- Medium: local impact, partial denial-of-service, or governance inconsistency.
- Low: informational issues, documentation errors, or low-risk edge cases.

## Timeline and Acknowledgement

- Acknowledge receipt within 72 hours.
- Provide an initial response or triage within 5 calendar days.
- Publish a mitigation plan or status update within 30 calendar days.

## Coordination and Disclosure

- Maintainers should coordinate with the reporter before public disclosure.
- If the reporter requests an embargo, respect it while the issue is being validated and fixed.
- Public disclosure may occur when:
  - a fix is available and deployed,
  - a coordinated disclosure date is agreed,
  - the issue is otherwise too severe to keep private.

## Response Process

1. Triage the report and assign a severity.
2. Identify affected components and potential scope.
3. Validate the issue and reproduce it if possible.
4. Draft a mitigation or fix plan.
5. Communicate status updates to the reporter and governance group.
6. Track the issue, planned release, and disclosure timeline.

## Documentation and Transparency

- Record security incidents in a governance registry or secure incident log.
- Publish security advisories for fixed issues, with severity and affected versions.
- Maintain a public version history of security fixes and advisories.

## Governance Integration

- Security advisories are governed by the release cadence policy.
- Critical or high-severity issues may trigger emergency review within the Phase 9 governance body.
- Security issues affecting adapter certification should prompt revalidation or revocation of trust listings.
