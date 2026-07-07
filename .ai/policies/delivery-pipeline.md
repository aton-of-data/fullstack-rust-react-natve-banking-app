# Delivery Pipeline Policy

Canonical reference: `context.md` and `.ai/manifest.yml`

## Pipeline (Non-Skippable)

```
Architecture → Implementation → Documentation → QA → Code Review → Commit
```

## Work Item Structure

Every change starts with a work item directory:

```text
docs/ai/work-items/<feature-id>/
  00-work-item.md          # optional summary (from template)
  01-architecture.md       # Architecture Agent
  02-implementation.md     # Implementation Agent
  03-documentation.md      # Documenter Agent
  04-qa-report.md          # QA Agent
  05-code-review.md        # Code Reviewer Agent
  06-commit-report.md      # Committer Agent
```

`<feature-id>` format: lowercase kebab-case (e.g., `auth-login`, `transfer-send-money`).

## Stage Contracts

### 1. Architecture

**Entry:** New or resumed work item request.

**Exit criteria:**

- `01-architecture.md` complete per template
- Acceptance criteria defined
- Test strategy defined
- Mermaid diagrams for non-trivial flows (when applicable)
- Explicit "Implementation Agent approved to proceed" section
- ADR created if required

**Forbidden:** Production code changes (except disposable investigation spikes, clearly marked).

### 2. Implementation

**Entry:** Approved `01-architecture.md`.

**Exit criteria:**

- Code, tests, migrations per architecture
- `02-implementation.md` with files changed, commands run, Documenter entry criteria
- Code is lint-clean and prettier-formatted (or Documenter/QA will fail)

**On material deviation:** Escalate to Architecture Agent before continuing.

**Note:** Implementation provides minimal inline comments; full JSDoc is Documenter Agent responsibility.

### 3. Documentation

**Entry:** Complete `02-implementation.md`.

**Exit criteria:**

- `03-documentation.md` with outcome `PASS`, `FAIL`, or `BLOCKED`
- JSDoc on all exported TypeScript/JavaScript symbols (frontend)
- `///` doc comments on public Rust items (backend)
- Mermaid diagrams in feature docs where flows/architecture need visualization
- `npm run lint` and `npm run format:check` pass (frontend, when scaffold exists)

| Outcome                                  | Action                     |
| ---------------------------------------- | -------------------------- |
| PASS                                     | Proceed to QA              |
| FAIL (missing exports/types to document) | Return to Implementation   |
| FAIL (doc-only gaps)                     | Documenter Agent continues |
| BLOCKED                                  | Return to Architecture     |

### 4. Quality Assurance

**Entry:** Documentation `PASS` and `03-documentation.md` complete.

**Exit criteria:**

- `04-qa-report.md` with outcome `PASS`, `FAIL`, or `BLOCKED`
- Every acceptance criterion validated with evidence
- Lint, prettier, typecheck, tests executed; outputs referenced

| Outcome | Action                   |
| ------- | ------------------------ |
| PASS    | Proceed to Code Review   |
| FAIL    | Return to Implementation |
| BLOCKED | Return to Architecture   |

### 5. Code Review

**Entry:** QA `PASS`.

**Exit criteria:**

- `05-code-review.md` with `APPROVED`, `CHANGES_REQUESTED`, or `BLOCKED`
- Independent review of correctness, security, architecture, **documentation quality**

| Outcome                  | Action                            |
| ------------------------ | --------------------------------- |
| APPROVED                 | Proceed to Commit (if authorized) |
| CHANGES_REQUESTED (code) | Return to Implementation          |
| CHANGES_REQUESTED (docs) | Return to Documenter              |
| BLOCKED                  | Return to Architecture            |

### 6. Commit

**Entry:** Review `APPROVED` + explicit commit authorization.

**Exit criteria:**

- `06-commit-report.md` with commit hash, message, files, evidence
- Commit message passes **commitlint** (Conventional Commits)
- Only Committer Agent runs `git commit`

## Handoff Rules

- Downstream agents must not trust upstream notes without verification.
- Upstream artifacts are read-only once approved.
- Re-opening a stage invalidates downstream artifacts unless explicitly noted.

## Authorization

- Commit requires human or explicit user authorization in the session.
- Agents must not commit unless the user requests it or pipeline authorization exists.
