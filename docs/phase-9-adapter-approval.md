# Phase 9 — Adapter Approval and Certification Workflow

This document defines the workflow, artifacts, and trust rules for approving adapters in Concordance.
It is intended to support independent validation, registry publication, and trusted adapter listings.

## Approval Objectives

- Ensure adapters are safe, interoperable, and auditable.
- Confirm that adapter announcements and normalization functions are tamper-evident.
- Validate that adapters are accompanied by canonical fixtures and conformance evidence.

## Required Artifacts

Each adapter submission must include:

- Signed `AdapterAnnouncement` with:
  - `scheme_uri`
  - `normalization_fn_uri`
  - `version`
  - `publisher`
  - `publisher_key`
  - `fixture_uri`
- Published `ConformanceReport` covering adapter behavior and expected results.
- Canonical fixture source metadata, including:
  - `source_class`
  - `source_identifier`
  - `verification_policy`
  - `reproducibility_notes`
  - `coverage`
- Reference implementation or test harness that can verify the adapter on demand.

## Review Workflow

1. Submit a proposal using the RFC template.
2. Provide the adapter announcement and conformance report in a registry-friendly format.
3. Perform an initial validation pass:
   - Verify announcement signature and payload integrity.
   - Confirm fixture source metadata is complete and accurate.
   - Confirm the adapter implementation passes declared expectations.
4. Conduct independent review:
   - A second party replays fixtures and validates the adapter’s reported results.
   - An independent reviewer confirms the verification policy is enforceable.
5. Issue a certification decision:
   - `Trusted`: meets requirements and is ready for listing.
   - `Provisional`: accepted with conditions, pending additional review or monitoring.
   - `Rejected`: fails criteria and must be corrected before resubmission.
6. Publish adapter metadata, approval status, and audit history in the reference registry.

## Certification Criteria

Adapters are evaluated against:

- Correctness: adapter produces expected normalization values or rejects invalid payloads.
- Completeness: coverage is declared for malformed, revoked, expired, and signature tamper cases.
- Transparency: source metadata explains where canonical payloads were obtained and how they were verified.
- Reproducibility: another participant can replay the adapter validation using the same artifacts.
- Security: adapter metadata does not expose secrets, and adapter behavior is bounded by the announced scheme.

## Trusted Listing and Publication

A trusted adapter listing should expose:

- `adapter_id`
- `scheme_uri`
- `normalization_fn_uri`
- `version`
- `publisher`
- `publisher_key`
- `source_identifier`
- `verification_policy`
- `reproducibility_notes`
- `coverage`
- certification status
- review timestamp and approver identity

Listings may be posted in the registry or a separate governance catalog.

## Ongoing Management

- Adapter versions are subject to the release cadence policy.
- Deprecated adapters must follow the deprecation policy and provide migration guidance.
- Certification may be revoked if:
  - the adapter announcement signature becomes invalid,
  - conformance reports no longer match observed behavior,
  - security or compatibility issues are discovered.

## Audit and Transparency

- Preserve approval and rejection history for every adapter submission.
- Log reviewer notes, rationale, and evidence used for the decision.
- Make certification records available through the registry observability API.
