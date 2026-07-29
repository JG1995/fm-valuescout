---
description: Plan one feature — PR and commit breakpoints, high-level work descriptions; per-feature from the development sequence
---

> **Not Cursor Plan mode.** `/plan` is Cursor's built-in mode. This command is **`/plan-feature`** — per-feature delivery planning for this repository.

## Target feature

${ARGUMENTS:-Use the next feature from **Plan next** in `.wiki/TODO.md` development sequence. If ambiguous, ask which feature to plan.}

## Mandatory reads

1. Read `AGENTS.md` and `.wiki/INDEX.md`.
2. Read `.cursor/skills/conventional-commits/SKILL.md` — provisional commit and PR titles must follow Conventional Commits.
3. Read `.wiki/TODO.md` — development sequence and which feature is next.
4. Read the feature source: `.wiki/features/planned/<slug>.md` when it exists; otherwise the CONCEPT MVP bullet and roadmap rationale for this feature.
5. Read `.wiki/CONCEPT.md`, `.wiki/ARCHITECTURE.md` (§1.1 when present), and any existing `.wiki/features/active/<slug>.md` ledger.
6. Scan `.cursor/skills/` for skills whose description matches this feature (architecture, stack, coding standards) and read each matching `SKILL.md`.
7. Inspect repository read-only — current-state map for this feature only.

Use Context7 MCP only when stack facts are needed for the plan — not for implementation detail.

## Recallium

Read `.cursor/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** planning — prior work on this feature or area, and decisions that affect PR/commit boundaries.
**Search with:** `search_memories` → `expand_memories` as needed; `recallium` / `session_recap` when resuming this feature across sessions.

**Save after:** planning complete — only when durable context changes the plan and is not already in the ledger.
**Save with:** `store_memory` — one concise memory; update existing when possible.

Skip save when unsure.

## Role boundary

| Command | Scope |
| --- | --- |
| `/roadmap` | Order of **features** across MVP |
| `/plan-feature` | **One feature** — PRs and commits as breakpoints |
| `/build` | **One active commit** — RED → GREEN → REFACTOR |
| `/build-loop` | **Manual opt-in:** same as `/build`, then automated checkpoint/fix loop + auto-commit |
| `/fix` | **Delegated review findings** on the current commit (default: CRITICAL, HIGH, MEDIUM) |
| `/checkpoint` | Stage, review report, commit after approval |

Do not implement, stage, commit, or push.

## Classify first

- **Trivial** — one coherent commit, no feature ledger. Produce a short work contract in chat and stop.
- **Feature work** — multi-commit delivery plan with PR and commit breakpoints.

## Planning rules (feature work)

**High-level only.** Describe *what* to do and *why*, affected areas, validation, and dependencies. **No code examples** unless one short snippet is essential to disambiguate a boundary or contract.

### Trunk-based development

This repository uses **trunk-based development** — `main` is the trunk; all work lands there through short-lived branches and small PRs.

When designing PR breakpoints:

- **Short-lived branches** — one branch per PR; merge to trunk soon after review, not a months-long feature branch.
- **Trunk stays green** — every commit and every merged PR must pass `./scripts/dev check` (and configured smoke/mutate when applicable).
- **Independently mergeable PRs** — each PR should be safe to merge to `main` on its own. Later PRs in the feature may depend on earlier merged work, but avoid changes that only work on a stale branch.
- **Incremental delivery** — prefer multiple small PRs over one large PR. Use feature flags, disabled routes, or schema additions that do not break existing behaviour when partial delivery is needed.
- **No "merge when feature complete"** — the feature completes through a **sequence of trunk merges**, not a single big bang.

### PR breakpoints

Group work that reviews and merges together:

- **Small feature:** one PR with an ordered atomic commit sequence, merged to trunk once.
- **Large feature:** multiple PRs merged to trunk in order (foundation → core → polish, or by review/risk boundary).
- Split PRs when: review surface is too large, risk should be isolated, trunk should receive value early, or layers must land in order.

**PR titles** use the same Conventional Commits shape as commits: `type(scope): imperative description` (optional body in the PR description, not the title). Scope is usually the feature slug or affected module. Example: `feat(auth): add session store and login route`.

Record a **provisional PR title** per PR in the plan.

### Atomic commit breakpoints

Each commit is one **atomic** unit — the strategy used for every `/build` and `/checkpoint`:

- **One coherent, revertible outcome** — one behavioural or structural change; if the subject needs "and", split the commit.
- **Reviewable** — a reviewer can understand and approve the diff without unrelated changes.
- **Independently reviewable** — complete for **this commit only**. State **out of scope for this commit** so reviewers do not treat planned later work as missing. A stub, partial UI, or unexported module is correct when a later commit owns the rest of the feature.
- **Trunk-safe** — lands on `main` via PR without breaking the gate; no WIP or "fix tests later" commits.
- **Ordered** — earlier commits establish foundations later commits depend on; keep the tree buildable where possible.

**Provisional commit messages** must follow [Conventional Commits](.cursor/skills/conventional-commits/SKILL.md): `type(scope): imperative description` — outcome, not file list; under 72 characters; no period.

**Walking skeleton** — name the thinnest path through the feature (first PR / first commits) that proves the approach on trunk.

When repository evidence is thin, deepen read-only reconnaissance before planning: inspect implementation and tests, read `.wiki/ARCHITECTURE.md` (§1.1 when present), read matching skills from `.cursor/skills/`, search Recallium per **## Recallium**, and use Context7 for stack facts. If a **gating unknown** needs a runtime experiment before the first commit, note it and suggest optional **`/spike`** — otherwise **ask the developer** with unresolved decisions stated explicitly.

## Chat output (feature work)

Present before or alongside the ledger:

```md
## Feature plan: <name>

### TL;DR
<One sentence. First PR and first commit to `/build`.>

### Scope recap
<From planned spec or CONCEPT — user-visible outcome, non-goals.>

### PR breakdown

#### PR 1 — <title>  (or sole PR for small features)
**Provisional PR title:** `type(scope): imperative description`
**Purpose:** …
**Merge to trunk when:** …
**Depends on:** …
**Commits:**
1. **<commit title>** — work: …; out of scope: …; validation: …; provisional: `type(scope): …`
2. …

#### PR 2 — … (only when feature warrants multiple PRs)

### Walking skeleton
<Thinnest path through the feature.>

### Risks and unknowns
<What could force replanning; gating unknowns that may need optional `/spike`.>

### Build next
`/build` → PR 1, commit 1 — <title>
```

## Ledger and TODO (feature work)

After the plan is coherent:

1. Create or update `.wiki/features/active/<feature-slug>.md` using the ledger template — **delivery plan** with PRs and commits, not slice graphs.
2. Mark exactly **one commit** `Active` (the first commit of the first `Active` or `Pending` PR).
3. Move the feature to **Active** in `.wiki/TODO.md` with a link to the ledger. Remove it from the development sequence table or mark it in progress.
4. Stop. Do not `/build` unless the developer asks.

Treat the plan as provisional. State unresolved decisions that block the first commit.
