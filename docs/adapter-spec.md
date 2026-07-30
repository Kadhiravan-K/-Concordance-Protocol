# Adapter contract

An adapter implements `TrustAdapter`: it declares one immutable `scheme_uri`,
one versioned `normalization_fn_uri`, and a pure `normalize(payload) -> [0,1]`
operation. It must not perform network I/O, mutate protocol state, or decide an
interaction outcome.

Each candidate production adapter must ship signed announcement metadata, input
fixtures, expected strengths, malformed-input fixtures, and a conformance run.
The two built-in synthetic adapters exist solely for deterministic MVP testing.
No remote adapter is downloaded or executed by the MVP.
