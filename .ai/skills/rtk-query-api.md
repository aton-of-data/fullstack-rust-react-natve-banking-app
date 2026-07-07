# Skill: RTK Query API Layer

## Base API

```text
src/app/api/baseApi.ts    # createApi, baseQuery with auth
src/features/<feature>/api.ts  # injectEndpoints
```

## Requirements

- Single `baseApi` with shared `baseQuery`
- Auth token injection in `prepareHeaders`
- Normalized error shape in `baseQuery` wrapper
- Tag types for cache invalidation
- Retry policy documented per endpoint type

## Endpoint Pattern

```typescript
// features/transfers/api.ts
export const transfersApi = baseApi.injectEndpoints({
  endpoints: (build) => ({
    sendTransfer: build.mutation<TransferResult, SendTransferRequest>({
      query: (body) => ({ url: "/transfers", method: "POST", body }),
      invalidatesTags: ["Balance", "Feed"],
    }),
  }),
});
```

## Mutations

- Invalidate correct tags (`Balance`, `Feed`, `User`)
- Optimistic updates only with rollback on failure
- Idempotency-Key header for transfer mutations (product requirement)

## DTO Mapping

- `transformResponse` or explicit mappers in feature layer
- UI models ≠ raw API JSON when shapes differ

## Real-Time Feed

Options (ADR required):

- WebSocket push → `onCacheEntryAdded` + streaming updates
- Polling with short interval (document tradeoff)
- SSE via polyfill/native module

## Prohibited

- `fetch` in components, hooks, screens, slices
- Parallel HTTP clients (Axios) outside baseApi

## Verification

- Loading/error/success states via RTK Query hooks in pages/organisms
- Cache invalidation tested after mutations
