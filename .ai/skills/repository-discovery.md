# Skill: Repository Discovery

Use at the start of Architecture stage or when onboarding to the codebase.

## Steps

1. **Read canonical context**
   - `context.md`
   - `.ai/manifest.yml`

2. **Map repository layout**

   ```bash
   find . -maxdepth 3 -type f \( -name "package.json" -o -name "Cargo.toml" -o -name "*.md" \) | head -50
   ls -la frontend/ backend/ src/ 2>/dev/null || true
   ```

3. **Identify stack**
   - Frontend: React Native, Redux Toolkit, RTK Query
   - Backend: Rust workspace crates
   - Database, CI, test runners

4. **Read conventions**
   - `.ai/policies/`
   - Existing ADRs in `docs/ai/adr/`
   - README if present

5. **Locate affected modules**
   - Grep for related symbols, routes, endpoints
   - Trace dependency direction (Rust crates, RN features)

6. **Record findings** in `01-architecture.md` → Existing-system findings

## Outputs

- Module map
- Test commands that work in this repo
- Gaps (missing CI, lint rules, etc.)

## Rules

- Do not assume package manager — verify `package-lock.json`, `pnpm-lock.yaml`, or `yarn.lock`
- Do not claim commands work until executed
- Note greenfield state if no app code exists yet
