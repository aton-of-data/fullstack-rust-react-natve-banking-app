# Ficus HTTP API

OpenAPI documentation and contract reference for the Ficus REST API.

## Live Documentation

When the API is running:

| Resource     | URL                                         |
| ------------ | ------------------------------------------- |
| Swagger UI   | http://localhost:8080/api-docs              |
| OpenAPI JSON | http://localhost:8080/api-docs/openapi.json |

Source of truth: `apps/api/crates/adapters-http/src/openapi.rs` (utoipa).

## Base URL

```
http://localhost:8080
```

Production base URL is deployment-specific. All versioned routes are under `/v1`.

## Authentication

Bearer JWT obtained from `POST /v1/auth/login`.

```http
Authorization: Bearer <access_token>
```

Token expiry configured via `JWT_EXPIRY_SECS` (default 3600).

## Endpoints Summary

| Method | Path                      | Auth | Description              |
| ------ | ------------------------- | ---- | ------------------------ |
| `POST` | `/v1/auth/login`          | No   | Username/password login  |
| `POST` | `/v1/auth/logout`         | Yes  | End session              |
| `GET`  | `/v1/auth/me`             | Yes  | Current user profile     |
| `GET`  | `/v1/users`               | Yes  | Search users by prefix   |
| `GET`  | `/v1/accounts/me/balance` | Yes  | Account balance          |
| `GET`  | `/v1/accounts/me/ledger`  | Yes  | Paginated ledger entries |
| `POST` | `/v1/transfers`           | Yes  | Create transfer          |
| `GET`  | `/v1/feed`                | Yes  | Paginated global feed    |
| `GET`  | `/v1/feed/stream`         | Yes  | SSE live feed            |
| `GET`  | `/health/live`            | No   | Liveness probe           |
| `GET`  | `/health/ready`           | No   | Readiness probe          |
| `GET`  | `/metrics`                | No   | Prometheus metrics       |

## Money Fields

All monetary amounts are **string-encoded integer minor units** (e.g., `"100000"` = $1,000.00 USD).

| Field           | Example    |
| --------------- | ---------- |
| `balance_minor` | `"100000"` |
| `amount_minor`  | `"1500"`   |

Currency is ISO 4217 code (currently `USD` only).

## Transfer Request

```json
POST /v1/transfers
Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000

{
  "recipient_username": "bob",
  "amount_minor": "1000",
  "currency": "USD",
  "description": "Lunch"
}
```

### Response

```json
{
  "transfer_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "status": "COMPLETED",
  "sender_balance_minor": "99000",
  "currency": "USD",
  "created_at": "2025-07-07T12:00:00Z"
}
```

### Error Responses

| Status | Code                   | When                              |
| ------ | ---------------------- | --------------------------------- |
| 400    | `VALIDATION_ERROR`     | Invalid amount, currency, or body |
| 401    | `UNAUTHORIZED`         | Missing or invalid token          |
| 404    | `RECIPIENT_NOT_FOUND`  | Unknown username                  |
| 409    | `IDEMPOTENCY_CONFLICT` | Key reused with different body    |
| 422    | `INSUFFICIENT_FUNDS`   | Declined transfer                 |
| 429    | `RATE_LIMITED`         | Login or transfer rate limit      |

Error body shape:

```json
{
  "error": {
    "code": "INSUFFICIENT_FUNDS",
    "message": "Insufficient funds for transfer"
  }
}
```

## Pagination

List endpoints return:

```json
{
  "items": [ ... ],
  "next_cursor": "opaque-cursor-or-null"
}
```

Pass `?cursor=<next_cursor>` on subsequent requests.

## SSE Feed Stream

```http
GET /v1/feed/stream HTTP/1.1
Authorization: Bearer <token>
Accept: text/event-stream
Last-Event-ID: <optional-transfer-uuid>
```

Events:

```
id: <transfer_id>
event: transfer
data: { ... FeedItemResponse ... }
```

## Rate Limits

| Endpoint              | Default limit      |
| --------------------- | ------------------ |
| `POST /v1/auth/login` | 10 / minute / IP   |
| `POST /v1/transfers`  | 30 / minute / user |

Configurable via `LOGIN_RATE_LIMIT_PER_MIN` and `TRANSFER_RATE_LIMIT_PER_MIN`.

## TypeScript Contracts

Shared types: `packages/contracts/src/index.ts`. Mobile imports `@ficus/contracts`.

## Generating Clients

```bash
# Example: openapi-generator (not bundled)
npx @openapitools/openapi-generator-cli generate \
  -i http://localhost:8080/api-docs/openapi.json \
  -g typescript-fetch \
  -o ./generated/api-client
```

## Related

- [Idempotency](../architecture/idempotency.md)
- [Real-time feed](../architecture/realtime-feed.md)
- [ADR-003](../ai/adr/003-jwt-authentication.md)
