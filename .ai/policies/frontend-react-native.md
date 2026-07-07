# Frontend Policy — React Native

Canonical reference: `context.md`

## Stack

- Pure React Native
- Redux Toolkit for all application state
- RTK Query for all remote API access
- Atomic Design for UI composition

## Directory Structure

```text
src/
  app/                    # store, providers, navigation shell
  shared/
    ui/
      atoms/
      molecules/
      organisms/
      templates/
    lib/
    hooks/                # thin adapters only; no application state
    types/
  entities/               # entity slices, selectors, types
  features/               # feature slices, RTK endpoints, listeners
  widgets/                # composed UI blocks
  pages/                  # route-level composition
```

## State Management Rules

| Rule               | Detail                                                                              |
| ------------------ | ----------------------------------------------------------------------------------- |
| Redux Toolkit only | Single store; feature slices colocated                                              |
| No custom Context  | Except Redux Provider                                                               |
| No local app state | No `useState`/`useReducer` for app, page, form, workflow, loading, error, selection |
| Pure reducers      | No side effects in reducers                                                         |
| Typed hooks        | `useAppDispatch`, `useAppSelector`                                                  |
| Selectors          | Derived state via selectors, not duplicated in slices                               |
| Entity adapters    | For normalized collections when appropriate                                         |
| Page state         | Feature slices keyed by route or entity ID                                          |

## Side Effects

Orchestrate via:

- RTK Query lifecycle (`onQueryStarted`, `invalidatesTags`, etc.)
- Listener middleware
- Explicitly named async thunks tied to slice actions (when not covered by RTK Query)

Each side effect must define: owner, trigger, success state, failure state, cancellation/cleanup.

**Never in reducers:** network calls, timers, storage, navigation, analytics, mutations.

## API Access

- Single RTK Query `baseApi` with typed endpoints
- Auth headers, error normalization, tags, invalidation, retries at API layer
- Mutations invalidate or optimistically update correct cache entries
- Map API DTOs before exposing to UI when shapes differ

**Prohibited in UI/hooks/screens/atoms/molecules/organisms/slices:**

- `fetch`
- `axios`
- Direct HTTP clients

## Atomic Design Rules

| Level     | Responsibility                                |
| --------- | --------------------------------------------- |
| Atoms     | Presentational, reusable                      |
| Molecules | Compose atoms                                 |
| Organisms | Compose molecules; narrow feature adapters OK |
| Templates | Page layout only                              |
| Pages     | Compose templates, widgets, features          |

Business rules do not belong in atoms, molecules, or templates.

## Real-Time Feed (Product Requirement)

Global transaction feed must update without manual refresh.

- Prefer WebSocket/SSE endpoint from Rust backend
- Integrate via RTK Query streaming, cache updates, or listener middleware
- ADR required for transport choice

## Tooling (Enforced)

Policy: `.ai/policies/frontend-tooling.md`. Templates: `.ai/templates/frontend-tooling/`.

| Command                | Enforces                               |
| ---------------------- | -------------------------------------- |
| `npm run lint`         | ESLint + architectural rules + JSDoc   |
| `npm run format:check` | Prettier                               |
| `npx commitlint`       | Conventional Commits (commit-msg hook) |

## Lint Enforcement (Required When Scaffold Exists)

ESLint rules must error on direct usage of:

```text
useState
useReducer
createContext
useContext
fetch
axios
```

Exceptions only via documented ADR for framework internals or approved infrastructure modules.

## Documentation

- JSDoc on all exported symbols (Documenter Agent)
- Mermaid diagrams for non-trivial feature flows
- Policy: `.ai/policies/documentation.md`

## Accessibility & Performance

- QA must validate loading, error, and empty states
- List virtualization for transaction feed when needed
- Memoized selectors for derived lists
- Avoid unnecessary re-renders from non-memoized inline objects in connected components
