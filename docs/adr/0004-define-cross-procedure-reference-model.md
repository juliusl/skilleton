# 4. Define cross-procedure reference model

Date: 2026-04-07
Status: Accepted
Last Updated: 2026-04-08
Links: ADR-0002, ADR-0003

## Context

The roadmap raises an open question: "Can Tasks or Steps reference other Procedures? If so, define cycle constraints and traversal semantics."

This decision affects composition and reuse patterns in skilleton. Agent skills often have shared sub-procedures (e.g., "validate token" appears in multiple flows). Without cross-references, shared behavior must be duplicated. With cross-references, the tool needs cycle detection and clear traversal semantics.

The type hierarchy (ADR-0002) defines the composition tree: Skill → Procedure → Step → Task. Cross-procedure references would allow items within one Procedure to point to another Procedure, creating a graph overlay on top of the tree.

The ID scheme (ADR-0003) uses hierarchical path IDs, which means references are path-based. A reference from `skill:s.procedure:a.step:1.task:call-auth` to `skill:s.procedure:auth-flow` is expressible as a path.

Decision drivers:
- **Reuse** — shared sub-procedures reduce duplication in complex skills
- **Analyzability** — the Policy compiler must be able to resolve all applicable policies for any execution path, including across references
- **Cycle safety** — circular references would cause infinite traversal in compilation and analysis
- **Simplicity** — the reference model should be understandable by both agents and humans

## Options

### Option A: No cross-references

Procedures are self-contained. Tasks and Steps cannot reference other Procedures. Shared behavior is duplicated.

- **Pro:** Simplest model — the type hierarchy is a pure tree with no graph overlay
- **Pro:** No cycle detection needed
- **Pro:** Policy scoping is trivial — walk up the tree
- **Con:** Shared sub-procedures must be copy-pasted across Procedures
- **Con:** Changes to shared behavior require updating every copy — high maintenance burden

### Option B: Read-only references (documentation links)

Tasks can include references to other Procedures as metadata links but cannot invoke or compose them. References are informational — "see also Procedure X."

- **Pro:** Enables documentation-style cross-referencing without affecting traversal
- **Pro:** No cycle risk — links are metadata, not execution edges
- **Pro:** Low implementation cost — just a `links: Vec<ItemId>` field
- **Con:** Does not solve the reuse problem — behavior is still duplicated
- **Con:** Links can become stale without validation

### Option C: Invocation references with DAG constraint

Tasks can reference other Procedures as invocations. The reference graph must be a DAG (Directed Acyclic Graph) — cycles are rejected at validation time.

- **Pro:** Full composition — shared sub-procedures are defined once and referenced
- **Pro:** Policy scoping works by merging: the calling Procedure's policies plus the referenced Procedure's own policies
- **Pro:** DAG constraint is well-understood and efficiently enforceable (topological sort)
- **Con:** Policy conflict detection becomes harder — a referenced Procedure may inherit conflicting policies from two different callers
- **Con:** More complex traversal — the compiler must follow references and detect DAG violations
- **Con:** Harder to understand execution flow when references chain deeply

## Evaluation Checkpoint (Optional)
<!-- Gate: Options → Decision. Agent assesses and recommends. -->

**Assessment:** Proceed

- [x] All options evaluated at comparable depth
- [x] Decision drivers are defined and referenced in option analysis
- [x] No unacknowledged experimentation gaps (ADR-0022 tolerance check)

**Validation needs:** The choice between Options A, B, and C does not require validation — all three are standard graph-modeling patterns with well-known tradeoffs. The exact policy merge semantics (acknowledged as uncertain in the Draft Worksheet) are a downstream design concern that does not affect the reference model selection. Merge semantics will be specified in the Policy compiler ADR.

## Decision

In the context of **enabling cross-procedure composition in skilleton**, facing **the tradeoff between simplicity and reuse**, we chose **Option C (invocation references with DAG constraint)** over **no references (A) and read-only links (B)** to achieve **full procedure composition and reuse without duplication**, accepting **increased complexity in Policy conflict detection and traversal**.

Concrete commitments:
1. A `Task` may include an optional `invokes: Option<ItemId>` field referencing another Procedure
2. The referenced Procedure must exist within the same Skill. Cross-Skill references are intentionally out of scope — this decision addresses intra-Skill composition only.
3. The reference graph across all Procedures in a Skill must form a DAG — the validator rejects cycles
4. Cycle detection uses DFS (three-color marking) during validation (not at construction time)
5. Policy resolution for an invoked Procedure merges the caller's inherited policies with the callee's own policies. Conflicts are surfaced as errors by the Policy compiler — a Skill with unresolved policy conflicts fails validation. The detailed merge specification (what constitutes a conflict, precedence rules between caller-inherited and callee-own policies) is deferred to the ADR that defines the Policy compiler's resolution algorithm.
6. Read-only documentation links (Option B) are also supported via the existing `Links:` metadata field — they are separate from invocation references

## Consequences

- **Positive:** Shared sub-procedures are defined once and reused — changes propagate automatically
- **Positive:** The DAG constraint is enforceable at validation time with DFS cycle detection, producing clear error messages when cycles are detected
- **Positive:** Documentation links and invocation references coexist — different use cases, different fields
- **Negative:** Policy conflict detection must handle transitive inheritance across invocation edges. A Procedure invoked from two different callers may see conflicting policies from each caller's scope. The detailed merge specification (conflict definition, precedence rules) is deferred to the Policy compiler ADR; this decision establishes that conflicts are validation errors, not silent merges.
- **Negative:** Deep invocation chains make it harder to reason about the full execution path. Mitigation: the compiler can produce a flattened view of any Procedure's full execution graph.
- **Neutral:** The `invokes` field on Task is optional. Skills that don't need cross-procedure references ignore it entirely — the feature adds no overhead when unused.

## Quality Strategy

- [x] Introduces major semantic changes
- [ ] ~~Introduces minor semantic changes~~
- [ ] ~~Fuzz testing~~
- [x] Unit testing
- [ ] ~~Load testing~~
- [ ] ~~Performance testing~~
- [x] Backwards Compatible
- [ ] ~~Integration tests~~
- [ ] ~~Tooling~~
- [ ] ~~User documentation~~

### Additional Quality Concerns

Unit tests should verify:
- DAG validation accepts valid reference graphs
- DAG validation rejects cycles with clear error messages
- Policy merging across invocation edges
- References to nonexistent Procedures are rejected

## Conclusion Checkpoint (Optional)
<!-- Gate: Quality Strategy → Review. Verify before requesting review. -->

**Assessment:** Ready for review

- [x] Decision justified (Y-statement or equivalent)
- [x] Consequences include positive, negative, and neutral outcomes
- [x] Quality Strategy reviewed — relevant items checked, irrelevant struck through
- [x] Links to related ADRs populated

**Pre-review notes:** Policy conflict detection across invocation edges is the highest-complexity consequence. The implementation should surface conflicts clearly rather than silently merging.

---

## Comments

### Draft Worksheet
<!-- Captures original intent and workflow calibration. -->

**Framing:**
Open question from the roadmap. Agent skills have shared sub-procedures (e.g., validation flows reused across multiple procedures). The question is whether and how to support cross-procedure references.

**Tolerance:**
- Risk: Medium — invocation references add graph complexity to what was a tree
- Change: Medium — this affects traversal, policy resolution, and validation
- Improvisation: Low — the options are standard graph-modeling patterns

**Uncertainty:**
- Certain: the tree structure (Skill → Procedure → Step → Task) is established
- Uncertain: how common cross-procedure references will be in practice
- Uncertain: exact policy merge semantics for conflicting inherited policies

**Options:**
- Target count: 3
- [ ] Explore additional options beyond candidates listed below

**Candidates:**
- No cross-references — pure tree, duplicate shared behavior
- Read-only links — metadata references, no composition
- Invocation references with DAG constraint — full composition, cycle-safe

### Review Round 1 — 2026-04-08

| # | Finding | Priority | Action | Detail |
|---|---------|----------|--------|--------|
| 1 | Policy merge semantics are undefined | H | Address | Tightened Commitment #5 to specify conflicts are validation errors and explicitly deferred detailed merge specification (conflict definition, precedence rules) to the Policy compiler ADR. Updated the negative consequence to match. Rationale: the commitment was making a behavioral claim without specifying the behavior — an accuracy gap. Full merge specification is out of scope for the reference model ADR but the behavioral commitment (conflicts = errors, not silent merges) belongs here. |
| 2 | Evaluation Checkpoint contradicts Draft Worksheet | M | Address | Updated Validation needs to acknowledge the merge semantics uncertainty from the Draft Worksheet and explain why it doesn't block the Options → Decision gate. The uncertainty is real but orthogonal to the choice between A, B, and C. |
| 3 | Same-Skill constraint is silent on future scope | L | Address | Added a clarifying sentence to Commitment #2 stating cross-Skill references are intentionally out of scope. This is scope documentation (framing context), not scope expansion. |
