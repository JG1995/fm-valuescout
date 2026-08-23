# 0012 — Secretlint for secret scanning

## Status

Accepted

## Context

Committed secrets (.env, API keys, tokens) are high-impact and common. The template already ignores local env files and documents `VITE_*` exposure rules, but Git history is permanent once a secret lands.

[Gitleaks](https://github.com/gitleaks/gitleaks) is widely used but requires a separate binary install on each machine. This template is pnpm-first — dependencies should install on `pnpm install` without extra local setup.

Husky runs `./scripts/dev check-fast` on every commit ([0011](./0011-husky-git-hooks.md)); CI runs `./scripts/dev check-app` when frontend or CI files change. Secret scanning should use the same `./scripts/dev` surface as Biome and TypeScript, not a second hook model or lint-staged.

## Decision

Use **secretlint** (`secretlint` npm package) with **`@secretlint/secretlint-rule-preset-recommend`**.

- Config: `.secretlintrc.json` at repo root.
- Ignore: `.secretlintignore` (e.g. `pnpm-lock.yaml` and `src-tauri/Cargo.lock` false positives).
- Full-tree scan: `pnpm exec secretlint "**/*"` (respects `.gitignore` by default since secretlint v13).
- Staged scan: staged paths from `git diff --cached --name-only` passed to `secretlint --no-glob` via `./scripts/dev secrets --staged` for optional fast checks without lint-staged.
- **Gate:** `./scripts/dev check` runs secretlint after Biome and `tsc`. `./scripts/dev check-app` exposes the same frontend checks for conditional CI.

Do **not** add lint-staged. Do **not** require a system binary.

## Alternatives considered

### Gitleaks

Strong rule set and GitHub Action. Requires local binary install or Docker — friction for a hobby template where `pnpm install` should be enough.

### lint-staged + secretlint (Bulletproof / secretlint docs pattern)

Fast staged-only runs. Rejected for this template: [0011](./0011-husky-git-hooks.md) chose a single full gate over split staged vs full behavior. `./scripts/dev secrets --staged` is available when a fork wants staged-only scans without lint-staged.

### CI-only scanning

Misses secrets when developers use `git commit --no-verify`. Local and CI must share `./scripts/dev check`.

### No automated scanning

Relies on `/skill:workflow-security-audit` and discipline. Too easy to commit `.env` once.

## Consequences

### Positive

- Secret scan installs with `pnpm install` — no extra local tooling.
- Same frontend checks locally and in GitHub Actions; local `check` also includes Rust.
- Pattern-based detection catches common credential formats before push.

### Negative

- Heuristic rules can false-positive; allowlist via `.secretlintignore` when needed.
- Does not replace `/skill:workflow-security-audit` for auth, IDOR, or novel secret encodings.
- Full-tree scan on every commit adds a small fixed cost to `./scripts/dev check`.
- Staged scan uses GNU `xargs -r` — document portable alternatives for macOS forks if needed.

### Follow-up

- Revisit staged-only pre-commit if full-tree scan becomes slow on large forks.

## Related work

- Git hooks: [0011](./0011-husky-git-hooks.md)
- Security audit skill references secret hygiene; this ADR owns automated commit/CI scanning.
