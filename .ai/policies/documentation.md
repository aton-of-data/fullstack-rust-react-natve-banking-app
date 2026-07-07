# Documentation Policy

Canonical reference: `context.md`

## Ownership

| Content | Owner Agent |
|---------|-------------|
| Architecture docs, ADRs, Mermaid system diagrams | Architecture Agent |
| JSDoc, module READMEs, feature flow diagrams | Documenter Agent |
| rustdoc (`///`) on public Rust APIs | Documenter Agent (backend changes) |
| Inline code comments (non-obvious logic only) | Implementation Agent |

## Documenter Agent Gate

Artifact: `docs/ai/work-items/<id>/03-documentation.md`

Must pass before QA.

## JSDoc Standards (TypeScript / JavaScript)

### Required on

- All `export` functions, classes, constants, types, interfaces, enums
- RTK Query endpoint definitions (describe cache behavior)
- Redux slice actions and selectors (public API)
- Custom hooks in `shared/hooks/`

### JSDoc Template

```typescript
/**
 * Sends a money transfer to another user by username.
 *
 * @param request - Transfer payload including recipient and amount in minor units.
 * @returns Promise resolving to the created transfer record.
 * @throws {TransferError} When insufficient funds or recipient not found.
 *
 * @example
 * await sendTransfer({ toUsername: 'alice', amountMinor: 1500, idempotencyKey: '...' });
 */
export async function sendTransfer(request: SendTransferRequest): Promise<Transfer> {}
```

### Type annotations

- Use TypeScript types in `@param {Type}` when not inferrable
- Prefer `@typedef` for complex object shapes in JS files
- `@template` for generics

### Not required on

- Private/non-exported helpers (unless complex)
- Test files (except exported test utilities)
- Generated code

## Mermaid Diagram Standards

Use Mermaid in:

- `01-architecture.md` — system context, sequence, state flows
- `03-documentation.md` — feature-specific flows
- `docs/ai/adr/*.md` — decision context diagrams
- Module `README.md` files when interaction is non-obvious

### Supported diagram types

| Type | Use for |
|------|---------|
| `flowchart` | Decision flows, pipeline, navigation |
| `sequenceDiagram` | API calls, transfer lifecycle, auth |
| `stateDiagram-v2` | Transfer states, session states |
| `erDiagram` | Data model (ADR/schema docs) |
| `classDiagram` | Type relationships (sparingly) |

### Example (sequence)

````markdown
```mermaid
sequenceDiagram
  participant UI as SendMoneyPage
  participant RTK as transfersApi
  participant API as Rust API
  participant DB as PostgreSQL

  UI->>RTK: sendTransfer mutation
  RTK->>API: POST /transfers (Idempotency-Key)
  API->>DB: BEGIN; check balance; debit; credit; COMMIT
  API-->>RTK: 201 Transfer
  RTK-->>UI: invalidate Balance, Feed tags
```
````

### Rules

- Diagrams must match implemented behavior (Documenter verifies against code)
- Keep diagrams focused — one concern per diagram
- Label participants with module/crate names
- Update diagrams when behavior changes

## Rust Documentation

- `///` on all `pub` items in `domain`, `application`, `contracts`
- Module-level `//!` docs for crate purpose
- `# Examples` section for non-trivial public APIs
- Mermaid in crate README or ADR (not rustdoc — Mermaid is markdown)

## Module README Pattern

For each feature with non-trivial flow:

```text
src/features/<feature>/README.md
```

Contents:

- Purpose
- State ownership
- Mermaid flow diagram
- Key exports (linked to JSDoc)
- Related ADRs

## Verification

Documenter and QA run:

```bash
npm run lint        # includes jsdoc rules
npm run format:check
```

Manual checklist in `03-documentation.md`:

- [ ] All new exports have JSDoc
- [ ] Mermaid diagrams present where required
- [ ] Diagrams match code
- [ ] No stale documentation

## Prohibited

- Exporting undocumented public APIs
- Mermaid diagrams that contradict implementation
- JSDoc that duplicates TypeScript types without adding meaning
- `@ts-ignore` to bypass doc requirements
