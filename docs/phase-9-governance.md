# Phase 9 — Governance

Goal: Build a sustainable protocol ecosystem.

This phase establishes the process, policies, and institutional controls needed to evolve Concordance beyond a single implementation.

## Purpose

Phase 9 organizes how Concordance changes, how adapters are approved, and how the registry is governed. It ensures that protocol evolution is transparent, accountable, and safe for independent implementers.

## What to build

- Version policy
  - Define `major.minor.patch` semantics for protocol, schema, and transport compatibility.
  - Specify compatibility guarantees for adapters, registries, and clients.
  - Document migration guidance for adopters.

- RFC process
  - Create a public proposal template for protocol extensions, schema changes, and ecosystem policies.
  - Define review, comment, and approval workflows.
  - Track decisions, dissent, and implementation status.

- Deprecation policy
  - Define how features, schemas, endpoints, and adapter capabilities are deprecated.
  - Specify sunset schedules, compatibility windows, and tooling deadlines.
  - Publish upgrade paths and migration guidance for consumers.

- Adapter approval
  - Establish criteria for adapter announcement, conformance reporting, and certification.
  - Document review requirements, compatibility tests, and required documentation.
  - Define a process for trusted adapter listings and independent validation.

- Registry governance
  - Define registry roles, operating responsibilities, and trust boundaries.
  - Document how the reference registry is administered and how independent registries cooperate.
  - Specify audit and transparency requirements for published records.

- Security disclosure process
  - Establish channels for reporting vulnerabilities and protocol issues.
  - Define response timelines, severity classifications, and disclosure policy.
  - Publish expectations for coordinated disclosure and community communication.

- Release cadence
  - Specify how often protocol releases, security advisories, and governance updates are issued.
  - Define checkpoints for independent implementation, testing, and publication.
  - Document the relationship between technical releases and governance reviews.

## Reference artifacts

- [RFC proposal template](phase-9-rfc-template.md)
- [Adapter approval and certification workflow](phase-9-adapter-approval.md)
- [Registry governance rules](phase-9-registry-governance.md)
- [Security disclosure process](phase-9-security-disclosure.md)
- [Release cadence policy](phase-9-release-cadence.md)

## Exit gate

External contributors can propose and standardize protocol extensions through a documented process.

## How to use this doc

This file should be the landing page for Phase 9 governance activities. Each bullet should be backed by a concrete specification, template, or process definition before the phase is considered complete.
