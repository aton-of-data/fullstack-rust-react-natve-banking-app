# Skill: React Native + Redux Toolkit

## Store Setup

```text
src/app/store.ts          # configureStore
src/app/hooks.ts          # useAppDispatch, useAppSelector
src/app/listenerMiddleware.ts
```

## Feature Slice Pattern

```text
src/features/<feature>/
  slice.ts
  selectors.ts
  types.ts
  index.ts
```

## Rules

- Reducers are pure — no async, no I/O
- Page-scoped state: slice name + entity/route key in state shape
- Derived data in selectors (`createSelector`)
- Entity collections: `createEntityAdapter` when normalized lists

## Listener Middleware

Use for:

- Cross-feature reactions
- Navigation side effects (triggered by actions)
- Analytics (non-blocking)

Define: trigger action, effect, cleanup on cancel.

## Prohibited

```typescript
// ❌ Application state in components
const [loading, setLoading] = useState(false);

// ❌ Side effect in reducer
someSlice.reducers.charge = (state) => { fetch('/api'); };

// ✅ RTK Query or listener for side effects
```

## Typed Hooks

Always use `useAppDispatch` and `useAppSelector` — never raw `useDispatch`/`useSelector` without types.

## Verification

- Selectors memoized for list screens (feed)
- No duplicated derived fields in slice state
