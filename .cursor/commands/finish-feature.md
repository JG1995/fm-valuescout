---
description: Validate a complete feature end-to-end, feature review, then documentation reconciliation
---

Complete feature-level validation and reconciliation for:

${ARGUMENTS:-The active feature ledger. Establish the comparison base from repository evidence (merge base with trunk, first feature commit, or named branch). Ask before proceeding if ambiguous.}

## Mandatory reads

Read `AGENTS.md`, `.wiki/INDEX.md`, the **active feature ledger**, the linked **planned feature spec** in `.wiki/features/planned/` when one exists, relevant `CONCEPT.md` bullets, and matching skills in `.cursor/skills/`.

Confirm the feature is **implementation-complete**: every commit in the delivery plan is `Completed — <hash>` (or `Removed — <reason>` with documented rationale). If work remains, stop and report what is still pending — do not run finish validation.

## Phase discipline

1. **Validate** — full tests and gates first.
2. **Review** — feature-complete reviewer pass on the whole implemented feature (not staged-only).
3. **Reconcile docs** — only after review is clear (Blocking: No, or developer explicitly approves proceeding with blocking findings recorded).
4. **Report** — delivered behaviour, architecture, validation, documentation, commit history, remaining risks.

Do not stage or commit unless the developer separately requests `/checkpoint`. Documentation edits from `/docs-review` are unstaged until checkpoint.

## Recallium

Read `.cursor/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** establishing scope and plan verification — prior decisions, constraints, and feature context not in wiki or ledger.
**Search with:** `search_memories` → `expand_memories` as needed; `recallium` / `session_recap` when resuming feature close-out across sessions.

**Reconcile after documentation reconciliation, before the close-out report** — non-blocking if Recallium is unavailable after the resilience retries in `.cursor/skills/recallium-usage/SKILL.md` § Call resilience; report failures and continue.

1. Search memories for this feature (constraints, roadmap, learnings, prior progress snapshots).
2. Compare the implemented feature, archived ledger, and updated wiki against what Recallium already holds.
3. Decide and report one of:
   - **No update** — completion context is already in wiki, Git, or the archived ledger
   - **New memory** — see triggers below
4. **Prefer new memories over editing old ones.** Recallium is temporal — store a fresh close-out snapshot; note what changed and optionally which earlier memory it supersedes. Do not rewrite prior entries in place.
5. Use `modify_memory` only to fix a mistake in a memory created this session, or when the developer explicitly asks to correct or inactivate an entry.

**New memory at finish-feature** — only when not already in wiki, Git, or the archived ledger:

| Trigger | Type | Example |
| --- | --- | --- |
| Feature complete — final constraints or boundaries settled | `feature` | Close-out constraints snapshot |
| Roadmap or plan-next changed after this feature | `progress` | MVP sequence after ledger archived |
| Non-obvious learnings from the feature as a whole | `learning` or `debug` | Cross-commit gotcha not in ledger |
| Completion summary worth resuming later | `progress` | What shipped, what was deferred, key risks |

Do not mirror the archived ledger or full feature diff. Skip when unsure.

## 1. Establish scope

1. Identify the feature, active ledger, and comparison base (`git merge-base`, first feature commit, or explicit ref from `${ARGUMENTS}`).
2. Inspect the **complete feature diff** (`git diff <base>...HEAD` or equivalent) and chronological commit history on the branch.

## 2. Plan and spec verification

Before review, confirm implementation against sources of truth:

| Source | Check |
| --- | --- |
| Feature ledger | Intent, user-visible behaviour, invariants, non-goals |
| Delivery plan | Every planned commit delivered or explicitly removed with reason |
| Walking skeleton | Thinnest path through the feature exists and works |
| Planned spec | `.wiki/features/planned/<slug>.md` when linked — behaviour and scope |
| `CONCEPT.md` | Relevant MVP bullets for this feature |
| Discoveries | Plan deviations documented with what and why |

Note gaps explicitly for the reviewer and final report. Do not silently treat missing planned work as done.

## 3. Validation (run before review)

Run in order. **Do not dispatch feature review until `./scripts/dev check` and the test suite have been run and results are known.**

0. **`./scripts/dev format`** — when the feature branch includes implementation or test changes; re-inspect the diff and include formatting fixes in the feature branch before gates.
1. **Full test suite** — `./scripts/dev test` with no target (or the project's documented full-suite invocation when `DEV_TEST_COMMAND` is configured). Add ledger-listed targeted tests when the full suite does not cover them.
2. **`./scripts/dev check`** — repository scripts and Cursor configuration gate.
3. **`./scripts/dev smoke`** — when configured; status **69** means unsupported — report as unsupported, not passed.
4. **`./scripts/dev mutate <target>`** — when configured for critical logic touched by this feature; status **69** means unsupported.
5. **Migration, preservation, rollback** — when the feature touches persistence or schema.
6. **Hygiene scan** — abandoned compatibility paths, TODOs, feature flags, dead code, disposable experiment artifacts (e.g. leftover `.cursor/work/spikes/` or spike branches merged by mistake).

Report pass/fail for each step. A failing gate or test suite blocks moving to review unless the developer explicitly overrides.

## 4. Feature-complete review

Require a **separate** reviewer pass — not self-review of implementation you wrote in this turn.

**Model:** Always use the pin in `.cursor/agents/reviewer.md` frontmatter. **Never** pass Task `model`. No exceptions.

Dispatch via Task when available:

    Task({
      subagent_type: "reviewer",
      // do not pass model — reviewer.md frontmatter owns it
      prompt: "Feature-complete review per .cursor/agents/reviewer.md — mode: feature-complete. Comparison base: <base>. Feature: <name>. Search Recallium per .cursor/rules/recallium.mdc when wiki/Git do not explain prior decisions relevant to the feature. Read active ledger, planned spec, and git diff <base>...HEAD. Judge the implemented feature as a whole against intent, user-visible behaviour, invariants, and delivery plan. Return full Review verdict with CRITICAL/HIGH/MEDIUM/NITPICK. Do not fix anything — report only.",
      description: "Feature-complete review",
    })

Or run `/review` in a separate read-only chat with the same mode and scope.

**Present the full Review verdict to the developer.** Do **not** automatically fix findings. Delegate via **`/fix`** or explicit instructions, then re-run validation and review from step 3.

**CRITICAL** and **HIGH** block documentation reconciliation and merge readiness until fixed or **explicitly approved** by the developer.

## 5. Documentation reconciliation (after review clears)

Only when **Blocking: No** on the feature-complete verdict, or the developer has **explicitly approved** proceeding with blocking findings recorded:

Run **`/docs-review`** as a foreground Documentation Steward pass scoped to this feature.

**Model:** Always use the pin in `.cursor/agents/documentation-steward.md` frontmatter. **Never** pass Task `model`. No exceptions.

    Task({
      subagent_type: "documentation-steward",
      // do not pass model — documentation-steward.md frontmatter owns it
      prompt: "Feature completion reconciliation per .cursor/agents/documentation-steward.md and AGENTS.md. Search Recallium per .cursor/rules/recallium.mdc when prior decisions affect reconciliation. Feature: <name>. Reconcile durable documentation with implemented state; condense and move the active ledger to features/completed/; update ARCHITECTURE.md, TODO.md, and INDEX-owned docs as needed. ${ARGUMENTS}",
      description: "Reconcile documentation",
    })

Do not run in the background or edit the worktree concurrently. After it returns, inspect the documentation diff and run `./scripts/dev check`.

If documentation reconciliation introduces blocking issues (contradictions, missing ADR for a structural change), report them — do not commit without developer direction.

## 6. Close out

1. Report:
   - **Delivered behaviour** — against ledger and spec
   - **Plan fidelity** — delivered vs planned; documented deviations
   - **Final architecture** — what changed in the repo
   - **Validation** — test suite, gate, smoke/mutate, migration checks
   - **Review verdict** — summary and blocking status
   - **Documentation** — steward changes, ledger archival, remaining doc risks
   - **Recallium** — `no update` / `new #… (type, one-line why)` / `skipped — Recallium unavailable`
   - **Commit history** — atomic outcomes, Conventional Commits quality on the feature branch
   - **Remaining risks** — follow-ups, MEDIUM/NITPICK backlog, merge readiness

2. Remind the developer to run **`/checkpoint`** to stage and commit documentation reconciliation (and any final ledger moves) when ready.
