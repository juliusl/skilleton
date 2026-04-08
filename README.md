# skilleton

A Rust-based tool for building and validating agent skills.

Skilleton provides a structured type system for defining agent skills as hierarchical trees of procedures, steps, tasks, policies, and criteria. It includes validation for references, type prefixes, and policy conflicts, plus a Markdown rendering pipeline that enforces policy-before-procedure ordering.

## Installation

```bash
cargo install --path .
```

## Usage

### Initialize a skill

```bash
skilleton init <path>
```

Creates a new skill directory with a `skill.toml` and an empty `procedures/` directory. The skill name is derived from the directory name.

**Exit codes:** `0` success, `1` error (path already exists)

### Validate a skill

```bash
skilleton check <path>
```

Loads the skill and runs all validators:
- Invocation reference integrity
- Criterion reference validity
- ItemId type prefix consistency
- Policy conflict detection

Reports all findings to stderr. **Exit codes:** `0` all checks pass, `1` validation errors found

### Build a skill

```bash
skilleton build <path>
```

Validates the skill, then renders it as Markdown to stdout with policies ordered before procedures at every hierarchy level.

```bash
skilleton build my-skill > my-skill.md
```

**Exit codes:** `0` success, `1` validation errors (no output produced)

## Skill Directory Layout

```
<skill-name>/
├── skill.toml           # Skill metadata, policies, criteria
└── procedures/
    ├── procedure-a.toml  # One file per procedure
    └── procedure-b.toml
```

## License

See project license.
