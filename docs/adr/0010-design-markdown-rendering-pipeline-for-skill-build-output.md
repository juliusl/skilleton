# 10. Design Markdown rendering pipeline for skill build output

Date: 2026-04-08
Status: Partially Superseded by ADR-0011
Last Updated: 2026-04-08
Links: ADR-0002, ADR-0006, ADR-0009

## Context

The `skilleton build` command (ADR-0009) renders a Skill as Markdown to stdout. The roadmap requires "correct ordering (policies before procedures)" — policies must appear before procedures in the output so that agents encounter constraints before instructions.

The existing type system provides a tree structure: Skill → (metadata, policies, criteria, procedures), Procedure → (meta, steps, entrance/exit criteria, policies, criteria), Step → (meta, tasks, completion criteria, policies, criteria). The renderer must walk this tree and emit Markdown with a deterministic, policy-first ordering at every level.

**Decision drivers:**
- Policy-before-procedure ordering at every hierarchy level
- Deterministic output (same input always produces same output)
- Readability — the Markdown should be usable as a standalone skill document
- Extensibility — new item types or sections should be addable without restructuring
- No external template engine dependency — keep the rendering self-contained

## Options

### Option 1: Hierarchical walk with inline policy-first ordering

Implement a `render` module that walks the Skill tree depth-first, emitting Markdown at each level with a fixed section order:

1. **Skill level:** title → metadata → policies → criteria → procedures
2. **Procedure level:** heading → policies → criteria → entrance criteria → steps → exit criteria
3. **Step level:** heading → policies → criteria → tasks → completion criteria

Each node implements a `render_markdown(&self, depth: usize) -> String` method (or similar trait). The `depth` parameter controls heading levels (`#`, `##`, `###`, etc.).

**Strengths:**
- Simple, self-contained — no external dependencies
- Ordering is explicit in code — easy to audit and test
- Each type owns its rendering logic
- Deterministic — traversal order is fixed by the code

**Weaknesses:**
- Rendering logic mixed with domain types (if using methods) or duplicated in a separate module (if using free functions)
- Heading depth management requires careful tracking

### Option 2: Two-pass rendering (collect then emit)

First pass: walk the tree and collect all items into ordered buckets (policies, criteria, procedures). Second pass: emit Markdown from the buckets.

**Strengths:**
- Clean separation between collection and rendering
- Easy to implement global transformations (e.g., cross-reference linking)

**Weaknesses:**
- Loses hierarchical context — policies at different levels get mixed
- More complex for nested structures where policy ordering must be per-scope
- Requires reconstructing the hierarchy during the emit phase

### Option 3: Template engine (Tera/Handlebars)

Use a template engine to define the Markdown output format as a template, then populate it from the Skill data.

**Strengths:**
- Template is human-editable — output format can change without code changes
- Familiar Jinja2-like syntax (Tera)

**Weaknesses:**
- Adds an external dependency for a straightforward rendering task
- Template logic for recursive tree traversal is awkward
- Harder to test — need to test template + data separately
- Overkill for a fixed output format

## Evaluation Checkpoint (Optional)
<!-- Gate: Options → Decision. Agent assesses and recommends. -->

**Assessment:** Proceed

- [x] All options evaluated at comparable depth
- [x] Decision drivers are defined and referenced in option analysis
- [x] No unacknowledged experimentation gaps (ADR-0022 tolerance check)

**Validation needs:** None — all options are standard rendering patterns.

## Decision

In the context of rendering skills as Markdown, facing the need for policy-before-procedure ordering at every hierarchy level, we chose **hierarchical walk with inline policy-first ordering** (Option 1) over two-pass rendering and template engines to achieve simple, deterministic, auditable output with no external dependencies, accepting that rendering logic lives in a dedicated module rather than being template-driven.

**Rendering module:** A new `src/render.rs` module with a public `render_skill(skill: &Skill) -> String` function. The function walks the Skill tree and emits Markdown with this section order at each level:

**Skill level:**
```markdown
# {skill.metadata.name}

{skill.metadata.description}

## Policies

{for each skill.policies → render_policy}

## Criteria

{for each skill.criteria → render_criterion}

## Procedures

{for each skill.procedures → render_procedure}
```

**Procedure level:**
```markdown
### {procedure.meta.id} — Procedure

**Policies:**
{for each procedure.policies → render_policy}

**Criteria:**
{for each procedure.criteria → render_criterion}

**Entrance Criteria:**
{for each procedure.entrance_criteria → render_criterion_ref}

{for each procedure.steps → render_step}

**Exit Criteria:**
{for each procedure.exit_criteria → render_criterion_ref}
```

**Step level:**
```markdown
#### {step.meta.id} — Step

**Policies:**
{for each step.policies → render_policy}

**Criteria:**
{for each step.criteria → render_criterion}

**Tasks:**
{for each step.tasks → render_task}

**Completion Criteria:**
{for each step.completion_criteria → render_criterion_ref}
```

**Task level:**
```markdown
- `{task.meta.id}` **{task.subject}**: {task.action}
  {if task.invokes} (invokes: {task.invokes}) {end}
```

**Rendering rules by type:**

- **`render_policy(policy)`** — `> **{policy.meta.id}**: {policy.text}` (blockquote for visual weight)
- **`render_criterion(criterion)`** — `- **{criterion.meta.id}**: {criterion.description}` (list item with full description — `Criterion` has a `description` field)
- **`render_criterion_ref(ref)`** — `- {ref.0}` (list item, ID only — `CriterionRef` is an `ItemId` wrapper with no description field)
- **`render_conditions(meta)`** — if `meta.conditions` is non-empty, emit `*Conditions: {ref1}, {ref2}, ...*` as an italic annotation line immediately after the item's heading. Items with empty `conditions` (the common case) produce no annotation.

**Intentional omissions:**
- `Policy.compatible_with` — conflict-detection metadata consumed by the validator (ADR-0007 §4), not meaningful in rendered output. Omitted.

**Empty section handling:** Sections with no items are omitted entirely — no empty headings. This produces clean output for skills that don't use criteria or policies at every level.

**Heading depth:** The skill title uses `#`, skill-level sections (Policies, Criteria, Procedures) use `##`, individual procedures use `###`, steps use `####`, tasks are list items. This keeps the hierarchy readable without exceeding four heading levels.

**Deterministic ordering:** Items within each section are emitted in their definition order (the order they appear in the `Vec` fields). The storage layer (SkillLoader) already sorts procedure files by filename, guaranteeing stable ordering.

## Consequences

**Positive:**
- Policy-before-procedure ordering is enforced by the code structure at every hierarchy level
- No external dependencies — the render module uses only `std::fmt::Write`
- Output is deterministic — same Skill struct always produces same Markdown
- Empty sections are omitted, producing clean output for simple skills

**Negative:**
- Changing the output format requires code changes, not template edits
- The render module needs updating when new item types are added to the type system

**Neutral:**
- The render module is decoupled from the CLI — it can be used as a library function
- Conditions on items (via `ItemMeta.conditions`) are rendered as italic annotations after the item heading when non-empty; items with no conditions produce no output

## Quality Strategy

- ~~Introduces major semantic changes~~
- [x] Introduces minor semantic changes
- ~~Fuzz testing~~
- [x] Unit testing
- ~~Load testing~~
- ~~Performance testing~~
- [x] Backwards Compatible
- ~~Integration tests~~
- [x] Tooling
- ~~User documentation~~

### Additional Quality Concerns

- Unit tests must verify policy-before-procedure ordering at every hierarchy level
- Tests should verify empty section omission behavior
- Round-trip property: build output should be human-readable as a standalone skill document

## Conclusion Checkpoint (Optional)
<!-- Gate: Quality Strategy → Review. Verify before requesting review. -->

**Assessment:** Ready for review

- [x] Decision justified (Y-statement or equivalent)
- [x] Consequences include positive, negative, and neutral outcomes
- [x] Quality Strategy reviewed — relevant items checked, irrelevant struck through
- [x] Links to related ADRs populated

**Pre-review notes:** The Markdown format is intentionally minimal — it renders the structural hierarchy, not a polished user document. Formatting refinements can be iterated on after the pipeline is established.

---

## Comments

### Draft Worksheet
<!-- Captures original intent and workflow calibration. -->

**Framing:**
The `build` command needs to output Markdown with policies before procedures. This requires a rendering pipeline that walks the Skill type hierarchy.

**Tolerance:**
- Risk: Low — straightforward tree-to-text rendering
- Change: Low — additive module
- Improvisation: Low — follow the type hierarchy

**Uncertainty:**
Certain: policy-before-procedure ordering, hierarchical output.
Uncertain: exact Markdown heading levels and formatting details — these can be refined during implementation.

**Options:**
- Target count: 2-3
- [ ] Explore additional options beyond candidates listed below

**Candidates:**
- Hierarchical walk — direct tree traversal with fixed ordering
- Two-pass — collect then emit
- Template engine — external template rendering

### Revision Q&A — Round 1

**F1 (H) — Heading level contradiction:** *Addressed.* Prose claimed "procedures use `##`, steps use `###`" but templates showed procedures at `###` and steps at `####`. Fixed prose to match the templates: skill title `#`, skill-level sections `##`, procedures `###`, steps `####`.

**F2 (H) — `ItemMeta.conditions` rendering unspecified:** *Addressed.* Added `render_conditions(meta)` rule: non-empty conditions emit an italic annotation line (`*Conditions: ...*`) after the item heading. Updated neutral consequence to match the spec.

**F3 (M) — `Criterion` vs `CriterionRef` undifferentiated:** *Addressed.* Added differentiated rendering rules: `render_criterion` emits ID + description (full `Criterion`), `render_criterion_ref` emits ID only (`CriterionRef` is an `ItemId` wrapper). Updated all template references to use the appropriate renderer.

**F4 (M) — Heading collision at `####`:** *Addressed.* Removed the redundant `#### Steps` section heading from the procedure template. Steps follow naturally after entrance criteria, and each step already has its own `####` heading — the intermediate heading was both redundant and caused an h4 collision.

**F5 (L) — `Policy.compatible_with` not addressed:** *Addressed.* Added to "Intentional omissions" section: `compatible_with` is conflict-detection metadata consumed by the validator (ADR-0007 §4), not meaningful in rendered output.

**F6 (L) — `Task.meta.id` not rendered:** *Addressed.* Added `{task.meta.id}` (code-formatted) as a prefix in the task list item template, consistent with ID rendering at every other hierarchy level.

**F7 (L) — Quality Strategy "Integration tests" struck through:** *Rejected — pragmatic staging.* The render module is a pure function (`&Skill → String`), fully testable with unit tests. Integration tests for the full `skilleton build` pipeline (load from disk → render) belong in ADR-0009's implementation scope, not this ADR's quality strategy.

<!-- Review cycle 1 — 2026-04-08 — Verdict: Revise. Findings: 7 (2H, 2M, 3L). Addressed: 6. Rejected: 1 (F7). -->
