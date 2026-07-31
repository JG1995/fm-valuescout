# Agent prompt: Fix Playwright smoke flake (React Query Devtools)

Use this prompt in other Cursor React Tauri template forks when CI smoke fails with a strict-mode violation on `Status:`.

---

## Task: Fix Playwright smoke flake caused by React Query Devtools

### Context

Smoke tests run Playwright against the **Vite dev server** (`import.meta.env.DEV === true`), so `@tanstack/react-query-devtools` and `@tanstack/react-router-devtools` are mounted. React Query Devtools renders a lowercase `status:` label in the DOM.

The smoke test in `e2e/smoke.spec.ts` likely uses:

```typescript
await expect(page.getByText("Status:")).toContainText("ok");
```

Playwright's `getByText("Status:")` is **case-insensitive**, so it can match **two** elements:

1. Health panel: `Status: ok`
2. Devtools: `status:`

That causes a **strict mode violation**. This is a timing flake:

- **Local** `./scripts/dev check` often passes (`CI` unset → different Playwright config, timing wins the race)
- **GitHub Actions** fails reliably (`CI=true` → fresh Vite server, single worker, different render timing)

This is **not** related to product rename or missing test updates — it's a pre-existing locator issue.

### Required fix

Scope smoke assertions to **`main`** (devtools render outside the app shell). `src/app/components/app-shell-layout.tsx` should already have `<main>`.

Update `e2e/smoke.spec.ts`.

**Before:**

```typescript
await expect(page.getByRole("heading", { name: "…" })).toBeVisible();
await expect(page.getByText("Status:")).toContainText("ok");
await expect(page.getByText("Stored value:")).toBeVisible();
```

**After:**

```typescript
const main = page.getByRole("main");

await expect(main.getByRole("heading", { name: "…" })).toBeVisible();
await expect(main.getByText("Status:")).toContainText("ok");
await expect(main.getByText("Stored value:")).toBeVisible();
```

Also scope any other content assertions that could collide with devtools, for example:

```typescript
await expect(page.getByRole("main").getByText("Stored value:")).toContainText(
  "smoke-value",
);
```

Run `./scripts/dev format e2e/smoke.spec.ts` before committing.

### Verification

Must pass locally **and** under CI Playwright config:

```bash
CI=true ./scripts/dev smoke
./scripts/dev check
```

Both must be green.

### Optional improvements (if you have scope)

1. **CONTRIBUTING.md** — note that `CI=true ./scripts/dev smoke` matches GitHub Actions Playwright settings (`retries`, `workers`, `reuseExistingServer` in `playwright.config.ts`).
2. **Superseded:** workflow-contract tests were removed on 2026-07-31. CI now runs `./scripts/dev smoke` directly.

Do **not** disable devtools for e2e unless the template explicitly wants that — scoping locators is the minimal fix.

### Commit message

```text
fix(e2e): scope smoke assertions to main content

React Query Devtools renders a lowercase "status:" label in dev mode,
which made getByText("Status:") ambiguous in CI Playwright smoke.
```

### Do not change

- Product rename / template identity (unless that's a separate task)
- Devtools mounting logic (`import.meta.env.DEV`) unless there's a separate architectural decision
