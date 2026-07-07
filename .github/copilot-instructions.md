# GitHub Copilot Instructions — Ficus Platforms

Canonical source: `context.md`. Full agent system: `AGENTS.md`.

## Project

Venmo-like money transfer app. React Native frontend, Rust backend.

## Mandatory Pipeline

Every code change requires a work item at `docs/ai/work-items/<feature-id>/` through:

**Architecture → Implementation → Documentation → QA → Code Review → Commit**

Do not skip stages. Only Committer Agent commits with **Conventional Commits** validated by commitlint.

## Enforced Tooling

- **ESLint** + `eslint-plugin-jsdoc` → `npm run lint`
- **Prettier** → `npm run format:check`
- **commitlint** → `commitlint.config.cjs` (no `--no-verify`)

## Documenter Agent

After implementation, Documenter Agent must:

- Add JSDoc to all exported TypeScript/JavaScript symbols
- Add `///` rustdoc to public Rust items
- Create Mermaid diagrams for feature flows (sequence, state, flowchart)
- Produce `03-documentation.md` with PASS before QA

Skill: `.ai/skills/jsdoc-and-mermaid.md`

## Invariants

1. Work item required for every change
2. Full 6-stage pipeline — no size exceptions
3. JSDoc on exports; Mermaid where flows are non-trivial
4. Lint + prettier must pass before QA PASS
5. Conventional Commits enforced on every commit
6. Never claim success without running verification

## Frontend (React Native)

- Redux Toolkit for app state; RTK Query for API
- **Do not use:** `useState`, `useReducer`, custom Context, `fetch`, `axios`
- JSDoc on all exports

## Backend (Rust)

- Layered crates; `///` on public APIs
- Integer money, idempotent transfers

## References

- Documenter: `.ai/agents/documenter.md`
- Documentation policy: `.ai/policies/documentation.md`
- Frontend tooling: `.ai/policies/frontend-tooling.md`
- Conventional Commits: `.ai/skills/conventional-commits.md`
