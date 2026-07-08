# ADR-012: Mobile Transfer Idempotency Lifecycle

## Status

Accepted

## Context

Transfer API requires `Idempotency-Key` header. Mobile clients must survive network timeouts without double-debiting. Component-local state (`useState`) is prohibited; Redux Toolkit owns submission state.

## Decision

Introduce `transferSubmissionSlice` with:

- `idempotencyKey` generated once when entering confirm step (`goToConfirm`)
- `status`: `idle | submitting | unknown_outcome | succeeded | failed`
- Key preserved on `unknown_outcome` and `failed` (retryable errors)
- Key cleared and regenerated only on `resetSubmission` / successful completion / explicit cancel

RTK Query `createTransfer` accepts `{ body, idempotencyKey }` and sets the header per request.

## Consequences

- Retry after timeout sends identical key + body
- 409 idempotency conflict blocks automatic retry (`retryable: false`)
- Idempotency key is preserved across Back→Review while an attempt is active
- `beginTransferAttempt` is a no-op if a key already exists (prevents rotation after unknown outcomes)
- 401 clears auth via `baseQueryWithAuth` (`clearCredentials`, form/submission reset, RTK cache reset)
- Cache invalidation unchanged (`Balance`, `Feed`, `Transfers` tags)

## Alternatives considered

- Persist keys in SecureStore — rejected; active-transfer memory is sufficient for MVP
- RTK Query `fixedCacheKey` — rejected; does not model unknown-outcome lifecycle
