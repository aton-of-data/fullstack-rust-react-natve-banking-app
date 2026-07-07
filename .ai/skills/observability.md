# Skill: Observability

## When to Use

Adding instrumentation, debugging production issues, configuring metrics/logs/traces, or reviewing observability-related changes.

## Stack

| Component      | Config path                       |
| -------------- | --------------------------------- |
| OTel Collector | `infra/otel/config.yaml`          |
| Prometheus     | `infra/prometheus/prometheus.yml` |
| Loki           | `infra/loki/config.yaml`          |
| Grafana        | `infra/grafana/provisioning/`     |

Start locally: `make up-obs`

## API Instrumentation

| Signal              | Implementation                                          |
| ------------------- | ------------------------------------------------------- |
| Traces              | `tracing` + OTLP when `OTEL_EXPORTER_OTLP_ENDPOINT` set |
| HTTP metrics        | `trace_metrics_middleware`, `/metrics` endpoint         |
| Request correlation | `X-Request-Id`, `X-Trace-Id` headers                    |
| SSE gauge           | `sse_connection_opened` / `sse_connection_closed`       |

## Environment Variables

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=ficus-api
RUST_LOG=info,ficus_api=debug
```

## Health Checks

| Endpoint        | Use           |
| --------------- | ------------- |
| `/health/live`  | Process alive |
| `/health/ready` | DB reachable  |

## Logging Rules

**Include:** `request_id`, `trace_id`, `user_id` (authenticated), transfer outcome

**Never log:** passwords, JWT tokens, raw idempotency keys in production

## Dashboards & Alerts

See `docs/operations/observability.md` for suggested panels and alert thresholds.

## Verification

```bash
curl -s http://localhost:8080/metrics | head
curl -s http://localhost:8080/health/ready
```

Grafana: http://localhost:3000 (admin/admin)

## ADR

ADR-009: Observability Stack

## CI Note

Full observability stack is not required for unit/integration CI. Security workflow runs container scans separately.
