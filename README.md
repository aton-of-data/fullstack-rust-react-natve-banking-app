# Ficus Platforms

Money transfer and balance tracking app (Venmo-like) with a live global transaction feed. React Native mobile client, Rust API, PostgreSQL ledger.

## Architecture Overview

```mermaid
flowchart TB
  subgraph client["Mobile (React Native)"]
    UI["Pages / Widgets / Atoms"]
    RTK["Redux Toolkit slices"]
    RQ["RTK Query + SSE feed"]
    UI --> RTK
    UI --> RQ
    RQ --> RTK
  end

  subgraph api["API (Rust / Axum)"]
    HTTP["adapters-http"]
  APP["application use cases"]
    DOM["domain invariants"]
    PERS["adapters-persistence"]
    HTTP --> APP
    APP --> DOM
    APP --> PERS
  end

  subgraph data["Data & Ops"]
    PG[(PostgreSQL)]
    OBS["OTel → Prometheus / Loki / Grafana"]
  end

  client -->|"REST + SSE /v1/*"| HTTP
  PERS --> PG
  api --> OBS
```

**Layering:** `domain ← application ← adapters & infrastructure`. See [docs/architecture/rust-layering.md](docs/architecture/rust-layering.md).

**Money:** Integer minor units (`i64` cents) per [ADR-0001](docs/ai/adr/0001-integer-money-representation.md). Double-entry ledger with materialized balances.

**Real-time feed:** Server-Sent Events (`GET /v1/feed/stream`) with in-process broadcast; mobile merges SSE into RTK Query cache.

## Repository Layout

| Path                 | Purpose                                                        |
| -------------------- | -------------------------------------------------------------- |
| `apps/mobile`        | Expo React Native app (Redux Toolkit, RTK Query)               |
| `apps/api`           | Rust workspace (domain, application, adapters, infrastructure) |
| `packages/contracts` | Shared TypeScript API types                                    |
| `packages/money`     | Client-side money formatting (no float math)                   |
| `infra/`             | Docker, observability configs                                  |
| `docs/`              | Architecture, operations, API, ADRs                            |
| `.ai/`               | Agent governance (policies, skills, pipeline)                  |

## Prerequisites

- **Node.js** ≥ 20, **pnpm** 9 (`corepack enable && corepack prepare pnpm@9.15.0 --activate`)
- **Rust** stable (2021 edition), `cargo-nextest`, `cargo-llvm-cov` (optional, for coverage)
- **Docker** (PostgreSQL and optional observability stack)
- **Expo CLI** (via `pnpm` in mobile app)

## Quick Start

### 1. Install dependencies

```bash
pnpm install
```

### 2. Start PostgreSQL

```bash
make up
# or: docker compose up -d postgres
```

### 3. Configure API environment

Create `apps/api/.env` (or export variables):

```bash
DATABASE_URL=postgres://ficus_app:ficus_app_password@localhost:5432/ficus
DATABASE_MIGRATION_URL=postgres://ficus_migrator:ficus_migrator_password@localhost:5432/ficus
JWT_SECRET=local-dev-jwt-secret-change-in-production-min-32-chars
JWT_EXPIRY_SECS=3600
API_HOST=0.0.0.0
API_PORT=8080
ENVIRONMENT=development
CORS_ORIGINS=http://localhost:8081,http://localhost:19006
```

### 4. Migrate and seed

```bash
make db-migrate
make db-seed
```

### 5. Run API

```bash
make api-dev
# API: http://localhost:8080
# Swagger UI: http://localhost:8080/api-docs
```

### 6. Run mobile

```bash
pnpm mobile:dev
# Expo dev tools — use iOS simulator, Android emulator, or Expo Go
```

Point the mobile app at the API via `apps/mobile/src/services/config.ts` (default `http://localhost:8080`).

## Seed Credentials (Local Dev Only)

| Username  | Password      | Starting balance                |
| --------- | ------------- | ------------------------------- |
| `alice`   | `password123` | $1,000.00 (100,000 minor units) |
| `bob`     | `password123` | $500.00 (50,000 minor units)    |
| `charlie` | `password123` | $250.00 (25,000 minor units)    |

Balances are funded from a system account with full ledger audit trail. **Never use these credentials outside local development.**

## Common Commands

### Monorepo (frontend + tooling)

| Command                 | Description                          |
| ----------------------- | ------------------------------------ |
| `pnpm lint`             | ESLint + JSDoc across workspace      |
| `pnpm format:check`     | Prettier check                       |
| `pnpm typecheck`        | TypeScript across packages           |
| `pnpm test`             | Unit tests (Turbo)                   |
| `pnpm test:performance` | k6 load test (requires k6 installed) |
| `pnpm security:scan`    | gitleaks + trivy wrappers            |

### API (Rust)

| Command             | Description                     |
| ------------------- | ------------------------------- |
| `make api-dev`      | Run API locally                 |
| `make api-test`     | `cargo nextest run --workspace` |
| `make api-lint`     | `cargo fmt --check` + clippy    |
| `make api-coverage` | llvm-cov with 90% thresholds    |
| `make db-migrate`   | Apply SeaORM migrations         |
| `make db-seed`      | Seed dev users                  |

### Docker profiles

| Command        | Description                                           |
| -------------- | ----------------------------------------------------- |
| `make up`      | PostgreSQL only                                       |
| `make up-full` | Postgres + API container                              |
| `make up-obs`  | Observability stack (OTel, Prometheus, Grafana, Loki) |
| `make down`    | Stop all services                                     |

## Testing Highlights

Money integrity and HTTP contract tests live in `apps/api/crates/testkit/tests/`:

- `auth_http_api_test.rs` — login/me/logout, 401/429 contracts, audit correlation
- `transfer_http_api_test.rs` — transfer idempotency, validation, exact 100-concurrent HTTP test
- `feed_http_api_test.rs` — feed pagination, SSE content-type, metrics scrape
- `transfer_concurrency_test.rs` — 100 concurrent transfers (exact 50 success / 50 decline)
- `transfer_concurrency_extended_test.rs` — cross-account contention, inverse-direction locking
- `transfer_idempotency_test.rs` — duplicate idempotency key does not double-charge
- `transfer_partial_state_test.rs` — failed transfers leave no partial state
- `ledger_db_immutability_test.rs` — append-only ledger/audit at database level
- `ledger_balance_reconciliation_test.rs` — balances match ledger sums

```bash
# Postgres via Docker testcontainers, or:
export TEST_DATABASE_URL=postgres://user@localhost:5432/postgres

cd apps/api && cargo test -p ficus-testkit -- --test-threads=1
cd apps/api && cargo test --workspace -- --test-threads=1
pnpm --filter @ficus/mobile test
node scripts/performance/run-k6.mjs --ci   # skips if k6 missing
bash scripts/performance/reconcile-after-load.sh
```

## Documentation Index

| Topic            | Location                                                                                     |
| ---------------- | -------------------------------------------------------------------------------------------- |
| Financial ledger | [docs/architecture/financial-ledger.md](docs/architecture/financial-ledger.md)               |
| Idempotency      | [docs/architecture/idempotency.md](docs/architecture/idempotency.md)                         |
| Concurrency      | [docs/architecture/concurrency-control.md](docs/architecture/concurrency-control.md)         |
| Real-time feed   | [docs/architecture/realtime-feed.md](docs/architecture/realtime-feed.md)                     |
| Mobile state     | [docs/architecture/mobile-state-management.md](docs/architecture/mobile-state-management.md) |
| Rust layering    | [docs/architecture/rust-layering.md](docs/architecture/rust-layering.md)                     |
| OpenAPI          | [docs/api/openapi.md](docs/api/openapi.md)                                                   |
| Observability    | [docs/operations/observability.md](docs/operations/observability.md)                         |
| Runbook          | [docs/operations/runbook.md](docs/operations/runbook.md)                                     |
| ADRs             | [docs/ai/adr/](docs/ai/adr/)                                                                 |
| AI pipeline      | [AGENTS.md](AGENTS.md), [context.md](context.md)                                             |

## Design Trade-offs

| Decision      | Choice                                      | Rationale                               | Trade-off                                                                         |
| ------------- | ------------------------------------------- | --------------------------------------- | --------------------------------------------------------------------------------- |
| Money storage | `BIGINT` minor units                        | Exact arithmetic, simple domain model   | Display formatting only at boundaries                                             |
| Balance model | Materialized balance + append-only ledger   | Fast reads, auditable history           | Must keep balance and ledger in sync in one transaction                           |
| Concurrency   | `SELECT … FOR UPDATE` + serialization retry | Correct debits under contention         | Latency spikes under heavy same-account load                                      |
| Idempotency   | Per-user key + request fingerprint          | Safe client retries                     | Conflicting fingerprint returns 409                                               |
| Real-time     | SSE + in-memory broadcast                   | Simple, works with RN via XHR streaming | Not horizontally scalable without shared pub/sub                                  |
| Mobile state  | Redux Toolkit + RTK Query only              | Enforced purity, single data layer      | No local `useState` for app state                                                 |
| Auth          | JWT bearer tokens                           | Stateless API, mobile-friendly          | No server-side revocation list (logout is client-side + optional blocklist later) |
| Monorepo      | pnpm + Turbo + Cargo workspace              | Shared contracts, unified CI            | Two package managers (npm ecosystem + Rust)                                       |

## Governance

All production changes flow through the delivery pipeline documented in [AGENTS.md](AGENTS.md):

```
Architecture → Implementation → Documentation → QA → Code Review → Commit
```

Work items: `docs/ai/work-items/<feature-id>/`. Only the Committer Agent creates commits after gates pass and commitlint validates Conventional Commits.

## License

Private — interview / evaluation project.
