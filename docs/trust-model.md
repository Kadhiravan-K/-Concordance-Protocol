# Trust model

Concordance trusts no score by default. A relying agent locally chooses trusted
native issuers, adapter publishers, adapter versions, and policies. The protocol
then provides verifiable handling of those chosen inputs: signed TOE integrity,
presenter binding, freshness, declared correlation capping, conflict surfacing,
and authorized revocation.

An adapter’s normalized strength is evidence, not a global truth value. A
receiver that lacks a trusted adapter or cannot verify a required claim must
escalate. The MVP intentionally has no automatic cross-scheme identity linking
or correlation discovery.
