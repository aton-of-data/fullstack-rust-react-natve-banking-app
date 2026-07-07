# ADR-009: Observability Stack

## Status

Accepted

## Context

Operators and developers need visibility into API health, transfer behavior, and real-time connection counts. Production debugging requires correlated logs and metrics without exposing secrets.

## Decision

Deploy a **metrics + logs + traces** stack for local and production parity:

| Signal     | Technology                                          |
| ---------- | --------------------------------------------------- |
| Metrics    | Prometheus scraping `/metrics` + OTel collector     |
| Traces     | OpenTelemetry OTLP export (`tracing-opentelemetry`) |
| Logs       | `tracing` structured logs → Loki via collector      |
| Dashboards | Grafana (provisioned datasources)                   |

Docker Compose profile `observability` runs OTel Collector, Prometheus, Loki, Grafana.

API env: `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_SERVICE_NAME`.

Custom metrics: HTTP latency histogram, transfer counters, SSE active connections gauge.

## Alternatives Considered

- **Logs only** — insufficient for latency and saturation analysis
- **Commercial APM only** — adds cost; OTel keeps portability
- **No local observability** — slows incident reproduction

## Consequences

- Additional containers for full stack (~4 services)
- Developers can run API-only without observability profile
- CI does not require observability stack for unit/integration tests

## Migration Plan

Configs in `infra/otel/`, `infra/prometheus/`, `infra/loki/`, `infra/grafana/`.

## Rollback Plan

Disable OTel exporter env var; API runs with stdout logs only.

## Approval Status

- Architecture Agent: Accepted
- Human reviewer: Pending
