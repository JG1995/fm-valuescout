# 0008 — Vitest and React Testing Library

## Status

Accepted

## Context

The template promotes test-first workflow for behavioural changes. Tests must run fast in watch mode and in CI through `./scripts/dev test`. Vite is the bundler — the test runner should share config and resolve aliases.

## Decision

Use **Vitest** with **jsdom**, **@testing-library/react**, **@testing-library/jest-dom**, and **@testing-library/user-event**. Configure Vitest in `vite.config.ts` (or `vitest.config.ts` if split later). Wire `./scripts/dev test` to `vitest run`.

## Alternatives considered

### Jest

Jest is familiar and works with RTL. It requires extra transforms for Vite, TypeScript, and ESM. Vitest is faster and shares Vite resolution and plugins.

### Playwright for unit tests

Playwright excels at E2E and real browsers. It is slower and heavier for component-level tests. Use Playwright for `./scripts/dev smoke` (`e2e/smoke.spec.ts`).

### No component tests in template

Skipping tests contradicts AGENTS.md behavioural workflow. A scaffold without Vitest forces every fork to reinvent test setup.

## Consequences

### Positive

- One config surface with Vite (`@/` aliases, React plugin).
- Fast watch mode for RED/GREEN loops.
- RTL tests user-visible behaviour, not implementation.

### Negative

- jsdom is not a real browser — some DOM APIs differ from production.
- Test files add scaffold size.

### Follow-up

- Done at scaffold (`41effa2`, `ba27c64`) — `src/testing/setup.ts` with `@testing-library/jest-dom`.
- Done at scaffold (`ba27c64`, `fc11b5a`) — health and layout component tests.
- Done at scaffold (`2c7f69c`) — `./scripts/dev test` defaults to `vitest run`.
- Done — Playwright smoke via `./scripts/dev smoke` and `e2e/smoke.spec.ts`.

## Related work

- Commits: `41effa2`, `fc11b5a`, `ba27c64`, `2c7f69c`
- Supersedes: none
