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
- 401 triggers existing auth listener logout flow
- Cache invalidation unchanged (`Balance`, `Feed`, `Transfers` tags)

## Alternatives considered

- Persist keys in SecureStore — rejected; active-transfer memory is sufficient for MVP
- RTK Query `fixedCacheKey` — rejected; does not model unknown-outcome lifecycle
