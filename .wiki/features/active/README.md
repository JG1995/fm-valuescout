# Active Feature Ledgers

One ledger per feature in active development. A ledger owns feature intent, the **delivery plan** (PRs and commits), and discoveries that change the plan. It does not own permanent current-state architecture.

Create a ledger with `workflow-plan-feature`. Keep exactly **one commit** marked `Active` during implementation.

## Canonical workflow

Read [`.agents/WORKFLOW.md`](../../../.agents/WORKFLOW.md) before creating or changing a ledger. It owns model routing, review evidence, escalation, and migration rules. This file owns the reusable ledger schema.

## Status vocabulary

**Feature:** `Draft` | `Shaping` | `Active` | `Blocked` | `Validation` | `Documentation reconciliation` | `Completed` | `Archived`

**PR:** `Pending` | `Active` | `Blocked` | `Merged` | `Removed — <reason>`

**Commit:** `Pending` | `Active` | `Blocked` | `Completed — hash pending checkpoint commit` | `Completed — <hash>` | `Removed — <reason>`

The pending-hash form is transient. Replace it from Git history in the next ledger or reconciliation commit.

At feature completion, reconcile documentation, mark the ledger `Completed`, condense it, move it to [completed features](../completed/README.md), and treat the moved record as `Archived`.

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
- Test ownership:
- Authoritative validation commands:
- Likely reuse points:
- Known technical risks:
- Applicable repository patterns:

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
- <Question> — blocks: planning | next commit | validation | later work only

## Risks

### <Concrete failure mode>

- **Trigger:** ...
- **Consequence:** ...
- **Mitigation:** ...
- **Proof:** <commit or validation stage>

## Walking skeleton

Thinnest path through this feature (usually first PR / first commits).

## Delivery plan

### PR 1 — <title>

**Status:** Active | Pending | Merged

**Provisional PR title:** `type(scope): imperative description`

**Purpose:** What this PR delivers and why it is a review/merge boundary on trunk.

**Depends on:** Prior PRs, features, or foundations.

**Merge boundary:** Why this PR is independently useful, reviewable, and safe to merge.

#### Commit 1 — <title>

**Status:** Active | Pending | Completed — …

**Work:** High-level description of what to implement — not code unless essential.

**Out of scope for this commit:**
- ...

**Validation:** Tests, gate, smoke — what proves this commit is done and trunk-safe.

**Provisional commit:** `type(scope): description` — Conventional Commits; one atomic outcome; split if the subject needs "and".

##### Implementation profile

**Assigned implementer:** Luna | Terra | Sol — `gpt-5.6-...` at none | low | medium | high | xhigh | max

**Routing summary:** State Capability Demand, Effort Demand, applied hard floor, and whether the Luna punch-up applies.

##### Review profile

**Assigned reviewer:** Luna | Terra | Sol — `gpt-5.6-...` at medium | high | xhigh | max

**Context:** Fresh. The reviewer receives the commit contract, relevant feature context, packet, diff, validation, and repository access before implementation notes.

**Mandate:**

- Challenge 3–8 commit-specific invariants, failure paths, boundaries, lifecycle transitions, validation gaps, accessibility concerns, or compatibility risks.

##### Implementation packet

###### Governing requirements and invariants

- Only the requirements and invariants relevant to this commit.

###### Existing patterns to follow

- Name exact repository files, symbols, tests, and modules. State explicitly when no useful analogue exists.

###### Expected change surface

- **Likely modified:** ...
- **Likely added:** ...
- **Ownership boundaries:** ...
- **Do not change without replanning:** ...

###### State and data design

- Source of truth, draft state, persisted state, cache state, identifiers, loading/error/stale state, mutation semantics, reconciliation, failure, reload, and replacement behavior where relevant.

###### Expected interfaces

- Types, DTOs, function or component responsibilities, hooks, IPC or API operations, persistence mutations, and read-model shape. Do not invent exact signatures unless the repository already determines them.

###### Execution order

1. Commit-specific sequence from contracts and pure logic through boundaries, callers, tests, and validation.

###### Validation ladder

1. Targeted unit or component tests.
2. Affected module tests.
3. Integration tests.
4. Static analysis or type checking.
5. `./scripts/dev check`.
6. `./scripts/dev smoke` when the browser path changes.
7. Manual or real-environment proof only where unavoidable.

###### Stop conditions

- Stop and return to planning when repository evidence disproves a Known fact, a required seam is absent, an invariant cannot hold, exclusions cannot remain intact, a public or persisted contract changes unexpectedly, meaningful validation cannot be built, a planned API differs materially, later-PR work becomes required, or a cross-feature dependency appears.

###### Allowed discretion

- Local naming, private helper structure, component decomposition inside approved boundaries, test organization, and other choices that do not change contracts.

###### Prohibited discretion

- Invariants, persistence ownership, public abstractions, migration strategy, feature scope, validation strength, API or IPC boundaries, and frontend/backend authority.

##### Escalation conditions

- **Increase effort when:** The model has the correct ownership and abstraction but misses paths, tests, integration detail, or repository exploration.
- **Increase model capability when:** The model misunderstands an invariant, ownership, architecture, root cause, or repository evidence.
- **Replan when:** A Known fact, invariant, architectural seam, persisted/public contract, PR boundary, validation contract, or cross-feature dependency changes.

##### Execution metadata

```yaml
execution_profile:
  planner:
    model: gpt-5.6-sol
    effort: high
  implementer:
    model: gpt-5.6-luna
    effort: high
    confidence: null
  capability_demand:
    residual_ambiguity: 0
    architectural_novelty: 0
    diagnostic_uncertainty: 0
    semantic_risk: 0
    context_synthesis: 0
    total: 0
    luna_punch_up_applied: false
    hard_floor: none
  effort_demand:
    implementation_breadth: 0
    branch_density: 0
    repository_discovery: 0
    validation_weakness: 0
    tool_coordination: 0
    adjustments: 0
    total: 0
  reviewer:
    model: gpt-5.6-terra
    effort: high
    context_mode: fresh
  review_demand:
    missed_defect_consequence: 0
    hidden_interaction_complexity: 0
    validation_weakness: 0
    architectural_discretion: 0
    blast_radius: 0
    total: 0
    hard_floor: none
  review_mandate:
    - Verify one concrete commit-specific invariant.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - The architecture is correct but validation exposes missed execution paths.
  escalate_model_when:
    - The implementer misunderstands ownership or a governing invariant.
  replan_when:
    - A documented invariant or architectural seam must change.
  adjudicator:
    model: gpt-5.6-sol
    effort: medium
    invoke_when:
      - Reviewer and implementer disagree about architecture.
      - A high-severity finding remains disputed.
      - A correction would change the feature plan.
```

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

### Assigned profiles

- **Implementation:** <model and effort from active commit>
- **Review:** <model and effort from active commit>

### Current blockers

- None | ...

### Discoveries that may require replanning

- None | ...

## Discoveries and replanning

Record deviations from the delivery plan, blockers, and decisions that changed remaining commits or PRs. Do not silently rewrite history. Each entry states **what was planned**, **what was discovered**, **why it matters**, **whether architecture or implementation changes**, **affected later work**, and **routing impact**.

- ...

## Completed work

| PR | Commit | Hash | Notes | Implementer | Reviewer | Deviations |
| --- | --- | --- | --- | --- | --- | --- |
| … | … | … | … | … | … | … |

## Final validation

At feature end, define the complete test suite, static checks, smoke paths, manual interactions, target viewports or platforms, keyboard-only paths, real-environment checks, migration and compatibility proof, and feature-complete review.

### Feature review profile

- **Reviewer:** Sol Medium by default; Sol High for high-risk features.
- **Mandate:** End-to-end intent, cross-commit integration, feature invariants, duplicated abstractions, lifecycle paths, temporary compatibility layers, and documentation accuracy.

## Documentation impact

During reconciliation.
~~~
