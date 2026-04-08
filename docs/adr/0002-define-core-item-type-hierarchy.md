# 2. Define core item type hierarchy

Date: 2026-04-07
Status: Accepted
Last Updated: 2026-04-08
Links: ADR-0003, ADR-0004

## Context

Skilleton is a Rust-based tool for building and modifying agent skills. Agent skills require well-defined structure — policies must appear before procedures, and semantic ordering matters. The roadmap defines a core item abstraction with two classification kinds:

- **Hierarchy-based** items compose into trees: Skill → Procedure → Step → Task
- **Singleton-based** items attach to hierarchy nodes as constraints: Policy, Criterion

Specific types from the roadmap:

| Type | Kind | Definition |
|------|------|------------|
| Policy | Singleton | A constraint or rule that MUST be followed |
| Criterion | Singleton | A state or outcome that is either satisfied or unsatisfied |
| Task | Hierarchy | A single instruction with a subject and action |
| Step | Hierarchy | A set of Tasks with a list of Completion Criteria |
| Procedure | Hierarchy | A list of Steps with entrance and exit Criteria |
| Skill | Hierarchy (root) | Root item with agentskills.io metadata; includes Policies inherited by children |

All items related to a Procedure can have conditional Criteria. If omitted, the item is implicitly Active.

ADR-0003 defines the `ItemId` type used in `ItemMeta` for unique item identification. ADR-0004 extends `Task` with cross-procedure reference capabilities, building on this hierarchy.

Decision drivers:
- **Type safety** — Rust's type system should enforce valid compositions at compile time
- **Extensibility** — new item types may be added in later milestones
- **Traversal** — the Policy compiler needs to walk the hierarchy for conflict detection
- **Idiomatic Rust** — prefer composition over inheritance patterns

## Options

### Option A: Enum-based type system

Model all items as variants of a single `Item` enum. Hierarchy vs. Singleton is a discriminant field.

```rust
enum Item {
    Skill(SkillData),
    Procedure(ProcedureData),
    Step(StepData),
    Task(TaskData),
    Policy(PolicyData),
    Criterion(CriterionData),
}
```

- **Pro:** Single type for storage and traversal; pattern matching is idiomatic
- **Pro:** Adding new variants is straightforward
- **Con:** No compile-time enforcement that a Step only contains Tasks — all relationships use `Item`
- **Con:** Every function that handles items needs exhaustive matching even when only one kind applies

### Option B: Trait-based polymorphism

Define an `Item` trait with `HierarchyItem` and `SingletonItem` sub-traits. Use trait objects for dynamic dispatch.

```rust
trait Item { fn id(&self) -> &ItemId; }
trait HierarchyItem: Item { fn children(&self) -> &[Box<dyn Item>]; }
trait SingletonItem: Item { fn scope(&self) -> Scope; }
```

- **Pro:** Clean abstraction boundary between hierarchy and singleton behavior
- **Con:** Trait objects lose type information — cannot pattern-match on concrete types without downcasting
- **Con:** Trait object overhead and complexity (lifetimes, object safety) for what is fundamentally a data model
- **Con:** Harder to serialize/deserialize

### Option C: Concrete structs with enum containers

Each type gets its own concrete struct. An enum container groups hierarchy children and singleton attachments by kind. Common metadata via a shared `ItemMeta` struct.

```rust
struct ItemMeta {
    id: ItemId,
    conditions: Vec<CriterionRef>,
}

struct Policy { meta: ItemMeta, text: String }
struct Criterion { meta: ItemMeta, description: String }
struct Task { meta: ItemMeta, subject: String, action: String }

struct Step {
    meta: ItemMeta,
    tasks: Vec<Task>,
    completion_criteria: Vec<CriterionRef>,
    policies: Vec<Policy>,
}

struct Procedure {
    meta: ItemMeta,
    steps: Vec<Step>,
    entrance_criteria: Vec<CriterionRef>,
    exit_criteria: Vec<CriterionRef>,
    policies: Vec<Policy>,
}

struct Skill {
    meta: ItemMeta,
    metadata: SkillMeta,
    procedures: Vec<Procedure>,
    policies: Vec<Policy>,
}
```

- **Pro:** Full compile-time type safety — a Step can only contain Tasks, a Procedure can only contain Steps
- **Pro:** Idiomatic Rust — composition over inheritance, no trait objects needed
- **Pro:** Straightforward serialization with serde
- **Pro:** Policy inheritance is modeled explicitly — each level has its own `policies: Vec<Policy>`
- **Con:** Adding a new hierarchy level requires updating parent structs
- **Con:** Generic traversal requires either a visitor pattern or an enum wrapper for cross-type operations

## Evaluation Checkpoint (Optional)
<!-- Gate: Options → Decision. Agent assesses and recommends. -->

**Assessment:** Proceed

- [x] All options evaluated at comparable depth
- [x] Decision drivers are defined and referenced in option analysis
- [x] No unacknowledged experimentation gaps (ADR-0022 tolerance check)

**Validation needs:** None — the type hierarchy is well-specified by the roadmap, and all three options are standard Rust patterns with known tradeoffs.

## Decision

In the context of **defining skilleton's core item abstraction**, facing **the need for compile-time type safety, idiomatic Rust patterns, and explicit hierarchy composition**, we chose **Option C (concrete structs with enum containers)** over **enum-based (A) and trait-based (B) approaches** to achieve **full compile-time enforcement of valid item compositions and straightforward serialization**, accepting **the need for a visitor pattern or enum wrapper for generic traversal, and manual updates when adding new hierarchy levels**.

Concrete commitments:
1. Each item type (Policy, Criterion, Task, Step, Procedure, Skill) is a concrete struct
2. Common metadata (ID, conditions) is composed via a shared `ItemMeta` struct
3. Hierarchy relationships are expressed as typed `Vec` fields — `Step.tasks: Vec<Task>`, `Procedure.steps: Vec<Step>`
4. Singleton items (Policy, Criterion) attach to hierarchy nodes via typed `Vec` fields at each level
5. Policy inheritance is explicit — `Skill.policies` propagate to all descendants during compilation/analysis
6. Conditional Criteria use `Vec<CriterionRef>` on `ItemMeta` — if empty, the item is implicitly Active

## Consequences

- **Positive:** Invalid compositions (e.g., a Task containing a Procedure) are rejected at compile time
- **Positive:** The data model maps directly to serialization formats without adapter layers
- **Positive:** Policy scoping is explicit in the type structure — the compiler can walk each level's policies for conflict detection
- **Neutral:** A visitor pattern or `ItemKind` enum will be needed for generic operations (e.g., "iterate all items in a skill"). This is standard Rust and can be added when traversal is needed.
- **Negative:** Adding a new hierarchy level (e.g., inserting a "Phase" between Procedure and Step) requires updating parent struct fields. This is acceptable because the type hierarchy is defined upfront and changes are architectural decisions requiring their own ADR.

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
- Construction of each item type with valid metadata
- Policy attachment at each hierarchy level
- Conditional criteria (present vs. omitted for Active default)

## Conclusion Checkpoint (Optional)
<!-- Gate: Quality Strategy → Review. Verify before requesting review. -->

**Assessment:** Ready for review

- [x] Decision justified (Y-statement or equivalent)
- [x] Consequences include positive, negative, and neutral outcomes
- [x] Quality Strategy reviewed — relevant items checked, irrelevant struck through
- [x] Links to related ADRs populated

**Pre-review notes:** The struct shapes shown are directional, not final API. Field names and access patterns will be refined during implementation.

---

## Comments

### Draft Worksheet
<!-- Captures original intent and workflow calibration. -->

**Framing:**
Milestone 1 of the skilleton roadmap defines 6 concrete item types organized into two kinds (Hierarchy and Singleton). This ADR decides how to model that type system in Rust — the foundation everything else builds on.

**Tolerance:**
- Risk: Low — the roadmap specifies types clearly; standard Rust patterns apply
- Change: Low — the type hierarchy is well-defined
- Improvisation: Low — follow the roadmap's type definitions

**Uncertainty:**
- Certain: the 6 types and their relationships (roadmap-specified)
- Certain: Hierarchy vs Singleton classification
- Uncertain: exact API surface and field names (deferred to implementation)

**Options:**
- Target count: 3
- [x] Explore additional options beyond candidates listed below

**Candidates:**
- Enum-based: single Item enum with variants
- Trait-based: Item trait with sub-traits
- Concrete structs: composition-based data model

<!-- Review cycle 1 — 2026-04-08 — Verdict: Accept. Polish pass applied. -->
