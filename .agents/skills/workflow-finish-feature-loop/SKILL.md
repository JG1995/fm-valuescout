---
name: workflow-finish-feature-loop
description: Finish a complete feature through automatic Sol High review/fix loops, documentation reconciliation, and local close-out commits
---

Validate and close one implementation-complete feature through an automated **feature review → fix → validation → review** loop. When the feature review clears, reconcile documentation and commit the reviewed corrections and documentation locally.

**Manual invocation only.** Run this skill only when the developer explicitly types `$workflow-finish-feature-loop`. Never suggest it, run it automatically, or substitute it for `$workflow-finish-feature`.

Typing `$workflow-finish-feature-loop` is explicit approval for the local commits described in **Phase 3** and **Phase 4**. It does not approve a push, amend, rebase, squash, merge, or history rewrite.

Use the developer-supplied feature or comparison base when present. Otherwise, use the active feature ledger and establish the base from repository evidence. Ask before proceeding when the base is ambiguous.

## Scope and preconditions

Follow `.agents/skills/workflow-finish-feature/SKILL.md` for mandatory reads, Recallium, scope, plan verification, and validation.

Before validation:

1. Confirm every planned commit is `Completed — <hash>` or `Removed — <reason>`.
2. Require a clean tracked worktree. Stop if unrelated or uncommitted tracked changes would make correction ownership or auto-commit scope ambiguous.
3. Confirm the comparison base and complete feature diff.
4. Do not start documentation reconciliation until the feature review clears.

## Loop contract

Maximum **3 fix rounds**. One fix round includes delegated correction, full feature validation, and another Sol High feature-complete review.

```text
validate feature
→ Sol High feature review 1
→ if CRITICAL | HIGH | MEDIUM: fix → validate → Sol High review 2
→ if still CRITICAL | HIGH | MEDIUM: fix → validate → Sol High review 3
→ if the same finding survives both corrections: increase correction capability or effort before the third and final fix
→ if still CRITICAL | HIGH | MEDIUM: fix → validate → Sol High review 4 (final)
→ if still CRITICAL | HIGH | MEDIUM: stop — do not commit or reconcile docs
→ if only NITPICK or none: commit reviewed corrections, reconcile docs, and commit docs
```

| Finding tier | Loop behavior |
| --- | --- |
| CRITICAL | Auto-fix when bounded; stop and replan when it changes an invariant, boundary, persisted contract, validation contract, or PR boundary |
| HIGH | Auto-fix when bounded; use the same replan conditions as CRITICAL |
| MEDIUM | Auto-fix by default before feature close-out |
| NITPICK | Fix only when the same verdict also has CRITICAL, HIGH, or MEDIUM; otherwise proceed without a fix round |

Track findings by violated contract and execution path. After two failed correction attempts on the same finding, use the third attempt only with increased implementation capability or reasoning effort. The feature reviewer remains Sol High for every pass. Stop after three fix rounds even when different findings appear in later passes.

## Phase 1 — Validate and review

Run `.agents/skills/workflow-finish-feature/SKILL.md` sections **1–3** in full.

Dispatch a separate fresh-context **generic** feature reviewer at **Sol High** (`gpt-5.6-sol`, `high`). Do not use the named reviewer because its pinned Terra High profile is for commit review. Give the reviewer the ledger, linked planned spec, comparison base, complete feature diff, commit review summaries, final validation results, and repository access. Do not provide the implementers' reasoning before the independent pass.

Require the evidence-shaped Review verdict from `.codex/agents/reviewer.toml`.

## Phase 2 — Automated correction loop

When the verdict contains CRITICAL, HIGH, or MEDIUM findings and rounds remain:

1. Validate each finding against its violated contract, execution path, consequence, and existing guards.
2. Route a bounded finding owned by one completed commit to that commit's original implementation profile. Follow `.agents/skills/workflow-fix/SKILL.md` with the explicit feature-correction scope.
3. Include NITPICK findings in the same correction pass when the verdict is mixed.
4. Stop and return to planning when a correction crosses commit boundaries or changes an invariant, persisted or public contract, architecture seam, validation contract, PR boundary, or later feature dependency.
5. Keep corrections uncommitted during the loop. Review the complete feature including working-tree corrections with `git diff <base>`, `git status --short`, and the contents of any untracked correction files, or use an equivalent complete diff.
6. Run the full validation sequence from `$workflow-finish-feature` after each correction round.
7. Ask the same Sol High reviewer context to verify corrected findings and newly exposed paths. Start a fresh Sol High review only when the correction materially changes the feature architecture or review mandate.

Do not run `$workflow-fix` for a NITPICK-only verdict. Proceed to Phase 3.

## Phase 3 — Commit reviewed corrections

When the final feature verdict has no CRITICAL, HIGH, or MEDIUM findings:

1. If no correction files changed, record that no content commit is needed.
2. If corrections changed implementation, tests, or intrinsic documentation:
   - inspect the complete correction diff and exclude unrelated changes;
   - stage exact correction files or hunks;
   - run `git diff --cached --check`, inspect the staged stat, and inspect the complete staged diff;
   - confirm the staged correction content is the same content covered by the final Sol High feature review;
   - assess a focused Conventional Commit message and commit locally without another approval wait.

The validation sequence already runs formatting before the final review. If staging exposes a content or formatting change, return to validation and Sol High review. The final Sol High feature review satisfies the independent review requirement for the correction commit only when it reviewed the same correction content. Dispatch a commit reviewer when staging changes the reviewed scope.

## Phase 4 — Reconcile and commit documentation

Run `$workflow-docs-review` as the foreground Documentation Steward pass from `.agents/skills/workflow-finish-feature/SKILL.md` section **5**.

After the steward returns:

1. Inspect the complete documentation diff and confirm it contains documentation and ledger archival only.
2. Run `./scripts/dev check`.
3. Stage exact documentation files.
4. Run `git diff --cached --check`, inspect the staged stat, and inspect the complete staged diff.
5. Commit the documentation reconciliation locally with a focused `docs(…)` message. No second reviewer pass is required for steward-owned documentation-only reconciliation.

Skip the documentation commit when reconciliation produces no changes.

## Phase 5 — Close out

Follow the Recallium reconciliation and close-out report from `$workflow-finish-feature`.

Report:

- correction rounds used (0–3) and each review verdict in brief;
- the final Sol High verdict and any remaining NITPICK;
- full validation results;
- correction commit hash, or `none`;
- documentation commit hash, or `none`;
- delivered behavior, plan fidelity, documentation and ledger status, Recallium result, and remaining risks.

## Failure exit

If the final review still has CRITICAL, HIGH, or MEDIUM findings after three fix rounds, stop without committing or running documentation reconciliation. Report the final verdict, all correction rounds, current worktree changes, and the required replanning or developer decision.

Do not push, amend, rebase, squash, merge, or rewrite history.
