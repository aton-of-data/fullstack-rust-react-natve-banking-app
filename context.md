# Ficus Platforms — Canonical Project Context

This file is the **single source of truth** for product requirements, technical direction, and AI engineering governance. All agent instruction files (`CLAUDE.md`, `AGENTS.md`, `.cursor/rules/`, `.github/copilot-instructions.md`) must mirror these principles without contradiction or weakening.

---

## Product: Money Transfer & Balance Tracking App

Develop a basic money transfer and balance tracking app with a global feed (think Venmo).

### Product Functionality

1. **Login** — Users log in with username and password. Basic auth is sufficient; no third-party integration required.
2. **Send money** — Users find other users by username and send money. The system must properly account for everyone's balances.
3. **Global feed** — A live-updating list of all transactions across the system. New transactions appear automatically in real time (no manual refresh).
4. **README** — Instructions for running the app, stack choices, and approach context.

### Money Integrity Requirements

This is a money app. Getting the math right under realistic conditions is non-negotiable.

Consider deliberately:

- How balances are represented and derived
- Concurrent transfers from the same account
- Client retries after timeout or network failure
- Auditability and account history reconstruction

We do not require a full production banking ledger, but implementation must have clear invariants, realistic failure behavior, and documented tradeoffs.

Research and apply judgment on:

- Idempotency keys for retry-safe APIs
- Integer-based money representation
- Append-only ledgers
- Double-entry accounting
- Database transactions and isolation

### Testing Expectations

Automated tests for money movement logic are required.

**Minimum:** a test with 100 concurrent transfers from the same funded account proving:

- No account balance goes negative
- Total money in the system is conserved
- Only fundable transfers succeed
- Failed transfers leave no partial state
- Idempotency handles duplicate requests

**Also required:** at least one retry-behavior test (e.g., duplicate submission must not double-charge).

### Tech Stack

| Layer    | Choice              |
|----------|---------------------|
| Frontend | Pure React Native   |
| Backend  | Rust                |

---

## AI Engineering Governance

### Shared Invariants (Non-Negotiable)

1. Every code change requires a tracked work item under `docs/ai/work-items/<feature-id>/`.
2. Every work item must move through **Architecture → Implementation → Documentation → QA → Code Review → Commit**.
3. A failing gate returns work to the **Implementation Agent** (or **Documenter Agent** for documentation-only fixes).
4. Reducers must remain pure. Side effects are never allowed inside reducers.
5. No feature is complete without acceptance criteria, tests, JSDoc documentation, verification evidence, and independent review.
6. The **Committer Agent** is the only agent allowed to create commits.
7. The Committer Agent may commit only after all prior gates have passed, **Conventional Commits** are validated, and authorization to commit exists.
8. Never bypass the pipeline because a change looks small.
9. Never introduce a new dependency, framework, abstraction, or architectural pattern without documenting the rationale.
10. Never claim success without evidence.

### Delivery Pipeline

```
Architecture → Implementation → Documentation → QA → Code Review → Commit
```

| Stage            | Agent              | Output Artifact                                      |
|------------------|--------------------|------------------------------------------------------|
| Architecture     | Architecture Agent | `docs/ai/work-items/<id>/01-architecture.md`         |
| Implementation   | Implementation Agent | `docs/ai/work-items/<id>/02-implementation.md`   |
| Documentation    | Documenter Agent   | `docs/ai/work-items/<id>/03-documentation.md`        |
| Quality Assurance| QA Agent           | `docs/ai/work-items/<id>/04-qa-report.md`            |
| Code Review      | Code Reviewer Agent| `docs/ai/work-items/<id>/05-code-review.md`        |
| Commit           | Committer Agent    | `docs/ai/work-items/<id>/06-commit-report.md`      |

**Gate outcomes:**

- Documentation: `PASS` | `FAIL` | `BLOCKED` — `FAIL` (code gaps) → Implementation; `FAIL` (doc-only) → Documenter; `BLOCKED` → Architecture
- QA: `PASS` | `FAIL` | `BLOCKED` — `FAIL` → Implementation; `BLOCKED` → Architecture
- Review: `APPROVED` | `CHANGES_REQUESTED` | `BLOCKED` — `CHANGES_REQUESTED` → Implementation or Documenter

No agent may skip, reorder, or self-approve a required stage.

### Frontend Tooling (Enforced)

| Tool | Purpose | Gate command |
|------|---------|--------------|
| ESLint | Lint + architectural restrictions + JSDoc rules | `npm run lint` |
| Prettier | Consistent formatting | `npm run format:check` |
| eslint-plugin-jsdoc | JSDoc completeness and validity | included in `npm run lint` |
| commitlint | Conventional Commits enforcement | `npx commitlint --edit` (commit-msg hook) |

Policy: `.ai/policies/frontend-tooling.md`. Templates: `.ai/templates/frontend-tooling/`.

### React Native Architecture (Frontend)

- **State:** Redux Toolkit only. No custom React Context, `useState`, or `useReducer` for application/page/form/workflow/loading/error/selection state.
- **Side effects:** RTK Query lifecycle handlers, listener middleware, or named async workflows — never inside reducers.
- **API:** RTK Query only. No `fetch`, Axios, or direct API clients from UI, hooks, screens, or slices.
- **UI:** Atomic Design under `src/` (`app/`, `shared/ui/`, `entities/`, `features/`, `widgets/`, `pages/`).
- **Lint restrictions:** prohibit `useState`, `useReducer`, `createContext`, `useContext`, `fetch`, `axios` except ADR-approved exceptions.
- **Documentation:** JSDoc on all exported symbols; Mermaid diagrams in module docs and architecture artifacts.

### Rust Backend Architecture

Layered workspace under `backend/`:

```
backend/crates/
  domain/  application/  contracts/
  adapters-http/  adapters-persistence/  infrastructure/  testkit/
```

Dependency direction: `domain ← application ← adapters & infrastructure ← composition root`.

Default stack: Tokio, Axum, Tower, Serde, PostgreSQL, SeaORM, tracing, utoipa, thiserror. Pin via `Cargo.lock`.

Quality gates: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo nextest run` (when configured), `cargo deny check`, `cargo audit`.

Rust documentation: `///` doc comments on public items; module docs with examples where non-obvious.

### Architecture Decision Records

ADRs live at `docs/ai/adr/`. Required when changing state management, API contracts, schema/migrations, auth, crate boundaries, dependencies, cross-cutting infrastructure, or repository rules. ADRs should include Mermaid diagrams when they clarify flows or boundaries.

### Quality & Security

Every change includes proportionate verification: typecheck, **lint**, **prettier**, **JSDoc**, unit tests, integration/contract tests when relevant, accessibility for mobile, security review for auth/secrets/validation, performance review for lists/API/DB.

**Prohibited bypasses:** `TODO: test later`, temporary direct fetch, temporary `useState`, temporary Context, temporary ORM in domain, skipped migrations, undocumented lint disables, non-conventional commit messages.

### Git & Commits

- **Conventional Commits enforced** via commitlint (commit-msg hook). Invalid messages block commit.
- Only the Committer Agent creates commits.
- Small, atomic, reversible commits.
- No amend, force-push, or rebase unless explicitly authorized.
- No `--no-verify` to skip commitlint.

### Governance File Map

| Path | Purpose |
|------|---------|
| `context.md` | Canonical source (this file) |
| `CLAUDE.md` | Claude Code / Opus entry point |
| `AGENTS.md` | Model-agnostic agent entry point |
| `.ai/manifest.yml` | Machine-readable pipeline manifest |
| `.ai/policies/` | Detailed policy documents |
| `.ai/agents/` | Per-agent contracts |
| `.ai/skills/` | Reusable skill playbooks |
| `.ai/templates/` | Handoff and report templates |
| `.cursor/rules/` | Cursor IDE rules |
| `.github/copilot-instructions.md` | GitHub Copilot instructions |
| `commitlint.config.cjs` | Conventional Commits schema |
| `docs/ai/work-items/` | Per-feature pipeline artifacts |
| `docs/ai/adr/` | Architecture Decision Records |

### Material Assumptions (Recorded)

1. **Greenfield repository** — no existing code; governance bootstrapped before implementation.
2. **Package managers** — npm for React Native; Cargo for Rust.
3. **Real-time feed** — WebSocket or SSE via backend; RTK Query integration on frontend (ADR required).
4. **Money representation** — integer minor units (cents) in backend domain; ADR-0001 proposed.
5. **CI** — GitHub Actions must run lint, format:check, test, and commitlint on PRs.
6. **Documentation** — TypeScript/JSDoc for frontend; rustdoc for backend public APIs.
