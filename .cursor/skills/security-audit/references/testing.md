# Testing infrastructure — security audit

Load this file when recon finds `src/testing/`, `e2e/`, `playwright.config.ts`, Vitest setup, `mockIPC`, MSW handlers, test fixtures, or CI jobs that run tests and publish artifacts.

Read `references/universal.md` first. This file covers **security risks in how tests stub trust boundaries** — not test quality or TDD ceremony (`coding-standards/references/testing.md` owns that).

Stack-specific stubs: `references/react.md` (client), `references/tauri.md` §Testing and mocks, `references/rust.md` §Testing vs production paths, `references/vite.md` (Vitest env inline).

## Core rule

Tests prove **functional** behavior under controlled conditions. They do **not** prove production security unless integration tests exercise real Rust commands, real SQLite, and real capability enforcement without permissive mocks.

A green test suite with `mockIPC` returning success for every command does **not** mean IPC validation, SQL parameterization, or authz are correct in Rust.

## Attack surface

| Layer | Risk class | Primary files |
| --- | --- | --- |
| **Committed credentials** | Secrets in repo history | `e2e/`, `src/testing/`, `__fixtures__/` |
| **Permissive mocks** | False confidence — mocks skip validation | `src/testing/setup.ts`, `e2e/tauri-ipc-stub.ts` |
| **Test-only code in prod** | Debug endpoints, `#[cfg(test)]` leaks | `lib.rs`, `main.tsx` imports |
| **CI artifacts** | Traces, coverage, logs with PII | Playwright `trace`, `coverage/` |
| **MSW / HTTP mocks (fork)** | Auth bypass in tests only | `src/testing/mocks/` |
| **Snapshot files** | PII or tokens frozen in snapshots | `**/__snapshots__/` |

## Recon signals

```text
mockIPC|clearMocks|stubTauriIpc|__TAURI_INTERNALS__
msw|setupServer|http\.(get|post)\(
fixtures?/|__snapshots__|\.snap
password|api[_-]?key|secret|token|Bearer
sk_live|service_role|AKIA|ghp_
process\.env\.|dotenv.*test
trace:\s*['"]on|video:\s*['"]on
#[cfg\(test\)]|cfg\(debug_assertions\)
TEST_|E2E_|PLAYWRIGHT_
```

Grep test trees for secret-shaped strings. Run secretlint on test paths — template includes full tree in `check`.

## Committed secrets and fixtures

| Pattern | Finding |
| --- | --- |
| Real API keys in `e2e/.env` or `playwright.config.ts` | Git leak |
| `sk_test_` / `pk_test_` Stripe keys in fixtures | Still credentials — rotate if real |
| Copy of production DB in `fixtures/` | PII dump in repo |
| JWT samples with valid signatures | Replay if same secret in prod |
| `.env.test` committed with non-placeholder values | History leak |

Safe practice:

- Placeholders only in committed fixtures (`test-api-key`, `fake-token`)
- Real keys only in CI secret store — injected at job runtime
- `.env.*.local` gitignored — verify test env files follow same rule

Template default: no secret patterns in test tree; `.env.example` empty — low baseline risk.

## IPC and Tauri mocks

Template Vitest (`src/testing/setup.ts`):

- `mockIPC` handles `get_status`, `get_demo_value`, `set_demo_value`
- In-memory `demoValue` — no Rust validation (length limits in production `service.rs` are **not** exercised)

Template Playwright (`e2e/tauri-ipc-stub.ts` via `stubTauriIpc`):

- Stubs IPC in browser context for frontend-only Vite server
- Smoke tests prove UI wiring — **not** Rust command security

| Audit question | Why it matters |
| --- | --- |
| Does mock accept any `set_demo_value` length? | Production may reject — mock hides gap |
| Does mock allow arbitrary command names? | Permissive stub vs production ACL |
| Is `mockIPC` imported in production `main.tsx` path? | Test mock in release bundle |
| E2E stub returns admin role for all users? | False authz confidence |

**Finding type:** Usually **process gap** (Medium) — "tests do not verify Rust validation" — not a runtime vulnerability unless mock ships in production.

Flag **High** when test setup is imported in production entry or `mockIPC` registers in non-test builds.

## MSW and HTTP mocks (forks)

Desktop template default uses IPC mocks — not MSW for domain data. If recon finds MSW:

| Pattern | Risk |
| --- | --- |
| MSW returns 200 for all auth endpoints | Client "logged in" without server |
| Handler mirrors happy path only | Error and auth failure paths untested |
| MSW imported in `main.tsx` when `import.meta.env.DEV` | Dev-only — verify tree-shaken in prod |
| MSW in production build via misconfigured `define` | Fake API in field |

Audit: MSW must not be the only place that enforces auth rules — server/Rust must match.

## Test-only commands and debug routes

| Pattern | Finding |
| --- | --- |
| `#[tauri::command]` only for tests but registered in release `generate_handler!` | Callable in production |
| `/api/debug` route in fork | Left enabled in prod |
| `if (process.env.NODE_ENV === 'test')` skip auth | Wrong env in misbuilt artifact |
| Vitest `vi.stubGlobal` for `fetch` in setup imported everywhere | Global stub leakage |

Search `generate_handler!` for test command paths. Search routes for `debug`, `test-only`, `__internal`.

## Playwright and E2E artifacts

`playwright.config.ts` template:

- `trace: 'on-first-retry'` — traces on failure may capture form input, tokens in DOM
- `webServer` runs local Vite on `127.0.0.1` — good; audit if changed to `0.0.0.0`
- `forbidOnly: Boolean(process.env.CI)` — prevents focused tests skipping coverage in CI

| Check | Risk |
| --- | --- |
| `video: 'on'` always | Large artifacts with sensitive UI |
| Traces uploaded to public CI artifact bucket | PII exposure |
| E2E against staging with real users | Test writes to prod data |
| `storageState` from logged-in session committed | Session cookie in JSON file |

Ensure `test-results/`, `playwright-report/`, `blob-report/` are gitignored — template should list them.

## Vitest coverage and output

| Risk | Mitigation |
| --- | --- |
| `coverage/` contains serialized API responses with PII | Gitignore; restrict CI artifact retention |
| `console.log` in tests dumps tokens | CI log leak |
| Shared `demoValue` in setup — cross-test pollution | Functional issue; rarely security |

Vitest shares Vite config — see `references/vite.md` for env inline in test bundles.

## Integration tests that matter for security

Positive signals when forks add them:

- `src-tauri/tests/` hits real commands against temp SQLite file
- Asserts **rejection** of invalid IPC args (length, charset, path traversal strings)
- Asserts error messages do not contain internal paths
- `cargo test` runs in CI gate — template runs via `check-rust`

Negative signal: only `mockIPC` tests for validation rules that exist only in Rust.

## CI and test jobs

| Check | Why |
| --- | --- |
| Test job env vars scoped to test keys | Prod keys in `pnpm test` job |
| Fork PR workflows run tests without secret access | Untrusted code exfiltrating secrets |
| `pnpm test` prints env on failure | Debug dump of CI secrets |
| Mutation/fuzz tools left running against prod URL | Accidental load or data corruption |

Template: tests run in same repo gate — no separate prod URL.

## Static audit methodology

1. **Grep test dirs** for secret patterns and real-looking credentials.
2. **Trace mock boundaries** — what production checks are skipped?
3. **Import graph** — `main.tsx` / `lib.rs` must not import test setup.
4. **List `#[cfg(test)]` and debug routes** — none in release handler.
5. **Playwright config** — host binding, trace/video, `storageState` files.
6. **`.gitignore`** — coverage, test-results, traces, local env.
7. **Cross-check production validation** — if Rust rejects 10k-char demo value, is there a test without mock?

Report mock gaps as **process / coverage** unless test code ships in production.

## False positives

- `mockIPC` in `src/testing/setup.ts` only imported by Vitest — expected template pattern.
- Playwright IPC stub for smoke on `127.0.0.1` — intentional frontend-only CI.
- Placeholder strings like `fake-token` or `test-user` in fixtures.
- `Unhandled IPC command` throw in mock — strict mock, good for catching drift.
- No MSW in template — skip HTTP mock section.
- secretlint on full tree including tests — catches committed secrets; not a finding when clean.

## Sources

| Source | Use in this file |
| --- | --- |
| `coding-standards/references/testing.md` | TDD quality — complementary, not duplicate |
| `references/tauri.md` §Testing and mocks | mockIPC, production registration |
| `references/react.md` §Dev-only paths | MSW, dev proxies |
| OWASP testing guidance | Fixture and CI artifact handling |
