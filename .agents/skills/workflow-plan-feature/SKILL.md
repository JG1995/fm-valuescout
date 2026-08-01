---
name: workflow-plan-feature
description: Plan one feature from the development sequence with a durable ledger, PR and commit boundaries, implementation packets, validation contracts, and independent implementation/review profiles
---

> Use **`$workflow-plan-feature`** for per-feature delivery planning in this repository.

## Target feature

Use the feature named by the developer when present. Otherwise, use the next feature from **Plan next** in `.wiki/TODO.md` development sequence. If still ambiguous, ask which feature to plan.

## Mandatory reads

1. Read `AGENTS.md`, `.agents/WORKFLOW.md`, `.wiki/INDEX.md`, and `.wiki/features/active/README.md`.
2. Read `.agents/skills/conventional-commits/SKILL.md` — provisional commit and PR titles must follow Conventional Commits.
3. Read `.wiki/TODO.md` — development sequence and which feature is next.
4. Read the feature source: `.wiki/features/planned/<slug>.md` when it exists; otherwise the CONCEPT MVP bullet and roadmap rationale for this feature.
5. Read `.wiki/CONCEPT.md`, `.wiki/ARCHITECTURE.md` (§1.1 when present), and any existing `.wiki/features/active/<slug>.md` ledger.
6. Scan `.agents/skills/` for skills whose description matches this feature (architecture, stack, coding standards) and read each matching `SKILL.md`.
7. Inspect repository read-only — current-state map for this feature only.

Use Context7 MCP only when stack facts are needed for the plan — not for implementation detail.

## Planning context

Use a separate planning context when agent dispatch is available. The named `planner` supplies the default Terra xhigh profile for features with established architecture and useful repository analogues. Use a generic Sol High planning agent when the feature meets the Sol conditions in `.agents/WORKFLOW.md`, and Sol xhigh only for the canonical exceptional conditions. If dispatch is unavailable, preserve the same planning-only boundary in the main context.

Do not override a pinned named role. Inspect the returned ledger and repository evidence before accepting the plan.

## Recallium

Read `.agents/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** planning — prior work on this feature or area, and decisions that affect PR/commit boundaries.
**Search with:** `search_memories` → `expand_memories` as needed; `recallium` / `session_recap` when resuming this feature across sessions.

**Save after:** planning complete — only when durable context changes the plan and is not already in the ledger.
**Save with:** `store_memory` — one concise memory; update existing when possible.

Skip save when unsure.

## Role boundary

| Command | Scope |
| --- | --- |
| `$workflow-roadmap` | Order of **features** across MVP |
| `$workflow-plan-feature` | **One feature** — PRs and commits as breakpoints |
| `$workflow-build` | **One active commit** — RED → GREEN → REFACTOR |
| `$workflow-build-loop` | **Manual opt-in:** same as `$workflow-build`, then automated checkpoint/fix loop + auto-commit |
| `$workflow-fix` | **Delegated review findings** on the current commit (default: CRITICAL, HIGH, MEDIUM) |
| `$workflow-checkpoint` | Stage, review report, commit after approval |
| `$workflow-finish-feature-loop` | **Manual opt-in:** feature validation, Sol High review/fix loop, documentation reconciliation, and local close-out commits |

Do not implement, stage, commit, or push.

## Classify first

- **Trivial** — one coherent commit, no feature ledger. Produce a short work contract in chat and stop.
- **Feature work** — multi-commit delivery plan with PR and commit breakpoints.

## Planning rules (feature work)

**High-level only.** Describe *what* to do and *why*, affected areas, validation, and dependencies. **No code examples** unless one short snippet is essential to disambiguate a boundary or contract.

Use the complete ledger template in `.wiki/features/active/README.md`. For every pending or active commit, include the implementation packet, Capability Demand, Effort Demand, implementation profile, Review Demand, review profile, evidence threshold, escalation conditions, replanning conditions, and machine-readable execution metadata. Score implementation capability, implementation effort, and review demand independently. Apply hard floors after raw scoring and document the Luna punch-up when used.

### Trunk-based development

This repository uses **trunk-based development** — `main` is the trunk; all work lands there through short-lived branches and PRs.

When designing delivery:

- **Short-lived branches** — one branch per PR; merge to trunk soon after review, not a months-long feature branch.
- **Trunk stays green** — every commit and every merged PR must pass `./scripts/dev check` (and configured smoke/mutate when applicable).
- **Independently mergeable PRs** — each PR should be safe to merge to `main` on its own. Later PRs in the feature may depend on earlier merged work, but avoid changes that only work on a stale branch.
- **No "merge when feature complete"** — the feature completes through trunk merges (often one PR, sometimes a short sequence), not an unmerged mega-branch.

**PRs and commits are different knobs.** Minimize PR count. Keep commits atomic and fine-grained. Do not merge commits just because you merged PRs, and do not invent extra PRs just because the commit list is long.

### PR breakpoints

**Default: one PR** with an ordered atomic commit sequence.

Add another PR only when there is a **clear boundary** — for example:

- A walking skeleton must land on trunk before the rest (so intermediate value is mergeable and green).
- A risky or reversible foundation (schema migration, new dependency, protocol change) should merge alone before dependent work.
- Two halves have no shared review surface and would be clearer as separate merges.

Do **not** split PRs merely to keep each PR “small,” to mirror layer cake (backend / UI / polish), or because the feature has many commits. A long commit list inside one PR is normal and preferred for this solo-hobbyist workflow.

When a second PR exists, state **why** the split is required in the plan. Prefer at most two PRs unless a third clear boundary is unavoidable.

**PR titles** use the same Conventional Commits shape as commits: `type(scope): imperative description` (optional body in the PR description, not the title). Scope is usually the feature slug or affected module. Example: `feat(auth): add session store and login route`.

Record a **provisional PR title** per PR in the plan.

### Atomic commit breakpoints

Each commit is one **atomic** unit — the strategy used for every `$workflow-build` and `$workflow-checkpoint`. **Retain fine-grained commits** even when the feature is a single PR.

- **One coherent, revertible outcome** — one behavioural or structural change; if the subject needs "and", split the commit.
- **Reviewable** — a reviewer can understand and approve the diff without unrelated changes.
- **Independently reviewable** — complete for **this commit only**. State **out of scope for this commit** so reviewers do not treat planned later work as missing. A stub, partial UI, or unexported module is correct when a later commit owns the rest of the feature.
- **Trunk-safe** — lands on `main` via PR without breaking the gate; no WIP or "fix tests later" commits.
- **Ordered** — earlier commits establish foundations later commits depend on; keep the tree buildable where possible.

**Provisional commit messages** must follow [Conventional Commits](.agents/skills/conventional-commits/SKILL.md): `type(scope): imperative description` — outcome, not file list; under 72 characters; no period.

**Walking skeleton** — name the thinnest path through the feature (often the first commits of the sole PR, or the whole first PR when a second PR is justified) that proves the approach on trunk.

When repository evidence is thin, deepen read-only reconnaissance before planning: inspect implementation and tests, read `.wiki/ARCHITECTURE.md` (§1.1 when present), read matching skills from `.agents/skills/`, search Recallium per **## Recallium**, and use Context7 for stack facts. If a **gating unknown** needs a runtime experiment before the first commit, note it and suggest optional **`$workflow-spike`** — otherwise **ask the developer** with unresolved decisions stated explicitly.

## Chat output (feature work)

Present before or alongside the ledger:

```md
## Feature plan: <name>

### TL;DR
<One sentence. First PR and first commit to `$workflow-build`.>

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

#### PR 2 — … (only when a clear PR boundary exists; omit by default)

### Walking skeleton
<Thinnest path through the feature.>

### Risks and unknowns
<What could force replanning; gating unknowns that may need optional `$workflow-spike`.>

### Build next
`$workflow-build` → PR 1, commit 1 — <title>
```

## Ledger and TODO (feature work)

After the plan is coherent:

1. Create or update `.wiki/features/active/<feature-slug>.md` using the complete ledger template — **delivery plan** with PRs, commits, packets, and execution profiles, not slice graphs.
2. Mark exactly **one commit** `Active` (the first commit of the first `Active` or `Pending` PR).
3. Move the feature to **Active** in `.wiki/TODO.md` with a link to the ledger. Remove it from the development sequence table or mark it in progress.
4. Stop. Do not `$workflow-build` unless the developer asks.

Treat the plan as provisional. State unresolved decisions that block the first commit.
