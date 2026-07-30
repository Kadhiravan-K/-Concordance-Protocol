---
name: agent-protocol-research
description: "Use this skill when asked to research AI-agent interaction/coordination protocols and invent a genuinely new protocol family (not a wrapper/extension of MCP, A2A, ACP, ANP, OAuth, gRPC, etc). Provides a literature-review checklist, an originality filter for rejecting disguised extensions, a gap-analysis framework across trust/economics/semantics/coordination/governance, a 1-10 scoring rubric for candidate protocols, and a spec template covering architecture, message formats, state machines, security, and an implementation roadmap. Trigger for requests like 'invent a new protocol for AI agents', 'what's missing from MCP/A2A', or any request to design next-generation multi-agent standards."
---

# AI-Agent Protocol Research & Invention Methodology

## Purpose
A repeatable method for (1) surveying the real state of AI-agent interaction
protocols, (2) finding gaps that are genuinely unsolved rather than merely
under-documented, (3) generating a wide candidate set of new protocol families,
and (4) specifying the strongest one to prototype-ready depth.

## Step 1 — Ground the survey in current reality, not memory
Model training data goes stale fast in this space (new protocols ship monthly).
Before analysis, search for the current state of at minimum:
- Model Context Protocol (MCP) — tool/context invocation
- Agent2Agent (A2A) — Google-originated, now Linux Foundation
- Agent Communication Protocol (ACP) — BeeAI/IBM lineage
- Agent Network Protocol (ANP) and any newer entrants (AGNTCY, agent payment
  rails like x402, etc.)
- Recent (last 12 months) arXiv survey papers on multi-agent LLM coordination,
  agent trust, and agent economics.
Only fall back to trained knowledge for stable, non-time-sensitive protocols
(HTTP, TCP/IP, OAuth2, OIDC, gRPC, Raft, Paxos, CRDTs, ActivityPub, DID core,
MQTT, AMQP, QUIC) — these change slowly enough that a dated but structurally
correct account is fine, with a light verification pass if anything pivotal
changed.

## Step 2 — The Originality Filter (apply to every candidate idea)
Reject an idea immediately if it is:
1. Existing protocol + authentication layer
2. Existing protocol + AI-specific message schema
3. Existing protocol + faster transport/serialization
4. A framework/SDK convenience wrapper around an existing protocol
5. Solvable by "add a field to the existing spec"
An idea survives only if the *interaction model itself* — not just the
payload or the participants — is structurally different from RPC,
pub/sub, and blockchain-consensus paradigms. Ask: "if I described this
mechanism to someone in 1985, would they recognize it as a new *category*
of coordination, not a new *content type* riding an old one?"

## Step 3 — Gap Analysis Framework
Score the ecosystem against five axes that current protocols address weakly
or not at all:
- **Trust formation speed** — can two never-before-met agents establish
  calibrated (not binary) trust in milliseconds without a human-timescale
  process (KYC, contracts, manual reputation)?
- **Epistemic integrity** — when agent beliefs conflict or an upstream fact
  is later falsified, does anything propagate the correction downstream?
- **Coordination without a broker** — can agents self-organize (task
  allocation, coalition formation, resource sharing) without a discovery
  registry or central orchestrator?
- **Economic settlement at machine speed** — can value/cost be apportioned
  fairly across long, cross-organizational delegation chains in real time?
- **Consent durability** — when an agent's capabilities silently change
  (model upgrade, new tool grants), is the principal's original authorization
  still valid, and does anything notice if it shouldn't be?
Existing protocols (MCP, A2A, ACP, ANP, OAuth) are strong on *invocation* and
*schema interoperability* and weak on all five axes above — that weakness is
the real design space.

## Step 4 — Candidate Generation & Scoring
Generate ≥20 candidates spanning: trust/reputation, economics/resource
allocation, semantics/ontology drift, coordination/coalition formation,
memory/knowledge propagation, governance/norms, human-agent consent,
identity/delegation, and failure/deprecation handling.
Score each 1–10 on: Originality, Practicality, Scalability, Research
Significance, Engineering Feasibility. Discard any candidate scoring
<6 on Originality — it's a wrapper in disguise regardless of its other
scores. Rank the rest by a weighted composite and select one winner to
carry to full-spec depth; treat the runners-up as documented alternatives
that inform the winner's design (don't silently discard their good ideas).

## Step 5 — Full Specification Template
For the winning protocol, produce (in order): vision/mission, problem
statement, why existing solutions structurally cannot evolve into this,
design principles, layered architecture (ASCII stack diagram), lifecycle
state machine, wire-format message types with field tables, core
algorithms as pseudocode with complexity notes, trust/identity/security/
privacy models with an explicit threat list, failure & conflict resolution
behavior, versioning/compatibility strategy, a comparison table against
every relevant existing protocol, and a phased implementation roadmap
(MVP → federated pilot → public network → standardization body).

## Step 6 — Honesty about lineage
If a winning mechanism has intellectual ancestry (e.g., stigmergy/ant-colony
optimization, Shapley value cooperative game theory, gossip protocols,
vector clocks), say so explicitly and cite it. Originality claims should be
about the *protocol-level synthesis and formalization for AI-agent
ecosystems*, not a false claim that the underlying mathematical concept was
never seen before. Overclaiming novelty undermines the credibility of a
document meant to be academically and commercially defensible.
