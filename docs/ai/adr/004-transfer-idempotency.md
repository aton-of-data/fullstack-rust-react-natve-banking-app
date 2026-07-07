# ADR-004: Transfer Idempotency

## Status

Accepted

## Context

Clients retry `POST /v1/transfers` after network timeouts. Without server-side deduplication, retries double-charge senders. Product requirements explicitly call for idempotency key support.

## Decision

Require **`Idempotency-Key` header** on transfer creation:

1. Key scoped per authenticated `user_id`
2. Store request fingerprint (recipient, amount, currency, description)
3. On replay with matching fingerprint, return stored response
4. On replay with mismatched fingerprint, return **409 Conflict**
5. Persist idempotency record in the **same transaction** as the transfer

## Alternatives Considered

- **Client-generated transfer UUID as body field** — duplicates header pattern; header is industry standard
- **Global idempotency key** — allows cross-user collision; rejected
- **At-least-once without idempotency** — unacceptable for money

## Consequences

- Clients must generate and persist keys per attempt
- Table growth requires TTL cleanup job (future)
- Integration tests must cover duplicate and conflict paths

## Migration Plan

N/A — schema includes `idempotency_requests` from initial migration.

## Rollback Plan

Removing idempotency would break safe retries; not planned.

## Approval Status

- Architecture Agent: Accepted
- Human reviewer: Pending
