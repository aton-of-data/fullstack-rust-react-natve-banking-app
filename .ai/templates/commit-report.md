# Commit Report: <feature-id>

## Commit
- **Hash:** `<full sha>`
- **Message:**
  ```
  <commit message>
  ```

## Pipeline Verification
| Artifact | Required | Present | Valid |
|----------|----------|---------|-------|
| 01-architecture.md | Approved | ✓ | ✓ |
| 02-implementation.md | Complete | ✓ | ✓ |
| 03-documentation.md | PASS | ✓ | ✓ |
| 04-qa-report.md | PASS | ✓ | ✓ |
| 05-code-review.md | APPROVED | ✓ | ✓ |

## Conventional Commits Validation
| Check | Result |
|-------|--------|
| `echo "<message>" \| npx commitlint` | exit 0 |
| Type/scope valid | ✓ |
| Work-Item footer | ✓ |

## Files Included
```
<git show --stat output or file list>
```

## Pre-Commit Checks
| Check | Result |
|-------|--------|
| git status clean (intended only) | |
| No secrets | |
| Authorization received | |
| commitlint passed | |
| lint + format:check (if applicable) | |

## Verification Evidence
| Command | Exit Code | Summary |
|---------|-----------|---------|
| | | |

## Remaining Follow-Up Work
- ...

## Commit Sign-Off
> **Agent:** Committer Agent
>
> **Date:** YYYY-MM-DD
