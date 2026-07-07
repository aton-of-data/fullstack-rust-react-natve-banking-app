# Skill: Conventional Commits

Use by Committer Agent before every commit.

## Format

```text
<type>(<scope>): <description>

[optional body]

Work-Item: <feature-id>
```

## Types

| Type | When |
|------|------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Formatting (no logic change) |
| `refactor` | Code change without feat/fix |
| `perf` | Performance improvement |
| `test` | Tests only |
| `build` | Build system, dependencies |
| `ci` | CI configuration |
| `chore` | Maintenance |
| `revert` | Revert prior commit |

## Scopes (examples)

`frontend`, `backend`, `auth`, `transfer`, `feed`, `docs`, `ci`, `deps`

## Rules

- **Imperative mood:** "add login" not "added login"
- **Lowercase** type and scope
- **No trailing period** in description
- **Header ≤ 100 characters**
- **Breaking changes:** `feat!: ` or footer `BREAKING CHANGE: description`

## Validation

```bash
# Dry-run message validation
echo "feat(transfer): add idempotent send endpoint" | npx commitlint

# commit-msg hook runs automatically on git commit (when husky configured)
```

## Examples

```text
feat(transfer): add concurrent-safe debit use case

Implement row-level locking and idempotency key storage.
Includes 100-concurrent-transfer integration test.

Work-Item: transfer-send-money
```

```text
fix(frontend): correct balance display formatting

Work-Item: feed-realtime
```

```text
docs(auth): add JSDoc and login sequence diagram

Work-Item: auth-login
```

## Forbidden

- `wip`, `fix stuff`, `update`, `changes` (non-conventional)
- `--no-verify` to skip commitlint
- Multi-topic commits (split into atomic commits)

## Committer Checklist

- [ ] Message matches Conventional Commits
- [ ] `npx commitlint` passes on message
- [ ] Work-Item footer present
- [ ] Type/scope match actual changes
