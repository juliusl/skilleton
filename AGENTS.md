# Contributing to skilleton

Instructions for agents and developers making changes to skills in this repo.

## Policies

All policies are listed here with identifiers. Detailed descriptions follow in the sections below.

| ID | Policy | Description |
|----|--------|-------------|
| P-1 | Writing Style | Technical, simple, concise — no double negatives, clear logic |
| P-1a | ADHD Friendly Guidelines | Logical ordering, frontloaded actions, visual chunking |
| P-2 | Broken Test Policy | Stop and fix broken tests before proceeding |
| P-4 | Git Policies | No commit/push without explicit user delegation |
| P-4a | Conventional Commits | Use `type(scope): summary` format for commit messages |
| P-16 | Broken Makefile Targets | Stop and fix broken Makefile targets before proceeding |
| P-17 | Autonomy Directives | Never take shortcuts when a procedure or plan has been established |
| P-18 | Broken References Policy | Stop and fix broken references before proceeding |

---

## P-1: Writing Style

All writing should follow this style

- **Technical and simple** — write for engineers/agents, not academics
- **No double negatives** — say what things *do*, not what they don't not do
- **Clear logic** — one idea per sentence, explicit cause-and-effect
- **Concise** — cut filler words; if a sentence works without a word, remove it
- **Do not arbitrarily wrap technical docs meant for agents** - If a document will be consumed by an agent, avoid manual wrapping in formatting.

### P-1a: ADHD Friendly Guidelines

In addition to the above writing style guidelines, writing must be presented in an ADHD friendly manner.

This DOES NOT mean:

- Using emojis
- Stating that information is ADHD friendly

This DOES mean:

- Order information logically — most important information first
- Frontload actions — put the command or instruction before the explanation, not after
- Use lists, flow-charts, and tables proactively
- Use whitespace and visual chunking — short paragraphs, consistent formatting patterns, clear separation between sections
- Do not use headers arbitrarily — organize around process flow, not arbitrary categories
- Keep justification brief
- State evidence and rationale explicitly — do not expect the reader to infer connections or fill in gaps
- Follow KISS — Keep It Simple, Stupid

NEVER:

- When giving a recommendation, do not preface the recommendation with this guideline, ADHD users do not need to be reminded that they have ADHD

## P-2: Broken Test Policy

ALWAYS, when encountering a broken test, STOP and fix the test before proceeding. A broken test is always in scope and ignoring it creates technical debt.

## P-4: Git Policies

1) Agents **must not** commit or push changes. Stage your work and let the
developer review, commit, and push manually.

This policy can **only** be bypassed with **EXPLICIT** instructions given or delegated by the user.

### P-4a: Conventional Commits

When asked to draft a commit message, use
<a href="https://www.conventionalcommits.org/">Conventional Commits</a> format:

```
<type>(<scope>): <short summary>

<optional body>
```

Common types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `build`.
Scope is optional but encouraged (e.g., `skill`, `makefile`, `tooling`).

## P-16: Broken Makefile Targets Policy

ALWAYS, when encountering a broken makefile target, STOP and fix the target before proceeding. A broken target is always in scope and ignoring it creates technical debt.

## P-17: Autonomy Directives

When operating autonomously, **NEVER** take shortcuts when a procedure or plan has been established. Resource constraints or session length are not valid reasons to skip procedures. Procedures are in place to safe-guard autonomously generated code.

## P-18: Broken References Policy

`make check-refs` must pass clean — zero broken references. Pre-existing broken references are not exempt. If `check-refs` fails at any point during a session, stop and fix the broken references before proceeding. Treating broken references as "pre-existing" and moving on is a policy violation.