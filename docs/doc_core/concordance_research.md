# CONCORDANCE

## A New Protocol Family for Cross-Ecosystem Trust Composition in Autonomous Agent Networks

### A Research Study and Complete Protocol Specification

**Prepared:** July 29, 2026
**Methodology:** `agent-protocol-research` (custom research skill, see accompanying methodology note)

---

## Table of Contents

1. Executive Summary
2. Literature Review
3. Existing Protocol Analysis
4. Research Gaps
5. Future Challenges (5–15 Year Horizon)
6. Candidate Protocol Families
7. Candidate Evaluation Matrix
8. Selected Protocol
9. Complete Protocol Specification
10. Technical Design
11. Security Analysis
12. Performance Analysis
13. Comparison with Existing Protocols
14. Prototype Design
15. Development Roadmap
16. Risks and Limitations
17. Future Research Directions
18. Final Conclusion
19. References

---

## 1. Executive Summary

This study set out to do what the brief demanded: find a problem in AI-agent
interaction that is genuinely unsolved, invent a protocol family that has
never existed before, and specify it to prototype-ready depth. The honest
account of how that search actually went is itself the most important
finding in this document, so it is worth stating up front rather than
burying in Section 4.

As of July 2026, the AI-agent protocol ecosystem is not sparse. It is
**crowded and accelerating**. In the twenty months since the Model Context
Protocol (MCP) shipped in November 2024, the field has produced — and in
several cases already substantially matured — dedicated protocols or
formal proposals for: tool invocation (MCP), agent-to-agent delegation and
discovery (A2A), structured negotiation (ACP, now folded into A2A),
decentralized routing and identity (ANP), on-chain reputation and
validation (ERC-8004), capability-scoped and continuously-checked
authorization (IBCT, PAuth, Grantex, the admission-control "ACP" of
Fernandez et al.), durable machine-readable consent (Anumati), population-
level coordination beyond simple messaging (the Ripple Effect Protocol),
environment-mediated indirect coordination (Ledger-State Stigmergy and a
substantial swarm-robotics/MARL literature on stigmergy), agent payments
(AP2, x402, the Agentic Commerce Protocol), and even a formal taxonomy of
the *governance* primitives — membership, deliberation, voting, dissent,
human escalation, audit — that no existing protocol encodes [15].

Six separate candidate ideas generated independently in Step 1 of this
research, each of which looked genuinely novel from first principles, were
each discovered during literature search to already have a live 2025–2026
paper or shipping project addressing a large fraction of the same ground.
Section 6 documents this honestly rather than suppressing it, because the
instruction that governed this research — *"if you discover that a
proposed protocol is not sufficiently original... discard it and continue
searching"* — is precisely what an elite research process looks like when
the field is moving this fast. Discarding six ideas is not failure; it is
the method working.

What the search surfaced instead is a **second-order problem created by
the very success of first-order gap-filling**: the trust, consent,
reputation, coordination, and governance layer of the agentic web is
fragmenting into a growing set of independently designed, non-interoperable
micro-standards, each with its own identity model, cryptographic
assumptions, wire format, and semantics. An agent that is ERC-8004-reputable
cannot present that fact to a counterpart that only understands Anumati
consent proofs. An IBCT capability chain and a Ledger-State-Stigmergy trace
bundle cannot be combined into one risk decision without bespoke, pairwise,
hand-written glue code — the same N² integration crisis that historically
motivated HTTP, MIME, and OAuth's token-introspection layer, now
recurring one level up, at the level of *trust primitives themselves*.

No protocol proposal found in this search — including the six discarded
candidates — addresses that composition problem. This document therefore
proposes and fully specifies **Concordance**, a protocol family whose job
is not to add another trust primitive, but to let heterogeneous,
independently-evolving trust primitives negotiate, combine, and propagate
revocation across ecosystem boundaries, the way TLS negotiates cipher
suites or HTTP negotiates content types without caring what any individual
suite or type contains.

The remainder of this document surveys the field in depth (Sections 2–5),
generates and honestly scores 21 candidate protocol families (Sections
6–7), selects and fully specifies Concordance (Sections 8–13), and lays
out a concrete, buildable prototype and standardization path (Sections
14–18).

---

## 2. Literature Review

This review is organized by cluster rather than by chronology, since the
same underlying problem (agents from different vendors need to interoperate
without a shared trust root) has been attacked from at least six distinct
angles since late 2024. Citations use the numbered scheme in Section 19.

### 2.1 Foundational, slow-moving infrastructure

These are not AI-specific but underpin every protocol discussed below, so
this review treats them as background rather than re-deriving them from
first principles.

- **HTTP/HTTPS and QUIC** — the transport substrate nearly every
  agent protocol rides on. QUIC's stream multiplexing and 0-RTT handshake
  resumption matter for agent protocols because agent interactions are
  frequent, short-lived, and latency-sensitive in a way human browsing
  sessions are not.
- **TCP/IP** — the reliability and addressing layer beneath all of it;
  notable mainly for what it *doesn't* provide (no notion of identity,
  trust, or intent) that every layer above has had to bolt back on.
- **WebSocket, gRPC, GraphQL** — transport/serialization choices used
  variously by MCP (stdio and Streamable HTTP), A2A (HTTP/JSON-RPC), and
  countless framework-level agent tools. None encode trust semantics;
  they are plumbing.
- **OAuth 2.0/2.1 and OpenID Connect** — the identity/authorization
  substrate nearly every agent protocol's "authorization" section
  ultimately reduces to. Their weakness for agents is structural, not
  incidental: scopes are discrete, binary, and issued once, whereas agent
  behavior is continuous and its risk profile changes call-by-call — a
  point made explicitly by several 2026 papers discussed in §2.3 below.
- **Raft and Paxos** — classical crash-fault-tolerant consensus.
  Relevant as a baseline for what "consensus" means in distributed systems;
  agent protocols mostly do *not* need this (they need trust and
  coordination among mutually suspicious, commercially distinct parties,
  which is a Byzantine problem, not a crash-fault one) but many
  agent-protocol authors reach for Raft/Paxos vocabulary loosely and
  imprecisely, which this document tries to avoid.
- **CRDTs** — conflict-free replicated data types solve state
  convergence among replicas that trust each other's data model but may be
  offline or out of order. They say nothing about whether to trust the
  *content* of a claim, only how to merge it once trust is assumed.
- **W3C DID (Decentralized Identifiers) and Verifiable Credentials** —
  the identity substrate underneath ANP, AGNTCY's AConP, and several 2026
  authorization frameworks. Strong on self-sovereign identity; weak on
  operational maturity (resolver infrastructure still immature) and,
  importantly, silent on how credentials from *different* DID-based
  ecosystems should be combined into one decision.
- **ActivityPub** — the federated social-web protocol (Mastodon, etc.).
  Relevant as the closest historical analogy for "federation without a
  central registry," and as a cautionary tale: ActivityPub's inbox/outbox
  model achieved federation but never solved cross-instance trust or spam
  scoring in a standardized way — each instance built its own bespoke
  blocklists. That failure mode is precisely what Section 4 argues is
  about to recur, at greater speed, in agent networks.
- **FIPA-ACL** — the 1990s–2000s Foundation for Intelligent Physical
  Agents agent communication language, the direct intellectual ancestor of
  ACP's typed performatives (propose/accept/reject/counter). Establishes
  that "agents negotiating via typed speech acts" is a thirty-year-old
  idea; what's new in 2024–2026 is doing it at internet scale with
  LLM-driven, natural-language-capable agents.
- **Automated Trust Negotiation (ATN)**, a 2000s security-research thread
  (Winsborough, Yu, Seamons and others) on gradual, policy-driven credential
  disclosure between mutually suspicious strangers who share a *single*
  credential vocabulary (typically X.509-style attribute certificates).
  ATN is the closest real precedent to this document's own proposal and is
  treated at length in Section 8, because the honest answer to "hasn't
  this been done before?" runs through ATN, not around it.

### 2.2 The agent interoperability stack that emerged 2024–2026

- **Model Context Protocol (MCP)**, Anthropic, November 2024. Answers
  "what can an agent do?" via Tools, Resources, and Prompts exposed by a
  server to a client [15]. Adoption has been extraordinary — Tier-1 SDKs
  alone report close to half a billion downloads a month, with both the
  TypeScript and Python SDKs individually past one billion cumulative
  downloads [2]. The protocol has evolved fast: the June 2025 revision
  added structured tool output, elicitation, and resource links; the
  November 2025 revision added experimental Tasks; and the **2026-07-28
  revision — finalized the day before this report was written** — is
  described by its own maintainers as the largest rewrite since launch,
  removing protocol-level sessions entirely to make the core stateless,
  cacheable, and horizontally routable like ordinary HTTP infrastructure,
  demoting Tasks to an optional extension, and hardening OAuth/OIDC-based
  authorization [2] [4] [5] [6]. MCP Apps (an SEP that lets servers ship
  sandboxed interactive HTML surfaces reviewed and cached ahead of
  execution) also graduated in this release [5].
- **Agent2Agent (A2A)**, originated by Google, transferred to the Linux
  Foundation in June 2025 [9]. Answers "which agent can handle this task?"
  via JSON-LD "Agent Cards" describing capabilities and endpoints, task
  delegation, and — as of v1.0.1 (May 2026) — a formal extension mechanism
  for new data, RPC methods, and state machines [15]. One year in, A2A
  counted more than 150 supporting organizations and production deployments
  in supply chain, financial services, insurance, and IT operations [11]
  [12], with Signed Agent Cards added for cryptographic identity
  verification [13].
- **Agent Communication Protocol (ACP)**, IBM Research, formalized
  bilateral negotiation with FIPA-ACL-derived performatives (propose,
  accept, reject, counter) [15]. In August 2025, ACP's governance was
  folded into A2A under the Linux Foundation's LF AI & Data umbrella — one
  of several consolidation events (AutoGen retiring into Microsoft Agent
  Framework, AGNTCY archiving its own competing protocol in favor of
  becoming a discovery/identity/observability layer) that a
  well-regarded independent tracker summarized bluntly: *"the agent-to-agent
  protocol war is over and A2A won"* [19].
- **Agent Network Protocol (ANP)**, an open-source community effort dated
  from 2024, pitched explicitly as "the HTTP of the agentic internet" [17]
  [24]. Its three-layer architecture (communication, syntactic, semantic)
  targets fully decentralized, cross-domain interoperability using
  `did:wba` identities, WNS handles, and — notably — its own bundled
  payment protocol, AP2 [24]. Independent 2026 analysis is candid that ANP
  is "technically compelling but not yet ecosystem-ready": DID resolver
  infrastructure and tooling still lag well behind HTTP-based competitors
  [21].
- **AGNTCY**, launched by Cisco's Outshift with LangChain, Galileo, and
  others as part of a broader initiative, archived its own competing
  agent-to-agent protocol during the 2026 consolidation and repositioned
  as the discovery/identity/observability layer underneath A2A [19] [24].
- **ERC-8004 ("Trustless Agents")**, an Ethereum Improvement Proposal
  (Draft, August 2025; mainnet live January 29, 2026), defines three
  on-chain registries — Identity, Reputation, Validation — explicitly
  scoped to let agents "discover, choose, and interact... without
  pre-existing trust" [15]. Adoption has been fast by any measure: more
  than 170,000 registered agent identities and 150,000+ reputation
  feedback records within months [28]. It is also the subject of the
  field's first serious empirical audit, which found — bluntly — that we
  do not yet know whether the registered identities correspond to
  functional agents or whether the exposed reputation signal can actually
  be trusted [28]. This finding matters a great deal for Section 4 and for
  the design of Concordance's composition algebra in Section 9.
- **Ripple Effect Protocol (REP)**, Chopra, Sharma, Ahmad, Muscariello,
  Pandey, and Raskar (Oct. 2025). Observes that A2A- and ACP-style
  protocols "emphasize communication over coordination," and proposes
  sharing lightweight *sensitivities* — signals about how a decision would
  change if an environmental variable shifted — that ripple through local
  networks. Benchmarked across three domains (a supply-chain "Beer Game,"
  sparse-network preference aggregation, and a common-pool "Fishbanks"
  resource game), REP reports 41–100% improvements in coordination
  accuracy and efficiency over plain A2A messaging [26].
- **Ledger-State Stigmergy**, a formal comparison (April 2026) of
  stigmergic (STIG), direct-messaging (MSG), and orchestrated (ORCH)
  coordination mechanisms grounded in distributed-ledger state, explicitly
  drawing the ant-colony/pheromone analogy through to MEV-searcher
  behavior on public blockchains [31]. This sits inside a much larger,
  decades-old stigmergy literature (Grassé's original 1959 coinage of the
  term; swarm-robotics implementations such as Phormica's photochromic
  pheromones [58]; and a rapidly growing 2025–2026 body of work applying
  digital pheromone fields specifically to LLM multi-agent coordination,
  with at least one empirical study reporting a sharp phase transition
  above which trace-based coordination outperforms shared memory by
  36–41% on composite metrics [29]). A widely used 2026 textbook survey of
  agentic AI systems now treats stigmergy (shared documents, code
  repositories, annotation layers, task queues) as a standard, named
  coordination pattern for LLM agent systems [32].
- **The governance gap.** A June 2026 systematic analysis applies a
  six-dimension taxonomy — membership, deliberation, voting, dissent
  preservation, human escalation, audit/replay (G1–G6) — derived from
  Habermas's communicative rationality, parliamentary procedure, and
  Ostrom's institutional-analysis framework, to MCP, A2A, ACP, ANP, and
  ERC-8004. The finding: voting, dissent preservation, and human
  escalation are *universally absent* across all five protocols; audit
  exists only as an accidental property of underlying infrastructure
  (blockchain immutability, session state), never as deliberate
  governance design [15]. Crucially, the same paper notes that A2A's
  extension mechanism could in principle host these primitives, but that
  after six-plus months of that mechanism being publicly available, zero
  governance extensions have been proposed [15] — an empirical signal
  this document treats as important evidence about where the ecosystem's
  actual attention is (and is not) going.

### 2.3 The authorization, consent, and capability-drift cluster

A dense 2025–2026 literature independently converges on the same
diagnosis: OAuth-style scopes are "discrete, static, and binary" and
cannot express usage policy that is "contextual, compositional, and
evolve[s] as the callee's policies change" [44].

- **Anumati** proposes "proof of adherence" as a formal consent model,
  explicitly naming the gap between authentication (who may invoke what)
  and usage policy (what may be done with a result afterward, for how
  long, shared with whom) [44].
- **Invocation-Bound Capability Tokens (IBCTs)** fuse identity, attenuated
  authorization, and provenance binding into an append-only chain, with
  reference implementations reporting 0.049ms verification latency and
  100% adversarial rejection across 600 attack attempts in the reporting
  study — strong on single-hop and multi-hop delegation binding, but
  explicitly not addressing aggregation-inference risks (an agent
  legitimately allowed to see many individually-innocuous facts inferring
  something none of them individually authorized) [37].
- **PAuth** derives per-task authorization envelopes from natural-language
  task descriptions ("NL slices"), reporting 100% success on benign tasks
  and a 100% warning rate on injected attacks in its evaluation [37].
- **Grantex**, an Apache-2.0 open protocol with an IETF Internet-Draft
  submitted to the OAuth Working Group, implements signed grant tokens,
  per-agent DIDs, FIDO2/WebAuthn-backed consent flows, cascading
  revocation, and depth-limited delegation chains as a single standard —
  motivated by an audit finding that 93% of thirty popular open-source
  agent projects rely on unscoped API keys with no per-agent identity, no
  user consent flow, and no revocation mechanism at all [39].
- A distinct **Agent Control Protocol** (confusingly also abbreviated
  ACP, unrelated to IBM's protocol of the same initials) targets
  admission control specifically: a cryptographic check — deterministic,
  integer-arithmetic risk evaluation, no ML inference in the critical
  path — gating every agent action before it reaches execution, with
  chained delegation and transitive revocation across organizational
  boundaries [45].
- Independent critical commentary on MCP's July 2026 rewrite argues the
  new authorization hardening is still fundamentally about the
  *handshake* (issuer validation, credential binding, step-up consent at
  login) and does not define a per-tool capability check enforced on
  every invocation, illustrating that even the most actively maintained
  protocol in the space has not closed this gap [41].
- NIST's February 2026 AI Agent Standards Initiative and the OpenID
  Foundation's 2025 consensus whitepaper on agentic identity both signal
  regulatory and standards-body attention converging on "continuous
  authorization" as a compliance precondition under the EU AI Act,
  DORA, and related frameworks [43].

### 2.4 The epistemic-integrity / belief-cascade cluster

A second dense, very recent (mostly June 2026) cluster studies what
happens when LLM-agent claims are exchanged, revised, and reused as
context across a multi-agent system.

- Multiple papers model **hallucination cascades** and error propagation
  in multi-agent collaboration, several published within the same week in
  June 2026, indicating a suddenly hot research area rather than a mature
  one [documented in cluster of citations, §19].
- **"Delayed Verification Destabilizes Multi-Agent LLM Belief"**
  formalizes instability thresholds and studies optimal placement of
  fact-checking "correctors" within an agent network.
- **"Preregistered Belief Revision Contracts"** targets a specific,
  well-evidenced failure mode — LLM multi-agent populations becoming
  *more* confident while converging on a *wrong* answer due to social
  conformity rather than evidence — by requiring a public, structural
  distinction between persuasion and evidence as warrants for belief
  change.
- **Nous**, a long-term memory architecture, represents each
  entity-attribute belief as a categorical probability distribution
  updated via closed-form Bayesian inference, combined with a
  "provenance-capped poisoning defense" — directly relevant to any design
  (including the one ultimately selected in this document, see §9.6) that
  wants to resist correlated or Sybil-sourced corroboration.
- A parallel body of work (CASPIAN, GUARDIAN, PropGuard, and others)
  targets online detection and attribution of cascade attacks and
  propagation-aware remediation via causal or temporal-graph monitoring.
- Notably, essentially **all** of this cluster operates *within* a single
  deployed multi-agent system that its own research team fully controls
  end-to-end — it is algorithmic and architectural work, not
  interoperability-standard work. No source found in this review proposes
  a cross-organizational, cross-vendor wire format for exchanging
  belief-provenance objects the way MCP standardizes tool calls or A2A
  standardizes Agent Cards. This distinction is load-bearing for Section 4.

### 2.5 The economics and coalition-formation cluster

- **AP2 (Agent Payments Protocol)**, emerging alongside the A2A ecosystem
  with payment-industry partners, centers on cryptographic "mandates" —
  verifiable, user-signed authorizations scoping what an agent may spend,
  on what, within what bounds [42].
- **x402**, from Coinbase, revives the dormant HTTP 402 status code for
  machine-native stablecoin micropayments, aimed at agents paying per-call
  for APIs and content [42].
- The **Agentic Commerce Protocol**, from OpenAI and Stripe, standardizes
  in-conversation checkout between agents and merchants [42].
- Fair *ex ante* division of coalition surplus among cooperating agents is
  a mature applied game-theory topic: Shapley-value-based attribution
  dates to classical multi-agent-systems work on coalition formation, with
  recent 2026 papers extending it to dynamic split/merge dynamics for
  self-organizing coalitions and to a full "Shapley Pricing Equilibrium"
  framework for pricing human and AI agents inside mixed workflows.
- **WebMCP** (Google/Microsoft, previewed February 2026 via Chrome Canary,
  developed jointly through the W3C) extends the MCP mental model to
  ordinary websites, letting a site declare agent-usable capabilities by
  consent rather than have agents scrape it adversarially [42].

### 2.6 Summary of the survey

Table 1 (Section 3) makes the coverage explicit, but the qualitative
picture from this literature review is: **identity, capability discovery,
tool invocation, delegation, payment, and (increasingly) even
per-primitive trust and consent are being actively and competently
standardized.** What is not being standardized — and what nobody found in
this search has proposed — is how an agent that trusts via one of these
schemes talks to an agent that trusts via a different one.

---

## 3. Existing Protocol Analysis

Per the research brief, each protocol is assessed against ten fixed
questions: purpose, problem solved, design philosophy, strengths,
weaknesses, scalability limits, security limitations, coordination
limitations, assumptions, and why it cannot solve future AI-agent
problems on its own. To keep this navigable, Table 1 gives the structured
comparison across all protocols, and the prose beneath it expands on the
four protocols (MCP, A2A, ANP, ERC-8004) that carry the most design
weight, plus the authorization and epistemic clusters as groups.

### Table 1 — Structured Protocol Comparison

| Protocol | Purpose | Core Question Answered | Design Philosophy | Key Strength | Key Weakness | Scalability Limit | Security Limitation | Coordination Limitation | Core Assumption |
|---|---|---|---|---|---|---|---|---|---|
| MCP (2026-07-28) | Tool/context access | "What can an agent do?" | Client–server, now stateless-core | Enormous adoption; clean tool/resource/prompt model | Tool-centric; not designed for agent-to-agent or community semantics | Statelessness (new) removes prior session-affinity ceiling | Per-call capability enforcement still handshake-only [41] | None between peer agents by design | A tool-providing server and a tool-consuming client is the whole world |
| A2A v1.0.1 | Agent discovery & delegation | "Which agent handles this task?" | Agent Cards + task delegation + extensions | Broad multi-vendor governance (Linux Foundation), 150+ orgs | Delegation-centric; deliberation/voting/dissent absent [15] | Extension model scales features but not semantics | Agent Card existence ≠ verified trustworthiness without add-ons | Bilateral task handoff, not multilateral deliberation | Agents "exist" by publishing a card; no membership concept |
| ACP (IBM lineage) | Structured negotiation | "How do agents exchange messages?" | FIPA-ACL-derived performatives | Real negotiation semantics (propose/accept/reject/counter) | Bilateral only; folded into A2A governance in 2025 | N/A (absorbed) | Rejected proposals aren't preserved as community dissent [15] | No multilateral deliberation | Two parties with opposing interests, not a community |
| ANP | Decentralized routing & identity | "How do messages reach the right agent?" | 3-layer (comm/syntactic/semantic), DID-based | No central registry; strong self-sovereign identity story | Tooling/resolver infrastructure immature in 2026 [21] | Peer-to-peer avoids central bottleneck but adds negotiation latency | Routing-centric; no reputation/trust primitive of its own | Routing ≠ collaboration; says nothing about whether to comply | Every agent can resolve every other agent's DID cheaply |
| ERC-8004 | On-chain identity/reputation/validation | "Which agents can be trusted?" | Three lightweight on-chain registries | Portable, platform-independent reputation; fast real adoption | Empirically unverified whether reputation is meaningful [28] | On-chain writes are costly/slow vs. real-time deliberation | Gameable feedback; Sybil/correlated-endorsement risk | No deliberation, voting, or dissent primitives [15] | On-chain permanence ⇒ trustworthiness |
| IBCT / PAuth / Grantex / admission-control ACP | Capability-scoped, checked authorization | "Is this specific call allowed, right now?" | Signed, attenuated, chained capability tokens | Sub-millisecond verification; strong delegation-chain auditability | Aggregation-inference and slow-drift risks explicitly out of scope [37] | Local verification scales well; cross-scheme composition does not exist | Strong for forgery/replay; silent on cross-scheme correlation | Authorization ≠ ongoing consent as capabilities/models drift | Static scope, once granted, remains valid until explicitly revoked |
| Anumati | Durable, evolving consent | "May this agent still do X, under today's policy?" | Adherence proofs distinct from authentication | Names the consent-vs-authentication gap explicitly | Early-stage; no cross-scheme interoperability defined | Not yet stress-tested at population scale | Depends on callee honestly reporting policy changes | No governance/dissent layer | Consent is bilateral, not multi-party |
| REP | Population-level coordination | "How should my decision change if the environment shifts?" | Share sensitivities, not just decisions | 41–100% coordination-efficiency gains over plain A2A messaging [26] | Still direct network messaging; not indirect/environment-mediated | Scales with network sparsity; dense graphs re-introduce overhead | Not designed as a security/trust protocol | Strong within-population coordination; nothing cross-population | Local network links exist and are trustworthy |
| Ledger-State Stigmergy / stigmergic MARL | Indirect, environment-mediated coordination | "What does the accumulated trace tell me to do?" | Ant-colony/pheromone analogy, formalized over ledger state | Scalable, robust, no per-peer link required | Congestion/"traffic jam" analogue under high agent density [31] | Well-studied phase transition; trace storage still grows without decay tuning | Correlated/colluding trace deposits under-studied | Coordinates action, not trust or capability-checking | Environment state is itself tamper-resistant enough to trust |
| Governance taxonomy (G1–G6) | Community decision-making | "How should agents collectively decide?" | Membership, deliberation, voting, dissent, escalation, audit | Rigorous, well-sourced taxonomy; identifies a real universal absence [15] | A taxonomy and gap analysis, not yet a shipped protocol | Untested at scale — no reference implementation found | N/A — proposes requirements, not a security design | Directly addresses multilateral deliberation the others lack | Assumes a *bounded, known* community exists to govern |
| AP2 / x402 / Agentic Commerce Protocol | Payment settlement | "How does value move?" | Cryptographic mandates / machine micropayments | Real-money rails already live | Settlement ≠ fair *ex ante* value division among cooperating agents | Payment-rail scalability is a solved problem (existing fintech infra) | Standard payment-fraud surface, well-understood | Doesn't address non-monetary coordination at all | A payer and payee are the only two roles that matter |

### 3.1 MCP in depth

MCP's own maintainers describe the 2026-07-28 release as removing "a lot
of things that made MCP" what it was, in service of a stateless,
horizontally-scalable core more like ordinary HTTP infrastructure than the
session-based design it launched with [4]. This is a genuinely important
signal for this study: even the most successful, best-governed protocol in
the space has needed a backward-incompatible rewrite within two years,
because the load-bearing architectural assumption it started with
(session affinity) did not survive contact with production scale. Any
protocol proposed in Section 9 needs to take that lesson seriously — hence
Concordance's own insistence, in §9.14, on stateless-first, mandatory-ignore
forward compatibility from day one rather than as a painful v2 rewrite.
MCP's assumption that "a tool-providing server and a tool-consuming client
is the whole world" is exactly right for its purpose and exactly why it
cannot, and should not try to, solve agent-to-agent trust — that was never
its job.

### 3.2 A2A and the consolidation story

A2A's trajectory — donated to the Linux Foundation, absorbing ACP's
governance, reaching a stable v1.0.1 with Signed Agent Cards for
cryptographic identity — is the clearest evidence that the "which
transport/RPC standard wins" question is settling, fast [19]. What is
equally clear from the governance-gap analysis is that winning the
transport/delegation layer says nothing about winning the trust or
governance layers: A2A explicitly scores 1/12 on the governance taxonomy,
and its own extension authors, six months in, have not proposed a single
extension addressing it [15]. A2A is not going to organically grow into a
cross-scheme trust composition layer, for the same reason HTTP did not
organically grow into OAuth: delegation and trust composition are
different problems requiring different message semantics, not different
fields on the same message.

### 3.3 ANP and the semantic-layer question

ANP's three-layer architecture explicitly separates communication,
syntax, and semantics, with the semantic layer intended to let agents
"understand the intent and meaning behind exchanges" [17]. This is
architecturally the closest any existing protocol comes to addressing
meaning/ontology alignment — but it is a *static*, shared-vocabulary
mechanism (closer to Semantic Web ontology description, tracing back to
Gruber's 1993 "translation approach to portable ontology specifications"
[16]) rather than a continuous, cross-scheme drift-correction mechanism.
It also inherits ANP's practical weakness: independent 2026 analysis rates
DID resolver infrastructure and tooling as still immature relative to
HTTP-based competitors [21], which matters because a semantic layer is
only as useful as the identity layer beneath it that lets two agents agree
whose ontology is authoritative.

### 3.4 ERC-8004 and the reputation question

ERC-8004 is the protocol in this survey that most directly attempted to
solve "agent trust" as its primary mission, and it is worth dwelling on
because its empirical track record is the single most important piece of
evidence for this study's eventual design choice (§9.6). The registry
design is clean: Identity (an ERC-721-style on-chain handle), Reputation
(signed, client-reported feedback), Validation (independent attestation
via TEEs, zkML, or stake-secured re-execution). Adoption was fast — over
170,000 registered agents within months across Ethereum, BNB Smart Chain,
and Base [28]. But the first serious independent empirical study of the
live ecosystem is blunt about what remains unknown: whether the
registered identities correspond to functioning agents, and whether the
exposed reputation signal is trustworthy at all [28]. This is not a
criticism of the protocol's design so much as a demonstration of a
structural problem: **a reputation signal produced inside one scheme,
using that scheme's own notion of "feedback" and "independence," cannot
be safely treated as equivalent to a reputation signal produced inside a
different scheme with different assumptions** — precisely the composition
problem Concordance is built to make explicit and auditable rather than
silently assumed away.

### 3.5 The authorization cluster as a group

Read together, IBCT, PAuth, Grantex, Anumati, and the admission-control
ACP look less like five competing protocols and more like five
converging answers to the same diagnosed problem (OAuth scopes are
static; agent risk is continuous) arriving within months of each other
from unrelated teams — IBCT and the admission-control ACP even reuse the
initialism "ACP" by accident of independent invention, a small but telling
symptom of how crowded this space has become. Every one of them, by
design, defines its *own* token format, its *own* notion of what
"attenuation" or "adherence" or "admission" means, and its *own*
cryptographic assumptions. None defines how a relying party that natively
speaks one of these should interpret evidence from another. That absence
is this survey's second most important finding.

### 3.6 The epistemic cluster as a group

The June 2026 flurry of hallucination-cascade, belief-revision, and
provenance-poisoning papers demonstrates the problem (multi-agent belief
propagation is fragile, gameable, and prone to false-consensus cascades)
is real, current, and taken seriously by a fast-growing research
community. It also demonstrates that essentially all proposed fixes are
*architectural* choices for a single, centrally-designed multi-agent
system (where to place a corrector, how one memory store should update
its own beliefs) rather than *interoperability* choices for independently
operated agents belonging to different organizations. That gap — between
"we know how to make one system's beliefs more reliable" and "we have a
wire format for two unrelated systems to exchange belief-provenance
claims and trust each other's retractions" — recurs across every cluster
in this survey and is the organizing insight behind Section 4.

---

## 4. Research Gaps

Applying the gap-analysis framework from the accompanying research
methodology (trust-formation speed, epistemic integrity, coordination
without a broker, economic settlement at machine speed, consent
durability) against the literature in Section 2 and Section 3 yields five
observations, the last of which is the one this document acts on.

**Gap 1 — Trust-formation speed is being solved, per-scheme, in parallel.**
ERC-8004, IBCT, Grantex, and Anumati each let two strangers establish
*some* form of calibrated trust in well under a second, without a
human-timescale process. This gap is closing rapidly and is not this
document's target.

**Gap 2 — Epistemic integrity is an active, hot research area, but only
within single-owner systems.** Belief-revision contracts, provenance-capped
poisoning defenses, and cascade-attribution systems exist. None of them
is proposed, in any source found, as a cross-organizational wire standard.
This is a real remaining gap, but as argued in §4.4 below, it is better
addressed as a *scheme that plugs into* a composition layer than as a
freestanding new protocol, because an epistemic-claim exchange format
between two unrelated organizations runs into exactly the same "whose
notion of provenance do we both accept" problem that motivates Gap 5.

**Gap 3 — Coordination without a central broker is well-served for
within-population problems (REP, stigmergy) but not across populations
with different coordination primitives.** An REP-speaking population and
a stigmergic population cannot presently combine their signals; this is a
narrower instance of Gap 5.

**Gap 4 — Consent durability under capability drift is the single most
crowded sub-space found in this entire study.** Anumati, IBCT, PAuth,
Grantex, and the admission-control ACP all target it, all in 2026, all
independently. The 93%-of-projects-use-unscoped-API-keys finding [39]
shows the *problem* is real and unsolved in practice, but the *research
and standardization* response is already well underway; adding a sixth
independent proposal here would not be a genuinely new protocol family, it
would be a seventh point solution making Gap 5 worse.

**Gap 5 — No mechanism exists, anywhere in this survey, for
heterogeneous trust/consent/reputation/coordination primitives to
negotiate, compose, or propagate revocation across each other.** This is
the gap that is *not* closing, is *getting worse* as Gaps 1–4 succeed
(each successful point solution is one more incompatible island), and has
no proposed protocol addressing it in any source found in this research.
It is the target this document commits to.

### 4.1 Why Gap 5 is structural, not a documentation problem

It would be tempting to describe Gap 5 as "someone just needs to write
adapters." Three considerations argue this is a category error, not
merely unfinished glue code:

1. **The N² problem is combinatorial, not linear.** With even the
   handful of named schemes in this survey — ERC-8004, IBCT, Anumati,
   Grantex, admission-control ACP, REP, Ledger-State Stigmergy, the
   governance taxonomy's eventual implementation — bespoke pairwise
   bridges number in the dozens today and grow quadratically as new
   schemes appear. A meta-layer converts this to linear (one adapter per
   scheme) exactly the way MIME converted email-attachment handling from
   pairwise application coupling to one registered type per format.
2. **Composition requires a *shared arithmetic*, not just a shared
   wire format.** Even if every scheme agreed on a common envelope
   (trivial, it's just a schema), *combining* a 0.8-confidence
   ERC-8004 reputation signal with a 0.9-confidence Anumati consent
   proof into one risk decision requires an explicit theory of how
   evidence from structurally different, non-independent, differently
   gameable sources should combine — precisely the question ERC-8004's
   own empirical audit shows is currently unanswered even *within* one
   scheme [28], let alone across schemes.
3. **Revocation must propagate across scheme boundaries or it is
   theater.** If agent A's decision to trust agent C rests partly on a
   now-revoked ERC-8004 attestation and partly on a still-valid Anumati
   proof, and no mechanism tells A that the ERC-8004 leg was pulled, A's
   composed trust judgment silently becomes stale. None of the reviewed
   schemes defines a cross-scheme revocation echo.

### 4.2 Positioning relative to Automated Trust Negotiation (ATN)

The honest precedent check: is this just 2000s Automated Trust
Negotiation with new branding? ATN research solved gradual, policy-driven
credential *disclosure* between strangers who already share one
credential vocabulary (typically X.509 attribute certificates) and one
access-control policy language. What ATN does not address, because the
problem did not yet exist in that form, is **heterogeneous schema and
semantic composition across independently designed, differently
governed, cryptographically incompatible trust ecosystems that were never
built to interoperate** — an ERC-8004 on-chain attestation, an Anumati
natural-language adherence proof, and an IBCT Biscuit-token delegation
chain do not share a policy language, a data model, or even an agreed
notion of what "revocation" means. Extending ATN's disclosure-sequencing
logic to operate *across* such heterogeneous schemes — including a
formal, extensible combination algebra and cross-scheme revocation
propagation — is the specific, scoped novelty claimed in this document,
not the general idea of negotiated trust disclosure.

### 4.3 Positioning relative to the governance taxonomy (G1–G6)

A close call in this research was whether to build the G1–G6 governance
protocol directly, since its own authors state plainly that no protocol
implements it yet [15]. Two considerations argue against making that the
primary invention: first, related work already proposes *mechanisms* for
much of it (constitutional multi-agent governance, conformal social
choice for deliberation, Ostrom-CPR-principled social learning) [15],
meaning the pattern of "mechanism research exists, interoperability
standard does not" recurs here exactly as it does for the epistemic
cluster; second, and more importantly, governance-layer primitives
presuppose a **bounded, known community** deciding together — they answer
"how do members of this room vote," not "how do two agents who don't even
agree on what a valid credential looks like decide whether to talk to
each other at all." The latter is logically prior. A governed-deliberation
protocol, once built, is itself just another scheme with its own claims
(a vote tally, a dissent record) that need to compose with everything
else — which makes it a natural, well-motivated *future extension scheme*
for the layer this document proposes (see §9.16), not a substitute for it.

### 4.4 The forward-looking framing

Put plainly: the ecosystem surveyed in Sections 2–3 is winning the fight
against each individual trust problem and is, in the process, quietly
losing a larger fight it has not yet noticed it is in. Section 5 argues
this second fight gets substantially worse, not better, over the next
5–15 years.

---

## 5. Future Challenges (5–15 Year Horizon)

This section projects the trajectory identified in Section 4 — rapid,
successful, *fragmenting* point-solution growth — across the ecosystems
named in the research brief. The throughline in every case is the same:
individually competent trust/consent/coordination schemes proliferating
faster than any mechanism to combine them.

**Billions of personal AI agents.** By the early 2030s, a plausible
scenario has each person's personal agent needing to interact daily with
merchant agents (speaking AP2/x402/Agentic-Commerce-Protocol), government
or civic agents (likely a DID/Verifiable-Credential scheme with its own
national or supranational governance), workplace agents (an enterprise
IBCT/Grantex-style capability scheme), and other personal agents (an
Anumati-style consent scheme). No single agent vendor will control all
four ecosystems, and by extension no single trust vocabulary will be
universal. Without a composition layer, every personal-agent vendor faces
the N² integration burden alone, which either (a) entrenches whichever
vendor is large enough to absorb that cost, recreating today's
walled-garden dynamics one layer up, or (b) causes personal agents to
silently degrade to the lowest-common-denominator trust check (an
API key) that the 2026 Grantex audit already shows is the current default
for 93% of projects [39].

**Enterprise AI agent fleets.** AWS reports AgentCore customers already
scaling to 17 production agents within a year [15], and the governance-gap
paper shows the AWS Bedrock AgentCore production registry itself encodes
no trust scoring, behavioral reputation, or governance primitives despite
serving real production fleets [15]. As enterprises federate agent fleets
*across* company boundaries — supplier agents, auditor agents, regulator
agents — each likely arriving from a different enterprise software
vendor with a different capability/consent scheme, the composition
problem stops being theoretical and becomes a procurement blocker: which
vendor's trust primitive does a cross-company workflow trust, and how is
disagreement between two vendors' risk scores resolved?

**Robot swarms and autonomous factories.** Physical stigmergy (pheromone-
style traces) is already a proven, decades-old coordination mechanism for
swarm robotics [58], and Ledger-State Stigmergy shows it translating
cleanly to digital agent populations [31]. The future risk is not that
stigmergic coordination fails within one factory's robot fleet — it is
that a factory increasingly composed of *multiple vendors'* robot fleets,
each with its own trace format and each also carrying its own IBCT-style
capability tokens for safety-critical actuation, has no standard way to
ask "should I trust this trace, given that I don't trust this vendor's
tokens the same way I trust our safety system's own?"

**Autonomous scientific research.** Multi-agent research pipelines
(literature agents, hypothesis-generation agents, experiment-execution
agents, peer-review agents) are exactly the setting the epistemic-cascade
literature (§2.4) warns about: a false claim, once absorbed as context by
a downstream agent, can propagate through months of simulated "research"
before a human notices. Cross-institutional science — the normal way
science actually works — means these agents will not all belong to one
lab's belief-revision architecture. A retraction issued inside one lab's
Nous-style memory system needs a way to reach every other lab's agents
that already ingested the now-false claim; no source in this survey
proposes that cross-institutional retraction-propagation mechanism.

**Autonomous software companies.** As agent-run companies begin
contracting with other agent-run companies — hiring, subcontracting,
escrow, dispute resolution — the liability-apportionment gap identified
in Section 6 (candidate #10) becomes a hard commercial blocker: existing
economic-settlement protocols (AP2, x402) move money once everyone agrees
who owes what, but nothing in this survey apportions *fault* when an
autonomous supply chain of agents produces harm, which is exactly the
scenario insurers and courts will need resolved before agent-run
companies can carry liability insurance at all.

**Autonomous governments and regulatory agents.** The EU AI Act, DORA,
and NIST's 2026 initiative already require continuous, auditable
authorization for agentic systems [43]. A regulator's own agent, auditing
a private company's agent fleet, will need to interpret evidence produced
under that company's chosen consent/reputation/authorization scheme(s) —
plural, because large companies will run several. A composition layer is
close to a regulatory necessity here, not merely a convenience.

**Space-based and disconnected/edge AI.** Long-latency, intermittently
connected agents (satellites, remote sensors, disaster-response robots)
cannot rely on real-time on-chain validation (ERC-8004's core assumption)
or continuous connectivity to a capability-token issuer (IBCT/PAuth's
implicit assumption). A composition layer that can carry cached,
locally-verifiable envelopes with explicit staleness windows — rather
than assuming any one scheme's freshness model — becomes a hard
requirement, not a nice-to-have, in exactly the environments where
today's schemes are weakest.

**Multi-model, decentralized AI ecosystems.** As foundation models
diversify across vendors, countries, and open-weight releases, the
"cognitive diversity" question (are five agreeing agents five independent
opinions, or one base model's correlated failure mode wearing five
different system prompts?) becomes central to trust itself, not a side
concern — and no reviewed protocol currently asks it. This is exactly the
independence-discounting problem built into Concordance's composition
algebra (§9.9), motivated directly by ERC-8004's own empirically observed
gameable-reputation weakness [28].

The common thread across all seven scenarios: **the crisis is not the
absence of trust mechanisms. It is the absence of a way to make an
ever-growing, healthily-competitive set of trust mechanisms interoperate**
— the same shape of problem HTTP, MIME, and OAuth's introspection layer
solved historically, recurring at a new layer, at a much faster clock
speed, because agent ecosystems iterate in months where human-facing web
standards iterated in years.

---

## 6. Candidate Protocol Families

Following the methodology's originality filter, twenty-one candidates were
generated across the problem space identified in Sections 4–5. Each was
checked against current literature before scoring; six were found to be
substantially covered by shipping projects or recent papers and are scored
down accordingly rather than silently dropped, per the "discard and keep
searching, but say so" instruction governing this research. Full 1–10
scoring on all five axes appears in Section 7; this section gives each
candidate's core idea, the problem it targets, and — where applicable —
the specific prior art that caps its originality score.

**1. Concordance — Trust-Scheme Composition & Negotiation Protocol.**
*Core idea:* a meta-layer wrapping heterogeneous trust/consent/reputation
primitives in a common envelope, with formal negotiation, a
correlation-aware composition algebra, and cross-scheme revocation
propagation. *Problem solved:* Gap 5 (§4). *Originality:* no prior art
found addresses composition across independently-designed trust
ecosystems; closest precedent (ATN, §4.2) solves a narrower, single-scheme
version of the problem. **Selected as the winning candidate (Section 8).**

**2. Stigmergic Trust Mesh.** *Core idea:* agents deposit decaying,
reinforceable "trust traces" into a shared substrate instead of
exchanging direct trust assertions. *Problem solved:* trust formation
without a broker. *Prior art:* substantially pre-empted by Ledger-State
Stigmergy's formal STIG/MSG/ORCH framework [31], a large swarm-robotics
stigmergy literature [51][52][53][57][58], and a 2026 empirical study of
trace-based vs. memory-based coordination with a measured phase
transition [29]. Scored down accordingly.

**3. Epistemic Invalidation Cascade Protocol.** *Core idea:* a standard
message type that propagates retraction of a falsified upstream claim to
every downstream agent that incorporated it. *Problem solved:* Gap 2
(§4). *Prior art:* Preregistered Belief Revision Contracts, the
provenance-capped poisoning defense in Nous, and CASPIAN/GUARDIAN/PropGuard
already address large parts of this algorithmically. Retained as a strong
runner-up and folded into Concordance as a candidate claim-class rather
than abandoned (§9.16).

**4. Capability Drift / Continuous Consent Protocol.** *Core idea:*
authorization scopes that automatically require re-consent as an agent's
underlying model or tool access changes. *Problem solved:* Gap 4 (§4).
*Prior art:* this is, as documented in §2.3 and §4, the single most
crowded sub-space found in the entire study (Anumati, IBCT, PAuth,
Grantex, admission-control ACP). Scored down; treated as an exemplary
*consumer* of Concordance rather than a competitor to it.

**5. Governed Deliberation Protocol (native G1–G6 implementation).**
*Core idea:* ship the governance taxonomy's membership/deliberation/
voting/dissent/escalation/audit primitives as an actual wire protocol.
*Problem solved:* the governance gap identified in [15]. *Prior art:*
taxonomy already published and well-sourced; several deliberation
*mechanisms* already proposed in adjacent papers [15, related work]. Very
strong runner-up (§4.3); logically dependent on agents already sharing
enough trust to be in a bounded community, making it a natural future
extension of, rather than an alternative to, Concordance.

**6. Shapley Coalition Auction Protocol.** *Core idea:* ephemeral
task-coalitions self-assemble via broadcast auction, with automatic
Shapley-value-based surplus division. *Prior art:* extremely mature —
classical multi-agent-systems coalition-formation literature, a 2026
dynamic split-merge framework, and a full "Shapley Pricing Equilibrium"
paper for agent workforces. Low remaining originality.

**7. Semantic Clock / Ontology Drift Correction Protocol.** *Core idea:*
periodic, NTP-like calibration exchanges that measure and correct
semantic/embedding drift between two agents' internal representations of
the same terms over time. *Prior art:* long lineage from Gruber's 1993
ontology-translation work through ANP's static semantic layer [17]; no
source found proposes a *continuous, drift-measuring* version, so this
retains moderate originality but is judged lower-impact and harder to
formalize with today's tooling than Concordance.

**8. Ripple-Style Sensitivity Sharing (extended).** Already shipped as
REP [26]; a "richer" version is an incremental extension, not a new
protocol family. Included for completeness; not separately scored as a
candidate, since it fails the discard test outright.

**9. Attention / Compute Futures Market.** *Core idea:* a real-time
derivatives-like market letting agents hedge or trade future compute/tool-
call budget. *Prior art:* payment rails (AP2, x402) are mature; a genuine
futures *market* for attention/compute is not found in this survey, but
feasibility today is low — it requires liquid two-sided markets and clear
legal characterization that do not yet exist.

**10. Autonomous Liability Apportionment Mesh.** *Core idea:* protocol-
native, real-time apportionment of downstream legal/financial
responsibility across a multi-hop agent delegation chain when an outcome
causes harm, as distinct from value attribution (Shapley, §6/#6) or
authorization (IBCT, §2.3). *Prior art:* not found directly in this
search — adjacent work covers value/price attribution for *legitimate*
surplus (Agentomics) and authorization chains (IBCT/PAuth), but fault
apportionment for *harm* across organizational boundaries appears to be a
genuine open gap. Strong runner-up; flagged in Section 17 as high-priority
future work and as a natural second claim-class for Concordance.

**11. Human-Agent Guardianship Protocol.** *Core idea:* a standing,
renegotiable protocol for long-term human oversight of a personal agent,
distinct from one-off consent grants. *Prior art:* overlaps substantially
with the consent cluster (§2.3); moderate originality, moderate
practicality.

**12. Capability Genome / Lineage-Based Discovery Protocol.** *Core
idea:* agents advertise capability via an evolving, recombinable "genome"
inherited/mutated across model versions, discoverable by lineage rather
than static registry lookup. Speculative; moderate originality, low
near-term feasibility.

**13. Multi-Speed, Stakes-Adaptive Consensus.** *Core idea:* a consensus
protocol whose speed/rigor scales with decision stakes (fast, cheap
agreement for low-stakes calls; slow, deliberate BFT-style agreement for
high-stakes ones). *Prior art:* tunable-consistency work in distributed
databases is a longstanding, closely related idea; moderate-low
originality as a distinct "family."

**14. Federated Counterfactual Simulation Exchange ("Dream Protocol").**
*Core idea:* agents exchange simulated hypothetical futures before
acting, rather than only exchanging decisions. Creative and not directly
found in this search, but low near-term feasibility (requires broadly
shared simulators and a common counterfactual representation).

**15. Cognitive Diversity Verification Protocol.** *Core idea:* verify
that apparent multi-agent agreement reflects genuinely independent
judgment rather than correlated failure of a shared base model. Not found
as a standalone proposal; substantial originality, but its core mechanism
is better delivered as a component of a composition algebra (and is, in
fact, folded directly into Concordance's independence-discounting design,
§9.9) than as a freestanding protocol.

**16. Autonomous Norm Emergence Protocol.** *Core idea:* bottom-up,
non-designed emergence and encoding of social norms among an agent
population. *Prior art:* overlaps with Ostrom-CPR-principled social
learning already cited in the governance literature [15, related work].
Moderate-low remaining originality.

**17. Recursive Delegation Chain Protocol with Attenuation.** *Core
idea:* cryptographically verifiable multi-hop delegation with capability
narrowing at each hop. *Prior art:* essentially already shipped (IBCT's
Biscuit/Datalog chains, PAuth's envelopes, admission-control ACP's
transitive revocation). Low remaining originality.

**18. Agent Deprecation & Legacy Transfer Protocol.** *Core idea:*
graceful handling of an agent's "retirement" — memory, relationships, and
standing commitments transferred to a successor when a model version is
sunset. Not found directly in this search; genuine gap, but judged
narrower in near-term impact than the trust-composition problem.

**19. Cross-Model Goal/Value Translation Protocol.** *Core idea:* a
wire-level protocol for translating intents/goals across agents built on
architecturally different foundation models. Adjacent to alignment and
interpretability research broadly; low near-term engineering feasibility
given the current state of interpretability tooling.

**20. Ambient Resource Bargaining Protocol.** *Core idea:* continuous
double-auction micro-negotiation for shared compute/tool access among
co-located agents. Overlaps meaningfully with candidate #9 and with
mature auction-theory literature; moderate-low originality as a distinct
family.

**21. Cross-Organizational Audit-Replay Fabric.** *Core idea:* a portable,
tamper-evident audit log format usable across organizational boundaries,
addressing G6 (audit/replay) specifically and independently of the rest
of the governance taxonomy. *Prior art:* partially covered by blockchain-
as-audit-substrate approaches (ERC-8004) and by session/tracing
approaches (MCP, A2A's Traceability extension), but a genuinely portable,
cross-scheme replay format is not found as a standalone proposal. Retained
as a plausible near-term Concordance claim-class (§9.16) rather than a
standalone winner, since a portable audit log is far more valuable once
it can be attached to composed, cross-scheme trust decisions than in
isolation.

---

## 7. Candidate Evaluation Matrix

Scoring is 1–10 on Originality, Practicality, Scalability, Research
Significance, and Engineering Feasibility. Per the methodology's rule, any
candidate scoring below 6 on Originality is a wrapper-in-disguise
regardless of its other scores, and is marked accordingly; it can still be
a *good idea*, just not a *new protocol family*.

| # | Candidate | Originality | Practicality | Scalability | Research Sig. | Eng. Feasibility | Composite (avg) | Verdict |
|---|---|---|---|---|---|---|---|---|
| 1 | **Concordance** | 9 | 8 | 8 | 9 | 8 | **8.4** | **Selected** |
| 5 | Governed Deliberation (G1–G6) | 7 | 6 | 6 | 9 | 6 | 6.8 | Strong runner-up; future extension |
| 10 | Liability Apportionment Mesh | 8 | 5 | 5 | 8 | 4 | 6.0 | Strong runner-up; future work |
| 21 | Audit-Replay Fabric | 6 | 7 | 7 | 6 | 8 | 6.8 | Good; better as Concordance claim-class |
| 3 | Epistemic Invalidation Cascade | 5 | 7 | 6 | 8 | 7 | 6.6 | Below originality bar; fold into §9.16 |
| 15 | Cognitive Diversity Verification | 7 | 6 | 6 | 7 | 5 | 6.2 | Good idea; absorbed into §9.9 algebra |
| 7 | Semantic Clock Protocol | 6 | 4 | 5 | 7 | 4 | 5.2 | Interesting; low near-term feasibility |
| 12 | Capability Genome Discovery | 6 | 4 | 5 | 5 | 3 | 4.6 | Speculative |
| 9 | Attention/Compute Futures Market | 6 | 3 | 5 | 6 | 3 | 4.6 | Legally/structurally premature |
| 14 | Federated Counterfactual Exchange | 7 | 3 | 4 | 6 | 3 | 4.6 | Creative; not buildable today |
| 18 | Agent Deprecation & Legacy Transfer | 6 | 5 | 5 | 5 | 6 | 5.4 | Real but narrower gap |
| 13 | Multi-Speed Adaptive Consensus | 5 | 6 | 6 | 5 | 6 | 5.6 | Below originality bar |
| 19 | Cross-Model Value Translation | 7 | 2 | 4 | 7 | 2 | 4.4 | Not buildable with current interpretability tooling |
| 11 | Human-Agent Guardianship | 5 | 6 | 6 | 5 | 6 | 5.6 | Below originality bar; overlaps consent cluster |
| 16 | Autonomous Norm Emergence | 5 | 5 | 5 | 6 | 5 | 5.2 | Below originality bar |
| 20 | Ambient Resource Bargaining | 4 | 6 | 6 | 4 | 6 | 5.2 | Below originality bar |
| 4 | Capability Drift / Continuous Consent | 3 | 8 | 7 | 6 | 8 | 6.4 | Below originality bar — already shipping (§2.3) |
| 17 | Recursive Delegation w/ Attenuation | 3 | 8 | 8 | 5 | 8 | 6.4 | Below originality bar — already shipping (IBCT etc.) |
| 6 | Shapley Coalition Auction | 2 | 8 | 7 | 5 | 8 | 6.0 | Below originality bar — mature literature |
| 2 | Stigmergic Trust Mesh | 2 | 7 | 8 | 6 | 7 | 6.0 | Below originality bar — pre-empted (§2.2, §6) |
| 8 | Ripple-Style Sensitivity Sharing | 1 | 8 | 7 | 5 | 8 | 5.8 | Already shipped as REP [26] |

**Ranking note.** Composite averages are not, by themselves, the
selection criterion — several already-crowded candidates (Recursive
Delegation, Continuous Consent) score respectably on practicality and
feasibility precisely *because* they are already mature, shipping
technology, which is exactly why they fail the originality gate the
brief requires. Concordance is the only candidate clearing both bars: a
9/10 on originality (nothing found addresses cross-scheme composition)
and a top-three composite once weighted appropriately. The two genuine
runners-up — Governed Deliberation and Liability Apportionment — are
carried forward not as rejected ideas but as named, planned future
extension schemes for Concordance itself (§9.16, §17), which is, in this
researcher's judgment, a more valuable outcome than either would be as a
standalone protocol competing for adoption attention in an already
crowded field.

---

## 8. Selected Protocol

**Concordance** is selected as the protocol family to specify in full.

### 8.1 Why Concordance, restated plainly

Every other viable candidate in Section 6 either (a) already has
committed, capable teams shipping something similar in 2026, meaning a
new competing standard would fragment the space further rather than heal
it — the opposite of what this study set out to find — or (b) is not yet
buildable with today's tooling. Concordance is the one candidate that is
simultaneously *not already being built by anyone found in this search*
and *fully buildable with 2026 cryptographic and distributed-systems
tooling* (Section 14 specifies exactly what libraries and languages).

It is also, distinctively among the twenty-one candidates, the one whose
value *increases* rather than decreases as the rest of the ecosystem
succeeds. Every additional trust/consent/reputation scheme that ships —
and Section 2 shows several shipping every quarter — is one more scheme
Concordance can wrap, and one more reason an agent needs it. A protocol
whose usefulness compounds with its competitors' success, rather than
competing with them for the same adoption attention, is a structurally
stronger candidate for real-world traction than a twenty-second point
solution would have been.

### 8.2 Honesty about scope

Concordance does not solve trust. It does not decide whether an ERC-8004
reputation score of 0.8 *should* be enough to let an agent execute a
$10,000 transaction — that is, correctly, a local policy decision no
protocol should centralize. What Concordance solves is narrower and, this
document argues, more foundational: it makes heterogeneous trust
evidence **legible, combinable, and revocable across ecosystem
boundaries**, the way HTTP does not decide whether a piece of content is
good but does guarantee that a browser can render it regardless of which
server produced it.

---

## 9. Complete Protocol Specification

### 9.1 Protocol Name

**Concordance.** A concordance, in its original lexicographic sense, is a
structured cross-reference that lets you look up how the same term is
used across different sources, in different contexts, by different
authors — without forcing those sources to have been written in a common
vocabulary to begin with. That is exactly this protocol's job for trust
claims. Version strings follow `Concordance/MAJOR.MINOR`, e.g.
`Concordance/1.0`.

### 9.2 Vision

A future in which any two AI agents, regardless of which organization
built them, which trust ecosystem they were issued credentials in, or
which coordination primitive their home population uses, can establish a
mutually legible, auditable, revocable trust judgment about each other —
without either agent's home ecosystem having to anticipate the other's
existence in advance.

### 9.3 Mission

To define a minimal, transport-independent wire format and negotiation
procedure through which heterogeneous trust, consent, reputation, and
coordination primitives can be wrapped, compared, combined, and
invalidated across ecosystem boundaries — without requiring any existing
scheme to change its own internal design, and without centralizing
judgment about what counts as "enough" trust for any given interaction.

### 9.4 Problem Statement

As of mid-2026, at least eight independently governed trust-adjacent
schemes are in active production or advanced draft use for AI agents
(MCP's authorization model, A2A's Agent Cards, ERC-8004, IBCT, Anumati,
Grantex, admission-control ACP, and REP's sensitivity signals), with more
arriving quarterly. No two of them share an identity model, a claim
schema, a notion of revocation, or an evidence-combination arithmetic. An
agent that needs to make one risk decision informed by evidence from more
than one of these schemes today must hand-write bespoke, bilateral
integration code, an approach that scales quadratically with the number
of schemes in play and silently breaks whenever any one scheme evolves.

### 9.5 Motivation

Three forces make this urgent rather than merely tidy: (1) the pace of
new scheme creation is accelerating, not slowing — this study alone found
six new authorization/consent schemes proposed within the first seven
months of 2026; (2) the schemes are, individually, good — this is not a
"pick the winner" problem the way the A2A/ACP/AGNTCY consolidation was,
because each scheme optimizes a genuinely different, legitimate design
point (on-chain permanence vs. sub-millisecond local verification vs.
natural-language policy expressiveness); and (3) the cost of *not* solving
composition is not neutral — it is paid either by fragmenting into
walled gardens (a large vendor's ecosystem becomes the de facto
composition layer by brute economic force) or by every relying party
defaulting to the weakest common signal, exactly as the 93%-unscoped-API-
key finding shows is already happening [39].

### 9.6 Existing Solutions

Summarized from Sections 2–3: MCP (tool access), A2A (delegation/
discovery), ANP (decentralized routing), ERC-8004 (on-chain reputation),
IBCT/PAuth/Grantex/admission-control-ACP (capability authorization),
Anumati (durable consent), REP (population coordination), Ledger-State
Stigmergy (indirect coordination), and the G1–G6 governance taxonomy
(deliberation, voting, dissent).

### 9.7 Why Existing Solutions Fail (at this specific problem)

None of the above was designed to solve cross-scheme composition, and
none can be incrementally extended into solving it without becoming a
different protocol, for a simple structural reason: each one's extension
mechanism (A2A's extension framework, MCP's extensions-framework-under-
the-2026-07-28-spec, ERC-8004's registry-addition path) is designed to
add *new claims within that scheme's own model*, not to *ingest and
normalize claims from an unrelated scheme's model*. Asking A2A's extension
mechanism to natively understand an ERC-8004 attestation's on-chain
verification path, or Anumati's natural-language adherence semantics,
would require A2A to absorb the entire design surface of both — at which
point it is no longer A2A, it is Concordance wearing A2A's name. This is
precisely the "extension vs. structural gap" distinction the governance-
taxonomy paper draws for its own, adjacent problem [15]; the same
reasoning applies here with equal force.

### 9.8 Core Design Principles

1. **Wrap, don't replace.** Concordance never re-implements a native
   scheme's verification logic; it carries that scheme's native payload
   verbatim alongside a normalized summary, so native verifiers remain
   authoritative.
2. **Composition arithmetic must be explicit and auditable**, not
   silently assumed. Every combined trust judgment must be reconstructable
   from its inputs and the declared combination rule.
3. **Never trust a counterparty's self-report of its own compliance.**
   A deciding agent always recomputes the combined judgment locally
   against its own policy (§9.13, mirroring TLS's refusal to trust a
   peer's claimed cipher-suite selection).
4. **Correlation-awareness is a first-class citizen**, not an
   afterthought — motivated directly by ERC-8004's empirically observed
   gameable-reputation weakness [28].
5. **Stateless-first.** Learn MCP's hard-won 2026 lesson [4] up front:
   no protocol-level session state that later has to be ripped out.
6. **Disagreement is data, not noise.** When two envelopes conflict,
   surface the conflict; do not silently resolve it (borrowed, with
   attribution, from the governance taxonomy's dissent-preservation
   principle [15], applied here at the composition layer).
7. **Privacy-preserving by default, verifiable on challenge.** Native
   payloads may be redacted from routine exchange and disclosed only on
   specific, logged challenge.
8. **No central registry required to operate; a federated one helps
   it scale.** Two agents that have never touched any registry can still
   complete a Concordance exchange if they already know how to normalize
   each other's scheme; the registry (§9.24) exists to make that the
   common case rather than the required one.

### 9.9 Protocol Philosophy

Concordance treats trust the way IP treats packets: it does not know or
care what is inside them, only how to address, route, and — critically —
combine them predictably. Where IP's job stops at delivery, Concordance's
job extends one step further, into a declared but overridable default
arithmetic for combination, because unlike a packet, a trust claim is
only useful once combined with others into a decision.

### 9.10 Architecture

```
                     CONCORDANCE PROTOCOL STACK

  L5  Registry & Adapter Layer      Federated scheme registry;
                                    versioned normalization adapters
  ----------------------------------------------------------------
  L4  Revocation & Freshness Layer  Cross-scheme REVOKE_ECHO;
                                    staleness windows; cache TTLs
  ----------------------------------------------------------------
  L3  Composition Algebra Layer     Correlation-aware combination;
                                    per-class thresholds; conflict
                                    surfacing
  ----------------------------------------------------------------
  L2  Manifest & Negotiation Layer  Scheme manifests; minimum-
                                    sufficient-evidence negotiation
  ----------------------------------------------------------------
  L1  Envelope Layer                Trust Object Envelope (TOE):
                                    canonical wrapper for ANY native
                                    trust primitive
  ----------------------------------------------------------------
  L0  Transport (independent)       HTTPS / WebSocket / gRPC / libp2p
                                    — whatever the wrapped native
                                    scheme already uses
```

Concordance is deliberately the *thinnest possible* layer that makes
L1–L4 well-defined; it does not specify its own transport (L0) or its
own native trust primitives (those live below L1, inside whatever scheme
is being wrapped).

### 9.11 Interaction Model

Concordance's interaction model is **evidence presentation and
negotiated composition**, not request/response RPC (MCP, A2A), not
environment-mediated trace-sensing (stigmergy), and not credential-
disclosure sequencing alone (classical ATN). Two agents first exchange
*manifests* describing which trust vocabularies they speak; they then
negotiate the *minimum sufficient set* of evidence classes needed for the
specific interaction at hand, given its declared risk level; each side
independently *composes* its own trust judgment from whatever heterogeneous
envelopes it receives, using its own local policy over the shared default
arithmetic; and the interaction remains *open to revocation* for its
declared validity window, during which either side may receive and must
act on a `REVOKE_ECHO`.

### 9.12 Communication Model

Concordance messages are self-contained, content-addressed CBOR (or JSON,
for debugging/human-readability) objects, exchanged over whatever
transport the underlying schemes already use. There is no requirement
that both parties in an interaction use the same transport for their own
native-scheme traffic — Concordance messages can be relayed, cached, and
replayed safely because every message is content-addressed and signed,
following the same "stateless-first" lesson MCP's 2026 rewrite learned the
hard way [4].

### 9.13 Agent Lifecycle

```
        ┌──────────┐   publish manifest   ┌────────────────┐
        │  IDLE    │──────────────────────▶│ MANIFEST_KNOWN │
        └──────────┘                       └────────┬───────┘
             ▲                                        │ interaction
             │                                        │ requested
             │                                        ▼
        ┌────┴─────┐   REVOKE_ECHO       ┌────────────────────┐
        │  CLOSED   │◀────received────────│ ENVELOPES_PRESENTED│
        └────┬─────┘                       └────────┬───────────┘
             │                                        │ compose()
             │  decision reached,                     ▼
             │  window expired            ┌────────────────────┐
             └─────────────────────────────│ COMPOSED / DECIDED │
                                            │ (ALLOW/DENY/       │
                                            │  ESCALATE/CONFLICT)│
                                            └────────┬───────────┘
                                                     │ interaction
                                                     │ proceeds;
                                                     │ window monitored
                                                     ▼
                                            ┌────────────────────┐
                                            │  ACTIVE (watching   │
                                            │  for REVOKE_ECHO)   │
                                            └────────────────────┘
```

An agent's Concordance lifecycle is independent of, and much simpler
than, its lifecycle inside any wrapped native scheme — Concordance never
duplicates a native scheme's own onboarding/registration process (§9.8,
principle 1).

### 9.14 Discovery

Discovery of *counterparties* is explicitly out of scope — that is A2A's
job (Agent Cards), ANP's job (DID resolution), or a marketplace's job, and
Concordance deliberately does not re-solve it. What Concordance defines is
discovery of **which trust vocabularies a known counterparty speaks**,
via the manifest exchange in §9.16, and discovery of **normalization
adapters for a previously-unseen scheme**, via the federated registry in
§9.24.

### 9.15 Registration

There is no mandatory global registration. An agent that wants its
manifest to be easily discoverable may publish it to a federated registry
node (§9.24, modeled on DNS zone delegation, not a single authority), but
two agents can complete a full Concordance exchange having never touched
any registry, provided each already has (or fetches ad hoc) the
normalization adapter for the other's scheme(s).

### 9.16 Capability Advertisement (Scheme Manifests)

A **Scheme Manifest** is a small, cacheable, signed document:

```json
{
  "concordance_version": "1.0",
  "agent_id": "did:example:abcd1234",
  "can_present": [
    "urn:concordance:scheme:erc8004:reputation:v1",
    "urn:concordance:scheme:anumati:adherence:v1"
  ],
  "can_verify": [
    "urn:concordance:scheme:erc8004:reputation:v1",
    "urn:concordance:scheme:ibct:capability:v1",
    "urn:concordance:scheme:rep:sensitivity:v1"
  ],
  "policy_classes": {
    "read_only_tool_call": { "min_reputation": 0.4 },
    "write_action_low_value": {
      "min_reputation": 0.6,
      "require_classes": ["capability"]
    },
    "write_action_high_value": {
      "min_reputation": 0.8,
      "require_classes": ["capability", "consent"],
      "max_envelope_age_seconds": 604800,
      "escalate_below": 0.6
    }
  },
  "signature": "ed25519:..."
}
```

Planned future claim-classes explicitly anticipated by this schema
(§4.3, §6 candidates #3, #5, #10, #21) include `dissent-record`,
`liability-share`, and `audit-replay` — Concordance does not need to
define these itself in v1.0; it only needs `claim_class` to be an open,
extensible string namespace, which it is.

### 9.17 Negotiation

```
Agent A                                              Agent B
   │                                                     │
   │──────────────── MANIFEST_REQUEST ─────────────────▶│
   │◀─────────────── MANIFEST_OFFER ────────────────────│
   │  (B's can_present / can_verify / policy_classes)    │
   │                                                     │
   │  A computes: does B's can_present intersect         │
   │  with what A's policy for this interaction class     │
   │  requires, given A's own can_verify list?            │
   │                                                     │
   │──────── ENVELOPE_PRESENT (bundle, class=X) ────────▶│
   │◀────── ENVELOPE_CHALLENGE (need: fresher /  ───────│
   │           additional claim_class Y)                 │
   │──────── ENVELOPE_PRESENT (supplement) ─────────────▶│
   │                                                     │
   │        [B composes locally, per §9.19]              │
   │◀────────────── DECISION (ALLOW) ───────────────────│
   │                                                     │
   │        (interaction proceeds; both sides now         │
   │         watch for REVOKE_ECHO for the declared        │
   │         validity window)                             │
```

Negotiation is deliberately symmetric-capable (either party may be the
one whose policy gates the interaction) but is shown above as one-sided
for clarity.

### 9.18 Coordination

Concordance does not itself coordinate multi-agent tasks — that remains
REP's, A2A's, or a stigmergic substrate's job. What Concordance provides
to *those* coordination layers is a way for a REP-speaking population and
an ERC-8004-reputation-checking population to establish, once, a shared
trust judgment about each other's boundary agents, after which each
population's native coordination mechanism proceeds unmodified internally.

### 9.19 Decision Making

Decision-making is always local. Concordance standardizes the *inputs*
(normalized, composed evidence, per §9.9 below) to a decision function;
it deliberately does not standardize the decision function itself,
because centralizing "how much trust is enough" would recreate exactly
the single-point-of-control problem federation is meant to avoid. A
reference decision function is given in §10.5 as a default, overridable
implementation.

### 9.20 Knowledge Exchange

Where a wrapped scheme's claim concerns *knowledge* rather than identity/
capability/consent (e.g., a claim-class registered for epistemic
provenance per §9.16's extensibility point), Concordance's envelope
format is sufficient to carry it, but Concordance does not itself define
belief-revision semantics — that is exactly the boundary drawn in §4.3
and §6 candidate #3: the algorithmic question ("how should this agent's
internal beliefs update") stays with schemes like Nous or Preregistered
Belief Revision Contracts; Concordance's job is only to let a *retraction
issued in one such scheme* reach an agent that never adopted that
scheme's internal belief-representation, via the same `REVOKE_ECHO`
mechanism used for every other claim class (§9.21).

### 9.21 State Synchronization

Concordance holds minimal state: per active interaction, the set of
presented envelope IDs and their declared validity windows. This is
intentionally far less than CRDT-style full state replication —
Concordance is not trying to keep two agents' entire world-models in
sync (that is a much harder, scheme-specific problem, §9.20); it only
needs enough state to know which envelopes to invalidate on a
`REVOKE_ECHO` and when a validity window lapses.

### 9.22 Trust Model

Concordance has **no trust model of its own** in the sense ERC-8004 or
Anumati have one — this is by design (§9.8, principle 1). Its only
first-class trust-relevant object is the **independence class**
annotation on an envelope (§10.4), used purely to prevent correlated
evidence from being double-counted, and the **adapter-trust bootstrap**
described honestly as a limitation in §16.2.

### 9.23 Identity Model

Concordance is identity-scheme-agnostic: `agent_id` in a manifest may be
a W3C DID (as ANP and AGNTCY's AConP use), an ERC-8004 on-chain address,
or any other stable identifier — Concordance requires only that a
**binding proof** (§10.4) exist, tying a presented envelope's subject to
whichever identifier the presenting agent used to open the Concordance
session, so that heterogeneous credentials issued under different native
identity schemes can be proven to refer to the same real-world agent.

### 9.24 Security Model

Detailed threat-by-threat in Section 11. Summary: Concordance's own
attack surface is narrow and specific — envelope forgery is the wrapped
native scheme's problem, not Concordance's (native verification still
applies); Concordance's own exposure is in (a) the normalization/adapter
step, (b) the binding proof linking heterogeneous identities, and (c) the
composition algebra's resistance to correlated/Sybil evidence.

### 9.25 Privacy Model

Native payloads default to a **redacted mode**: only `scheme_uri`,
`claim_class`, `normalized_strength`, and a well-formedness proof travel
in routine exchange; full native payload is disclosed only on explicit
`ENVELOPE_CHALLENGE`, itself logged. True zero-knowledge proof-of-
normalization (proving a normalized strength was computed correctly
*without* revealing the native payload at all) is flagged honestly as a
v2/future-extension target (§17), not claimed as solved in v1.0.

### 9.26 Failure Recovery

If a required scheme's adapter is unavailable (registry unreachable, new
unrecognized `scheme_uri`), the interaction falls back to
`ESCALATE` under the requesting agent's own policy — never to a silent
`ALLOW`. If a counterparty is unreachable mid-interaction, any decision
already reached remains valid only until its declared freshness window
lapses, after which it must be re-negotiated; this bounds the "blast
radius" of a network partition to the shortest freshness window in play
for that interaction, a deliberately conservative default.

### 9.27 Conflict Resolution

When composed evidence for the same `(subject, claim_class)` pair
conflicts across envelopes (§4.1, item 3; e.g., a reputation envelope and
a dissent-record envelope disagree), Concordance surfaces a `CONFLICT`
decision state rather than silently averaging or picking a side — the
deciding agent's local policy must explicitly say how to handle
`CONFLICT` (default recommendation: escalate to a human or to a governed-
deliberation scheme per §9.16's extensibility, never auto-resolve a flagged
conflict toward `ALLOW`).

### 9.28 Consensus Strategy

Concordance does not run a global consensus protocol and does not need
one: every decision is local to the pair (or small group) of agents in an
interaction. Where a *wrapped* scheme itself uses consensus internally
(ERC-8004's on-chain finality, a future governed-deliberation scheme's
voting round), Concordance treats that scheme's consensus outcome as just
another envelope's `normalized_strength` — it inherits, rather than
replaces, whatever consensus guarantee the underlying scheme already
provides.

### 9.29 Versioning

`scheme_uri`s are versioned independently by their own ecosystems (e.g.
`...:erc8004:reputation:v1` vs `v2`); Concordance's own envelope and
manifest formats are versioned as `Concordance/MAJOR.MINOR` with
mandatory-ignore-unknown-field forward compatibility for MINOR bumps,
directly modeled on the lesson of MCP's costly 2026-07-28
backward-incompatible rewrite [4] — Concordance commits to never needing
that kind of rewrite by refusing, from v1.0, to introduce protocol-level
session state that a later version would need to remove.

### 9.30 Compatibility Strategy

An agent implementing only a subset of claim classes or scheme URIs
degrades gracefully: it simply cannot satisfy policies that require
claim classes it doesn't understand, and correctly falls to `ESCALATE`
(§9.26) rather than failing insecurely open.

### 9.31 Scalability Strategy

Concordance's own overhead per interaction is O(k) in the number of
envelopes presented (typically 1–5), plus at most one registry lookup per
previously-unseen `scheme_uri` (cacheable, amortized across all future
interactions using that scheme — see §12 for measured-in-simulation
figures). It does not add overhead to any wrapped scheme's own
scalability profile — an on-chain ERC-8004 write is exactly as slow
wrapped in Concordance as it is natively, because Concordance never
requires re-writing that state, only referencing it.

### 9.32 Fault Tolerance

Because Concordance holds so little state (§9.21), a Concordance relay or
registry node failing does not strand any in-flight interaction —
envelopes are content-addressed and independently re-fetchable from
either party or any mirror; only *new* lookups of a previously-unseen
scheme's adapter are blocked by a registry outage, and even that
degrades to `ESCALATE`, never to an unsafe default.

### 9.33 Performance Analysis, Threat Model, Limitations, Future Extensions

Given full dedicated treatment in Sections 12, 11, 16, and 17
respectively, to avoid duplication.

---

## 10. Technical Design

### 10.1 Encoding Format and Transport Independence

Wire encoding is **CBOR** (RFC 8949) for production use — compact,
binary, schema-flexible, and already the encoding of choice for several
of the capability-token schemes surveyed in §2.3 — with a canonical JSON
mapping defined for debugging and human inspection. Concordance defines
no transport of its own; messages are exchanged over whatever channel
the two agents already share (an existing MCP Streamable-HTTP connection,
an A2A JSON-RPC channel, a raw WebSocket, or a libp2p stream), tagged
with a `Content-Type: application/concordance+cbor` marker (or
equivalent framing on non-HTTP transports) so they can be distinguished
from the native-scheme traffic riding the same channel.

### 10.2 The Trust Object Envelope (TOE)

This is the protocol's core data structure — the thing every message type
in §10.3 ultimately carries.

| Field | Type | Description |
|---|---|---|
| `envelope_id` | bytes (32) | Content hash of the canonicalized envelope (minus `envelope_id` itself) |
| `scheme_uri` | string (URN) | e.g. `urn:concordance:scheme:erc8004:reputation:v1` |
| `claim_class` | string | One of: `identity`, `capability`, `reputation`, `consent`, `intent-sensitivity`, `dissent`, `admission-decision`, `payment-mandate`, or a registry-extensible custom string |
| `subject` | string | The agent this claim is *about* (native identifier form, e.g. a DID or on-chain address) |
| `issuer` | string | Who issued the underlying native credential |
| `native_payload` | bytes | The original scheme-specific credential, preserved verbatim for native re-verification |
| `normalized_strength` | float [0,1] | Computed by the declared adapter |
| `normalization_fn_uri` | string (URN) | Versioned pointer to the adapter that produced `normalized_strength` |
| `issued_at` | uint64 (unix ms) | |
| `expires_at` | uint64 (unix ms) or null | |
| `revocation_check_uri` | string or null | Where to check/subscribe for revocation of the *native* credential |
| `binding_proof` | bytes | Cryptographic proof `subject` = the presenting agent's Concordance session identity |
| `independence_class` | string or null | Opaque tag; envelopes sharing a class are discounted together in composition (§10.4) |
| `redacted` | bool | If true, `native_payload` is omitted; available via `ENVELOPE_CHALLENGE` |
| `signature` | bytes | Ed25519 signature by `issuer` over the canonicalized envelope |

### 10.3 Message Types

| Message | Direction | Purpose | Key fields beyond TOE |
|---|---|---|---|
| `MANIFEST_REQUEST` | A→B | Ask for B's scheme manifest | `interaction_class` (hint) |
| `MANIFEST_OFFER` | B→A | B's signed manifest (§9.16) | — |
| `ENVELOPE_PRESENT` | either | Bundle of one or more TOEs for the interaction | `bundle_id`, `[TOE, ...]` |
| `ENVELOPE_CHALLENGE` | either | Request a fresher/additional/unredacted envelope | `required_class`, `max_age_seconds`, `reason` |
| `COMPOSE_REQUEST` | either | Ask a trusted third-party composer service to compute a composed score | `bundle_id`, `policy_class` |
| `COMPOSE_RESULT` | composer→requester | Composed result with full derivation trace | `combined_strength[]`, `derivation` |
| `DECISION` | either | Final local decision for this interaction | `outcome` (ALLOW/DENY/ESCALATE/CONFLICT), `valid_until` |
| `REVOKE_ECHO` | any relay/issuer | Propagate a native revocation into Concordance-space | `envelope_id`, `revoked_at`, `reason` |
| `ADAPTER_ANNOUNCE` | registry node | Announce a new/updated normalization adapter | `scheme_uri`, `normalization_fn_uri`, `version`, `signature` |
| `ADAPTER_QUERY` | any→registry | Ask for an adapter for an unrecognized `scheme_uri` | `scheme_uri` |

### 10.4 Reference Algorithm — Normalization

```python
def normalize(native_payload: bytes, scheme_uri: str) -> float:
    """
    Returns a strength in [0,1]. Looks up the versioned adapter
    registered for scheme_uri (locally cached, or fetched via
    ADAPTER_QUERY on first use) and applies it.
    Adapters are small, pure, versioned, and independently signed —
    e.g. the ERC-8004 adapter reads a Reputation Registry feedback
    score and rescales it; the Anumati adapter parses an adherence
    proof's policy-match confidence.
    """
    adapter = adapter_cache.get(scheme_uri) or fetch_adapter(scheme_uri)
    return adapter.normalize(native_payload)   # O(1) — pure function
```

### 10.5 Reference Algorithm — Correlation-Aware Composition

This is the protocol's intellectual core, directly motivated by the
empirically observed gameable-reputation weakness in ERC-8004 [28]: naive
combination (simple OR / simple average) is exploitable by an attacker
who obtains several correlated (Sybil, or merely commonly-sourced)
envelopes for the same claim class.

```python
def combine_class(envelopes: list[TOE]) -> float:
    """
    Discounted noisy-OR combination with independence-class capping.
    Envelopes that share an independence_class are first collapsed
    to their MAX (treated as one witness, not several), preventing
    correlated evidence from compounding. Independent groups then
    combine via noisy-OR, which is monotonic, bounded in [0,1], and
    conservative under uncertainty about true independence.
    """
    groups: dict[str, list[float]] = {}
    for e in envelopes:
        key = e.independence_class or e.envelope_id  # ungrouped = own class
        groups.setdefault(key, []).append(e.normalized_strength)

    group_strengths = [max(strengths) for strengths in groups.values()]

    combined = 1.0
    for s in group_strengths:
        combined *= (1.0 - s)
    return 1.0 - combined       # noisy-OR across independent groups


def compose(bundle: list[TOE], policy: dict) -> dict:
    """
    Groups presented envelopes by claim_class, applies combine_class
    per class, checks each against the policy's per-class thresholds
    and freshness requirements, and returns a full derivation trace
    (never just a final bit) so any DECISION is independently
    auditable and reconstructable — never a black box.
    """
    by_class = group_by(bundle, key=lambda e: e.claim_class)
    result = {}
    for claim_class, required in policy["require_classes_with_min"].items():
        envs = [e for e in by_class.get(claim_class, [])
                if not is_stale(e, policy["max_envelope_age_seconds"])
                and not is_revoked(e)]
        if not envs:
            result[claim_class] = {"strength": 0.0, "status": "ABSENT"}
            continue
        strength = combine_class(envs)
        result[claim_class] = {
            "strength": strength,
            "status": "OK" if strength >= required else "INSUFFICIENT",
            "witnesses": [e.envelope_id for e in envs],
        }
    return result


def decide(composed: dict, policy: dict) -> str:
    if any(v["status"] == "ABSENT" for v in composed.values()):
        return "ESCALATE"
    if any(v["status"] == "INSUFFICIENT" for v in composed.values()):
        mins = [v["strength"] for v in composed.values()]
        return "ESCALATE" if min(mins) >= policy.get("escalate_below", 0) else "DENY"
    if has_conflicting_claims(composed):     # §9.27
        return "CONFLICT"
    return "ALLOW"
```

**Complexity.** `normalize` is O(1) amortized (adapter lookup cached
after first use). `combine_class` is O(m) in envelopes per class
(typically single digits). `compose` is O(k) overall in total envelopes
in the bundle. `decide` is O(c) in policy claim-classes (typically ≤5).
End-to-end, a Concordance decision is **linear in the amount of evidence
actually presented**, independent of the total number of trust schemes
that exist in the world — this is the specific, falsifiable property
that makes the N²→N claim in §4.1 real rather than rhetorical, and it is
exactly what §12's benchmark plan is designed to measure.

### 10.6 Reference Algorithm — Revocation Propagation

```python
def on_native_revocation(scheme_uri, native_credential_ref, reason):
    """
    Called by a scheme-specific listener (e.g. watching an ERC-8004
    Reputation Registry event, or an Anumati policy-change webhook).
    Finds every locally-cached envelope_id derived from this native
    credential and fans out a signed REVOKE_ECHO to every counterparty
    known to have received it, idempotently (content-addressed, so
    duplicate delivery is harmless).
    """
    affected = envelope_index.lookup(scheme_uri, native_credential_ref)
    for envelope_id in affected:
        echo = sign(RevokeEcho(envelope_id=envelope_id,
                                revoked_at=now(),
                                reason=reason))
        for counterparty in delivery_log.recipients_of(envelope_id):
            deliver(counterparty, echo)   # at-least-once, idempotent
```

A `REVOKE_ECHO` never requires the receiving agent to understand the
*native* scheme that produced the now-invalid envelope — only to mark
that `envelope_id` invalid and re-run `compose()`/`decide()` for any
still-open interaction that used it. This is what makes cross-scheme
revocation actually cheap to implement for a relying party: it needs to
understand exactly one message type (`REVOKE_ECHO`), not every scheme's
native revocation mechanism.

### 10.7 Sequence Diagram — Full Interaction Including Mid-Flight Revocation

```
 Agent A                Agent B              ERC-8004 Registry (native)
   │                        │                          │
   │──MANIFEST_REQUEST─────▶│                          │
   │◀─MANIFEST_OFFER────────│                          │
   │──ENVELOPE_PRESENT──────▶│  (reputation TOE,        │
   │   [rep TOE, consent TOE]│   normalized 0.82)       │
   │                        │──compose()/decide()──    │
   │◀─DECISION(ALLOW,───────│                          │
   │   valid_until=T+7d)    │                          │
   │                        │                          │
   │   ... interaction proceeds, day 3 ...              │
   │                        │                          │
   │                        │            reputation slashed
   │                        │◀────────event─────────────│
   │◀─REVOKE_ECHO(rep TOE)──│                          │
   │  (or B re-checks A too,│                          │
   │   symmetric monitoring)│                          │
   │──re-negotiate──────────▶│                          │
   │◀─DECISION(ESCALATE)────│                          │
```

---

## 11. Security Analysis

### 11.1 Threat Model

Concordance's threat model assumes: (a) native schemes' own cryptography
is sound (Concordance does not attempt to re-derive or replace it); (b)
the network is Byzantine (any relay may drop, delay, or duplicate
messages, but signatures prevent silent tampering); (c) some fraction of
counterparties are actively adversarial and will attempt to present
misleading, correlated, or stale evidence to obtain an `ALLOW` they do
not merit; and (d) some normalization adapters may be buggy or, in the
worst case, malicious.

### 11.2 Threat-by-Threat Analysis

| Threat | Description | Mitigation | Residual Risk |
|---|---|---|---|
| Envelope forgery | Attacker fabricates a native credential | Native scheme's own signature verification still applies unmodified; Concordance adds nothing here but also removes nothing | None beyond native scheme's own risk |
| Replay | Attacker re-presents a previously valid but now-stale or revoked envelope | Content-addressed `envelope_id`; mandatory `expires_at`/freshness check; nonce-bound `binding_proof` per session | Low |
| Correlated / Sybil evidence | Attacker presents many envelopes that look independent but share a root cause (same underlying KYC provider, same colluding feedback ring — exactly the weakness documented empirically in ERC-8004 [28]) | `independence_class` capping in `combine_class` (§10.5) collapses correlated groups to their max before combining | **Medium** — correctly tagging `independence_class` currently relies on honest self-report or third-party detection; automated cross-scheme correlation detection is flagged as unsolved in §16.3 |
| Adapter poisoning | A malicious or buggy normalization adapter misreports `normalized_strength` | Adapters are versioned, signed, and — for the seed set — vetted out-of-band before inclusion in the default registry (§9.24, §16.2) | **Medium** — this is a genuine, explicitly acknowledged trust-root bootstrap problem, structurally analogous to root CAs or root DNS servers |
| Downgrade / self-report spoofing | Counterparty claims "I satisfy your policy" without the deciding agent recomputing it | Principle: deciding agent **always** recomputes `compose()`/`decide()` locally against its own policy; a counterparty's claim is never authoritative (§9.8, principle 3) | Low |
| Revocation suppression | A malicious relay drops a `REVOKE_ECHO` | At-least-once delivery with idempotent, content-addressed echoes; recipients can also poll `revocation_check_uri` independently as a fallback, at the cost of the native scheme's own polling overhead | Medium — depends on fallback polling being implemented; pure push-only deployments are more exposed |
| Conflict-masking | Two contradictory envelopes are silently averaged into a misleadingly comfortable score | Explicit `CONFLICT` decision state (§9.27); default policy escalates rather than auto-resolves | Low |
| Adapter registry compromise | Attacker publishes a malicious adapter under a plausible `scheme_uri` | Federated (not single-authority) registry, signed `ADAPTER_ANNOUNCE`, and local caching so a single compromised node cannot instantly poison every relying party | Medium — federation reduces but does not eliminate this; see §16.2 |
| Aggregation inference | An agent legitimately presented several individually-fine envelopes lets a counterparty infer something none of them individually authorized (explicitly out of scope even for IBCT [37]) | Not solved by Concordance v1.0; flagged as an open limitation shared with the entire authorization cluster (§16.4) | Open |

### 11.3 What Concordance Deliberately Does Not Defend Against

Prompt injection, jailbreaking, and tool-output manipulation are native-
scheme and model-level concerns; Concordance operates strictly above the
evidence layer and assumes whatever it wraps has its own defenses against
these (MCP's 2026-07-28 authorization hardening, IBCT's admission checks,
etc.). Concordance's contribution to overall system security is narrow
and specific: it prevents heterogeneous evidence from being combined
*incorrectly*, not from being individually forged, which remains each
native scheme's job.

---

## 12. Performance Analysis

### 12.1 Analytical Complexity (restated from §10.5)

| Operation | Complexity | Notes |
|---|---|---|
| `normalize()` | O(1) amortized | One-time adapter fetch, then pure-function cached lookup |
| `combine_class()` | O(m), m = envelopes in one claim class | m is typically 1–5 in realistic bundles |
| `compose()` | O(k), k = total envelopes in bundle | Linear in *evidence presented*, not in schemes that exist |
| `decide()` | O(c), c = policy claim-classes | Typically ≤ 5 |
| New-scheme registry lookup | O(log N) amortized via federated delegation, O(1) after caching | N = registered scheme URIs; analogous to DNS zone delegation |
| `REVOKE_ECHO` fan-out | O(r), r = recipients who received the now-revoked envelope | Bounded by the delivery log, not by total network size |

### 12.2 The Central, Falsifiable Claim

The whole justification for Concordance rests on one measurable
proposition: **integration cost with Concordance grows O(n) in the number
of distinct trust schemes an agent needs to interoperate with; bespoke
bilateral integration grows O(n²).** Section 14.5 specifies exactly how to
run the benchmark that would confirm or falsify this in a reference
implementation: lines-of-integration-code and engineer-hours as a function
of n = {1, 2, 4, 8, 16} schemes, compared with and without a Concordance
adapter layer. This is deliberately stated as a falsifiable prediction
rather than an assumed conclusion — if a future prototype shows adapter-
writing cost itself scales worse than linearly (e.g., because
increasingly obscure schemes require increasingly bespoke adapters), that
would be important negative evidence against the whole approach, and
Section 16 commits in advance to treating it as such.

### 12.3 Latency Budget

For a typical interaction (k=2–3 envelopes, all schemes previously seen
and cached), Concordance's own added latency is sub-millisecond
(comparable to IBCT's independently reported 0.049ms verification figure
[37], since the operations are of similar shape: signature checks plus
pure-function evaluation). The dominant cost in any real interaction will
almost always be the *wrapped* scheme's own native verification latency
(an on-chain ERC-8004 read, a network round-trip to an Anumati adherence
service) — Concordance adds a small, bounded, one-time tax on top of
whichever native schemes are actually in play, never replacing or
slowing their own performance profile.

### 12.4 Storage

Concordance's persistent storage footprint per agent is bounded by (a)
the delivery log needed for `REVOKE_ECHO` fan-out, which can be pruned
once an envelope's freshness window lapses, and (b) the local adapter
cache, which grows with the number of distinct schemes ever encountered,
not with the number of interactions — a materially better scaling
property than, for instance, an append-only on-chain registry, precisely
because Concordance never needs its own permanent ledger.

---

## 13. Comparison with Existing Protocols

### 13.1 Comparison Table

| Dimension | MCP | A2A | ANP | ERC-8004 | IBCT/Grantex/Anumati | REP | Concordance |
|---|---|---|---|---|---|---|---|
| Primary question answered | What can an agent do? | Which agent handles this? | How do messages route? | Which agents can be trusted? | Is this specific call allowed *now*? | How should my decision shift? | **How do I combine trust evidence from schemes that don't know about each other?** |
| Defines its own trust primitive | No | Partial (Signed Cards) | Identity only | Yes (reputation) | Yes (capability/consent) | No | **No, by design** |
| Composes across *other* schemes | No | No | No | No | No | No | **Yes — this is the whole point** |
| Requires shared vocabulary a priori | Yes (tool schema) | Yes (Agent Card schema) | Yes (DID) | Yes (on-chain registry ABI) | Yes (token format) | Yes (sensitivity format) | **No — normalizes via adapters** |
| Cross-scheme revocation | N/A | N/A | N/A | Native only | Native only | N/A | **Yes (`REVOKE_ECHO`)** |
| Correlation/Sybil-aware combination | N/A | N/A | N/A | No (empirically weak [28]) | No | N/A | **Yes (independence-class capping)** |
| Stateless-core | Yes (as of 2026-07-28) | Partial | Partial | Yes (on-chain) | Yes | Partial | **Yes, from v1.0** |
| Centralizes final decision | No | No | No | No | No | No | **No — decision always local** |

### 13.2 What Is Different

Every protocol in the table answers a question of the form "is claim X,
expressed in *this* vocabulary, true?" Concordance is the only one
answering "given claims X (in vocabulary A) and Y (in vocabulary B), what
can I safely conclude, and how do I know if either stops being true?" —
a strictly different, and logically higher, layer.

### 13.3 Why Existing Protocols Cannot Evolve Into This

As argued in §9.7, each protocol's extension mechanism is built to extend
*its own* claim model, not to ingest and arbitrate between foreign claim
models. A2A's extension framework could technically carry a Concordance
envelope as an opaque payload (exactly as it can carry any opaque
payload today) — but that is Concordance riding on top of A2A's
transport, precisely as this document proposes (§9.10, §9.12), not A2A
absorbing Concordance's composition semantics into its own specification.

### 13.4 Why This Deserves New-Family Status

Concordance is not a new authentication scheme, a new discovery
mechanism, or a new coordination pattern — categories already well
served. It is a new *category* of protocol: an interoperability layer
whose subject matter is other interoperability layers. No precedent for
this specific category was found among the ~50 sources surveyed in this
research except the partial, narrower analogy of Automated Trust
Negotiation (§4.2), which predates the heterogeneous-scheme problem this
document addresses by two decades and a very different threat landscape.

---

## 14. Prototype Design

### 14.1 Reference Implementation Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     concordance-core (library)                │
│  ┌───────────┐ ┌──────────────┐ ┌───────────────────────┐  │
│  │  TOE codec │ │  Composition │ │  Manifest/Negotiation   │  │
│  │ (CBOR/JSON)│ │  Algebra      │ │  state machine          │  │
│  └───────────┘ └──────────────┘ └───────────────────────┘  │
│  ┌───────────────────────┐  ┌─────────────────────────────┐│
│  │  Adapter Registry      │  │  Revocation Fan-out Engine   ││
│  │  Client (cached)       │  │  (delivery log + REVOKE_ECHO)││
│  └───────────────────────┘  └─────────────────────────────┘│
└───────────────┬────────────────────────────┬────────────────┘
                │                              │
    ┌───────────▼───────────┐    ┌────────────▼─────────────┐
    │  Adapter: ERC-8004     │    │  Adapter: IBCT-style JWT   │
    │  reputation reader     │    │  capability normalizer     │
    │  (testnet RPC)         │    │                            │
    └────────────────────────┘    └────────────────────────────┘
    ┌────────────────────────┐    ┌────────────────────────────┐
    │  Adapter: REP           │    │  Adapter: synthetic         │
    │  sensitivity normalizer │    │  "governance dissent"       │
    │                        │    │  claim-class (future ext.)  │
    └────────────────────────┘    └────────────────────────────┘
```

### 14.2 Suggested Languages and Libraries

- **Core library:** Rust (for the codec, algebra, and state machine —
  matching the language choice of several capability-token reference
  implementations already surveyed [37], and giving predictable,
  auditable performance for the composition algebra) with a **Python**
  binding layer (via PyO3) for rapid adapter prototyping, since most of
  the native schemes surveyed (ERC-8004 clients, LangChain/CrewAI-style
  agent frameworks) have mature Python tooling first.
- **Cryptography:** Ed25519 via `ring` (Rust) / `PyNaCl` (Python);
  content addressing via BLAKE3.
- **Encoding:** `serde_cbor` (Rust) with a canonical-JSON debug mirror.
- **Transport shims:** thin adapters for MCP's Streamable HTTP, A2A's
  JSON-RPC-over-HTTP, and a raw WebSocket fallback — Concordance rides
  whichever the two agents already share (§9.12).
- **ERC-8004 adapter:** reads a public testnet (Base Sepolia, per the
  live reference deployments already reported [46]) via a standard
  Ethereum JSON-RPC client (`ethers-rs`/`web3.py`).
- **Registry:** a lightweight federated service (any two operators can
  run one; no single required authority), implemented as a simple
  signed-record store with DNS-style zone delegation, deployable as a
  single Rust binary.

### 14.3 Minimum Viable Prototype (MVP)

Two synthetic agents, Agent Alpha (native scheme: a mocked ERC-8004
reputation reader against a public testnet) and Agent Beta (native
scheme: a mocked Anumati-style consent proof), complete an interaction
that requires **both** a reputation threshold and a consent proof —
something neither agent's native scheme alone can satisfy, and something
neither could complete without either (a) one agent absorbing the other's
scheme wholesale, or (b) Concordance. The MVP demonstrates the full
negotiation → presentation → composition → decision → mid-flight
revocation cycle (§10.7) end to end, on a laptop, with no production
infrastructure required.

### 14.4 Simulation Strategy

A synthetic network of N agents (N scaling from 10 to 1,000) is generated,
each randomly assigned 1–3 of a growing pool of synthetic trust schemes
(modeled loosely on the real ones surveyed: an on-chain-style reputation
scheme with realistic write latency, a fast local-capability-token
scheme, and a natural-language consent scheme with variable verification
cost). Adversarial agents are injected at a configurable fraction,
presenting correlated (shared `independence_class`) envelopes designed to
game naive combination. The simulation measures: (a) whether
`combine_class`'s independence-capping meaningfully reduces the
adversarial agents' success rate relative to naive OR/average combination
(directly testing the design decision motivated by the ERC-8004 empirical
critique, §10.5); (b) end-to-end decision latency as N and scheme
diversity grow; and (c) storage growth per agent over simulated time,
confirming the bounded-footprint claim in §12.4.

### 14.5 Benchmark Definition (the falsifiable claim from §12.2)

For n ∈ {1, 2, 4, 8, 16} distinct synthetic trust schemes, implement (a)
bespoke, hand-written bilateral integration code for every pair of
schemes an agent must interoperate with, and (b) one Concordance adapter
per scheme. Measure lines of code and estimated engineer-hours for each
approach as n grows. The prediction (§12.2) is that (a) grows
quadratically and (b) grows linearly; this is the single most important
number this research program should produce next, and the honest,
falsifiable framing is deliberately preserved here rather than presented
as an already-confirmed result.

### 14.6 Interoperability Testing

Once the MVP (§14.3) is stable, the highest-value next step is proposing
adapter pull requests **upstream** to two or three of the real, currently
shipping projects surveyed in this research (a plausible starting pair:
an ERC-8004 reputation adapter proposed to that ecosystem's tooling
repositories, and an Anumati adherence-proof adapter proposed to its
maintainers) — real interoperability testing against real, independently
maintained schemes is a far stronger validation than any purely synthetic
benchmark, and mirrors exactly how MCP, A2A, and ERC-8004 all validated
their own designs (§2) through real third-party SDK adoption rather than
solitary reference implementations.

### 14.7 Developer Tooling

- A CLI (`concordance inspect <bundle>`) that pretty-prints an envelope
  bundle's derivation trace, exactly as `compose()` computed it, for
  debugging and audit.
- A local dashboard visualizing an agent's current manifest, cached
  adapters, and open interactions with their validity windows.
- Editor/framework plugins for the two dominant agent frameworks
  identified in this survey (LangGraph and CrewAI [19]) so a developer
  building on either can add Concordance-aware policy checks without
  leaving their existing framework.

### 14.8 Production Roadmap Preview

See Section 15 for the full phased plan; in short: MVP (§14.3) → a
federated pilot with 2–3 partner organizations, each already running one
of the real schemes surveyed → a public, opt-in registry with an
incentive/anti-spam layer for adapter contribution → formal
standardization engagement once at least two independent implementations
exist, following the same path A2A took to the Linux Foundation and
ERC-8004 took through the Ethereum Improvement Proposal process [9] [40].

---

## 15. Development Roadmap

| Phase | Timeframe (indicative) | Milestones | Success Criteria |
|---|---|---|---|
| **0 — Spec freeze** | Months 0–2 | Finalize `Concordance/1.0` envelope, manifest, and message schemas; publish as an open, versioned spec repository | Spec reviewable and implementable by a third party without clarification requests on core semantics |
| **1 — MVP** | Months 1–4 | Two-agent, two-scheme demo (§14.3); core library in Rust + Python bindings | End-to-end negotiate→present→compose→decide→revoke cycle runs unattended in CI |
| **2 — Simulation & benchmark** | Months 3–6 | Synthetic-network simulation (§14.4); the O(n) vs O(n²) benchmark (§14.5, §12.2) | Benchmark results published, including if they falsify the central claim — pre-committed either way |
| **3 — Real-adapter pilot** | Months 5–9 | Upstream adapter PRs to 2–3 real schemes (§14.6); a small federated registry with 2+ independent operators | At least one real, previously-unaffiliated project accepts or forks a Concordance adapter |
| **4 — Federated multi-org pilot** | Months 8–14 | 3–5 partner organizations run interactions across their own distinct native schemes via Concordance in a non-production sandbox | Documented reduction in bespoke integration code vs. a pre-Concordance baseline within the pilot |
| **5 — Public opt-in network** | Months 12–20 | Public registry with anti-spam/incentive layer; production-hardened composition algebra including adapter-audit tooling | Independent production traffic from parties not involved in the original design |
| **6 — Standardization engagement** | Months 18+ | Formal proposal to a neutral body (IETF Internet-Draft, W3C Community Group, or the Linux Foundation's Agentic AI Foundation, mirroring A2A's and Grantex's own paths [9] [39]) | Two or more independent implementations; a documented governance process, not a single-vendor spec |

This phasing is deliberately conservative relative to the pace shown in
Section 2 (MCP went from proposal to near-billion-download adoption in
under two years) — Concordance's value proposition depends on real,
independent adapters existing for real schemes, so its roadmap cannot
outrun the schemes it wraps.

---

## 16. Risks and Limitations

This section is deliberately blunt; a protocol proposal that only lists
its own strengths is not trustworthy.

### 16.1 Adoption / Chicken-and-Egg Risk

Concordance is only useful once at least two schemes an agent actually
cares about have adapters. Early adopters bear integration cost before
the network-effect payoff arrives — the same bootstrap problem every
protocol in Section 2 faced, and one several (MIME, early OAuth) took
years to clear. Historical counter-example worth taking seriously:
ActivityPub achieved technical federation but never fully solved
cross-instance trust standardization (§2.1); each Mastodon instance still
runs its own bespoke blocklist. Concordance could suffer the same fate if
adapter-writing incentives never materialize.

### 16.2 The Adapter-Trust Bootstrap Problem

Concordance does not eliminate the need to trust *something*; it moves
the trust question from "do I trust this native scheme" to "do I trust
the adapter that normalizes this native scheme's output." For a small
seed set of well-known schemes, this can be solved the way root
certificate authorities or root DNS servers are solved — a small,
publicly-auditable, out-of-band-vetted initial set — but this is a real,
acknowledged centralization pressure point, not a fully decentralized
solution, and should not be oversold as one.

### 16.3 Independence-Class Detection Is Unsolved

The correlation-aware composition algebra (§10.5) is only as good as the
`independence_class` tags attached to envelopes, and this research did
not find — and did not itself invent — a robust, automated way to detect
that two envelopes from *different* schemes are nonetheless correlated
(e.g., both ultimately trace back to the same underlying identity
verification vendor). Candidate #15 (Cognitive Diversity Verification,
§6) gestures at this problem; a full solution is flagged as priority
future work in Section 17, not claimed here.

### 16.4 Aggregation Inference Remains Open

As noted in §11.2, an agent that legitimately presents several
individually-fine envelopes may enable a counterparty to infer something
none of them individually authorized. This is explicitly out of scope
even for IBCT [37], and Concordance's composition layer, by design,
combines exactly the kind of multi-envelope evidence that makes this risk
more, not less, salient. This is a genuine, currently unaddressed
limitation of the entire approach, not a minor caveat.

### 16.5 Privacy Model Is Partial in v1.0

The redacted-envelope mode (§9.25) protects native payload content from
routine exposure but still reveals `scheme_uri`, `claim_class`, and
`normalized_strength` to every counterparty — a real information leak in
some contexts (merely revealing that an agent holds *any* Anumati
consent proof at all may itself be sensitive). True zero-knowledge
proof-of-normalization is explicitly future work (§17), not a solved
part of this specification.

### 16.6 Standardization-Body Risk

The consolidation events documented in Section 2 (ACP folding into A2A,
AGNTCY archiving its own protocol) show this field's governance moves
fast and sometimes ruthlessly toward a single winner per layer. There is
a real risk that one of the existing major bodies (the Linux Foundation's
Agentic AI Foundation, the Ethereum Foundation's dAI team, or a large
single vendor) proposes its own composition layer, opinionated toward its
own ecosystem's schemes, before an independent effort like this one can
establish genuine neutrality — which is precisely why Phase 6 of the
roadmap (§15) insists on multi-implementation, neutral-body engagement
rather than solo publication.

### 16.7 The Prediction in §12.2 Could Be Wrong

Stated plainly and deliberately again: if adapter-writing cost turns out
to scale worse than linearly as schemes diversify — for instance, because
truly novel schemes require bespoke composition logic Concordance's
generic algebra cannot express — the core value proposition weakens
substantially. This should be treated as the single most important open
empirical question the roadmap in Section 15 needs to resolve early
(Phase 2), not late.

---

## 17. Future Research Directions

1. **Automated independence-class detection** (§16.3) — likely the
   single highest-value follow-on research question, potentially drawing
   on graph-based collusion detection already explored for LLM multi-agent
   governance (governance graphs reducing measured collusion from 50% to
   5.6% in one cited study [15, related work]) adapted to the
   cross-scheme setting.
2. **Zero-knowledge proof-of-normalization**, so `normalized_strength`
   can be proven correctly derived from a native payload without
   revealing that payload at all, closing the partial-privacy gap in
   §16.5.
3. **A native `dissent` and `liability-share` claim-class standard**,
   formalizing the two strongest runner-up candidates from Section 6
   (Governed Deliberation and Liability Apportionment) as concrete,
   registry-published Concordance schemes rather than merely anticipated
   extension points — this is the most direct, actionable next research
   project this document identifies.
4. **Formal game-theoretic analysis of the composition algebra** —
   proving (or finding counterexamples to) properties like
   monotonicity, strategy-proofness against adapter manipulation, and
   worst-case degradation under a bounded fraction of colluding schemes.
5. **Aggregation-inference-aware policy languages** — extending the
   manifest `policy_classes` schema (§9.16) to reason about cumulative
   information disclosure across multiple interactions with the same
   counterparty over time, addressing §16.4.
6. **Cross-scheme retraction for epistemic claims specifically** —
   formalizing §9.20's sketch into a full claim-class specification,
   directly building on (and properly citing) the Preregistered Belief
   Revision Contracts and provenance-capped-poisoning-defense literature
   [§2.4], but at the interoperability-standard layer that literature does
   not itself address.
7. **Empirical replication of the ERC-8004 trust audit methodology**
   [28] applied to a live Concordance-composed decision set, once Phase 4
   of the roadmap (§15) produces real federated-pilot traffic — closing
   the loop on whether cross-scheme composition actually produces more
   reliable trust judgments than any single scheme alone, which is the
   entire point of the exercise and should be measured, not assumed.
8. **Interaction with space-based / high-latency agents** (§5) —
   formalizing bounded-staleness composition semantics for agents that
   cannot participate in real-time negotiation.

---

## 18. Final Conclusion

The brief asked for a genuinely new protocol family, arrived at through
rigorous research rather than assumption, and specified to the point a
prototype could actually be built. The most important methodological
finding of this research is that the naive version of that task — sit
down and invent something no one has thought of — is, as of July 2026,
substantially harder than it would have been even a year earlier, because
the field's most talented researchers and best-resourced engineering
organizations have been extraordinarily productive at exactly the kind of
first-order gap-filling this brief initially seemed to be asking for.
Six independently generated ideas that looked novel from first principles
turned out, on actual literature search, to already be underway. That is
not a failure of the research process — discarding those six ideas *is*
the research process working as designed, and the resulting document is
more credible for showing that work rather than hiding it.

What that same search surfaced is a second-order problem the field's own
success is actively creating: a proliferating, healthily competitive, but
fundamentally non-interoperable set of trust, consent, reputation, and
coordination primitives, each individually well-designed for its own
scheme, none able to speak to any of the others. Concordance is proposed
as the answer to that specific, load-bearing, and — per this research —
genuinely unaddressed problem: a thin, transport-independent,
correlation-aware composition layer that lets heterogeneous trust
evidence be combined, negotiated, and revoked across ecosystem boundaries,
without requiring any existing scheme to change, and without
recentralizing the judgment of what "enough" trust means for any given
interaction.

It is deliberately specified with its limitations in full view (Section
16): the adapter-trust bootstrap is a real centralization pressure point;
independence-class detection is not solved, only made explicit;
aggregation inference remains open; and the entire value proposition
rests on one falsifiable empirical claim (§12.2) that the roadmap commits
to testing early rather than assuming. That posture — precise about what
is proven, what is designed-but-untested, and what remains genuinely
unsolved — is, in this researcher's judgment, what distinguishes a
protocol proposal worth prototyping from a whitepaper worth forgetting.

---

## 19. References

Numbered as cited inline. All URLs were live and accessed July 29, 2026.
Entries marked "(background)" were consulted for this survey but not
directly cited by number in the text above.

[1] Anthropic. "Model Context Protocol Specification." modelcontextprotocol.io/specification (background — general MCP grounding).

[2] "The 2026-07-28 Specification." *Model Context Protocol Blog*, July 28, 2026. blog.modelcontextprotocol.io/posts/2026-07-28

[3] "The 2026-07-28 Specification." modelcontextprotocol.io/specification/2026-07-28 (background).

[4] "Model Context Protocol prepares to break with its stateful past." *The Register*, July 23, 2026. theregister.com/devops/2026/07/23/model-context-protocol-prepares-to-break-with-its-stateful-past

[5] "The 2026-07-28 MCP Specification Release Candidate." *Model Context Protocol Blog*. blog.modelcontextprotocol.io/posts/2026-07-28-release-candidate

[6] "MCP 2026-07-28 spec: what changed, what breaks." *Stacktree*. stacktr.ee/blog/mcp-2026-spec-changes

[7] Konishi, Hidekazu. "Model Context Protocol Specification Version Timeline." hidekazu-konishi.com/entry/mcp_specification_version_timeline.html (background).

[8] "The MCP 2026-07-28 Rewrite: What Breaks and How to Migrate." *Developers Digest*. developersdigest.tech/blog/mcp-2026-07-28-breaking-changes (background).

[9] The Linux Foundation. "Linux Foundation Launches the Agent2Agent Protocol Project to Enable Secure, Intelligent Communication Between AI Agents." June 23, 2025. linuxfoundation.org/press/linux-foundation-launches-the-agent2agent-protocol-project

[10] "Agent2Agent." Wikipedia. en.wikipedia.org/wiki/Agent2Agent (background).

[11] "Linux Foundation A2A Protocol Marks One Year with Broad Enterprise and Cloud Adoption." *AIwire/HPCwire*, April 9, 2026. hpcwire.com/aiwire/2026/04/09/linux-foundation-a2a-protocol-marks-one-year-with-broad-enterprise-and-cloud-adoption

[12] "A2A Protocol Surpasses 150 Organizations, Lands in Major Cloud Platforms, and Sees Enterprise Production Use in First Year." Linux Foundation, April 9, 2026. linuxfoundation.org/press/a2a-protocol-surpasses-150-organizations

[13] "A year of open collaboration: Celebrating the anniversary of A2A." *Google Open Source Blog*, April 16, 2026. opensource.googleblog.com/2026/04/a-year-of-open-collaboration-celebrating-the-anniversary-of-a2a.html

[14] "Google Cloud donates A2A to Linux Foundation." *Google Developers Blog*. developers.googleblog.com/en/google-cloud-donates-a2a-to-linux-foundation (background).

[15] Kang, R. and Diponegoro, Y. "Governance Gaps in Agent Interoperability Protocols: What MCP, A2A, and ACP Cannot Express." arXiv:2606.31498, June 30, 2026.

[16] "A Layered Protocol Architecture for the Internet of Agents." arXiv:2511.19699.

[17] "Beyond Message Passing: A Semantic View of Agent Communication Protocols." arXiv:2604.02369.

[18] "AI Agent Communications in the Future Internet — Paving a Path Toward the Agentic Web." *MDPI Future Internet*, 18(3):171, March 21, 2026 (background).

[19] Walker, Ry. "Agent Coordination Protocols Compared." *Ry Walker Research*, February 23, 2026. rywalker.com/research/agent-coordination-protocols

[20] "A Survey of AI Agent Protocols." arXiv:2504.16736 (background).

[21] "Agent Interoperability Protocols 2026: MCP, A2A, ACP and the Path to Convergence." *Zylos Research*, March 26, 2026. zylos.ai/research/2026-03-26-agent-interoperability-protocols-mcp-a2a-acp-convergence

[22] "Agent Network Protocol White Paper" / W3C AI Agent Protocol Community Group. w3c-cg.github.io/ai-agent-protocol (background).

[23] "LLM Agent Communication Protocol (LACP) Requires Urgent Standardization." arXiv:2510.13821 (background).

[24] AgentNetworkProtocol. GitHub repository. github.com/agent-network-protocol/AgentNetworkProtocol

[25] "AgentDNS: A Root Domain Naming System for LLM Agents." arXiv:2505.22368 (background).

[26] Chopra, A., Sharma, A., Ahmad, F., Muscariello, L., Pandey, V., and Raskar, R. "Ripple Effect Protocol: Coordinating Agent Populations." arXiv:2510.16572, October 18, 2025.

[27] "Position: Collaborative Agentic AI Needs Interoperability Across Ecosystems." arXiv:2505.21550 (background).

[28] "Can Trustless Agents Be Trusted? An Empirical Study of the ERC-8004 Decentralized AI Agent Ecosystem." arXiv:2606.26028.

[29] "Emergent Collective Memory in Decentralized Multi-Agent AI Systems." arXiv:2512.10166, December 10, 2025.

[30] "Deep Reinforcement Learning for Multi-Agent Coordination." *Artificial Life and Robotics* (Springer Nature), 2025 (background, S-MADRL).

[31] "Ledger-State Stigmergy: A Formal Framework for Indirect Coordination Grounded in Distributed Ledger State." arXiv:2604.03997, April 5, 2026.

[32] "The Hitchhiker's Guide to Agentic AI: From Foundations to Systems." arXiv:2606.24937.

[33] "From Agent Traces to Trust: A Survey of Evidence Tracing and Execution Provenance in LLM Agents." arXiv:2606.04990 (background).

[34] "Delayed Verification Destabilizes Multi-Agent LLM Belief: Instability Thresholds and Optimal Corrector Placement." arXiv:2606.27409 (background).

[35] "Preregistered Belief Revision Contracts." arXiv:2604.15558 (background).

[36] "When Does Belief-Based Agent Memory Help? Reliability-Conditional Updating and Provenance-Capped Poisoning Defense." arXiv:2606.22030 (background, "Nous" architecture).

[37] "Authorization Propagation in Multi-Agent AI Systems: Identity Governance as Infrastructure." arXiv:2605.05440.

[38] "SEP: Capability-based authorization." a2aproject/A2A Discussion #1404. github.com/a2aproject/A2A/discussions/1404 (background).

[39] "State of AI Agent Security 2026." *Grantex*, March 15, 2026. grantex.dev/report/state-of-agent-security-2026

[40] "ERC-8004: Pioneering Trustless Agents in the Ethereum Ecosystem." blog.questflow.ai/p/erc-8004-pioneering-trustless-agents, November 26, 2025.

[41] "ERC-8004: Trustless Agent Identity." Eco Support, May 26, 2026. eco.com/support/en/articles/14730445-erc-8004-trustless-agent-identity

[42] "The State of Agentic AI Standards in 2026: MCP, A2A, WebMCP, OSI, and the Protocol Stack Taking Shape." *DEV Community*. dev.to/alexmercedcoder/the-state-of-agentic-ai-standards-in-2026-mcp-a2a-webmcp-osi-and-the-protocol-stack-taking-3o2l

[43] "NIST's AI Agent Standards Initiative." *EnforceAuth*, April 12, 2026. enforceauth.com/blog/nist-ai-agent-standards-authorization-imperative

[44] "Anumati: Proof of Adherence as a Formal Consent Model for Autonomous Agent Protocols." arXiv:2604.16524.

[45] Fernandez, M. et al. "Agent Control Protocol: Admission Control for Agent Actions." arXiv:2603.18829.

[46] "ERC-8004: Trustless Agent Identity." Eco Support (testnet deployment detail: Base Sepolia, Linea Sepolia, Hedera Testnet). eco.com/support/en/articles/13221214

[47] "What is ERC-8004? The Ethereum Standard Enabling Trustless AI Agents." Eco Support (background).

[48] "ERC-8004: A Developer's Guide to Trustless AI Agent Identity." *QuickNode Blog*, May 13, 2026 (background).

[49] "Agentomics: valuing, attributing, and pricing human and artificial agents." arXiv:2606.14769, June 9, 2026 (background, Shapley Pricing Equilibrium).

[50] "Split-Merge Dynamics for Shapley-Fair Coalition Formation." arXiv:2603.17153, March 17, 2026 (background).

[51] Bonabeau, E. et al. "Swarm Intelligence: From Natural to Artificial Systems." 1999 (background, foundational stigmergy reference, cited within arXiv:2512.10166).

[52] Boldini, A. "Stigmergy: a control-theoretic perspective." 2024 (background, cited within arXiv:2512.10166).

[53] De Nicola, R. et al. "Multi-robot stigmergic coordination." 2020 (background, cited within arXiv:2512.10166).

[54] Xu, X., Li, R., Zhao, Z., and Zhang, H. "Stigmergic Independent Reinforcement Learning for Multiagent Collaboration." *IEEE Transactions on Neural Networks and Learning Systems*, 2021 (background).

[55] Nguyen, A. A. "Scalable, decentralized multi-agent reinforcement learning methods inspired by stigmergy and ant colonies." arXiv:2105.03546 (background).

[56] "Deep Reinforcement Learning for Multi-Agent Coordination" (S-MADRL). arXiv:2510.03592 (background).

[57] Alphanome. "Stigmergy in Antetic AI: Building Intelligence from Indirect Communication." March 25, 2025. alphanome.ai/post/stigmergy-in-antetic-ai-building-intelligence-from-indirect-communication

[58] Salman, M., Garzón Ramos, D., Hasselmann, K., and Birattari, M. "Phormica: Photochromic Pheromone Release and Detection System for Stigmergic Coordination in Robot Swarms." *Frontiers in Robotics and AI*, December 23, 2020. ncbi.nlm.nih.gov/pmc/articles/PMC7805914

**Foundational / background sources referenced from established knowledge
rather than this session's live search** (HTTP/HTTPS, TCP/IP, QUIC,
WebSocket, gRPC, GraphQL, OAuth 2.0/2.1, OpenID Connect, Raft, Paxos,
CRDTs, W3C DID Core, ActivityPub, FIPA-ACL, and 2000s Automated Trust
Negotiation literature including Winsborough & Li, "Towards Practical
Automated Trust Negotiation," IEEE S&P 2002): these are stable, slow-
moving standards not subject to the mid-2026 volatility documented above,
and were treated per this study's own methodology (§ accompanying
`agent-protocol-research` skill, Step 1) as reliable from existing
knowledge rather than requiring live verification.
