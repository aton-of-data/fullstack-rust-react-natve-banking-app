# Git and Commits Policy

Canonical reference: `context.md`

## Commit Authority

**Only the Committer Agent may create commits.**

Other agents must not run `git commit`, `git add` for commit purposes, amend, rebase, or force-push.

## Preconditions for Commit

1. `01-architecture.md` — approved
2. `02-implementation.md` — complete
3. `03-documentation.md` — outcome `PASS`
4. `04-qa-report.md` — outcome `PASS`
5. `05-code-review.md` — outcome `APPROVED`
6. Explicit authorization to commit (user request or documented approval)
7. Working tree contains only intended changes
8. No secrets, debug code, or unrelated files
9. **Commit message passes commitlint** (Conventional Commits)

## Conventional Commits (Enforced)

[Conventional Commits](https://www.conventionalcommits.org/) are **mandatory**, not optional.

**Enforcement:**

- `commitlint.config.cjs` at repository root
- Husky `commit-msg` hook: `npx commitlint --edit $1`
- Committer Agent validates message before commit:
  ```bash
  echo "<proposed message>" | npx commitlint
  ```

**Format:**

```text
<type>(<scope>): <short description>

[optional body]

Work-Item: <feature-id>
```

**Types:** `feat`, `fix`, `refactor`, `test`, `docs`, `style`, `chore`, `ci`, `perf`, `build`, `revert`

**Scope examples:** `frontend`, `backend`, `transfer`, `auth`, `feed`, `docs`

Skill: `.ai/skills/conventional-commits.md`

## Commit Practices

- Small, atomic, reversible commits
- One logical change per commit when possible
- Reference work item ID in body: `Work-Item: transfer-send-money`
- Imperative mood, lowercase, no trailing period in subject

## Forbidden Unless Explicitly Authorized

- `git commit --amend`
- `git push --force`
- `git rebase` on shared branches
- Skipping hooks (`--no-verify`) — **blocks commitlint**
- Non-conventional commit messages
- Committing `.env`, credentials, or generated secrets

## Commit Report

Committer Agent produces `06-commit-report.md` with:

- Commit hash
- Commit message
- commitlint validation evidence
- Files included
- Verification evidence (re-run critical checks if needed)
- Remaining follow-up work

## Git Safety

- Never update git config
- Never force-push to main/master
- Verify `git status` and `git diff` before commit
- Do not commit if hooks fail; fix message or code and create new commit (no amend unless authorized)
