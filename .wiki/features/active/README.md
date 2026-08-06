# Active Feature Ledgers

Keep one ledger per feature in active development. The ledger owns feature intent, the delivery plan, PR boundaries, commit packets, validation evidence, and discoveries that change the plan. It does not own permanent current-state architecture.

Create a ledger with `$workflow-plan-feature`. When a planned spec exists, absorb its accepted intent, behavior, boundaries, dependencies, non-goals, open questions, and acceptance detail into the ledger, then delete the spec in the same planning change. Keep one PR active and exactly one commit marked `Active` inside it.

## Status vocabulary

**Feature:** `Shaping` | `Active` | `Blocked` | `Validation` | `Documentation reconciliation` | `Ready for final publication`

**PR:** `Pending` | `Awaiting prior PR merge` | `Active` | `Ready for publication` | `Merged` | `Removed — <reason>`

**Commit:** `Pending` | `Active` | `Blocked` | `Completed` | `Removed — <reason>`

The content commit can mark its own plan item `Completed`, but it cannot contain its own hash. Record the hash in the next normal ledger-bearing commit or during feature reconciliation. Do not create a ledger-only commit only to record a hash.

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
- Project validation commands:
- Primary risks:

## Feature architecture

Responsibilities and boundaries for this feature.

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

The thinnest path through this feature.

## Delivery plan

### PR 1 — <title>

**Status:** Active

**PR ref:** Not published | <number or URL>

**Merge ref:** Not merged | <immutable merge commit or equivalent>

**Provisional PR title:** `type(scope): imperative description`

**Purpose:** What this PR delivers and why it is a review and merge boundary.

**Depends on:** Prior PRs, features, or foundations.

#### Commit 1 — <title>

**Status:** Active

**Provisional commit:** `type(scope): description`

**Work:** One coherent outcome.

**Out of scope:**

- ...

**Implementation packet:**

- Owners and files:
- Existing patterns to verify:
- Constraints and invariants:
- Dependencies and ordering:

**Implementation profile:** <model and effort> — <short repository-specific reason>

**Review profile:** <model and effort> — <short consequence and validation reason>

**Validation:** Exact project commands and expected evidence.

**Stop conditions:** Conditions that require escalation, replanning, or developer input.

**Review mandate:** Three to eight concrete concerns derived from this commit's risks and invariants.

#### Commit 2 — <title>

...

### PR 2 — <title>

**Status:** Awaiting prior PR merge

**Depends on:** PR 1

...

## Active work

**PR:** <number or title>

**Commit:** <title>

### RED proof

State the smallest failing test or reproducible proof and the plausible wrong behavior it detects. When automation is not practical, name the focused command or runtime probe and explain the limitation.

### Expected outcome

Observable repository state when this commit is complete.

### Explicit exclusions

What this commit must not include.

## Discoveries and replanning

Record material deviations, blockers, and decisions that change remaining work. State what was planned, what changed, and why.

- ...

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| ... | ... | Pending record | ... | ... | None |

Resolve `Pending record` from Git in the next normal ledger update or during feature reconciliation.

## Final validation

**Feature review profile:** <Sol High | Sol xhigh | Sol Max> — <short cross-commit risk and consequence reason>

List the exact project commands and manual evidence required before feature review.

## Documentation impact

Complete during reconciliation.
~~~
