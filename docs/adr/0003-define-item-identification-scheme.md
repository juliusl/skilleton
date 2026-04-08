# 3. Define item identification scheme

Date: 2026-04-07
Status: Accepted
Last Updated: 2026-04-08
Links: ADR-0002, ADR-0004

## Context

Every item in skilleton needs a unique identifier. The ID scheme affects storage, cross-referencing, display, source-control diffs, and the Policy compiler's ability to resolve references.

The roadmap flags this as an open question: "Define item ID scheme (e.g., UUIDs, namespaced paths, human-readable slugs)."

Requirements from the roadmap and ADR-0002's type hierarchy:
- IDs must be unique within a Skill (the root container)
- IDs must support cross-procedure references (ADR-0004)
- IDs should be source-control friendly (readable diffs, merge-friendly)
- IDs should encode enough context for the Policy compiler to determine scope
- IDs must be stable across renames of display titles

Decision drivers (ranked by priority):
1. **Scope encoding** — the Policy compiler needs to determine where a Policy applies. This is the primary differentiator between options.
2. **Human readability** — operators and agents need to reference items in diagnostics and instructions
3. **Diff friendliness** — changes to IDs should produce meaningful diffs in source control
4. **Collision resistance** — IDs must be unique without a central registry

## Options

### Option A: UUIDs

Each item gets a v4 UUID (e.g., `550e8400-e29b-41d4-a716-446655440000`).

- **Pro:** Globally unique — no collision risk, no coordination needed
- **Pro:** Stable across renames — changing a title doesn't change the ID
- **Con:** Opaque — a UUID conveys no information about what it identifies or where it lives
- **Con:** Terrible source-control diffs — UUIDs are meaningless to reviewers
- **Con:** The Policy compiler cannot derive scope from the ID alone — needs a separate scope lookup

### Option B: Hierarchical path IDs

Each item is identified by its path from the Skill root, using type prefixes and dot separators:
`skill:my-skill.procedure:auth-flow.step:validate-token.task:check-jwt`

- **Pro:** Self-documenting — the ID encodes the item's position in the hierarchy
- **Pro:** Scope is derivable from the path — `skill:my-skill.procedure:auth-flow.policy:no-plaintext` clearly scopes the policy to `auth-flow`
- **Pro:** Readable diffs — path changes tell you exactly what moved
- **Con:** IDs change when items are reparented (moved to a different parent)
- **Con:** Verbose — deeply nested items produce long IDs
- **Con:** Singleton items (Policy, Criterion) need an attachment path to their parent

### Option C: Type-prefixed slugs (scoped to parent)

Each item gets a slug unique within its parent container. The full path is computable but not stored in the ID field itself:
- Stored ID: `policy:no-plaintext`
- Full path (computed at resolution time): `skill:my-skill.procedure:auth-flow.policy:no-plaintext`

- **Pro:** Short, readable, diff-friendly
- **Pro:** Scope is determined by traversal — the compiler walks the tree, it doesn't parse the ID
- **Pro:** Stable under reparenting if slugs don't collide in the new parent
- **Con:** Uniqueness is only guaranteed within a parent — cross-reference resolution needs tree context
- **Con:** Two items in different parents can have the same slug (e.g., `task:validate` in two different Steps)

## Evaluation Checkpoint (Optional)
<!-- Gate: Options → Decision. Agent assesses and recommends. -->

**Assessment:** Proceed

- [x] All options evaluated at comparable depth
- [x] Decision drivers are defined and referenced in option analysis
- [x] No unacknowledged experimentation gaps (ADR-0022 tolerance check)

**Validation needs:** None — all options use well-understood identification patterns. The tradeoffs are clear from prior art in similar domain-modeling tools.

## Decision

In the context of **identifying items in skilleton's type hierarchy**, facing **the need for human-readable, scope-aware, source-control-friendly identifiers**, we chose **Option B (hierarchical path IDs)** over **UUIDs (A) and type-prefixed slugs (C)** to achieve **self-documenting identifiers where scope is derivable directly from the ID**, accepting **verbosity for deeply nested items and ID instability on reparenting**.

Option C was rejected primarily because cross-reference resolution requires tree context — the compiler cannot determine scope from the ID alone, which conflicts with our top decision driver (scope encoding).

Concrete commitments:
1. An `ItemId` is a structured path: `<type>:<slug>` segments joined by `.` separators
2. Valid type prefixes: `skill`, `procedure`, `step`, `task`, `policy`, `criterion`
3. Slugs are lowercase, hyphenated, max 50 characters per segment (derived from keeping full 4-level paths under ~250 characters to avoid file-system and tooling friction; adjustable if real usage warrants it)
4. The full path from root (Skill) to leaf is the canonical ID
5. Singleton items include their attachment point: `skill:my-skill.procedure:auth.policy:no-plaintext`
6. `ItemId` is a newtype wrapping a `String` with parsing and validation methods
7. Equality is path-based — two `ItemId`s are equal if their full paths match

## Consequences

- **Positive:** The Policy compiler can determine scope by prefix-matching paths — no separate scope lookup needed
- **Positive:** Diagnostics and error messages reference items by readable paths
- **Positive:** Source-control diffs show exactly which item was modified or moved
- **Negative:** Moving an item to a different parent changes its ID and breaks existing references. A rename/move operation must update all references — rename-propagation tooling is deferred to the implementing ADR for ItemId.
- **Negative:** Deep hierarchies produce long IDs. Mitigation: the max nesting depth is fixed at 4 levels (Skill → Procedure → Step → Task), keeping paths manageable.
- **Neutral:** Serialization stores the full path. This is slightly more verbose than a UUID but carries more information per byte.

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
- Parsing valid and invalid path strings
- Path equality and prefix matching for scope resolution
- Slug validation (length, character set)
- ItemId construction from parent path + local slug

## Conclusion Checkpoint (Optional)
<!-- Gate: Quality Strategy → Review. Verify before requesting review. -->

**Assessment:** Ready for review

- [x] Decision justified (Y-statement or equivalent)
- [x] Consequences include positive, negative, and neutral outcomes
- [x] Quality Strategy reviewed — relevant items checked, irrelevant struck through
- [x] Links to related ADRs populated

**Pre-review notes:** The max 50-char slug limit per segment is a starting constraint. It can be adjusted if real-world usage shows it's too restrictive.

---

## Comments

### Draft Worksheet
<!-- Captures original intent and workflow calibration. -->

**Framing:**
Open question from the roadmap. Need an ID scheme that works for source-control-friendly files, supports cross-procedure references, and enables the Policy compiler to determine scope.

**Tolerance:**
- Risk: Low — prefer proven identification patterns
- Change: Medium — this is a foundational choice that's hard to reverse
- Improvisation: Low — the roadmap lists the candidates

**Uncertainty:**
- Certain: IDs must be unique within a Skill, support cross-references, be source-control friendly
- Uncertain: optimal slug length limits, whether reparenting is common enough to worry about

**Options:**
- Target count: 3
- [ ] Explore additional options beyond candidates listed below

**Candidates:**
- UUIDs — universally unique, opaque
- Namespaced/hierarchical paths — self-documenting, scope-aware
- Human-readable slugs — simple, parent-scoped

<!-- Review cycle 1 — 2026-04-08 — Verdict: Accept. Polish pass applied. -->
