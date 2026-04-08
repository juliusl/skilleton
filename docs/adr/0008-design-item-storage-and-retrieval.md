# 8. Design item storage and retrieval

Date: 2026-04-08
Status: Planned
Last Updated: 2026-04-08
Links: ADR-0002, ADR-0003, ADR-0005, ADR-0006

## Context

The roadmap requires "Item storage implementation, that can store and fetch items." This is the API for loading skill definitions from disk and accessing them in memory at runtime.

The storage layer sits between serialization (ADR-0005, which selects the on-disk format) and the type hierarchy (ADR-0002, which defines the in-memory types). File organization (ADR-0006) determines how skill definitions are laid out on disk. The storage API must bridge these layers — reading files according to ADR-0006's layout, deserializing per ADR-0005's format, and producing the typed structs from ADR-0002.

Key operations the storage layer must support:
- **Load** a Skill from disk by path or identifier
- **Fetch** individual items by `ItemId` (ADR-0003) from a loaded Skill
- **List** available Skills in a directory
- **Validate** loaded items against the type hierarchy constraints

Testability is a first-order concern. Unit tests for the Policy compiler (ADR-0007) and cross-procedure validation (ADR-0004) need to construct in-memory skill graphs without touching the file system.

Decision drivers:
1. **Testability** — an in-memory backend for tests is non-negotiable; file I/O in unit tests is slow and fragile
2. **Simplicity** — the 0.x API should be minimal; premature abstraction is a cost
3. **Extensibility** — future storage backends (database, network) should be addable without rewriting consumers
4. **Rust idioms** — trait-based abstraction, `Result` error handling, ownership-friendly API

## Options

### Option A: Direct file I/O

Load files directly into Rust types using serde. No abstraction layer. Functions like `load_skill(path: &Path) -> Result<Skill>` read TOML files and return typed structs.

```rust
pub fn load_skill(path: &Path) -> Result<Skill> {
    let content = std::fs::read_to_string(path)?;
    let skill: Skill = toml::from_str(&content)?;
    Ok(skill)
}

pub fn find_item(skill: &Skill, id: &ItemId) -> Option<&dyn Item> {
    // walk the hierarchy
}
```

- **Pro:** Minimal code — no traits, no generics, no indirection
- **Pro:** Easy to understand — the call path is `read file → deserialize → return`
- **Pro:** Fast to implement — a few functions cover the initial use cases
- **Con:** Unit tests must use real files or `tempdir` — no way to construct test data without I/O
- **Con:** Storage format is coupled to every call site — changing from TOML to YAML touches all consumers
- **Con:** No path to alternative backends — adding database storage requires a different API

### Option B: Repository pattern with trait abstraction

A `SkillRepository` trait defines the storage interface. Implementations for file-based (TOML) and in-memory backends. Consumers depend on the trait, not the implementation.

```rust
pub trait SkillRepository {
    fn load_skill(&self, name: &str) -> Result<Skill>;
    fn list_skills(&self) -> Result<Vec<String>>;
    // ItemRef: enum over concrete item kinds from ADR-0002's type hierarchy
    // (Procedure, Policy, Criterion, etc.) — owns the data so the trait
    // avoids returning references tied to &self's lifetime.
    fn find_item(&self, skill: &str, id: &ItemId) -> Result<Option<ItemRef>>;
}

pub struct FileRepository { root: PathBuf }
pub struct InMemoryRepository { skills: HashMap<String, Skill> }
```

- **Pro:** In-memory implementation enables fast, deterministic unit tests with no file system dependency
- **Pro:** Storage format changes only affect the `FileRepository` implementation — consumers are insulated
- **Pro:** Future backends (database, network, compressed archive) implement the same trait
- **Pro:** Idiomatic Rust — trait objects or generics, `Result` error handling, composable
- **Con:** More code upfront — trait definition, two implementations, error types
- **Con:** Trait design must anticipate future needs without over-engineering — getting the API surface right requires judgment
- **Con:** Indirection adds a layer between the caller and the data

### Option C: Lazy-loading with cache

Load items on demand as they are accessed. A cache stores previously loaded items to avoid redundant I/O. The cache is invalidated when files change (via timestamp or watcher).

```rust
pub struct SkillCache {
    root: PathBuf,
    cache: HashMap<String, CachedSkill>,
}

struct CachedSkill {
    skill: Skill,
    loaded_at: SystemTime,
}

impl SkillCache {
    pub fn get_skill(&mut self, name: &str) -> Result<&Skill> {
        if self.is_stale(name) {
            self.reload(name)?;
        }
        Ok(&self.cache[name].skill)
    }
}
```

- **Pro:** Memory-efficient for large skill collections — only loads what is accessed
- **Pro:** Automatic freshness for long-running processes (e.g., a language server)
- **Con:** Cache invalidation is a hard problem — timestamp checks are racy, file watchers add platform-specific complexity
- **Con:** Mutable borrow on `get_skill` conflicts with Rust's borrow checker when multiple items are accessed simultaneously
- **Con:** Adds complexity (staleness checks, cache state) before there is evidence that eager loading is a bottleneck
- **Con:** Harder to test — cache behavior introduces non-determinism unless carefully mocked

## Evaluation Checkpoint (Optional)
<!-- Gate: Options → Decision. Agent assesses and recommends. -->

**Assessment:** Proceed

- [x] All options evaluated at comparable depth
- [x] Decision drivers are defined and referenced in option analysis
- [x] No unacknowledged experimentation gaps (ADR-0022 tolerance check)

**Validation needs:** None — repository pattern is a standard Rust idiom (widely used in projects like `cargo`, `rustup`). The tradeoffs between abstraction cost and testability are well-understood. No prototype needed.

## Decision

In the context of **designing skilleton's item storage and retrieval API**, facing **the need for testable, format-decoupled, extensible storage with idiomatic Rust patterns**, we chose **Option B (repository pattern with trait abstraction)** over **direct file I/O (A) and lazy-loading with cache (C)** to achieve **immediate testability via an in-memory backend and clean separation between storage format and consumers**, accepting **additional upfront code for the trait definition and two initial implementations**.

Concrete commitments:
1. A `SkillRepository` trait defines the storage interface with methods: `load_skill`, `list_skills`, `find_item`
2. `FileRepository` implements the trait for file-system-based storage using serde deserialization (format per ADR-0005, layout per ADR-0006)
3. `InMemoryRepository` implements the trait for test usage — skills are constructed programmatically with no I/O
4. Both implementations return `Result<T, RepositoryError>` where `RepositoryError` is a dedicated error enum
5. The trait uses `&self` (shared reference) — implementations manage interior mutability if needed. This keeps the API borrow-checker-friendly for consumers holding multiple references.
6. Lazy loading (Option C) is not precluded — a `CachingRepository` wrapper can be added in a future milestone by composing over any `SkillRepository` implementation. The trait abstraction supports this without breaking existing consumers.

## Consequences

- **Positive:** Unit tests for the Policy compiler and cross-procedure validation can construct arbitrary skill graphs in memory without file I/O — faster, deterministic, and isolated
- **Positive:** Changing the serialization format (e.g., TOML → RON) requires updating only `FileRepository`, not every consumer of the storage API
- **Positive:** The trait boundary creates a natural integration test seam — `FileRepository` is tested with real files, consumers are tested with `InMemoryRepository`
- **Negative:** The trait API surface must be designed upfront. An overly narrow API may require breaking changes later; an overly broad API wastes effort on unused methods. Mitigation: start with the minimal three-method API and extend via default methods or a `SkillRepositoryExt` trait.
- **Negative:** Two implementations must be kept in sync — any new trait method must be implemented in both `FileRepository` and `InMemoryRepository`.
- **Neutral:** `FileRepository.load_skill` must implement ADR-0006's directory traversal and multi-file assembly (reading `skill.toml` plus each procedure file under `procedures/`). This complexity is contained behind the trait boundary — consumers and `InMemoryRepository` are unaffected — but it makes `FileRepository` the heavier of the two initial implementations.
- **Neutral:** The abstraction adds one layer of indirection between callers and data. This is standard Rust practice and has negligible runtime cost (monomorphized generics or thin trait objects).

## Quality Strategy

- [ ] ~~Introduces major semantic changes~~
- [x] Introduces minor semantic changes
- [ ] ~~Fuzz testing~~
- [x] Unit testing
- [ ] ~~Load testing~~
- [ ] ~~Performance testing~~
- [x] Backwards Compatible
- [x] Integration tests
- [ ] ~~Tooling~~
- [ ] ~~User documentation~~

### Additional Quality Concerns

Unit tests (using `InMemoryRepository`):
- Load a skill by name and verify the returned struct matches the input
- List skills returns all registered skills
- Find item by `ItemId` returns the correct item from the hierarchy
- Find item with nonexistent ID returns `None`
- Error cases: load nonexistent skill, invalid item ID format

Integration tests (using `FileRepository`):
- Round-trip: write a skill definition to disk, load via `FileRepository`, verify correctness
- Load from a directory with multiple skill files
- Graceful error on malformed files

## Conclusion Checkpoint (Optional)
<!-- Gate: Quality Strategy → Review. Verify before requesting review. -->

**Assessment:** Ready for review

- [x] Decision justified (Y-statement or equivalent)
- [x] Consequences include positive, negative, and neutral outcomes
- [x] Quality Strategy reviewed — relevant items checked, irrelevant struck through
- [x] Links to related ADRs populated

**Pre-review notes:** The trait API shown is directional. The exact method signatures (return types, lifetimes, error variants) will be refined during implementation. The key commitment is the trait boundary itself, not the specific method shapes.

---

## Comments

### Draft Worksheet
<!-- Captures original intent and workflow calibration. -->

**Framing:**
The roadmap requires item storage and retrieval. This ADR decides the API shape — not the serialization format (ADR-0005) or file layout (ADR-0006), but how consumers interact with stored skill definitions.

**Tolerance:**
- Risk: Low — repository pattern is proven in the Rust ecosystem
- Change: Medium — the trait API is a public interface that downstream code depends on
- Improvisation: Low — the options are standard software architecture patterns

**Uncertainty:**
- Certain: need both file-based and in-memory access (testability requirement)
- Certain: consumers should not depend on the storage format directly
- Uncertain: exact trait API surface — will the initial three methods suffice?
- Uncertain: whether lazy loading will be needed (depends on skill collection sizes in practice)

**Options:**
- Target count: 3
- [ ] Explore additional options beyond candidates listed below

**Candidates:**
- Direct file I/O — minimal, no abstraction
- Repository trait — abstract, testable, extensible
- Lazy-loading with cache — demand-loaded, memory-efficient

### Review Addendum — V-1

**Q (Finding 1, M):** `ItemRef` is used in `find_item`'s return type but never defined. Define it as an enum over concrete references, a type alias, or replace with a concrete return type.

**A: Address.** Added a comment block in the Option B code snippet clarifying that `ItemRef` is an enum over concrete item kinds from ADR-0002's type hierarchy (Procedure, Policy, Criterion, etc.) and explaining *why* it owns the data (avoiding lifetime ties to `&self`). The pre-review notes already acknowledge signatures are directional, but an undefined type in the illustrative code creates unnecessary ambiguity.

**Q (Finding 2, L):** Missing consequence: `FileRepository.load_skill` must implement ADR-0006's directory traversal and multi-file assembly. Add as a negative or neutral consequence.

**A: Address.** Added as a neutral consequence. ADR-0006 line 171 explicitly calls out multi-file assembly and directory traversal as implementation complexity. ADR-0008's decision already says "layout per ADR-0006," but the consequence section should surface the cost explicitly — this is impact documentation. Filed as neutral because the complexity is real but fully contained behind the trait boundary.
