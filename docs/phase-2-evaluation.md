# Phase 2 evaluation contract

The deterministic simulator reports one CSV row per run. Its `naive_*` fields
apply noisy-OR directly to all currently valid supporting reputation TOEs;
`capped_*` fields invoke the reference composition algorithm, which first
collapses declared common independence classes. `*_adversarial_allows` is the
primary synthetic safety comparison. A lower capped value is useful evidence
only for the declared synthetic scenario; it is not evidence of real-world
Sybil detection.

## Required scenario matrix

Run each scale (10, 100, 1,000 agents) with the same seed and 1, 2, and 3
maximum schemes per agent. For each scale, collect: baseline; 10% adversarial;
10% revoked; 10% expired; and 10% conflicting evidence. Store the CSV output
with the command/configuration used to produce it.

```powershell
cargo run -p concordance-simulator -- --agents 1000 --max-schemes 3 --adversarial-percent 10 --revoked-percent 10 --expired-percent 10 --conflict-percent 10 --seed 7 --format csv
cargo run -p concordance-benchmarks -- --format csv
```

The integration benchmark establishes implementation-count growth only. Phase
2 is not complete until the model is replaced or supplemented by independently
measured adapter LOC, engineering time, and conformance time for identical
fixture contracts.
