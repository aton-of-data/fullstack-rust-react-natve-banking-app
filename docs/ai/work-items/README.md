# Work Items

Each feature, bug fix, refactor, or migration gets a directory:

```text
docs/ai/work-items/<feature-id>/
  00-work-item.md          # optional summary
  01-architecture.md
  02-implementation.md
  03-documentation.md      # Documenter Agent — JSDoc + Mermaid
  04-qa-report.md
  05-code-review.md
  06-commit-report.md
```

`<feature-id>`: lowercase kebab-case (e.g., `auth-login`, `transfer-send-money`).

## Pipeline

```
Architecture → Implementation → Documentation → QA → Code Review → Commit
```

See `.ai/templates/` and `.ai/policies/delivery-pipeline.md`.
