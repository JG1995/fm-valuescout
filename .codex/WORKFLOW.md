# Adaptive Development Workflow

This document defines shared planning, implementation, validation, review, escalation, and PR-boundary policy. `AGENTS.md` owns standing repository rules. Installed `workflow-*` skills own phase procedures. `.wiki/features/active/README.md` owns the feature-ledger template.

## Invocation policy

Every `workflow-*` skill requires explicit user invocation. Do not select one from an ordinary natural-language request. In Codex, select the skill through `/skills` or mention `$workflow-<name>`. Every workflow skill must set `policy.allow_implicit_invocation: false` in `agents/openai.yaml` and repeat the boundary in its description.

## Lifecycle

```text
feature plan
  → implement one commit
  → project-defined validation
  → fresh-context commit review
  → correction, escalation, or replanning
  → PR publication and merge boundary
  → feature-complete validation and review
  → documentation reconciliation
```

Trivial work can use a short work contract instead of a feature ledger. Workflow invocation does not grant Git authority except for the two loop permissions centralized in `AGENTS.md`.

Every phase uses the repository as project memory. Read the `project-context` skill for the targeted inspection order and documentation routing. Update the narrowest durable owner in the phase that makes the information true.

## 1. Plan

Use `$workflow-plan-feature`. Create one ledger in `.wiki/features/active/` and ground it in actual files, symbols, tests, commands, current-state documents, ADRs, and debug reports.

The plan must:

- distinguish invariants from implementation preferences;
- split work into independently reviewable, trunk-safe PRs and commits;
- give each commit a bounded implementation packet, implementation profile, review profile, validation commands, stop conditions, and review mandate;
- identify exact PR dependencies and publication boundaries;
- mark exactly one commit `Active` inside the active PR;
- absorb accepted content from a planned feature spec into the ledger and delete the promoted spec;
- keep discoveries and material deviations in the ledger.

Use **Terra xhigh** for planning when the architecture and feature boundary are established. Use **Sol High** when several architectures remain plausible, persistence or migration design is unsettled, requirements conflict, security or concurrency boundaries change, ownership is unclear, or implementation discoveries require material replanning. Reserve **Sol xhigh** for several combined Sol conditions or difficult-to-reverse security, corruption, or data-loss consequences.

## 2. Implement one commit

Use `$workflow-build` or the manually requested `$workflow-build-loop`. The main session implements the active commit and later review fixes. Do not dispatch a separate implementation agent only to satisfy model routing.

Before editing, read the active work, governing requirements, invariants, exclusions, implementation packet, named analogues, validation commands, and stop conditions. Verify analogues before copying them. Start with the planned RED test or smallest failing proof when practical. Implement only the active commit.

The implementation handoff states files changed, behavior implemented, tests changed, commands run, unresolved uncertainty, packet deviations, and escalation or replanning status.

## 3. Validate

Use the project-defined commands recorded in the active commit, from cheapest to broadest:

1. Targeted tests or a focused reproducible proof.
2. Affected module tests.
3. Integration or end-to-end tests.
4. Static analysis, formatting, or type checking.
5. The project's documented commit gate.
6. Manual or real-environment checks only where automation cannot prove the contract.

Use the stable `./scripts/dev` commands documented in `AGENTS.md`. Do not claim a command passed when it was unavailable, unsupported, skipped, or replaced with weaker evidence. A command result is evidence; confidence is not.

## 4. Review in a fresh context

Every non-trivial commit gets a separate read-only reviewer after deterministic validation. Use the review profile in the ledger. Without a ledger, use the named `reviewer` at its default profile. When the ledger assigns another profile, use a generic read-only reviewer with the same contract.

Give the reviewer the original commit specification, relevant invariants and non-goals, implementation packet, staged diff, validation results, review mandate, and repository access. Do not initially give the implementer's reasoning or defense.

After corrections, reuse the same reviewer context when available. Start a fresh review when that context is unavailable or the correction materially changes scope, architecture, or mandate. Main-session self-review does not replace independent review.

Retain a defect only when the reviewer identifies:

1. A violated requirement or invariant.
2. A concrete execution path.
3. An observable incorrect consequence.

Every finding includes severity, location, violated contract, execution path, consequence, existing guard considered, reproduction or missing test, and confidence. Put plausible but unproven concerns under investigation notes.

Keep the owning commit `Active` through review and correction. Advance delivery state only after no CRITICAL, HIGH, or MEDIUM finding remains, or after the developer explicitly accepts committing with the remaining findings recorded.

## 5. Correct, escalate, or replan

Correct bounded execution defects in the main session. Increase reasoning effort when the ownership and abstraction are correct but branches, tests, or integration details are incomplete. Increase model capability when the implementation chose the wrong abstraction, misunderstood ownership or an invariant, patched a symptom, or invented project facts.

After two failed corrections for the same defect, stop and request a profile change or replan. Replan when a known fact is disproved, an invariant or approved boundary changes, a required seam does not exist, a public or persisted contract changes materially, validation cannot be meaningful, work crosses a PR boundary, or review exposes an architectural disagreement.

Replanning updates the uncertainty register, decisions, risks, affected packets, profiles, and delivery order. Preserve completed history and record why the plan changed.

## 6. Respect PR boundaries

The ledger owns publication state as well as implementation state.

- Work on one active PR at a time.
- When its final commit clears checkpoint, mark the PR `Ready for publication` and stop. Do not activate the next PR.
- Record the PR number or URL when published and the merge commit or equivalent immutable ref when merged.
- Mark a dependent PR `Awaiting prior PR merge` until its dependencies are merged.
- Activate the next PR only after its dependencies are recorded as merged.
- Do not merge, push, publish, or rewrite history without explicit developer approval.

For the final implementation PR, run `$workflow-finish-feature` after all planned commits are complete and all earlier PRs are merged. The final PR can remain unmerged while feature review and documentation reconciliation run. Scope feature review to the exact recorded commit and PR refs, not an assumed `base...HEAD` range.

## 7. Finish the feature

Use `$workflow-finish-feature` after every planned commit is completed or removed with a reason. Use `$workflow-finish-feature-loop` only when the developer explicitly opts into automatic review corrections and the local commits described by that skill.

Run the ledger's feature-level validation, then dispatch a fresh **Sol High** feature reviewer. Review the exact recorded implementation set, including merged earlier PRs and the final PR's recorded commits. The reviewer checks end-to-end intent and cross-commit interactions rather than repeating each commit review.

After review clears, reconcile durable documentation, condense and archive the ledger, and prepare the final PR for publication. Record the final PR and merge ref when the merge occurs; do not claim the feature is merged before repository evidence shows it.

## Model routing

Choose profiles with short repository-specific reasons. Do not calculate numeric scores unless a project supplies a validator or consumer for them.

### Implementation profiles

- **Luna:** Use for defined, reversible execution with strong validation and useful analogues. Increase effort for breadth, many branches, or slow feedback.
- **Terra:** Use when the outcome is defined but local design judgment, diagnosis, persistence, lifecycle, or cross-layer integration is material.
- **Sol:** Use when the framing, invariant, architecture, or solution remains uncertain, or when security, concurrency, corruption, data loss, or irreversible change raises the consequence.

Use at least Terra High for unknown root cause, meaningful persistence semantics, cache invalidation, several asynchronous states, third-party API discovery, or missing analogues. Use at least Terra xhigh for existing-data migration, retry or idempotency behavior, or several interacting persistence, cache, API, and UI layers. Use at least Sol High for authorization boundaries, credible corruption or data-loss risk, concurrency, conflicting requirements, novel boundaries, or difficult-to-reverse changes.

### Review profiles

- **Luna Medium:** Mechanical work with strong deterministic validation and little discretion.
- **Terra Medium or High:** Ordinary behavioral work; use High for persistence, lifecycle, external APIs, cross-layer behavior, or incomplete coverage.
- **Terra xhigh:** Existing-data migration, retries, idempotency, or difficult partial-failure behavior.
- **Sol High:** Authorization, corruption or data-loss risk, destructive migration, concurrency, cryptography, major public contracts, or architectural contradiction.

The final feature reviewer is always Sol High unless project guidance requires a stronger profile.

## Incremental migration

Do not rewrite completed feature history to match a new schema. For a legacy active ledger, preserve intent, decisions, discoveries, completed refs, and deviations. Add missing packets, profiles, validation commands, review mandates, and PR state only for active and pending work.
