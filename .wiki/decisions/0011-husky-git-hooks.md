# 0011 — Husky for Git hooks

## Status

Accepted (amended 2026-07-27)

## Context

Every commit should catch lint, type, and secret issues before they land on trunk. The template previously used a plain Git hook at `scripts/hooks/pre-commit` activated manually with `git config core.hooksPath scripts/hooks`. Developers often skip that step on a new clone.

Bulletproof React uses **Husky** plus **lint-staged** to run ESLint and TypeScript on staged files only. This template uses **Biome** for lint and format.

After the Tauri + Rust stack landed, running the **full** `./scripts/dev check` on every commit (workflow contracts, browser smoke, and Rust gates) made local commits unnecessarily slow — often 40+ seconds on a healthy machine.

## Decision

Use **Husky** to install Git hooks on `pnpm install` (`prepare` script). Use **no lint-staged**.

The pre-commit hook runs a **fast** local gate:

```bash
./scripts/dev check-fast
# plus ./scripts/dev check-rust when staged files include src-tauri/
```

`check-fast` runs **full-tree** Biome (`biome check`) and TypeScript (`tsc -b`), plus **staged-only** secretlint (`./scripts/dev secrets --staged`).

The **full** code-quality gate remains `./scripts/dev check` — Biome, TypeScript, full-tree secretlint, and Rust gates. CI selects frontend, browser, Rust, and bridge checks from changed paths, then aggregates the applicable results in its required `check` status. Run `check` manually before merge.

Replace `scripts/hooks/` with `.husky/pre-commit` at scaffold. Remove the manual `core.hooksPath` setup from contributor docs.

## Alternatives considered

### Plain Git hooks (`scripts/hooks` + `core.hooksPath`)

Zero npm dependencies. Hooks are easy to forget on a fresh clone because nothing runs `git config` automatically.

### Husky + lint-staged (Bulletproof React pattern)

Faster commits when only a few files change. Staged Biome runs can miss project-wide issues that full `biome check` catches. `tsc --noEmit` still checks the whole project when included — lint-staged does not speed up typecheck meaningfully.

Rejected for this template: prefer one full gate in CI over split staged vs full behavior on commit.

### Full `./scripts/dev check` on every commit (original ADR)

Local commits and CI used the same command. Predictable, but slow once contract tests and Rust gates joined the check surface.

Superseded by fast pre-commit + full CI gate (2026-07-27).

### No Git hooks (CI only)

Relies on discipline or `--no-verify`. Too easy to push broken commits from local machines.

## Consequences

### Positive

- Hooks install automatically after `pnpm install` — no per-clone `core.hooksPath` step.
- Pre-commit stays fast for docs and frontend-only changes.
- Manual `./scripts/dev check` retains the full safety net; CI runs the applicable product suites for changed paths.

### Negative

- Husky is an extra devDependency and `.husky/` directory.
- Local commits can skip Rust when `src-tauri/` is untouched; CI runs the applicable product suites on `main`.

### Follow-up

- Done at scaffold (`41effa2`, `2c7f69c`) — `husky` in `devDependencies`, `"prepare": "husky"`, `.husky/pre-commit`.
- Done at scaffold (`2c7f69c`) — `scripts/hooks/` removed.
- Amended 2026-07-27 — `check-fast` + conditional `check-rust` on pre-commit; full `check` in CI.
- Amended 2026-07-31 — remove workflow contract tests; run browser smoke and bridge tests as explicit CI product checks.
- Amended 2026-07-31 — select CI product checks from changed paths; `check` aggregates applicable results.

## Related work

- Lint/format policy: [0009](./0009-biome.md)
- Secret scanning: [0012](./0012-secretlint.md)
- Commits: `41effa2`, `2c7f69c`
- Supersedes: manual `scripts/hooks` workflow; full-gate-only pre-commit (original ADR wording)
