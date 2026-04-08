# 7. Define policy conflict detection model

Date: 2026-04-08
Status: Ready
Last Updated: 2026-04-08
Links: ADR-0002, ADR-0003, ADR-0004

## Context

The roadmap requires "a Policy compiler must be able to check for Policy conflict." ADR-0004 deferred detailed policy merge semantics to this decision. The roadmap flags as open: "Define what constitutes a Policy conflict (contradictory text, overlapping scope, incompatible constraints, etc.)"

Policies attach at each hierarchy level (ADR-0002): `Skill.policies`, `Procedure.policies`, and `Step.policies` are all `Vec<Policy>`. A Procedure inherits its parent Skill's policies. A Step inherits its Procedure's policies plus the Skill-level policies. When a Task invokes another Procedure (ADR-0004), the callee may see policies from both its own Skill-level ancestry and the caller's scope — this is the primary source of potential conflicts.

Policy identity uses hierarchical path IDs (ADR-0003). Two policies at different scopes have different `ItemId` paths (e.g., `skill:s.policy:no-plaintext` vs. `skill:s.procedure:auth.policy:no-plaintext`), so identity-based deduplication is insufficient — semantically overlapping policies will have distinct IDs.

The core question: what constitutes a "conflict" that the compiler should flag?

Decision drivers:
1. **Implementability** — the detection model must be buildable in Milestone 2 without NLP or ML
2. **Actionability** — reported conflicts must be resolvable by the user with clear guidance
3. **False-positive rate** — excessive false positives erode trust in the compiler's output
4. **Extensibility** — the model should accommodate future semantic analysis without redesign

## Options

### Option A: Text-based duplicate detection

Detect policies with identical or near-identical text at overlapping scopes. Use string normalization (lowercase, whitespace collapse) and optionally Levenshtein distance for near-match detection.

```
Conflict: policy:no-secrets at skill:s and policy:no-credentials at skill:s.procedure:auth
  → text similarity: 85% (threshold: 80%)
```

- **Pro:** Simple to implement — string comparison with a configurable threshold
- **Pro:** Catches copy-paste duplication across scopes
- **Con:** Misses semantic conflicts — "always log requests" and "never log PII" have low text similarity but conflict in practice
- **Con:** Near-match thresholds produce false positives — "no plaintext passwords" and "no plaintext tokens" are similar text but not conflicting
- **Con:** Threshold tuning is fragile — no single threshold works across all policy styles

### Option B: Tag-based conflict detection

Policies carry semantic tags (e.g., `#security`, `#performance`, `#logging`). The compiler detects conflicts when policies with the same tag at overlapping scopes have contradictory directives. Tags are author-supplied metadata.

```rust
struct Policy {
    meta: ItemMeta,
    text: String,
    tags: Vec<String>,
}
```

A conflict is: two policies with the same tag applying to the same scope where the author has not explicitly marked them as compatible.

- **Pro:** Semantic grouping enables meaningful conflict detection — "security" policies are compared against other "security" policies
- **Pro:** Extensible — new tags and conflict rules can be added without changing the detection algorithm
- **Con:** Requires policy authors to tag every policy correctly — untagged policies are invisible to conflict detection
- **Con:** Tag taxonomy must be defined and maintained — what tags exist? What counts as contradictory?
- **Con:** Does not solve the hard problem — determining whether two tagged policies actually contradict requires semantic understanding the compiler doesn't have
- **Con:** Requires amending the accepted `Policy` struct from ADR-0002 to add a `tags` field — an already-accepted decision would need modification

### Option C: Scope-overlap reporting

Report all cases where multiple policies apply to the same scope. The compiler computes effective policy sets by walking the hierarchy and merging inherited policies at each node. Any node with more than one effective policy is reported as a "policy overlap" for user review.

```
Overlap at skill:s.procedure:auth.step:validate:
  - skill:s.policy:no-plaintext (inherited from Skill)
  - skill:s.procedure:auth.policy:encrypt-tokens (defined at Procedure)
  Resolution: user confirms compatibility or refactors
```

For cross-procedure invocations (ADR-0004), the compiler merges the caller's effective policies with the callee's effective policies and reports any overlap.

- **Pro:** No semantic analysis needed — purely structural computation
- **Pro:** Zero false negatives — every potential conflict is surfaced
- **Pro:** Produces a complete "policy map" showing which policies apply where — useful for auditing and debugging
- **Pro:** Extensible — semantic analysis (text similarity, tagging) can be layered on top as filters to reduce noise
- **Con:** High false-positive rate for skills with many non-conflicting policies — every overlap is reported, even benign ones
- **Con:** Requires user judgment to resolve overlaps — the compiler cannot distinguish real conflicts from intentional layering

## Evaluation Checkpoint (Optional)
<!-- Gate: Options → Decision. Agent assesses and recommends. -->

**Assessment:** Proceed

- [x] All options evaluated at comparable depth
- [x] Decision drivers are defined and referenced in option analysis
- [x] No unacknowledged experimentation gaps (ADR-0022 tolerance check)

**Validation needs:** None — all three options use well-understood analysis patterns (string comparison, tagging, set intersection). The tradeoff between precision and recall in conflict detection is a design choice, not a technical unknown.

## Decision

In the context of **defining what constitutes a policy conflict in skilleton**, facing **the need for a conflict detection model that is implementable without NLP, produces actionable output, and supports future extension**, we chose **Option C (scope-overlap reporting), refined to cross-origin convergence** over **text-based duplicate detection (A) and tag-based conflict detection (B)** to achieve **complete visibility into policy interactions where policies from different origins converge, with zero false negatives for genuine overlap scenarios**, accepting **that overlaps between non-conflicting policies still require user judgment to confirm compatibility**.

Option C as evaluated reports *all* multi-policy scopes. In practice, this flags normal single-branch inheritance (a Step inheriting its Procedure's policy and the Skill's policy) alongside genuine cross-origin convergence. Worked example: a Skill with 1 policy, 2 Procedures with 1 policy each, and 1 Step per Procedure — the broad trigger flags 6/6 nodes (every node with ≥2 effective policies), but 0 have genuine cross-origin overlap. The refinement narrows the trigger to cases where policies from *different origins* converge, which is the actual conflict surface.

Concrete commitments:
1. The Policy compiler computes effective policy sets by walking the hierarchy — each node's effective set is its own `policies` plus all ancestor `policies`
2. For cross-procedure invocations (ADR-0004), the effective set at the invocation point merges the caller's effective policies with the callee's effective policies
3. An "overlap" is reported when policies from different origins converge at a scope — specifically: (a) cross-procedure invocations where caller and callee effective sets merge, and (b) multiple policies defined at the same hierarchy level within a single scope. Single-branch inheritance (a child scope inheriting its parent's policies along one ancestry chain) is normal policy layering and is *not* reported. Each overlap includes the policies involved, their origin scopes, and the target scope
4. Overlaps are warnings, not errors — the compiler surfaces them for user review. Users can suppress overlaps via compatibility annotations. Full annotation design is deferred to implementation, constrained to: (a) an annotation targets a specific overlap (a pair of policy origins at a specific scope), not policies globally; (b) annotations are co-located with the policy definition in the skill file, not in external config; (c) each suppression is per-overlap — no global rules or pattern-based suppression
5. The overlap report is structured data (not free-form text) so downstream tools can filter, sort, and present it
6. Text-based similarity (Option A) and tag-based grouping (Option B) may be layered on top as optional filters in future milestones. The overlap report's structured format supports this extension without redesign.

## Consequences

- **Positive:** Every potential policy conflict is surfaced — no silent interactions between policies at overlapping scopes
- **Positive:** The detection algorithm is purely structural (set computation over the hierarchy) and implementable in Milestone 2 without external dependencies
- **Positive:** The overlap report doubles as an audit tool — users can inspect which policies apply at any scope, even when there are no conflicts
- **Negative:** Skills with cross-procedure invocations or multiple same-level policy definitions will produce overlap reports requiring manual review, even when the overlapping policies are intentionally complementary. Mitigation: compatibility annotations let users suppress known-good overlaps, and future filtering (text similarity, tags) can reduce noise programmatically.
- **Negative:** The compiler cannot distinguish real conflicts from intentional policy layering. Users must review each overlap manually until semantic analysis is added.
- **Neutral:** The overlap model does not define precedence rules (e.g., "child policy overrides parent"). Precedence is an orthogonal concern — this ADR establishes detection, not resolution. A future ADR may define precedence semantics if the overlap reports indicate demand.
- **Revisit trigger:** Revisit overlap criteria after overlap reports are generated for ≥3 real skills. If >80% of reported overlaps are false positives (user-confirmed benign), reconsider trigger criteria — the threshold may need further narrowing or default filtering.

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
- Effective policy set computation at each hierarchy level (Skill, Procedure, Step)
- Inherited policy propagation from parent to child scopes
- Single-branch inheritance does NOT produce overlaps (child inheriting parent policies only)
- Overlap detection for cross-procedure invocations (caller + callee policy merge)
- Overlap detection for multiple policies defined at the same hierarchy level
- Cross-procedure invocation merging produces correct effective sets
- Empty policy sets produce no overlaps
- Single-policy scopes produce no overlaps

## Conclusion Checkpoint (Optional)
<!-- Gate: Quality Strategy → Review. Verify before requesting review. -->

**Assessment:** Ready for review

- [x] Decision justified (Y-statement or equivalent)
- [x] Consequences include positive, negative, and neutral outcomes
- [x] Quality Strategy reviewed — relevant items checked, irrelevant struck through
- [x] Links to related ADRs populated

**Pre-review notes:** This ADR intentionally avoids defining precedence rules (child-overrides-parent, caller-overrides-callee). Precedence is a resolution mechanism; this ADR covers detection only. If overlap reports prove too noisy in practice, filtering and precedence can be addressed in a follow-up ADR.

---

## Comments

### Draft Worksheet
<!-- Captures original intent and workflow calibration. -->

**Framing:**
ADR-0004 deferred policy merge semantics to this decision. The roadmap requires conflict detection as part of the Policy compiler. The core question is what "conflict" means — this ADR defines the detection model, not the resolution strategy.

**Tolerance:**
- Risk: Low — scope-overlap reporting is a conservative, well-understood approach
- Change: Low — detection model is additive and does not alter existing types
- Improvisation: Low — the options are standard static analysis patterns

**Uncertainty:**
- Certain: policies attach at Skill, Procedure, and Step levels (ADR-0002)
- Certain: cross-procedure invocations create merged policy scopes (ADR-0004)
- Uncertain: how noisy overlap reports will be in practice — depends on real-world skill structure
- Uncertain: whether compatibility annotations are sufficient to manage false positives

**Options:**
- Target count: 3
- [ ] Explore additional options beyond candidates listed below

**Candidates:**
- Text-based duplicate detection — string similarity at overlapping scopes
- Tag-based conflict detection — semantic tags with conflict rules
- Scope-overlap reporting — report all multi-policy scopes for user review

### Q&A Addendum — Revision V-1 (2026-04-08)

**F1 (H): Overlap trigger too broad — flags all normal inheritance.**
Decision: **Address.** The reviewer's worked example (Skill with 1 policy, 2 Procedures with 1 each, 1 Step each → 6/6 nodes flagged, 0 real conflicts) demonstrates that the original "2+ effective policies" trigger conflates normal single-branch inheritance with genuine cross-origin convergence. Narrowed commitment #3 to report overlaps only when policies from *different origins* converge: cross-procedure invocations and same-level definitions. Added the worked example to the Decision section to justify the refinement.

**F2 (H): Annotation mechanism defers entire suppression design with no constraints.**
Decision: **Address.** Deferring annotation *implementation* is appropriate, but the ADR should constrain the design space so the suppression strategy is grounded. Expanded commitment #4 with three minimum constraints: annotations target specific overlaps (not global), live in the skill file (not external config), and are per-overlap (not pattern-based). These constraints prevent the annotation system from becoming a second policy language.

**F3 (H): Add revisit trigger for overlap criteria.**
Decision: **Address.** "≥3 real skills" and ">80% false positives" are concrete, measurable milestones — exactly the kind of revisit trigger worth recording. Added to Consequences.

**F4 (M): Negative consequence "verbose reports for skills with many policies" is understated.**
Decision: **Address.** With the original broad trigger, the verbosity applied to ANY skill with >1 policy, not just "skills with many policies." With the narrowed trigger from F1, the verbosity is proportional to cross-procedure invocations and same-level definitions. Reframed the consequence to match the revised trigger and to be more direct about the manual review burden.

**F5 (M): Option B cons should mention amending ADR-0002.**
Decision: **Address.** Factual gap — Option B proposes `tags: Vec<String>` on the Policy struct, but the accepted Policy struct in ADR-0002 has only `{ meta: ItemMeta, text: String }`. Added con noting the amendment requirement.

**F6 (L): Define a measurable false-positive threshold for decision driver #3.**
Decision: **Reject.** No empirical data exists — we haven't generated overlap reports for any real skill. The revisit trigger from F3 provides the measurable checkpoint (>80% false positives across ≥3 skills). Stating a threshold now without data would be an ungrounded assertion. The threshold will emerge from practice, not from theory.
