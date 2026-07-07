# Skill: JSDoc and Mermaid Documentation

Use during Documentation stage.

## JSDoc Workflow

1. **Inventory exports** in changed files:

   ```bash
   # After frontend scaffold exists
   git diff --name-only | grep -E '\.(ts|tsx)$'
   ```

2. **For each exported symbol**, add JSDoc with:
   - One-line summary (imperative or descriptive)
   - `@param` for each parameter (with `{Type}` and name)
   - `@returns` for non-void functions
   - `@throws` when errors are part of contract
   - `@example` for non-obvious usage (RTK mutations, hooks)

3. **RTK Query endpoints:**

   ```typescript
   /**
    * Creates a money transfer between users.
    * Invalidates `Balance` and `Feed` cache tags on success.
    * Sends `Idempotency-Key` header for retry safety.
    */
   sendTransfer: build.mutation<Transfer, SendTransferRequest>({ ... })
   ```

4. **Redux slices:**

   ```typescript
   /**
    * Feature slice owning send-money workflow state keyed by draft ID.
    * Side effects: transfer submission via `transfersApi` listener.
    */
   ```

5. **React components (exported):**
   ```typescript
   /**
    * Presentational transfer amount input (minor units).
    * @param props - Component props.
    * @param props.value - Amount in cents.
    * @param props.onChange - Called when amount changes.
    */
   ```

## Rust rustdoc Workflow

```rust
/// Debits `amount` from `from` and credits `to` atomically.
///
/// # Errors
/// Returns `TransferError::InsufficientFunds` when balance is too low.
///
/// # Idempotency
/// Duplicate `idempotency_key` returns the original transfer without double-charging.
pub async fn execute(&self, cmd: SendTransferCommand) -> Result<Transfer, TransferError>
```

## Mermaid Workflow

1. Identify flows needing diagrams (from architecture or code inspection)
2. Choose diagram type (sequence for API, flowchart for decisions, state for lifecycle)
3. Add to `03-documentation.md` and/or `src/features/<name>/README.md`
4. Verify labels match actual module/route names

## Verification

```bash
cd frontend
npm run lint
npm run format:check
```

Record exit codes in `03-documentation.md`.

## Coverage Checklist

| Area                     | JSDoc              | Mermaid                      |
| ------------------------ | ------------------ | ---------------------------- |
| New exports              | Required           | If flow is non-trivial       |
| RTK endpoints            | Required           | Sequence diagram recommended |
| Feature slice public API | Required           | State diagram if complex     |
| Page components          | Required           | Optional                     |
| Atoms/molecules          | Required on export | Rare                         |

## Common Failures

| Issue                 | Fix                          |
| --------------------- | ---------------------------- |
| `jsdoc/require-jsdoc` | Add block above export       |
| `jsdoc/require-param` | Document all params          |
| Missing `@returns`    | Add for non-void functions   |
| Stale Mermaid         | Update diagram to match code |
