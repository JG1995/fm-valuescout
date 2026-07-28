# 0009 — Biome

## Status

Accepted

## Context

Every commit runs `./scripts/dev check`. Lint and format must be fast on a solo developer machine and in GitHub Actions. Bulletproof React uses ESLint plus Prettier plus many plugins. The template should avoid that multi-tool setup unless a clear gap forces a hybrid.

## Decision

Use **Biome only** for lint and format. Run `biome check` in `./scripts/dev check`. Use **`tsc --noEmit`** separately for type checking — Biome does not replace the TypeScript compiler.

Do **not** add ESLint or Prettier by default. Biome covers formatting, most lint rules, React hook dependencies via **`useExhaustiveDependencies`** (recommended, error level), import restrictions via **`noRestrictedImports`**, and filename rules via **`useFilenamingConvention`** when configured in `biome.json`.

Add ESLint only when a **specific, proven gap** appears in practice (for example full `jsx-a11y` parity). Do not add ESLint preemptively for `react-hooks/exhaustive-deps` — Biome owns that rule family.

## Alternatives considered

### ESLint + Prettier + typescript-eslint (Bulletproof React)

Maximum plugin ecosystem. Configuration spans many packages and files. CI lint steps are slower on typical projects. Duplicates what Biome bundles for solo React + TypeScript work.

### ESLint flat config only (no Prettier)

Still heavier than Biome for the same baseline checks.

### Biome + lint-staged on commit

Rejected with Husky decision [0011](./0011-husky-git-hooks.md). Full `biome check` runs via `./scripts/dev check` on every commit, not staged files only.

### No linter (format on save only)

Formatting without lint misses unused variables, suspicious patterns, and import issues before review.

## Consequences

### Positive

- Single tool and config file (`biome.json`).
- Faster than ESLint + Prettier on medium repos.
- Built-in React rules including `useExhaustiveDependencies` for hook dependency arrays.
- Aligns with solo-dev scope — less config ceremony than BPR's ESLint matrix.

### Negative

- Not every ESLint plugin from Bulletproof React has a Biome equivalent (`jsx-a11y`, `testing-library`, Tailwind class lint are partial or manual).
- Teams with mandatory corporate ESLint configs may hybridize at fork time.

### Follow-up

- Done at scaffold (`41effa2`) — `biome.json` with React TSX defaults, `useExhaustiveDependencies`, kebab-case filenames, and scoped import zones.
- Done at scaffold (`2c7f69c`) — `biome check` in `./scripts/dev check`.
- Revisit ESLint only if Biome misses a real bug or a product requirement needs plugin coverage Biome lacks.

## Related work

- Git hooks: [0011](./0011-husky-git-hooks.md)
- Commits: `41effa2`, `2c7f69c`
- Supersedes: none
