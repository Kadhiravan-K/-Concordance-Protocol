# Phase 9 — Registry Governance Rules

This document defines the operating rules, trust boundaries, and transparency expectations for Concordance registries.
It applies to the reference registry and independent peer registries participating in Phase 4 federation.

## Registry Purpose

The registry is a publication and discovery mechanism for:

- adapter announcements
- scheme manifests
- revocation echoes
- conformance reports
- audit and observability records

It is not a trust authority; registry records are evidence of publication and synchronization, not endorsement.

## Governance Principles

- Transparency: registry operations, record history, and governance decisions must be auditable.
- Decentralization: independent registries may peer and synchronize records without requiring a single authoritative operator.
- Auditability: append-only logs, durable storage, and observable event streams must support replay and verification.
- Accountability: node operators must document their identity, data retention, and peer sync behavior.

## Roles and Responsibilities

- Reference Registry Operator
  - Maintains the canonical reference node.
  - Publishes trusted adapter, manifest, and revocation records.
  - Supports synchronization and audit APIs.

- Peer Registry Operator
  - Runs an independent node.
  - Peers with other registries to replicate records.
  - Applies governance rules for record acceptance.

- Validator / Auditor
  - Reviews registry contents, audit logs, and revocation delivery.
  - Confirms compliance with governance rules.

- Observer
  - Monitors registry health, metrics, and transparency records.

## Record Acceptance Rules

A registry may accept only records that are:

- syntactically valid for the declared `SignedRecord` kind,
- correctly signed by a recognized issuer where required,
- not stale or otherwise invalid according to protocol rules,
- within the scope of peer sync agreement.

Revocations must be represented as `RevokeEcho` records and must be verified against the original envelope issuer before being applied.

## Synchronization and Federation

- Peer sync is pull-based and best-effort.
- Registries may synchronize records from peers using `sync/events` and `revoke stream` APIs.
- Revocation delivery is supported via SSE; polling fallback is permitted.
- Peer endpoints should be documented and discoverable by governance participants.

## Registry Auditing

A registry must expose:

- audit logs for published records,
- decision-history or observability history views,
- event cursors and replay support,
- health and metrics information.

Auditors should verify that:

- record cursors advance monotonically,
- published adapter and revocation records are preserved,
- governance decisions affecting registry operation are recorded.

## Trust Boundaries and Disclaimers

- Registry publication does not imply correctness of adapter logic.
- A registry operator may choose to retain or exclude records according to policy, but exclusions must be documented.
- Records from foreign peers must be labeled clearly and, when appropriate, treated as provisional until independently verified.

## Incident Response

- Node operators must document incident handling procedures.
- In the event of a compromise, the operator must notify governance participants and publish a mitigation plan.
- Critical registry incidents may trigger an emergency security advisory under the disclosure process.
