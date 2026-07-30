# Composition algebra

For each claim class, discard invalid, stale, and revoked evidence. Split the
remaining supporting TOEs by `independence_class`; an absent class is unique to
the TOE. Let each group strength be the maximum strength in that group. The
combined score is `1 - ∏(1 - group_strength)`.

Contradicting TOEs are not averaged with supporting evidence. When a support
and contradiction differ by at least `conflict_delta`, the result is
`CONFLICT`. This conservative rule is deliberate: policy authors must make an
explicit choice before a conflict can lead to an action.

This algebra limits double counting of declared common sources. It does not
prove source independence and must be evaluated by the simulator's adversarial
scenarios before any pilot claim is made.
