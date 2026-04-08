# 6. Define file organization for skill definitions

Date: 2026-04-08
Status: Ready
Last Updated: 2026-04-08
Links: ADR-0002, ADR-0003, ADR-0004, ADR-0005

## Context

The roadmap requires "a DSL enables procedural-agent analysis." This ADR defines how skilleton project files are organized on disk — how the type hierarchy (ADR-0002) maps to files and directories, and whether users write TOML directly or use a custom syntax.

This decision depends on the serialization format (ADR-0005, TOML; currently Proposed — this decision assumes ADR-0005 acceptance). The file organization determines diff granularity (one large file vs. many small files), merge conflict frequency, and how agents navigate a skill's structure.

The type hierarchy is: Skill → Procedure → Step → Task, with Policy and Criterion attached as singletons at multiple levels. ItemId paths (ADR-0003) encode this hierarchy — the directory layout should mirror it.

Requirements from the roadmap:
- **Source-control friendly** — small files produce targeted diffs and fewer merge conflicts
- **Agent navigable** — agents should locate a specific procedure or policy by file path without parsing the entire skill
- **Procedural-agent analysis** — the DSL must be parseable for static analysis (policy conflict detection, DAG validation per ADR-0004)
- **MVP simplicity** — avoid building a custom parser when standard formats suffice

Decision drivers (ranked):
1. **Diff granularity** — changes to one procedure should not affect other procedures' files
2. **Agent navigability** — path-based lookup without full-skill parsing
3. **Parser complexity** — building and maintaining a custom parser is significant effort
4. **Convention clarity** — developers and agents must understand the layout without external documentation

## Options

### Option A: Single-file TOML

One `.toml` file per Skill containing all procedures, steps, tasks, and policies inline.

```
skills/
  onboarding.toml       # entire skill definition
```

```toml
[skill]
id = "skill:onboarding"
name = "Onboarding"

[[skill.policies]]
id = "policy:no-plaintext"
text = "Never store passwords in plaintext"

[[skill.procedures]]
id = "procedure:welcome"

[[skill.procedures.steps]]
id = "step:greet"

[[skill.procedures.steps.tasks]]
id = "task:send-message"
subject = "User"
action = "Send greeting message"
```

- **Pro:** Simplest model — one file, one skill, no directory conventions to learn
- **Pro:** Atomic operations — reading or writing a skill is a single file operation
- **Pro:** No cross-file reference resolution needed
- **Con:** Large skills produce large files — a skill with 10 procedures and 50+ tasks creates a file too large for meaningful diffs
- **Con:** Merge conflicts — two developers editing different procedures in the same file will conflict
- **Con:** 4 levels of TOML nesting (`[[skill.procedures.steps.tasks]]`) is verbose and hard to read

### Option B: Directory-based with TOML files

One directory per Skill. Each procedure gets its own TOML file. Skill-level metadata and policies live in a root file.

```
skills/
  onboarding/
    skill.toml                      # skill metadata + skill-level policies
    procedures/
      welcome.toml                  # procedure with steps, tasks, policies
      auth-flow.toml                # another procedure
```

Each procedure file contains its steps, tasks, and procedure-level policies:

```toml
[procedure]
id = "procedure:welcome"

[[procedure.policies]]
id = "policy:greet-first"
text = "Always greet before asking for credentials"

[[procedure.steps]]
id = "step:greet"

[[procedure.steps.tasks]]
id = "task:send-message"
subject = "User"
action = "Send greeting message"
```

- **Pro:** Targeted diffs — editing one procedure only affects that procedure's file
- **Pro:** Merge-friendly — developers working on different procedures never conflict
- **Pro:** Agent navigability — an agent can read `procedures/welcome.toml` without parsing the entire skill
- **Pro:** Max TOML key depth is 3 segments within any file (`[[procedure.steps.tasks]]`) — manageable compared to Option A's 4-segment depth
- **Pro:** Directory structure mirrors the type hierarchy — conventions are self-documenting
- **Con:** Multiple files require assembly — loading a full skill means reading the directory tree
- **Con:** Cross-file references between procedures require path resolution at load time
- **Con:** More files to manage — rename/move operations must update the directory structure

### Option C: Custom DSL with dedicated parser

Define a purpose-built `.skilleton` syntax optimized for readability and agent analysis.

```skilleton
skill onboarding {
  name: "Onboarding"

  policy no-plaintext {
    "Never store passwords in plaintext"
  }

  procedure welcome {
    step greet {
      task send-message {
        subject: "User"
        action: "Send greeting message"
      }
    }
  }
}
```

- **Pro:** Maximum expressiveness — syntax is tailored to the domain
- **Pro:** Compact — less syntactic overhead than TOML for nested structures
- **Pro:** Natural visual hierarchy — nesting is indentation-based, not encoded in key paths
- **Con:** Requires building and maintaining a parser — significant engineering effort for MVP
- **Con:** No ecosystem tooling — no existing linters, formatters, or IDE plugins
- **Con:** Agents need a custom parser or the skilleton binary to read files — cannot use generic TOML/YAML parsers
- **Con:** Two formats to maintain if TOML is kept for Cargo-style config files alongside the DSL

## Evaluation Checkpoint (Optional)
<!-- Gate: Options → Decision. Agent assesses and recommends. -->

**Assessment:** Proceed

- [x] All options evaluated at comparable depth
- [x] Decision drivers are defined and referenced in option analysis
- [x] No unacknowledged experimentation gaps (ADR-0022 tolerance check)

**Validation needs:** None for this decision — directory-based file organization is a well-established pattern (Cargo workspaces, Terraform modules, Kubernetes manifests). The split-granularity question (procedure-level vs. step-level files) is acknowledged as uncertain (see Worksheet), but procedure-level splitting is a safe starting point — if it proves insufficient, step-level splitting can be introduced without invalidating the directory-based approach.

## Decision

In the context of **organizing skilleton project files on disk**, facing **the need for source-control-friendly diffs, agent-navigable structure, and MVP-appropriate parser complexity**, we chose **Option B (directory-based with TOML files)** over **single-file TOML (A) and a custom DSL (C)** to achieve **targeted diffs per procedure, path-based agent navigation, and zero custom parser overhead**, accepting **the need for multi-file assembly at load time and directory-level rename coordination**.

Concrete commitments:
1. Each Skill is a directory under a configurable root (default: `skills/`)
2. `skill.toml` at the directory root contains skill metadata (`SkillMeta`), skill-level policies, and skill-level criteria
3. `procedures/` subdirectory contains one `.toml` file per Procedure, named by the procedure's slug
4. Each procedure file contains the procedure's steps, tasks, and procedure-level policies and criteria — max TOML key depth of 3 segments (`[[procedure.steps.tasks]]`)
5. Skill-level policies and criteria live in `skill.toml` under `[[skill.policies]]` and `[[skill.criteria]]`
6. Step-level and task-level policies and criteria are inline within their parent procedure file
7. File and directory names derive from the slug portion of the ItemId — `skill:onboarding` maps to the `onboarding/` directory, `procedure:auth-flow` maps to `procedures/auth-flow.toml`
8. Loading a Skill reads `skill.toml` then iterates `procedures/*.toml` and assembles the full type tree
9. No custom parser is built — all files are standard TOML parsed by the `toml` crate (ADR-0005)

## Consequences

- **Positive:** Diffs are scoped to the procedure being edited — adding a step to `welcome.toml` produces a diff confined to that file
- **Positive:** Agents locate a procedure by path (`skills/onboarding/procedures/welcome.toml`) without parsing other files
- **Positive:** TOML key depth is capped at 3 segments within any file (`[[procedure.steps.tasks]]`), avoiding the 4-segment verbosity flagged in ADR-0005
- **Positive:** No custom parser needed — the `toml` crate handles all file parsing, reducing implementation scope for MVP
- **Negative:** Loading a skill requires reading multiple files and assembling the hierarchy in memory. A `SkillLoader` must handle directory traversal, file parsing, and cross-file reference validation.
- **Negative:** Renaming a procedure slug requires renaming the file and updating all cross-procedure references (ADR-0004 `invokes` fields). Tooling to automate renames is deferred to a later milestone.
- **Negative:** A procedure with many steps and tasks produces a single large file, since all steps and tasks are inline. If procedures grow large enough to reintroduce the diff-granularity problem that eliminated Option A, step-level file splitting would be needed — deferred to a future ADR if procedure files prove unwieldy during the first 3 skill implementations.
- **Neutral:** The directory structure is one convention, not the only possible convention. A future ADR could introduce a custom DSL (Option C) as a layer on top of the TOML files, though it would likely require revisiting some directory and naming conventions.

## Quality Strategy

- [x] Introduces major semantic changes
- [ ] ~~Introduces minor semantic changes~~
- [ ] ~~Fuzz testing~~
- [x] Unit testing
- [ ] ~~Load testing~~
- [ ] ~~Performance testing~~
- [x] Backwards Compatible
- [x] Integration tests
- [x] Tooling
- [ ] ~~User documentation~~

### Additional Quality Concerns

Unit tests should verify:
- `SkillLoader` assembles a multi-file skill correctly
- Missing `skill.toml` produces a clear error
- Procedure slug ↔ filename mapping is consistent
- Empty `procedures/` directory produces a valid skill with zero procedures

Integration tests should verify:
- Round-trip: write a Skill to the directory structure, read it back, compare equality
- Cross-procedure `invokes` references resolve correctly across files

Tooling: implement `SkillLoader` and `SkillWriter` modules. Wire into CLI commands for `skilleton init` (bootstrap directory) and `skilleton load` (parse and validate).

## Conclusion Checkpoint (Optional)
<!-- Gate: Quality Strategy → Review. Verify before requesting review. -->

**Assessment:** Ready for review

- [x] Decision justified (Y-statement or equivalent)
- [x] Consequences include positive, negative, and neutral outcomes
- [x] Quality Strategy reviewed — relevant items checked, irrelevant struck through
- [x] Links to related ADRs populated

**Pre-review notes:** This ADR pairs with ADR-0005. The directory-based approach directly mitigates TOML's nesting verbosity by capping key depth at 3 segments per file. Review both together.

---

## Comments

### Draft Worksheet
<!-- Captures original intent and workflow calibration. -->

**Framing:**
The roadmap requires "a DSL enables procedural-agent analysis." Need to decide how skill definitions are organized on disk — file layout and whether TOML is used directly or wrapped in a custom syntax. Depends on ADR-0005 (TOML as format). Leaning toward directory-based TOML — each procedure in its own file, directory structure mirrors the type hierarchy, no custom parser needed for MVP.

**Tolerance:**
- Risk: Low — prefer established file organization patterns
- Change: Medium — this defines the project file layout going forward
- Improvisation: Low — the options are well-known patterns (single-file, directory-based, custom DSL)

**Uncertainty:**
- Certain: TOML is the serialization format (ADR-0005)
- Certain: the hierarchy has 4 levels (ADR-0002)
- Uncertain: whether step-level file splitting is needed or if procedure-level splitting is sufficient
- Uncertain: optimal naming conventions for edge cases (slugs with special characters)

**Options:**
- Target count: 3
- [ ] Explore additional options beyond candidates listed below

**Candidates:**
- Single-file TOML — one file per skill, all inline
- Directory-based TOML — directory per skill, file per procedure
- Custom DSL — purpose-built syntax with dedicated parser

### Revision — Review Findings (V-1 through V-5b)

| # | Pri | Finding | Disposition | Rationale |
|---|-----|---------|-------------|-----------|
| 1 | H | ADR-0004 missing from Links header | **Address** | Factual omission — ADR-0004 is referenced in Context (DAG validation) and Consequences (invokes fields) but was absent from Links metadata. Added. |
| 2 | H | Dependency on unaccepted ADR-0005 unacknowledged | **Address** | Procedural accuracy — ADR-0005 is Proposed, not Accepted. Added inline qualification in Context: "currently Proposed — this decision assumes ADR-0005 acceptance." |
| 3 | M | "2 levels of TOML nesting" is ambiguous | **Address** | "Levels" counted from the root table, which is non-obvious. Restated as "3 dotted segments" with concrete example (`[[procedure.steps.tasks]]`) in Option B pro, commitment 4, and positive consequence. |
| 4 | M | Missing consequence: large procedures → unwieldy files | **Address** | Real known risk — the Worksheet flags uncertainty about procedure-level vs. step-level splitting. Added as negative consequence deferred to future ADR, with concrete revisit trigger: "first 3 skill implementations." |
| 5 | M | "Validation needs: None" contradicts worksheet uncertainty | **Address** | Qualified to "None for this decision" and acknowledged split-granularity uncertainty explicitly. Procedure-level splitting is a safe starting point; the uncertainty doesn't gate this decision. |
| 6 | L | Neutral consequence re: future DSL is optimistic | **Address** | "Without breaking this decision" overstated compatibility. Softened to "would likely require revisiting some directory and naming conventions." |
| 7 | L | Title promises "DSL syntax" but none is designed | **Address** | Scope mismatch — the decision is about file organization, not DSL design. Retitled to "Define file organization for skill definitions." The DSL was evaluated (Option C) and rejected; the title implied it was designed. |
| 8 | L | Criterion placement in file layout unspecified | **Address** | Criteria parallel Policies in the ADR-0002 type hierarchy but were absent from every commitment. Added "and criteria" to commitments 2, 4, 5, and 6. |
| 9 | L | Skill directory ↔ ItemId slug mapping underspecified | **Address** | Commitment 7 only gave the procedure example. Extended to cover skill-directory mapping: `skill:onboarding` → `onboarding/`. Inferrable, but should be explicit. |
