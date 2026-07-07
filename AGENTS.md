# AGENTS.md — Ficus Platforms

Model-agnostic agent operating system. **Canonical source: `context.md`**.

## Purpose

Every feature, bug fix, refactor, migration, and architectural change flows through a non-skippable pipeline. This file applies to Claude Code, Cursor, GitHub Copilot, and any other coding agent.

## Pipeline

```
Architecture → Implementation → Documentation → QA → Code Review → Commit
```

| Stage | Agent | Output |
|-------|-------|--------|
| 1 | Architecture Agent | `docs/ai/work-items/<id>/01-architecture.md` |
| 2 | Implementation Agent | `docs/ai/work-items/<id>/02-implementation.md` |
| 3 | Documenter Agent | `docs/ai/work-items/<id>/03-documentation.md` |
| 4 | Quality Assurance Agent | `docs/ai/work-items/<id>/04-qa-report.md` |
| 5 | Code Reviewer Agent | `docs/ai/work-items/<id>/05-code-review.md` |
| 6 | Committer Agent | `docs/ai/work-items/<id>/06-commit-report.md` |

**Gate rules:**
- Documentation: `PASS` → QA; `FAIL` (code) → Implementation; `FAIL` (docs) → Documenter; `BLOCKED` → Architecture
- QA: `PASS` → Review; `FAIL` → Implementation; `BLOCKED` → Architecture
- Review: `APPROVED` → Commit; `CHANGES_REQUESTED` → Implementation or Documenter; `BLOCKED` → Architecture
- Only **Committer Agent** runs `git commit` with **commitlint-valid** Conventional Commits

## Shared Invariants

1. Tracked work item for every code change
2. Full pipeline required — no exceptions for size
3. Failing gate → Implementation or Documenter; BLOCKED → Architecture
4. Pure reducers; no side effects inside reducers
5. Acceptance criteria + tests + **JSDoc** + evidence + independent review required
6. Committer-only commits after all gates pass + commitlint + authorization
7. No new dependency/pattern without rationale
8. Never claim success without evidence

## Enforced Tooling

| Tool | Command | Purpose |
|------|---------|---------|
| ESLint | `npm run lint` | Lint + JSDoc rules |
| Prettier | `npm run format:check` | Formatting |
| commitlint | `npx commitlint` | Conventional Commits |

Config: `commitlint.config.cjs`. Templates: `.ai/templates/frontend-tooling/`.

## Stack

| Layer | Technology |
|-------|------------|
| Frontend | React Native, Redux Toolkit, RTK Query, Atomic Design, JSDoc, Mermaid |
| Backend | Rust (Tokio, Axum, PostgreSQL, SeaORM), rustdoc |

## Key Prohibitions

**Frontend:** `useState`, `useReducer`, `createContext`, `useContext`, `fetch`, `axios`; undocumented exports

**Backend:** ORM entities in domain; business logic in handlers

**All:** pipeline bypass, non-conventional commits, `--no-verify`, unverified "tests pass"

## Where to Read More

| Topic | Path |
|-------|------|
| Canonical context | `context.md` |
| Machine manifest | `.ai/manifest.yml` |
| Documenter Agent | `.ai/agents/documenter.md` |
| Documentation policy | `.ai/policies/documentation.md` |
| Frontend tooling | `.ai/policies/frontend-tooling.md` |
| JSDoc + Mermaid skill | `.ai/skills/jsdoc-and-mermaid.md` |
| Conventional Commits | `.ai/skills/conventional-commits.md` |

## Starting a New Change

1. Create `docs/ai/work-items/<feature-id>/`
2. **Architecture Agent** → `01-architecture.md`
3. **Implementation Agent** → `02-implementation.md`
4. **Documenter Agent** → `03-documentation.md` (JSDoc + Mermaid)
5. Continue through QA → Review → Commit
