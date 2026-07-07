# ADR-001: Monorepo Tooling

## Status

Accepted

## Context

Ficus ships a React Native mobile app, shared TypeScript packages, a Rust API workspace, infrastructure configs, and AI governance docs. We need a single repository with consistent developer workflows and CI orchestration.

## Decision

Adopt a **pnpm + Turborepo** JavaScript workspace alongside a **Cargo workspace** for Rust:

| Layer                 | Tooling                                        |
| --------------------- | ---------------------------------------------- |
| JS package manager    | pnpm 9 (`packageManager` field pinned)         |
| JS task orchestration | Turborepo (`turbo.json`)                       |
| Rust                  | Cargo workspace under `apps/api/`              |
| Shared TS types       | `packages/contracts`, `packages/money`         |
| Root scripts          | `package.json` delegates to Turbo and Makefile |

Workspace definition: `pnpm-workspace.yaml` includes `apps/*` and `packages/*`.

## Alternatives Considered

- **npm workspaces** — slower installs, weaker workspace isolation
- **Nx** — heavier; Turbo sufficient for current scale
- **Separate repos** — fragments contracts and governance pipeline

## Consequences

- Developers need both Node ≥ 20 and Rust stable
- CI runs parallel frontend and backend jobs
- Contract changes require coordinating TS package version and Rust DTOs
- `Makefile` bridges Docker/DB commands not suited to npm scripts

## Migration Plan

N/A — greenfield monorepo layout from initial delivery.

## Rollback Plan

Splitting repos would require extracting `packages/contracts` and duplicating governance; not planned.

## Approval Status

- Architecture Agent: Accepted
- Human reviewer: Pending
