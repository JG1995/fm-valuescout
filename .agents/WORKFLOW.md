# Adaptive Development Workflow

This document is the canonical policy for planning, implementation, validation, review, escalation, and replanning. `AGENTS.md` owns standing repository rules. `.agents/skills/workflow-*/SKILL.md` files own phase procedures. `.wiki/features/active/README.md` owns the reusable feature-ledger template.

## Lifecycle

Use this default path for non-trivial feature work:

```text
planning context
  → durable feature ledger
  → active commit implementer
  → deterministic validation
  → fresh-context commit reviewer
  → correction, escalation, or replanning
  → feature-complete validation and review
  → documentation reconciliation and ledger archival
```

Trivial work can use a short work contract instead of a ledger. Keep the existing approval and Git rules in `AGENTS.md`; this policy does not grant permission to commit, push, or rewrite history.

### 1. Plan

Use `workflow-plan-feature`. The planning context inspects the repository and creates one durable ledger in `.wiki/features/active/`.

The default planning profile is **Sol High** (`gpt-5.6-sol`, `high`). Use Sol xhigh when several architectures remain plausible, persistence or migration design is unsettled, requirements conflict, security or concurrency boundaries change, repository ownership is unclear, or implementation discoveries require material replanning.

The planner must:

- ground the current-state map and implementation packets in actual files, symbols, tests, and commands;
- distinguish invariants from implementation preferences;
- split work into independently reviewable, trunk-safe PRs and commits;
- assign an implementation profile and a separate review profile to every pending or active commit;
- define validation, stop conditions, escalation conditions, and a commit-specific review mandate;
- mark exactly one commit `Active`;
- write discoveries back to the ledger instead of leaving them in chat context.

### 2. Implement one commit

Use `workflow-build` or the manually requested `workflow-build-loop`. The implementer receives only the active commit, relevant feature context, and repository access.

Before editing, the implementer must read:

- the active commit work and exclusions;
- its governing requirements and invariants;
- its implementation packet and execution profile;
- the named repository patterns and relevant surrounding code;
- the validation contract and stop conditions.

The implementer verifies named analogues before copying them, starts with the planned RED test or smallest failing proof when practical, implements only the active commit, validates incrementally, and reviews the diff against the active invariants.

The implementation handoff must state files changed, behavior implemented, tests added or changed, commands run, unresolved uncertainty, material deviations from the packet, and whether escalation or replanning occurred. This self-review does not replace independent review.

### 3. Validate

Use the validation ladder in the active commit from cheapest to broadest:

1. Targeted unit or component tests.
2. Affected module tests.
3. Integration tests.
4. Static analysis or type checking.
5. `./scripts/dev check`.
6. `./scripts/dev smoke` when browser behavior changes.
7. Manual or real-environment checks only where automation cannot prove the contract.

Use only the stable commands in `AGENTS.md` and `./scripts/dev`. Status 69 means unsupported, not passed. A command result is evidence; an agent's confidence is not.

### 4. Review in a fresh context

Every non-trivial commit gets a separate read-only reviewer after deterministic validation. Dispatch the model and reasoning effort assigned by the commit's review profile. The repository's named `reviewer` is the default Terra xhigh role; use a generic read-only reviewer with the same contract when the ledger assigns another profile.

Give the reviewer:

- the original commit specification;
- relevant invariants and non-goals;
- the implementation packet;
- the actual staged diff;
- validation results;
- repository access.

Do not initially give the reviewer the implementer's chain of reasoning, self-review, or a defense of the chosen design. Give implementation notes only after the independent pass when they are needed to resolve disputed intent.

The reviewer follows the commit-specific mandate and reconstructs intended behavior from the ledger and code. It checks guards and tests before retaining a finding.

### 5. Correct, escalate, or replan

Return a confirmed bounded execution defect to the original implementation model. Increase reasoning effort when ownership, abstractions, and the governing invariant are correct but branches, tests, or integration details are incomplete. Increase model capability when the implementer chose the wrong abstraction, misunderstood ownership or an invariant, patched a symptom, or invented repository assumptions.

After one clear structural misunderstanding, do not retry the same model only with more effort. Escalate capability or return to planning.

Use Sol High or xhigh replanning when a Known fact is disproved, an invariant or approved boundary must change, a required seam does not exist, a public or persisted contract changes materially, validation cannot be meaningful, the commit leaks into a later PR, a cross-feature dependency appears, or review exposes an architectural disagreement.

Replanning must update the uncertainty register, decisions, risks, affected implementation packets, model and review assignments, and delivery order. Preserve completed history and record why the plan changed.

### 6. Finish the feature

Use `workflow-finish-feature` after every planned commit is completed or explicitly removed with a reason.

Run feature-level validation from the ledger, then dispatch a fresh feature-complete reviewer. The default is Sol Medium; use Sol High for security, concurrency, data-loss, destructive migration, difficult-to-reverse architecture, or similarly high-risk integration. The feature reviewer checks end-to-end intent and cross-commit interactions instead of repeating commit reviews.

After review clears, reconcile durable documentation and archive the ledger according to `.wiki/INDEX.md` and `.wiki/features/completed/README.md`.

## Model roles

Model capability answers **what must be understood**. Reasoning effort answers **how extensively the selected model must explore and verify it**. Score them independently.

### Luna: executor

Use **Luna** (`gpt-5.6-luna`) when the solution is sufficiently defined and the main challenge is execution and verification. Typical work includes planned UI, established CRUD, repetitive multi-file changes, explicit tests, mechanical refactors, known-cause fixes, and long but bounded implementation with deterministic feedback.

### Terra: engineer and diagnostician

Use **Terra** (`gpt-5.6-terra`) when the outcome is defined but the implementation requires local design judgment. Typical work includes persistence invariants, state lifecycle, cache invalidation, cross-layer integration, bounded diagnosis, asynchronous state, compatibility, and several plausible repository-consistent implementations.

### Sol: architect and high-consequence reasoner

Use **Sol** (`gpt-5.6-sol`) when the framing, invariant, or solution is uncertain or the consequences justify stronger judgment. Typical work includes architecture, conflicting requirements, novel boundaries, concurrency, security, destructive migration, corruption or data-loss risk, public contract strategy, and material replanning.

## Capability Demand

Score each category from 0 to 3:

| Category | 0 | 1 | 2 | 3 |
| --- | --- | --- | --- | --- |
| Residual requirement ambiguity | Exact behavior | Minor local choices | Several design choices | Conflicting or unclear requirements |
| Architectural novelty | Exact analogue | Known pattern in a new place | New abstraction or boundary | Novel cross-cutting architecture |
| Diagnostic uncertainty | None | Cause strongly indicated | Cause uncertain but bounded | Broad or systemic unknown |
| Semantic and consequence risk | Cosmetic or easy reversal | Local behavior | Persistence, compatibility, or contract | Security, corruption, data loss, or irreversible impact |
| Context synthesis | One or two files | One cohesive module | Several layers, languages, or subsystems | Multiple services, repositories, runtimes, or external systems |

Add the five values:

| Total | Initial implementation model |
| --- | --- |
| 0–4 | Luna |
| 5–8 | Terra |
| 9–15 | Sol |

### Luna punch-up exception

Luna can handle a score of 5–6 when all of these conditions hold:

- residual ambiguity and diagnostic uncertainty are each 0 or 1;
- architecture is approved;
- deterministic validation exists;
- the change is reversible;
- the score comes mainly from breadth or cross-layer volume;
- the implementation packet names useful repository patterns;
- failure can be detected before merge.

Use Luna xhigh or max and set `luna_punch_up_applied: true`. Do not use more Luna effort to hide an architectural misunderstanding.

## Effort Demand

Score each category from 0 to 3:

| Category | 0 | 1 | 2 | 3 |
| --- | --- | --- | --- | --- |
| Implementation breadth | Tiny change | A few files | Several modules | Large or long-horizon commit |
| Branch and failure-path density | Straight line | A few alternatives | Many edges or failures | Complex temporal or combinatorial state |
| Repository discovery | Exact files known | Clear analogue | Meaningful exploration | Architecture reconstruction |
| Validation weakness | Fast and strong | Good with minor gaps | Slow or integration-heavy | Weak, manual, flaky, or unavailable |
| Tool and environment coordination | Edit only | Edit plus one check | Several tools or runtimes | Repeated experiments, services, or external systems |

Apply these adjustments, then clamp the total to 0–15:

- Subtract one for an exact named analogue.
- Subtract one for fast deterministic coverage.
- Subtract one when expected files and ownership boundaries are known.
- Subtract one when execution order is explicit.
- Add one for slow, flaky, or mostly manual tests.
- Add one for crossing a language or process boundary.
- Add one for a poorly documented external dependency.
- Add one after a non-trivial failed attempt.
- Add one when historical data or backward compatibility must be preserved.

| Adjusted total | Reasoning effort |
| --- | --- |
| 0 | none |
| 1–3 | low |
| 4–6 | medium |
| 7–9 | high |
| 10–12 | xhigh |
| 13–15 | max |

Use at least low for autonomous mutation.

## Implementation hard floors

Use at least **Terra High** when the root cause is unknown, persistence semantics change, cache invalidation is central, state survives reload or partial failure, several asynchronous states interact, a third-party API must be discovered, or no useful repository analogue exists.

Use at least **Terra xhigh** when existing data migrates, uniqueness or ordering spans several mutation types, retry or idempotency matters, frontend state plus IPC or API plus persistence plus cache interact, critical behavior lacks adequate tests, or stale references must survive replacement.

Use at least **Sol High** when authorization boundaries change, realistic data loss or corruption is possible, concurrency or distributed consistency is central, requirements conflict, a novel boundary is required, the change is difficult to reverse, a foundational planning fact is false, or a lower-tier model misunderstood the task structurally.

Lower a hard floor only with a concrete repository-specific justification in the ledger.

## Review Demand

Score review independently from implementation. Each category is 0–3:

| Category | 0 | 1 | 2 | 3 |
| --- | --- | --- | --- | --- |
| Consequence of a missed defect | Cosmetic | Local and reversible | Persistent or significant user-visible failure | Security, corruption, data loss, irreversible impact, or outage |
| Hidden interaction complexity | Straight line | A few branches | Lifecycle or cross-layer interaction | Concurrency, retries, temporal state, or partial failure |
| Validation weakness | Fast and nearly exhaustive | Strong with minor gaps | Partial or integration-heavy | Weak, manual, flaky, or unobservable |
| Architectural discretion | Exact pattern | Minor local choices | Several plausible implementations | Novel or unresolved boundary |
| Blast radius | Isolated unit | One module | Several layers | Shared contract, historical data, external system, or repository-wide behavior |

| Total | Reviewer profile |
| --- | --- |
| 0–3 | Luna Medium or High, or deterministic validation only for truly mechanical work |
| 4–6 | Terra High |
| 7–9 | Terra xhigh |
| 10–12 | Sol Medium or High |
| 13–15 | Sol High or xhigh plus a bounded specialist review |

Use at least **Terra High** for persistence, state lifecycle, cache invalidation, external APIs, multiple frontend/backend layers, meaningful implementation discretion, or incomplete test coverage.

Use at least **Terra xhigh** for existing-data migration, stale-state survival, uniqueness across mutations, retries, idempotency, partial failure, cancellation, ordering semantics, linked asynchronous states, or persistence plus cache plus UI reconciliation.

Use at least **Sol High** for authorization, credible corruption or data-loss risk, destructive migration, concurrency, cryptography, major public contracts, architectural contradiction, or difficult-to-reverse consequences.

For high-consequence work, the final adjudicator is normally at least as capable as the implementer.

## Review mandate and evidence

Each commit gets a mandate with approximately three to eight concrete concerns derived from its invariants, failure paths, boundaries, lifecycle, error handling, validation, accessibility, compatibility, or data integrity.

Retain a defect only when all three are present:

1. A violated requirement or invariant.
2. A concrete execution path.
3. An observable incorrect consequence.

Every retained finding must include severity, file and location, violated contract, execution path, consequence, existing guard considered, a reproduction or precise missing test, and confidence. Put plausible but unproven concerns under investigation notes. Exclude style-only comments unless style or repository convention is part of the commit contract.

The reviewer distinguishes confirmed defects, missing tests with concrete failure scenarios, investigation notes, and architectural disagreements. It recommends one of: accept, correct locally, escalate capability, or replan.

Optional specialist reviewers may generate candidates for distinct concerns such as lifecycle, persistence, accessibility, or security. Agreement is not proof. A stronger reviewer or adjudicator must validate every retained candidate against code, tests, requirements, and invariants. Prefer three distinct mandates to five generic reviews.

## Correction and adjudication

After a correction, rerun targeted and affected broad validation. Ask the reviewer to verify the corrected findings and newly exposed paths. Repeat the whole review only when the correction materially changes the commit.

Invoke Sol adjudication when the reviewer and implementer disagree about architecture, a high-severity finding remains disputed, or fixing a finding would change an invariant, persisted contract, PR boundary, or later commit.

## Incremental migration

Do not rewrite completed feature history to match this schema.

For an active legacy ledger:

1. Preserve existing intent, decisions, discoveries, completed hashes, and plan deviations.
2. Add complete packets and execution profiles to the active commit before implementation continues.
3. Add profiles to pending commits before each becomes active. Prefer migrating all pending commits during the same planning pass when repository evidence is already loaded.
4. Add implementation and review model columns to Completed work. Use `unknown (pre-routing)` when the historical model cannot be verified.
5. Record migration under Discoveries and replanning only when it changes delivery order, scope, architecture, or an assignment. Schema-only enrichment does not rewrite history.

## Worked example: Squad Planner depth matrix

Representative planned commit: **Add the three-team depth matrix** from `.wiki/features/active/squad-planner.md`.

- **Implementation:** Luna xhigh. Capability Demand is 5: ambiguity 1, novelty 1, diagnosis 0, semantic risk 1, context synthesis 2. The Luna punch-up applies because the Rust read model and UI architecture are approved, the work is reversible, and deterministic React tests exist. Effort Demand is 10 after adjustments because the commit spans route and component composition, multiple display states, keyboard structure, and viewport behavior.
- **Packet summary:** follow the existing Planner route, shared tactic editor, `ScoreBadge`, and route-test patterns; consume the Rust-owned read model; keep matrix state presentational; do not add picker mutations or string controls; validate team tabs, sticky lanes, overflow, scores, and unresolved states.
- **Review:** Terra High. Review Demand is 6: consequence 1, hidden interactions 2, validation weakness 1, discretion 1, blast radius 1. The mandate challenges truthful unresolved/outside-pool rendering, shared tactic-row identity across tabs, keyboard reachability, horizontal overflow at the target viewport, and leakage of later picker or string-management scope.
- **Escalation:** increase Luna effort if the planned ownership is correct but states or tests are incomplete. Escalate to Terra if the read model cannot support truthful rendering without local domain reconstruction. Replan with Sol if the matrix requires a new persisted contract or work assigned to the candidate-picker commit.
