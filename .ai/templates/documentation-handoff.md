# Documentation Handoff: <feature-id>

## Outcome

<!-- PASS | FAIL | BLOCKED -->

**PASS**

## Summary

<what was documented>

## JSDoc Coverage

| File | Symbol | Documented | Notes |
| ---- | ------ | ---------- | ----- |
|      |        | ✓          |       |

### Exports Without JSDoc (must be empty for PASS)

None

## Rust Doc Coverage (if backend)

| Crate | Item | Documented |
| ----- | ---- | ---------- |
|       |      | ✓          |

## Mermaid Diagrams

| Diagram | Location              | Type            | Describes |
| ------- | --------------------- | --------------- | --------- |
|         | `03-documentation.md` | sequenceDiagram |           |

### Diagram Sources

````markdown
<!-- Paste or link Mermaid source here -->

```mermaid
sequenceDiagram
  ...
```
````

## Module READMEs Created/Updated

| Path | Summary |
| ---- | ------- |
|      |         |

## Tooling Verification

| Command                | Exit Code | Summary             |
| ---------------------- | --------- | ------------------- |
| `npm run lint`         | 0         | JSDoc + ESLint pass |
| `npm run format:check` | 0         | Prettier pass       |

## Documentation Decisions

- ...

## Known Gaps

None | <list with owner>

## QA Entry Criteria

- [ ] All exported symbols documented per policy
- [ ] Mermaid diagrams match implementation
- [ ] Lint and format:check pass
- [ ] No stale docs from prior features

## Documenter Sign-Off

> **Agent:** Documenter Agent
>
> **Date:** YYYY-MM-DD
