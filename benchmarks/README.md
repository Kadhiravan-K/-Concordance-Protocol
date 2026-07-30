# Integration-cost benchmark

Run `cargo run -p concordance-benchmarks` to emit the transparent baseline for
1, 2, 4, 8, and 16 schemes. The baseline counts one adapter per scheme versus
one bespoke implementation per unordered scheme pair. It is a falsifiable
integration-count model, not a claim that all adapters take equal effort.

The phase-2 gate is to replace the two work-unit columns with independently
measured LOC, implementation time, and conformance time for the same fixture
set, and publish negative findings if adapter complexity grows superlinearly.
