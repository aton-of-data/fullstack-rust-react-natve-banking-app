# Docs screenshot capture

- **Date:** 2026-07-08
- **Flow:** `flows/docs-capture.yaml` (manual / not part of suite)
- **Platform:** iOS iPhone 16 Pro
- **Result:** PASS (exit 0)
- **Assets:** `e2e/screenshots/01`–`06` (login → home → search → confirm → success → insufficient funds)

Regenerate (repo root, Expo Go + API + seed, Java 17 + Maestro):

```bash
export JAVA_HOME="$(/usr/libexec/java_home -v 17)"
maestro test apps/mobile/e2e/flows/docs-capture.yaml --platform ios
```
