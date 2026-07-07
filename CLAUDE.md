# CLAUDE.md — Ficus Platforms

Canonical source: **`context.md`**. If anything conflicts, `context.md` wins.

## Project

Money transfer and balance tracking app (Venmo-like). **React Native** frontend, **Rust** backend.

## Non-Negotiable Invariants

1. Every code change requires a work item under `docs/ai/work-items/<feature-id>/`.
2. Pipeline: **Architecture → Implementation → Documentation → QA → Code Review → Commit** — no skipping.
3. Failing QA/Review → Implementation; doc gaps → Documenter; `BLOCKED` → Architecture.
4. Reducers must be pure; no side effects in reducers.
5. No feature complete without acceptance criteria, tests, **JSDoc**, evidence, and independent review.
6. **Only Committer Agent** may create commits, after all gates pass, **commitlint** passes, and commit is authorized.
7. Never bypass pipeline for "small" changes.
8. New dependencies/patterns require documented rationale (ADR when material).
9. Never claim success without executed verification.

## Agent Roles

| Agent          | Artifact               | Read                              |
| -------------- | ---------------------- | --------------------------------- |
| Architecture   | `01-architecture.md`   | `.ai/agents/architecture.md`      |
| Implementation | `02-implementation.md` | `.ai/agents/implementation.md`    |
| Documenter     | `03-documentation.md`  | `.ai/agents/documenter.md`        |
| QA             | `04-qa-report.md`      | `.ai/agents/quality-assurance.md` |
| Code Review    | `05-code-review.md`    | `.ai/agents/code-reviewer.md`     |
| Committer      | `06-commit-report.md`  | `.ai/agents/committer.md`         |

Manifest: `.ai/manifest.yml`

## Frontend Tooling (Enforced)

- **ESLint** — lint + architectural rules + `eslint-plugin-jsdoc`
- **Prettier** — `npm run format:check`
- **commitlint** — Conventional Commits on every commit (`commitlint.config.cjs`)

## Frontend Rules (React Native)

- Redux Toolkit only for app state
- RTK Query only for API
- Atomic Design under `src/`
- JSDoc on all exports; Mermaid for non-trivial flows

## Backend Rules (Rust)

- Layered workspace; `///` rustdoc on public items
- Integer money, idempotency, transactional transfers
- Gates: `cargo fmt --check`, `clippy -D warnings`, `cargo test`

## Policies & Skills

- Policies: `.ai/policies/` (including `documentation.md`, `frontend-tooling.md`)
- Skills: `.ai/skills/` (including `jsdoc-and-mermaid.md`, `conventional-commits.md`)
- Templates: `.ai/templates/`
- ADRs: `docs/ai/adr/`

## Git

Conventional Commits **enforced** via commitlint. Only Committer Agent commits. No `--no-verify`.
