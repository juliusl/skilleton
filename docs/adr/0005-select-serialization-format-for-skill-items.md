# 5. Select serialization format for skill items

Date: 2026-04-08
Status: Accepted
Last Updated: 2026-04-08
Links: ADR-0002, ADR-0003, ADR-0006

## Context

Skilleton needs to persist skill items to disk. The serialization format determines how the Rust type hierarchy (ADR-0002) — Skill → Procedure → Step → Task with Policy/Criterion attachments — is stored, version-controlled, and read by both Rust tooling and agents consuming raw files.

The roadmap lists this as an open question: "Serialization format decision (TOML, YAML, custom DSL, etc.)" It also specifies a key constraint: files must be source-control friendly (readable diffs, merge-friendly).

Requirements:
- **Source-control friendly** — readable diffs, meaningful merge conflicts
- **Hierarchical data support** — the type system has 4 levels with singleton attachments
- **Rust serde ecosystem** — first-class `Serialize`/`Deserialize` derives
- **Parse-deterministic** — format must not silently coerce types; what is written must be what is parsed
- **Round-trip fidelity** — serialization must preserve ItemId paths (ADR-0003) and all metadata

Decision drivers (ranked):
1. **Source-control friendliness** — the roadmap states this explicitly as a constraint
2. **Rust serde support** — must derive `Serialize`/`Deserialize` on existing types (ADR-0002)
3. **Parse determinism** — agents parse files directly; the format must not silently coerce types
4. **Hierarchical fitness** — the fixed 4-level hierarchy must map naturally

## Options

### Option A: TOML

Use TOML as the serialization format with `serde` + `toml` crate.

```toml
[skill]
id = "skill:onboarding"
name = "Onboarding"
description = "New user onboarding flow"
conditions = []

[[skill.policies]]
id = "policy:no-plaintext"
text = "Never store passwords in plaintext"
conditions = []

[[skill.procedures]]
id = "procedure:welcome"
conditions = []
entrance_criteria = ["criterion:user-registered"]
exit_criteria = ["criterion:onboarded"]

[[skill.procedures.steps]]
id = "step:greet"
conditions = []
completion_criteria = ["criterion:greeted"]

[[skill.procedures.steps.policies]]
id = "policy:greet-by-name"
text = "Always address the user by name"
conditions = []

[[skill.procedures.steps.tasks]]
id = "task:send-message"
subject = "User"
action = "Send greeting message"
conditions = []

[[skill.procedures.steps.tasks]]
id = "task:log-event"
subject = "System"
action = "Log onboarding event"
conditions = ["criterion:logging-enabled"]
invokes = "procedure:audit-log"
```

- **Pro:** Source-control friendly — key-value pairs produce clean, line-oriented diffs
- **Pro:** Rust-native — `toml` crate has mature serde support; derives are straightforward
- **Pro:** Widely known — agents and humans can read TOML without documentation
- **Pro:** No implicit type coercion — values are what they appear to be
- **Con:** Nested arrays of tables (`[[a.b.c]]`) become verbose at 3+ levels
- **Con:** Inline tables are single-line only — complex nested objects can't use them ergonomically

### Option B: YAML

Use YAML as the serialization format with `serde` + `serde_yaml` crate.

```yaml
skill:
  id: "skill:onboarding"
  name: Onboarding
  description: New user onboarding flow
  conditions: []
  policies:
    - id: "policy:no-plaintext"
      text: Never store passwords in plaintext
      conditions: []
  procedures:
    - id: "procedure:welcome"
      conditions: []
      entrance_criteria: ["criterion:user-registered"]
      exit_criteria: ["criterion:onboarded"]
      steps:
        - id: "step:greet"
          conditions: []
          completion_criteria: ["criterion:greeted"]
          policies:
            - id: "policy:greet-by-name"
              text: Always address the user by name
              conditions: []
          tasks:
            - id: "task:send-message"
              subject: User
              action: Send greeting message
              conditions: []
            - id: "task:log-event"
              subject: System
              action: Log onboarding event
              conditions: ["criterion:logging-enabled"]
              invokes: "procedure:audit-log"
```

- **Pro:** Natural nesting — indentation maps directly to hierarchy depth
- **Pro:** Widely supported — agents, CI tools, and humans all read YAML
- **Pro:** Compact for hierarchical data — less syntactic overhead than TOML for nested structures
- **Con:** Implicit type coercion — `yes`, `no`, `on`, `off` silently become booleans; `3.10` becomes a float
- **Con:** Indentation-sensitive — whitespace errors cause silent data loss or parse failures
- **Con:** Multiple YAML specs — YAML 1.1 vs 1.2 behavior differs; `serde_yaml` uses 1.2 but agents may assume 1.1
- **Con:** `serde_yaml` is unmaintained as of 2024 — `serde_yml` is the successor but less mature

### Option C: RON (Rusty Object Notation)

Use RON as the serialization format with `serde` + `ron` crate.

```ron
Skill(
    meta: ItemMeta(id: "skill:onboarding", conditions: []),
    metadata: SkillMeta(name: "Onboarding", description: "New user onboarding flow"),
    procedures: [
        Procedure(
            meta: ItemMeta(id: "procedure:welcome", conditions: []),
            steps: [
                Step(
                    meta: ItemMeta(id: "step:greet", conditions: []),
                    tasks: [
                        Task(
                            meta: ItemMeta(id: "task:send-message", conditions: []),
                            subject: "User",
                            action: "Send greeting message",
                            invokes: None,
                        ),
                    ],
                    completion_criteria: [],
                    policies: [],
                ),
            ],
            entrance_criteria: [],
            exit_criteria: [],
            policies: [],
        ),
    ],
    policies: [],
)
```

- **Pro:** Preserves Rust type names — struct and enum names appear in the output
- **Pro:** First-class serde support — the `ron` crate handles derives cleanly
- **Pro:** No ambiguity — `None`, `Some()`, tuples, and enums serialize naturally
- **Con:** Not widely known — agents outside the Rust ecosystem cannot parse RON without a RON-specific parser
- **Con:** Verbose — every struct name and field is explicit, producing longer files than TOML or YAML
- **Con:** Poor diff ergonomics — deeply nested parentheses make diffs harder to read than key-value formats
- **Con:** Not a standard format — no language-agnostic tooling (linters, formatters, schema validators)

### Excluded: Custom DSL

Not evaluated. A custom DSL would require building and maintaining a parser, which is unjustified when three established formats with existing serde support cover the requirements.

## Evaluation Checkpoint (Optional)
<!-- Gate: Options → Decision. Agent assesses and recommends. -->

**Assessment:** Proceed

- [x] All options evaluated at comparable depth
- [x] Decision drivers are defined and referenced in option analysis
- [x] No unacknowledged experimentation gaps (ADR-0022 tolerance check)

**Validation needs:** None — all three formats have mature Rust crate support and well-documented behavior. The tradeoffs are deterministic from the format specifications.

## Decision

In the context of **persisting skilleton's type hierarchy to disk**, facing **the need for source-control-friendly, parse-deterministic files with first-class Rust serde support**, we chose **Option A (TOML)** over **YAML (B) and RON (C)** to achieve **clean line-oriented diffs, no implicit type coercion, and idiomatic Rust serialization**, accepting **verbosity in deeply nested array-of-tables syntax**.

Concrete commitments:
1. All item types (Skill, Procedure, Step, Task, Policy, Criterion) derive `Serialize` and `Deserialize` via serde
2. `ItemMeta`, `ItemId`, `CriterionRef`, and `SkillMeta` also derive serde traits
3. The `toml` crate is the serialization backend — add `toml` and `serde` as dependencies in `Cargo.toml`
4. `ItemId` serializes as its string representation (the hierarchical path), not as a struct
5. TOML array-of-tables (`[[...]]`) syntax is used for `Vec` fields (policies, procedures, steps, tasks)
6. Files use the `.toml` extension
7. Round-trip fidelity: `deserialize(serialize(item)) == item` for all types

## Consequences

- **Positive:** Source-control diffs are line-oriented — adding a policy or task shows as appended lines, not a restructured block
- **Positive:** No implicit type coercion — string values stay strings, preventing silent data corruption that YAML allows
- **Positive:** The `toml` crate is actively maintained, has a stable API, and provides complete TOML v1.0 spec coverage with first-class serde support
- **Negative:** Nested array-of-tables syntax (`[[skill.procedures.steps.tasks]]`) is verbose at the 4th level. Mitigation: ADR-0006 addresses this by splitting files so no single file exceeds 2 levels of nesting. **Dependency risk:** ADR-0006 is currently Ready — if it is rejected or deferred, the full 4-level nesting verbosity applies and may require revisiting this decision.
- **Negative:** TOML does not support `null` — `Option::None` fields must use serde's `skip_serializing_if` or a sentinel value. Mitigation: use `#[serde(skip_serializing_if = "Option::is_none")]` on the `invokes` field.
- **Neutral:** TOML is less expressive than YAML for arbitrary nesting. This is acceptable because the hierarchy is fixed at 4 levels — skilleton does not need arbitrary-depth data structures.

## Quality Strategy

- [ ] ~~Introduces major semantic changes~~
- [x] Introduces minor semantic changes
- [ ] ~~Fuzz testing~~
- [x] Unit testing
- [ ] ~~Load testing~~
- [ ] ~~Performance testing~~
- [x] Backwards Compatible
- [ ] ~~Integration tests~~
- [x] Tooling
- [ ] ~~User documentation~~

### Additional Quality Concerns

Unit tests should verify:
- Round-trip serialization for each item type: `deserialize(serialize(x)) == x`
- `ItemId` serializes as a string, not a struct
- Optional fields (e.g., `invokes: None`) are omitted from output
- Malformed TOML produces clear parse errors

Tooling: `Cargo.toml` must be updated to add `serde` (with derive feature) and `toml` crate dependencies.

## Conclusion Checkpoint (Optional)
<!-- Gate: Quality Strategy → Review. Verify before requesting review. -->

**Assessment:** Ready for review

- [x] Decision justified (Y-statement or equivalent)
- [x] Consequences include positive, negative, and neutral outcomes
- [x] Quality Strategy reviewed — relevant items checked, irrelevant struck through
- [x] Links to related ADRs populated

**Pre-review notes:** The TOML nesting verbosity concern is directly addressed by ADR-0006's file organization decision. The two ADRs should be reviewed together.

---

## Comments

### Draft Worksheet
<!-- Captures original intent and workflow calibration. -->

**Framing:**
Open question from the roadmap: "Serialization format decision (TOML, YAML, custom DSL, etc.)." Skilleton needs to persist the type hierarchy (ADR-0002) to disk in a format that is source-control friendly, serde-compatible, and readable by agents consuming raw files. Leaning toward TOML — it's Rust-native, diff-friendly, and avoids YAML's type coercion footguns.

**Tolerance:**
- Risk: Low — prefer widely adopted formats with proven Rust support
- Change: Low — persistence format, not a runtime architecture change
- Improvisation: Low — the roadmap lists the candidate formats

**Uncertainty:**
- Certain: serde derive support is required for all types (ADR-0002)
- Certain: source-control friendliness is a roadmap constraint
- Uncertain: whether TOML's nested table verbosity is tolerable in practice (mitigated by ADR-0006 file splitting)

**Options:**
- Target count: 3
- [ ] Explore additional options beyond candidates listed below

**Candidates:**
- TOML — structured, Rust-native, diff-friendly
- YAML — flexible nesting, widely supported, type coercion risks
- RON — Rust-native, type-preserving, not widely known

### Q&A Addendum (Review V-1)

**F1 (H) — "Validation needs: None" is premature; a PoC would confirm round-trip claims.**
**Decision: Reject.** The round-trip behavior of TOML + serde is deterministic from format specifications — there is no ambiguity about whether `toml::to_string` followed by `toml::from_str` preserves data for types that implement `Serialize`/`Deserialize`. The PoC would confirm implementation correctness (e.g., that our specific serde attributes are right), which is an implementation-phase concern. Quality Strategy already specifies round-trip unit tests for exactly that purpose.

**F2 (H) — TOML example omits real fields (conditions, criteria, invokes).**
**Decision: Address.** Updated the TOML example to include `conditions`, `entrance_criteria`, `exit_criteria`, `completion_criteria`, `policies` at step level, and `invokes` on a Task — matching the fields already shown in the RON example. Also updated the YAML example to the same data for apples-to-apples comparison. The realistic example makes the verbosity Con honest.

**F3 (M) — "Agent-readable" driver is undefined; doesn't differentiate TOML from YAML.**
**Decision: Address.** The finding is right — "agent-readable" is true of both TOML and YAML. The actual differentiator is parse determinism (no implicit type coercion). Renamed the driver to "Parse determinism" and the requirement to "Parse-deterministic" with concrete definition. Updated the Decision Y-statement to match.

**F4 (M) — Nesting mitigation depends on ADR-0006 (Proposed); state dependency risk.**
**Decision: Address.** Added explicit dependency risk language to the nesting consequence: if ADR-0006 is rejected or deferred, full 4-level verbosity applies and may require revisiting this decision.

**F5 (L) — Drop Cargo endorsement; state crate quality directly.**
**Decision: Address.** Replaced "used by Cargo itself" with direct quality claims: actively maintained, stable API, complete TOML v1.0 spec coverage.

**F6 (L) — Custom DSL exclusion rationale missing.**
**Decision: Address.** Added "Excluded: Custom DSL" section after Option C with one-line rationale: building a parser is unjustified when three established formats with serde support cover the requirements.
