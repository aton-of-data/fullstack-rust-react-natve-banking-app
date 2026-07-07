# Skill: Git Commit

**Only Committer Agent uses this skill.**

## Preconditions

Verify artifacts:

```bash
ls docs/ai/work-items/<feature-id>/
# 01-architecture.md 02-implementation.md 03-documentation.md
# 04-qa-report.md 05-code-review.md
```

Confirm outcomes:

- Documentation: `PASS`
- QA: `PASS`
- Review: `APPROVED`
- User authorized commit

## Pre-Commit Inspection

```bash
git status
git diff
git diff --staged
```

Reject if:

- Secrets (`.env`, keys, tokens)
- Debug `console.log` / `dbg!` left intentionally
- Unrelated files
- Failing tests (re-run if needed)

## Validate Conventional Commit Message

```bash
echo "feat(scope): description

Work-Item: <feature-id>" | npx commitlint
```

Must exit 0. See `.ai/skills/conventional-commits.md`.

## Stage and Commit

```bash
git add <intended files only>
git commit -m "$(cat <<'EOF'
feat(scope): short description

Work-Item: <feature-id>

EOF
)"
```

Husky `commit-msg` hook runs commitlint automatically. **Never use `--no-verify`.**

## Post-Commit

```bash
git log -1 --format='%H %s'
git status
```

## Record in `06-commit-report.md`

- Commit hash
- Full message
- commitlint validation evidence
- File list (`git show --stat`)
- Verification commands re-run (optional but recommended)

## Forbidden

- `git commit --amend` (unless hook-modified files per user rules)
- `git push --force`
- `--no-verify`
- Non-conventional commit messages
- Committing without PASS documentation, PASS QA, APPROVED review
