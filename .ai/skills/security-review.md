# Skill: Security Review

Use when QA or Code Review touches security-sensitive areas.

## Triggers

- Authentication / authorization
- Password handling
- Session/token storage
- Money transfers
- Input validation
- Logging and error messages
- New dependencies
- CORS, headers, rate limits

## Checklist

### Authentication

- [ ] Passwords hashed (argon2/bcrypt), never logged
- [ ] Timing-safe comparison where applicable
- [ ] Generic error messages on login failure

### Authorization

- [ ] Users cannot access others' balances/transfers
- [ ] Transfer debit only from authenticated user's account

### Input Validation

- [ ] Amounts positive, bounded, integer minor units
- [ ] Usernames/recipients validated server-side
- [ ] Idempotency key format validated

### API Security

- [ ] HTTPS assumed in production docs
- [ ] No secrets in repo
- [ ] Structured errors without stack traces to clients

### Dependencies

- [ ] `cargo audit` / `npm audit` reviewed for new deps
- [ ] Minimal dependency surface

### Logging

- [ ] No passwords, tokens, or full PAN in logs
- [ ] Request IDs for audit trail

## Output

Document in `03-qa-report.md` or `04-code-review.md`:

```markdown
## Security Review
- **Scope:** ...
- **Findings:** None | List with severity
- **Status:** PASS | FAIL
```

## Severity

- **Critical** — blocks PASS/APPROVED
- **High** — blocks unless fixed or accepted via ADR
- **Medium/Low** — track as follow-up with owner
