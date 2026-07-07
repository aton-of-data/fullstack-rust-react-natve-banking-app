# ADR-003: JWT Authentication

## Status

Accepted

## Context

Mobile clients need stateless authentication after username/password login. Third-party OAuth is out of scope. Tokens must work with standard `Authorization: Bearer` headers for REST and SSE.

## Decision

- **Login:** `POST /v1/auth/login` validates credentials (Argon2 password hash) and returns a signed **JWT access token**
- **Transport:** `Authorization: Bearer <token>` on protected routes
- **Middleware:** `require_auth` validates signature, expiry, and extracts `user_id`
- **Storage (mobile):** Expo SecureStore via Redux listener middleware
- **Secret:** `JWT_SECRET` env var, minimum 32 characters

## Alternatives Considered

- **Session cookies** — poor fit for React Native
- **Opaque server sessions** — requires session store; deferred
- **OAuth2/OIDC** — out of product scope

## Consequences

- Stateless verification scales horizontally
- Logout is primarily client-side token discard; server `POST /v1/auth/logout` is advisory
- No built-in refresh token rotation in v1
- Token theft mitigated by TLS in production and short expiry

## Migration Plan

N/A — initial auth model.

## Rollback Plan

Introduce refresh tokens or session store via superseding ADR.

## Approval Status

- Architecture Agent: Accepted
- Human reviewer: Pending
