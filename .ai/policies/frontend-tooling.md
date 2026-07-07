# Frontend Tooling Policy

Canonical reference: `context.md`

## Required Tooling

| Tool                | Config location                      | Enforced by             |
| ------------------- | ------------------------------------ | ----------------------- |
| ESLint              | `frontend/eslint.config.mjs`         | `npm run lint`          |
| Prettier            | `frontend/prettier.config.mjs`       | `npm run format:check`  |
| eslint-plugin-jsdoc | in ESLint config                     | `npm run lint`          |
| commitlint          | `commitlint.config.cjs` (repo root)  | Husky `commit-msg` hook |
| Husky               | `frontend/package.json` or repo root | git hooks               |

Bootstrap templates: `.ai/templates/frontend-tooling/`

## Required npm Scripts (frontend/package.json)

```json
{
  "scripts": {
    "lint": "eslint .",
    "lint:fix": "eslint . --fix",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "typecheck": "tsc --noEmit",
    "test": "jest",
    "prepare": "husky"
  }
}
```

## ESLint Requirements

### Architectural rules (error)

Prohibit direct usage of:

- `useState`, `useReducer` — application state
- `createContext`, `useContext` — custom context (Redux Provider exempt via ADR)
- `fetch`, `axios` — direct HTTP in UI layers

### JSDoc rules (via eslint-plugin-jsdoc)

| Rule                        | Level | Applies to                                     |
| --------------------------- | ----- | ---------------------------------------------- |
| `jsdoc/require-jsdoc`       | error | exported functions, classes, types, interfaces |
| `jsdoc/require-description` | error | all JSDoc blocks                               |
| `jsdoc/require-param`       | error | functions with parameters                      |
| `jsdoc/require-returns`     | error | functions with non-void return                 |
| `jsdoc/check-types`         | error | `@param` and `@returns` types                  |
| `jsdoc/valid-types`         | error | TypeScript-compatible JSDoc types              |

### Exemptions

- Test files: relaxed JSDoc (describe blocks exempt)
- `*.config.*` files: lint-only, no JSDoc required
- ADR-documented exceptions only

## Prettier Requirements

- Single source of truth for formatting — **no manual style debates**
- Run `format:check` in CI and QA gate
- Prettier wins over ESLint formatting rules (`eslint-config-prettier`)

Default settings (template):

```javascript
{ semi: true, singleQuote: true, trailingComma: 'all', printWidth: 100, tabWidth: 2 }
```

## Conventional Commits (commitlint)

Enforced at commit time. Committer Agent must not use `--no-verify`.

**Format:**

```text
<type>(<scope>): <description>

[optional body]

Work-Item: <feature-id>
```

**Allowed types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

**Rules:**

- `type` required, lowercase
- `scope` optional, lowercase
- `description` required, imperative mood, no trailing period
- Header max 100 characters

Config: `commitlint.config.cjs`

Validate before commit:

```bash
echo "feat(auth): add login" | npx commitlint
```

## CI Gates (when GitHub Actions added)

```yaml
# Required jobs
- npm run typecheck
- npm run lint
- npm run format:check
- npm test
```

## Prohibited

- `// eslint-disable` without ADR justification in work item
- `// prettier-ignore` without comment explaining why
- Committing unformatted code
- Non-conventional commit messages
