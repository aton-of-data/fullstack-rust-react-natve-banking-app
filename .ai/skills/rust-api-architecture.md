# Skill: Rust API Architecture

## Crate Responsibilities

| Crate                  | Contents                                                  |
| ---------------------- | --------------------------------------------------------- |
| `domain`               | `Money`, `Account`, `Transfer`, invariants, domain errors |
| `application`          | `SendTransfer`, `GetFeed`, ports, unit-of-work            |
| `contracts`            | HTTP request/response DTOs, OpenAPI schemas               |
| `adapters-http`        | Axum routes, extractors, status mapping                   |
| `adapters-persistence` | SeaORM entities, repos, migrations                        |
| `infrastructure`       | Config, DB pool, tracing, auth middleware                 |
| `testkit`              | Fixtures, test DB helpers                                 |

## Handler Pattern (Thin)

```rust
// adapters-http: parse → command → use case → response
async fn post_transfer(
    State(ctx): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, ApiError> {
    let idempotency_key = extract_idempotency_key(&headers)?;
    let cmd = req.into_command(idempotency_key)?;
    let result = ctx.send_transfer.execute(cmd).await?;
    Ok(Json(result.into()))
}
```

## Money Invariants

- Store amounts as `i64` minor units (or dedicated `Money` newtype)
- Transfer use case runs in DB transaction
- Check balance + debit + credit atomically
- Idempotency table keyed by `(user_id, idempotency_key)` or global key
- Ledger entries append-only

## Error Mapping

- Domain errors → `application` errors → HTTP status in adapter
- Never leak internal DB errors to clients

## Testing

```bash
cargo test -p domain
cargo test -p application
cargo test --workspace
```

Integration tests with testcontainers for PostgreSQL when configured.

## Required Product Tests

- `concurrent_transfers_from_same_account` — 100 parallel debits
- `duplicate_idempotency_key_does_not_double_charge`

## Verification Commands

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
