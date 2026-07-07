# Skill: Feature Planning

Use during Architecture stage to structure a work item.

## Steps

1. **Assign feature ID** — kebab-case (e.g., `auth-login`, `feed-realtime`)

2. **Create work item directory**
   ```text
   docs/ai/work-items/<feature-id>/
   ```

3. **Draft from template** — `.ai/templates/feature-work-item.md`

4. **Define scope**
   - User-visible behavior
   - API changes
   - State ownership (which slice owns what)
   - Non-goals (explicit)

5. **Acceptance criteria** — testable, numbered

   Example:
   ```text
   AC-1: User can log in with valid username/password
   AC-2: Invalid credentials return error without leaking which field failed
   AC-3: Session persists across app restart (if in scope)
   ```

6. **Test matrix**

   | AC | Unit | Integration | E2E | Manual |
   |----|------|-------------|-----|--------|
   | AC-1 | | ✓ | | |

7. **Identify ADR needs** — see `.ai/skills/architecture-decision-record.md`

8. **Pipeline checklist**
   - [ ] `01-architecture.md`
   - [ ] `02-implementation.md`
   - [ ] `03-documentation.md`
   - [ ] `04-qa-report.md`
   - [ ] `05-code-review.md`
   - [ ] `06-commit-report.md`

## Rules

- No implementation before architecture approval
- Every AC must map to at least one test or verification step
