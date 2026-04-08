# Roadmap

## Background

This is rust-based tool for stream-lining building and making changes to agent skills.

Agent skills benefit from well-defined structure. Specifically, instructions that are "policies" must be before "procedures" otherwise agents can ignore or miss "policies".

Since semantics and ordering matters, it's actually difficult to make changes and updates via agents as it's possible for the agents to miss places that need updates or remove existing instructions that are outdated or contradictory. It's also difficult for human operators to see the entire skill as a whole to inspect the organization.

## Constraints
- Rust-based
- Item-based (meaning all data is identified by an item id) - More detail in Milestone 1
    - `Procedures` must be described as a Hierarchal Tree of `Steps` and `Tasks`
    - `Policies` must use clear language
    - `Policies` can be scoped at each level of the `Procedure` Hierarchy
    - A `Policy` compiler must be able to check for `Policy` conflict
- Include a DSL w/ a Visual Editor
    - A DSL enables procedural-agent analysis, a visual-editor enables human analysis
- Should be able to build and output a markdown file from artifacts
- Any DSL or config should be source-control friendly

## Milestones

### Milestone 1 <!-- status: complete -->
- Initial item abstraction and relationship definitions:
    - Two types `Hierarchy` based and `Singleton` based
    - `Policy`: A constraint or rule that **MUST** be followed. Must be a `Singleton`.
    - `Criterion`: A state or outcome that is either satisfied or unsatisfied. Must be a `Singleton`.
    - `Task`: A single instruction w/ a subject and action to apply.
    - `Step`: A set of `Task`s w/ a list of "Completion" `Criterion`
    - `Procedure`: A list of `Steps` w/ a list of entrance and exit `Criterion`
    - All items related to a `Procedure` can have a set of conditional `Criterion`, if omitted it implies the item is `Active`
    - `Skill`: Root item which should contain all metadata based on agentskills.io specification
        - Should include `Policy` items that any child items inherent implicitly
        - Will be the entrypoint for any type of compilation or analysis tooling
- **Open**: Define item ID scheme (e.g. UUIDs, namespaced paths, human-readable slugs)
- **Open**: Design cross-procedure reference model — can `Tasks` or `Steps` reference other `Procedures`? If so, define cycle constraints and traversal semantics
- **Follow-up (code review)**: Add type-prefix validation to `CriterionRef` constructor so it enforces the inner `ItemId` has `TypePrefix::Criterion`

### Milestone 2
- Initial working data-format draft
    - **Open**: Serialization format decision (TOML, YAML, custom DSL, etc.) — must be settled before DSL and Rust type design are finalized
- DSL implementation for defining policies and procedures
    - Initial draft file extension, directory organization, etc.
- DSL compiler and analysis tool, to analyze conflicts
    - **Open**: Define what constitutes a `Policy` conflict (contradictory text, overlapping scope, incompatible constraints, etc.)
- Item storage implementation, that can store and fetch items

### Milestone 3
- Create CLI to initialize, check, and build `Procedures` and `Policies`
    - Build command should output Markdown with correct ordering (policies before procedures)
- Define simple skill toy to validate procedure

### Milestone 4
- Finalize all schemas and specifications
- Stabilize data model before building visual tooling

### Milestone 5
- Visual node-based editor for skilleton based skills
    - Should be able to make and test changes
    - Should be able to list and access multiple skills
    - Should be able to view/edit/add item inventory
- MCP server integration w/ skill instructions
    - Enables `Criterion` or `Policy` lookup to reduce output Markdown size
    - Enables defining new `Criterion` and `Policy` items for the project

### Milestone 6
- Initial Package and Release for User Testing
- Write documentation and user guides

### Milestone 7
- Gather usage and improvements

### Milestone 8
- Publish `0.1` to crates.io
