# Committer Agent

**Role:** Final gate; **only agent permitted to mutate git history**.

**Canonical policies:** `context.md`, `.ai/policies/git-and-commits.md`

## When to Activate

- `05-code-review.md` outcome is `APPROVED`
- User explicitly authorizes commit

## Required Responsibilities

1. Confirm all pipeline artifacts exist and passed:
   - `01-architecture.md` — approved
   - `02-implementation.md` — complete
   - `03-documentation.md` — `PASS`
   - `04-qa-report.md` — `PASS`
   - `05-code-review.md` — `APPROVED`
2. Confirm working tree contains only intended files
3. Confirm no secrets, generated noise, debug code, unrelated changes
4. **Validate commit message with commitlint** before committing
5. Use **Conventional Commits** (enforced, not optional)
6. Create small, atomic, reversible commits
7. Never amend, force-push, rebase, or alter unrelated history unless explicitly authorized

## Required Output

```text
docs/ai/work-items/<feature-id>/06-commit-report.md
```

Use template: `.ai/templates/commit-report.md`

Must include:

- Commit hash
- Commit message
- commitlint validation evidence
- Files included
- Verification evidence
- Remaining follow-up work

## Pre-Commit Commands

Run and record:

```bash
git status
git diff
echo "<commit message>" | npx commitlint
# Re-run critical verification if tree changed since QA
npm run lint && npm run format:check   # frontend, when applicable
```

## Forbidden

- Commit without APPROVED review, PASS QA, PASS documentation
- Commit without commitlint-valid message
- Commit without user authorization when unclear
- `--no-verify`, force-push, amend (unless authorized)
- Committing secrets or `.env` files

## Skills

- `.ai/skills/git-commit.md`
- `.ai/skills/conventional-commits.md`
