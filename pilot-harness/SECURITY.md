Security hardening for external manifest and fixture fetching

This document describes operator-configurable policies and the secure defaults implemented in `pilot-harness/src/external.rs`.

Defaults (safe):
- Only HTTPS remote fetches are permitted by default. `http://` is rejected.
- Local file access (`file://` and raw filesystem paths) is disabled by default.
- Max download size: 1 MiB (configurable via `PILOT_HARNESS_MAX_BYTES`).
- HTTP connect timeout: 5s (`PILOT_HARNESS_HTTP_CONNECT_TIMEOUT_SECS`).
- HTTP read timeout: 10s (`PILOT_HARNESS_HTTP_READ_TIMEOUT_SECS`).
- Redirect limit: 5 (`PILOT_HARNESS_MAX_REDIRECTS`).
- Optional host allowlist: set `PILOT_HARNESS_ALLOWLIST_HOSTS` (comma-separated) to restrict HTTPS hosts.
- Manifest signature verification: if a manifest includes `publisher_key` and `signature` they are verified (Ed25519) against the canonical CBOR preimage. Set `PILOT_HARNESS_REQUIRE_SIGNED_MANIFESTS=1` to reject unsigned manifests.

How to opt-in to allow local fixtures (operator-responsibility):
- Set `PILOT_HARNESS_ALLOW_LOCAL=1` and `PILOT_HARNESS_LOCAL_BASE_DIR` to the directory root that local fixture paths must reside under.
- The harness will canonicalize paths and reject any that escape the base dir.

Content-Type validation:
- Remote manifest fetches must provide a `Content-Type` header of `application/json` or a `+json` subtype. Local files do not provide content-type information and are accepted when local access is enabled.

Errors and failure modes:
- Fetch failures return descriptive errors and do not proceed to process untrusted payloads.
- Large responses are rejected before being processed.

Operator guidance:
- In production, set `PILOT_HARNESS_REQUIRE_SIGNED_MANIFESTS=1` and maintain a list of approved publisher keys in your deployment pipeline.
- Run the harness behind an outbound network allowlist and monitor its network activity.
