# 11. Add template-based Markdown rendering to build command

Date: 2026-04-08
Status: Planned
Last Updated: 2026-04-08
Links: ADR-0009, ADR-0010

## Context

ADR-0010 chose a hardcoded hierarchical walk in `render_skill()` to emit Markdown with policy-first ordering. That decision explicitly rejected external template engines as "overkill for a fixed output format." The hardcoded approach worked for M3's goal of proving the build pipeline, but ADR-0010 noted a consequence: "Changing the output format requires code changes, not template edits."

Milestone 4 requires template-based rendering — the build command should accept a Markdown template file, letting users customize output layout without modifying Rust code. The user has expressed a preference for Mustache-style templates.

The current `render_skill()` implementation (`src/render.rs`) is ~185 lines of `write!`/`writeln!` calls with inline conditionals for empty-section omission. This logic maps directly to a template: sections map to conditional blocks, iteration maps to list sections, and the fixed ordering is expressed by template structure rather than code flow.

**Decision drivers:**
- User customizability — users control output format via template files
- Policy-first ordering — must remain enforceable regardless of template content
- Backwards compatibility — default output must match current `render_skill()` behavior
- Dependency weight — prefer lightweight crates
- Simplicity — template format should be obvious to read and edit

## Options

### Option 1: Mustache via `ramhorns` crate

Use `ramhorns` — a zero-copy, compile-time Mustache template engine for Rust. The Skill data model is serialized to a template context, and the template defines the Markdown layout.

**Template structure:**
```mustache
# {{name}}
{{#description}}

{{description}}
{{/description}}
{{#has_policies}}

## Policies

{{#policies}}
> **{{id}}**: {{text}}

{{/policies}}
{{/has_policies}}
{{#has_procedures}}

## Procedures
{{#procedures}}

### {{id}} — Procedure
{{#has_policies}}

#### Policies

{{#policies}}
> **{{id}}**: {{text}}

{{/policies}}
{{/has_policies}}
{{#has_steps}}

{{#steps}}
#### {{id}} — Step

{{#has_tasks}}
{{#tasks}}
- [ ] **{{id}}**: {{text}}
{{/tasks}}
{{/has_tasks}}
{{/steps}}
{{/has_steps}}
{{/procedures}}
{{/has_procedures}}
```

**CLI integration:**
- `skilleton build <path>` — uses a built-in default template (compiled into the binary)
- `skilleton build <path> --template <file>` — uses a user-provided template file

**Strengths:**
- Logic-less — templates cannot execute arbitrary code; safe for user-provided files
- Lightweight — `ramhorns` is a small, zero-copy crate
- User preference — matches the stated candidate
- Natural fit — Mustache's conditional sections (`{{#list}}...{{/list}}`) handle empty-section omission without helper logic

**Weaknesses:**
- No built-in iteration index or conditional logic beyond truthy/falsy — complex formatting (e.g., "render separator only between items") requires data preprocessing
- Partials (`{{>partial}}`) need a partition loader; multi-file templates add distribution complexity
- Adds an external dependency

### Option 2: Handlebars via `handlebars-rust` crate

Use `handlebars-rust` — a superset of Mustache with helpers, partials, and block expressions.

**Strengths:**
- More expressive than Mustache — built-in helpers (`#if`, `#each`, `#with`, `@index`)
- Mature crate with wide adoption
- Partials are first-class

**Weaknesses:**
- Heavier dependency (~3x the size of ramhorns)
- Helpers introduce logic into templates — harder to audit user-provided templates
- Expressiveness exceeds the requirements — the rendering model is data-in/text-out

### Option 3: Keep hardcoded layout with override hooks

Extend the current `render_skill()` with hook points — callback functions or configuration flags that control specific sections. No template engine, but limited customization.

**Strengths:**
- No new dependency
- Preserves ADR-0010's simplicity rationale

**Weaknesses:**
- Limited customization — users can toggle sections but not change layout
- Hook API grows with each customization request
- Does not satisfy the roadmap requirement for template-file-based rendering

## Evaluation Checkpoint (Optional)
<!-- Gate: Options → Decision. Agent assesses and recommends. -->

**Assessment:** Proceed

- [x] All options evaluated at comparable depth
- [x] Decision drivers are defined and referenced in option analysis
- [x] No unacknowledged experimentation gaps (ADR-0022 tolerance check)

**Validation needs:** None — all options use established rendering patterns. Mustache semantics are well-documented and the data model is small enough that template context construction is straightforward.

## Decision

In the context of the `skilleton build` command, facing the need for user-customizable Markdown output, we chose **Mustache templates via `ramhorns`** (Option 1) over Handlebars and hardcoded hooks to achieve lightweight, logic-less, user-customizable rendering with minimal dependency weight, accepting that complex formatting requires data preprocessing rather than template logic.

This decision **supersedes** ADR-0010's prohibition on external template engines. ADR-0010's core contribution — policy-first ordering at every hierarchy level — is preserved: the default template encodes this ordering, and the template context provides data in a structure that makes policy-first natural. Upon acceptance, ADR-0010's status becomes "Partially Superseded by ADR-0011" — its ordering model remains active; only the template engine prohibition is superseded.

**Implementation design:**

1. **Template context struct** — A `RenderContext` struct derived from `Skill` that flattens the hierarchy into template-friendly fields. Each level (skill, procedure, step) has boolean `has_*` fields for conditional sections and pre-formatted string fields where needed.

2. **Default template** — A built-in template compiled as a `const &str` that reproduces the current `render_skill()` output exactly. This is the backwards-compatibility baseline.

3. **CLI integration** — Add `--template <path>` optional argument to the `build` subcommand. This extends the `Build` variant defined in ADR-0009 (currently `Build { path: PathBuf }`). When omitted, use the default template. When provided, read the file and use it as the rendering template.

4. **`render_skill()` refactor** — Replace the current hardcoded implementation with:
   ```rust
   pub fn render_skill(skill: &Skill) -> String {
       render_skill_with_template(skill, DEFAULT_TEMPLATE)
   }

   pub fn render_skill_with_template(skill: &Skill, template: &str) -> String {
       let context = RenderContext::from(skill);
       // Apply template via ramhorns
   }
   ```

5. **Partials strategy** — For M4, use a single-file template with inline rendering for all hierarchy levels. Partial support (multi-file templates) is deferred to M5 if needed.

## Consequences

**Positive:**
- Users can customize build output without modifying Rust code
- Default template is designed to reproduce current `render_skill()` output exactly — verified by byte-for-byte backward-compatibility test
- Logic-less templates are safe for user-provided files — no code execution risk
- `ramhorns` is lightweight with zero-copy parsing

**Negative:**
- Adds `ramhorns` as a runtime dependency
- Complex formatting (e.g., conditional separators) requires `RenderContext` preprocessing rather than template logic
- Template errors are runtime failures, not compile-time — need clear error messages
- Policy-first ordering shifts from code-enforced to template-expressed — custom templates can deviate from ADR-0010's ordering guarantees. Accepted because the default template preserves ordering for the common case; custom templates are a power-user opt-in.

**Neutral:**
- The default template serves as documentation of the output format
- Policy-first ordering is expressed by the default template structure rather than code flow

## Quality Strategy

- ~~Introduces major semantic changes~~
- [x] Introduces minor semantic changes
- ~~Fuzz testing~~
- [x] Unit testing
- ~~Load testing~~
- ~~Performance testing~~
- [x] Backwards Compatible
- [x] Integration tests
- ~~Tooling~~
- ~~User documentation~~

### Additional Quality Concerns

- **Backwards compatibility test** — default template output must match current `render_skill()` output byte-for-byte for the onboarding fixture
- **Custom template test** — verify `--template` flag reads and applies a user-provided template
- **Error handling** — invalid template syntax should produce clear error messages with line/column context
- **Empty section handling** — verify that the template context correctly enables empty-section omission

## Conclusion Checkpoint (Optional)
<!-- Gate: Quality Strategy → Review. Verify before requesting review. -->

**Assessment:** Ready for review

- [x] Decision justified (Y-statement or equivalent)
- [x] Consequences include positive, negative, and neutral outcomes
- [x] Quality Strategy reviewed — relevant items checked, irrelevant struck through
- [x] Links to related ADRs populated

**Pre-review notes:** This supersedes ADR-0010's template engine prohibition while preserving its core ordering contribution. The default template ensures backwards compatibility.

---

## Comments

### Draft Worksheet
<!-- Captures original intent and workflow calibration. -->

**Framing:**
The build command's hardcoded Markdown layout cannot be customized without code changes. The roadmap requires template-based rendering. Mustache is the user's preferred template format. ADR-0010 rejected template engines but acknowledged the layout-change cost.

**Tolerance:**
- Risk: Low — template rendering is a well-understood pattern
- Change: Medium — reverses ADR-0010's template engine prohibition
- Improvisation: Low — direction is clear (mustache, --template flag)

**Uncertainty:**
Certain: need template-based rendering, mustache preference, backwards compatibility with current output.
Uncertain: partials strategy (single-file vs multi-file templates), exact RenderContext shape.

**Options:**
- Target count: 2-3
- [ ] Explore additional options beyond candidates listed below

**Candidates:**
- Mustache via ramhorns — user preference, lightweight, logic-less
- Handlebars — superset of mustache, more powerful
- Keep hardcoded with hooks — no dependency, limited customization

### Revision Q&A — Round 1

**F1 (M) — Policy-first ordering guarantee weakened, misclassified as neutral:** *Addressed.* Moved the ordering-shift consequence from Neutral to Negative with an explanation of why the tradeoff is acceptable (default template preserves ordering; custom templates are power-user opt-in). Retained a narrower neutral consequence noting the default template expresses ordering via structure rather than code.

**F2 (M) — `ramhorns` crate fitness unvalidated:** *Rejected — staging.* The `RenderContext` struct (Decision item 1) explicitly flattens the hierarchy into template-friendly fields, which addresses the nesting concern by design — templates won't traverse deeply nested structures. Verifying that `ramhorns` works with the concrete data model is implementation evidence produced by the first implementation task, not a decision-stage validation need. The partial inconsistency in the template example is addressed separately in F3.

**F3 (M) — Option 1 template example contradicts single-file decision:** *Addressed.* Replaced the `{{>procedure_body}}` partial in the Option 1 template example with inline rendering showing the procedure → step → task hierarchy. The template example now matches the Decision's single-file approach (item 5).

**F4 (L) — ADR-0010 supersession scope ambiguous:** *Addressed.* Added a sentence to the Decision specifying that ADR-0010's status becomes "Partially Superseded by ADR-0011" — its ordering model remains active; only the template engine prohibition is superseded.

**F5 (L) — Backwards compatibility stated as fact, not goal:** *Addressed.* Rephrased the positive consequence from a certainty ("produces identical output") to a design goal verified by testing ("is designed to reproduce... verified by byte-for-byte backward-compatibility test").

**F6 (L) — ADR-0009 update not noted:** *Addressed.* Added a note under CLI integration (Decision item 3) that the `--template` flag extends the `Build` variant defined in ADR-0009 (currently `Build { path: PathBuf }`).

**F7 (L) — Decision drivers not explicitly prioritized:** *Rejected — the Y-statement already communicates priority through its goal ordering ("lightweight, logic-less, user-customizable rendering with minimal dependency weight"). Adding explicit weighting to the drivers list is process overhead for a clear decision.*

<!-- Review cycle 1 — 2026-04-08 — Verdict: Revise. Findings: 7 (3M, 4L). Addressed: 5. Rejected: 2. -->
<!-- Review cycle 2 — 2026-04-08 — Verdict: Accept. All findings resolved, rejections justified, no regressions. -->
