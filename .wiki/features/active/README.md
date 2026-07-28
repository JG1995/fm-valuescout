# Active Feature Ledgers

One ledger per feature in active development. A ledger owns feature intent, the **delivery plan** (PRs and commits), and discoveries that change the plan. It does not own permanent current-state architecture.

Create a ledger with `/plan-feature`. Keep exactly **one commit** marked `Active` during implementation.

## Status vocabulary

**Feature:** Shaping | Active | Blocked | Validation | Documentation reconciliation

**PR:** `Pending` | `Active` | `Merged` | `Removed — <reason>`

**Commit:** `Pending` | `Active` | `Blocked` | `Completed — hash pending checkpoint commit` | `Completed — <hash>` | `Removed — <reason>`

The pending-hash form is transient. Replace it from Git history in the next ledger or reconciliation commit.

At feature completion, reconcile documentation, condense the ledger, and move it to [completed features](../completed/README.md).

## Ledger template

~~~markdown
# <Feature Name>

## Status

Active

## Intent

Why the feature exists and what capability it introduces.

## User-visible behavior

- ...

## Invariants

- ...

## Non-goals

- ...

## Current-state map

- Relevant components:
- Data model:
- Persistence and migrations:
- Existing behavioral assumptions:
- Architectural seams:
- Tests and validation:
- Primary risks:

## Feature architecture (this feature)

Responsibilities and boundaries for this feature — not every file or method.

## Uncertainty register

### Known
- ...

### Assumptions
- ...

### Decisions
- ...

### Unknowns
- ...

### Risks
- ...

## Walking skeleton

Thinnest path through this feature (usually first PR / first commits).

## Delivery plan

### PR 1 — <title>

**Status:** Active | Pending | Merged

**Provisional PR title:** `type(scope): imperative description`

**Purpose:** What this PR delivers and why it is a review/merge boundary on trunk.

**Depends on:** Prior PRs, features, or foundations.

#### Commit 1 — <title>

**Status:** Active | Pending | Completed — …

**Work:** High-level description of what to implement — not code unless essential.

**Out of scope for this commit:**
- ...

**Validation:** Tests, gate, smoke — what proves this commit is done and trunk-safe.

**Provisional commit:** `type(scope): description` — Conventional Commits; one atomic outcome; split if the subject needs "and".

#### Commit 2 — …

### PR 2 — <title> (omit section when one PR suffices)

**Status:** Pending

**Purpose:** …

#### Commit 1 — …

## Active work

**PR:** <number or title>

**Commit:** <title>

### RED test (active commit)

What the smallest failing test should assert — and **what wrong behavior it would catch** (test quality gate in `coding-standards/references/testing.md`). Skip only when the commit is trivial per that reference.

### Expected outcome

Observable repository state when this commit is complete.

### Explicit exclusions

What this commit must not include.

## Discoveries and replanning

Record deviations from the delivery plan, blockers, and decisions that changed remaining commits or PRs. Each entry: **what was planned**, **what happened instead**, **why**.

- ...

## Completed work

| PR | Commit | Hash | Notes |
| --- | --- | --- | --- |
| … | … | … | … |

## Final validation

At feature end.

## Documentation impact

During reconciliation.
~~~
