# 9. Select CLI framework and define command structure

Date: 2026-04-08
Status: Accepted
Last Updated: 2026-04-08
Links: ADR-0005, ADR-0006, ADR-0007, ADR-0008, ADR-0010

## Context

Skilleton is a Rust library for building and modifying agent skills. It provides core types (Skill, Procedure, Step, Task, Policy, Criterion), file-based storage (SkillLoader, SkillWriter, FileRepository), reference validation, and policy conflict detection. All of this exists as a library crate — there is no binary target.

Milestone 3 requires a CLI binary that exposes three operations:
- **init** — scaffold a new skill directory layout
- **check** — validate a skill's references, criteria, type prefixes, and policy conflicts
- **build** — render a skill as Markdown with policies ordered before procedures

The CLI is the first user-facing interface. Its design determines how users interact with skilleton and how the library's capabilities are composed into workflows.

**Decision drivers** (listed in priority order):
- Subcommand support with typed arguments
- Derive macro ergonomics for defining commands declaratively
- Help text and error message quality
- Ecosystem adoption and long-term maintenance
- Minimal dependency footprint for a focused CLI

## Options

### Option 1: clap (derive)

Use `clap` with its derive macros to define the CLI structure. clap is the de facto standard Rust CLI framework with broad ecosystem adoption.

**Command structure** (framework-independent — same interface regardless of parser choice):
```
skilleton init <path>          # scaffold skill directory
skilleton check <path>         # validate skill
skilleton build <path>         # render Markdown
```

Each subcommand maps to a struct with `#[derive(Parser)]`. Arguments are typed fields with doc comments generating help text.

**Strengths:**
- Most widely used Rust CLI framework — de facto standard with the broadest ecosystem integration
- Derive macros produce clean, declarative command definitions
- Built-in help, version, completions, and error formatting
- Mature subcommand support with nested commands
- Extensive documentation and community examples

**Weaknesses:**
- Heavy compile-time dependency (proc macros, multiple features)
- Large dependency tree for a simple 3-subcommand CLI

### Option 2: argh

Use Google's `argh` crate — a derive-based CLI parser designed for minimal binary size and compile time.

**Strengths:**
- Zero proc-macro overhead at runtime
- Small dependency tree
- Clean derive syntax

**Weaknesses:**
- Limited ecosystem adoption compared to clap
- Fewer features (no completions, limited error formatting)
- Less active maintenance
- Missing some ergonomics (custom value parsers, value hints)

### Option 3: Manual argument parsing (std::env::args)

Parse arguments manually using the standard library. No external dependencies. Included as a zero-dependency baseline — a genuine consideration given this CLI has only three subcommands with simple positional arguments.

**Strengths:**
- Zero dependencies
- Full control over parsing logic
- Fastest compile time

**Weaknesses:**
- No automatic help generation
- Manual error handling for every argument
- Brittle — adding new commands requires rewriting match blocks
- No completions or standard CLI conventions

## Evaluation Checkpoint (Optional)
<!-- Gate: Options → Decision. Agent assesses and recommends. -->

**Assessment:** Proceed

- [x] All options evaluated at comparable depth
- [x] Decision drivers are defined and referenced in option analysis
- [x] No unacknowledged experimentation gaps (ADR-0022 tolerance check)

**Validation needs:** None — all three options are well-understood Rust CLI patterns.

## Decision

In the context of creating a CLI for skilleton, facing the need for typed subcommands with good help text and maintainability, we chose **clap with derive macros** (Option 1) over argh and manual parsing to achieve declarative command definitions, automatic help generation, and ecosystem alignment, accepting the heavier compile-time dependency.

**Command design:**

```rust
#[derive(Parser)]
#[command(name = "skilleton", about = "Build and validate agent skills")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new skill directory layout
    Init {
        /// Path to create the skill directory
        path: PathBuf,
    },
    /// Validate a skill's structure and references
    Check {
        /// Path to the skill directory
        path: PathBuf,
    },
    /// Build a skill into Markdown output
    Build {
        /// Path to the skill directory
        path: PathBuf,
    },
}
```

**Binary target:** Add `[[bin]]` to `Cargo.toml` with `src/main.rs` as the entry point. The binary depends on the library crate for all skill logic.

**Exit codes:**
- `0` — success
- `1` — validation errors (check) or build errors (build)
- `2` — usage errors (bad arguments)

**Init behavior:** `skilleton init <path>` creates the directory layout defined by ADR-0006. It constructs a default `Skill` struct with placeholder metadata and calls `SkillWriter::write`:
- `<path>/skill.toml` — with `name` derived from the directory name, an empty `description`, and a generated `skill:<name>` ItemId
- `<path>/procedures/` — empty directory (no default procedures)

The resulting directory is immediately loadable by `SkillLoader::load`.

**Check behavior:** `skilleton check <path>` loads a skill via `SkillLoader::load`, then runs all validators:
1. `validate_invocation_references` — cross-procedure reference integrity
2. `validate_criterion_references` — criterion reference validity
3. `validate_type_prefixes` — ItemId type prefix consistency
4. `detect_policy_overlaps` — policy conflict detection

Reports all findings and exits with code 1 if any validation fails.

**Build behavior:** `skilleton build <path>` loads a skill, validates it (same as check), and renders Markdown to stdout. See ADR-0010 for the rendering pipeline design.

## Consequences

**Positive:**
- Users can scaffold, validate, and build skills from the command line
- clap provides professional-quality help text and error messages automatically
- Derive macros keep command definitions maintainable as new subcommands are added
- The binary target introduces a clear separation between library API and user interface

**Negative:**
- clap adds compile-time cost — incremental builds remain fast, but clean builds take longer (to be measured during implementation; expect single-digit seconds on modern hardware based on similar-sized CLIs)
- The clap dependency tree increases `Cargo.lock` size

**Neutral:**
- The `src/main.rs` entry point becomes the composition root, importing from `src/lib.rs`
- Future subcommands (e.g., `lint`, `format`, `watch`) can be added without restructuring

## Quality Strategy

- ~~Introduces major semantic changes~~
- [x] Introduces minor semantic changes
- ~~Fuzz testing~~
- [x] Unit testing
- ~~Load testing~~
- ~~Performance testing~~
- [x] Backwards Compatible
- [x] Integration tests
- [x] Tooling
- [x] User documentation

### Additional Quality Concerns

- Integration tests should exercise all three subcommands end-to-end using a fixture skill directory
- The `init` command must produce output loadable by `SkillLoader::load`
- The `check` command must report all validation errors, not just the first

## Conclusion Checkpoint (Optional)
<!-- Gate: Quality Strategy → Review. Verify before requesting review. -->

**Assessment:** Ready for review

- [x] Decision justified (Y-statement or equivalent)
- [x] Consequences include positive, negative, and neutral outcomes
- [x] Quality Strategy reviewed — relevant items checked, irrelevant struck through
- [x] Links to related ADRs populated

**Pre-review notes:** The command design intentionally uses simple positional `path` arguments. Flags and options can be added later without breaking the interface.

---

## Comments

### Draft Worksheet
<!-- Captures original intent and workflow calibration. -->

**Framing:**
Milestone 3 requires a CLI to expose skilleton's library capabilities. The CLI needs init, check, and build subcommands. clap is the obvious framework choice for Rust CLIs.

**Tolerance:**
- Risk: Low — standard CLI pattern
- Change: Low — additive (new binary target)
- Improvisation: Low — follow Rust CLI conventions

**Uncertainty:**
Certain: subcommand structure (init/check/build), clap as framework.
Uncertain: none — this is well-trodden territory.

**Options:**
- Target count: 2-3
- [ ] Explore additional options beyond candidates listed below

**Candidates:**
- clap (derive) — de facto standard
- argh — lightweight alternative
- Manual parsing — zero-dep baseline

<!-- Review cycle 1 — 2026-04-08 — Verdict: Accept with suggestions. Findings: 7 (2M, 5L). -->
