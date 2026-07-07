# ADR-006: SSE Global Feed

## Status

Accepted

## Context

The product requires a live global transaction feed without manual refresh. React Native must receive updates on device without polling.

## Decision

- **Protocol:** Server-Sent Events (`text/event-stream`)
- **Endpoint:** `GET /v1/feed/stream` (authenticated)
- **Broadcast:** `tokio::sync::broadcast` channel in `FeedService`
- **Replay:** `Last-Event-ID` header replays missed events before live stream
- **Mobile:** XHR-based SSE parser (no native EventSource) integrated via RTK Query `onCacheEntryAdded`
- **Initial load:** `GET /v1/feed` REST pagination

## Alternatives Considered

- **WebSocket** — bidirectional overhead not needed
- **Long polling** — higher latency and load
- **Client-only polling** — fails product real-time requirement

## Consequences

- Single-instance broadcast; multi-instance needs shared pub/sub (future ADR)
- SSE connections counted in Prometheus for leak detection
- 15s keep-alive prevents proxy timeouts

## Migration Plan

N/A — initial feed implementation.

## Rollback Plan

Fall back to REST polling with degraded UX; document in mobile app.

## Approval Status

- Architecture Agent: Accepted
- Human reviewer: Pending
