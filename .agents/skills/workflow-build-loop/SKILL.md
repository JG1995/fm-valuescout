---
name: workflow-build-loop
description: Build the active commit, then auto checkpoint/fix until review is clean (NITPICK-only) — manual opt-in only
---

Build the **active commit** (same scope as `$workflow-build`), then run an automated **checkpoint → fix → checkpoint** cycle until the reviewer reports no **CRITICAL**, **HIGH**, or **MEDIUM** findings. **NITPICK-only** verdicts skip `$workflow-fix` and proceed to auto-commit; **mixed** verdicts fix NITPICK alongside delegated tiers in the same pass. When clean, commit the content and advance the ledger without waiting for approval.

**Manual invocation only.** Run this command only when the developer explicitly types `$workflow-build-loop`. Never suggest it, never run it automatically, and never substitute it for the default `$workflow-build` → `$workflow-checkpoint` flow unless the developer asked for it.

Requested focus:

Use the developer-supplied commit scope when present. Otherwise, use the active commit from the active feature ledger.

## Scope

Same as `$workflow-build`:

- **One active commit** only — do not start the next commit or batch work.
- Same mandatory reads, RED/GREEN discipline, ledger updates during build, and explicit exclusions from the ledger.
- Do not push, amend, rebase, squash, or rewrite history unless the developer explicitly asks later.

## Loop contract

Maximum **5 fix rounds**. One fix round = `$workflow-fix` then another `$workflow-checkpoint`.

```text
$workflow-build
→ $workflow-checkpoint (review 1)
→ if CRITICAL | HIGH | MEDIUM: $workflow-fix → $workflow-checkpoint (review 2)
→ if still CRITICAL | HIGH | MEDIUM: $workflow-fix → $workflow-checkpoint (review 3)
→ if still CRITICAL | HIGH | MEDIUM: $workflow-fix → $workflow-checkpoint (review 4)
→ if still CRITICAL | HIGH | MEDIUM: $workflow-fix → $workflow-checkpoint (review 5)
→ if still CRITICAL | HIGH | MEDIUM: $workflow-fix → $workflow-checkpoint (review 6 — final)
→ if still CRITICAL | HIGH | MEDIUM: stop — report verdict, do not commit
→ if only NITPICK or none: auto-commit content + ledger advancement
```

| Finding tier | In loop | Blocks manual `$workflow-checkpoint` commit? |
| --- | --- | --- |
| CRITICAL | Auto-fix; stop without commit if still present after 5 fix rounds | Yes |
| HIGH | Auto-fix; stop without commit if still present after 5 fix rounds | Yes |
| MEDIUM | Auto-fix by default (same as `$workflow-fix` with no args); stop without commit if still present after 5 fix rounds | No — developer may approve commit with MEDIUM in manual `$workflow-checkpoint` |
| NITPICK | Fix only when the same verdict also has CRITICAL, HIGH, or MEDIUM; otherwise leave as-is and exit | No |

**NITPICK-only verdicts:** When a review lists **only** NITPICK findings (no CRITICAL, HIGH, or MEDIUM), do **not** run `$workflow-fix`. Proceed directly to Phase 3 (auto-commit).

**Mixed verdicts:** When a review lists CRITICAL, HIGH, and/or MEDIUM **and** NITPICK items, run `$workflow-fix` for the delegated tiers **and** include the NITPICK items in the same fix pass.

**Exit success:** Review verdict has no CRITICAL, HIGH, or MEDIUM items (residual NITPICK allowed only if you chose not to fix them in a prior mixed pass — normally none remain).

**Exit failure:** After **5** `$workflow-fix` rounds, the final `$workflow-checkpoint` review still lists any CRITICAL, HIGH, or MEDIUM finding. Present the full verdict and loop summary; do not commit.

## Phase 1 — Build

Follow `.agents/skills/workflow-build/SKILL.md` in full through implementation, format, gate, and ledger updates for the active commit.

Do **not** stop for manual `$workflow-checkpoint` at the end of build — continue into Phase 2 in the **same command invocation** (do not wait for the developer between phases).

## Phase 2 — Automated checkpoint / fix loop

For each checkpoint pass in the loop, follow `.agents/skills/workflow-checkpoint/SKILL.md` steps **1–11** (inspect, intrinsic docs, stage exact hunks, format, gate, reviewer subagent, commit message assessment, Recallium reconcile, present verdict).

Use the same named `reviewer` Codex agent selection as `$workflow-checkpoint`. If named-agent dispatch is unavailable, use its generic read-only fallback. Do not add a model pin.

Differences from manual `$workflow-checkpoint`:

- **Do not wait for developer approval** between loop iterations.
- When the verdict still has CRITICAL, HIGH, or MEDIUM and fix rounds remain, run `$workflow-fix` with default delegation (**CRITICAL**, **HIGH**, and **MEDIUM** from that verdict) per `.agents/skills/workflow-fix/SKILL.md`. When the same verdict also lists NITPICK items, include them in that `$workflow-fix` pass. Include the **build-loop carve-out** (continue to the next checkpoint; do not stop for the developer).
- When the verdict is NITPICK-only or empty, proceed to Phase 3 — do **not** run `$workflow-fix` for NITPICK-only.

Track and report in the final summary:

- Fix round count (0–5)
- Each review verdict (at least final; brief notes on earlier rounds if findings changed)

## Phase 3 — Auto-commit (success only)

When the loop exits successfully:

1. Commit the staged content locally with the assessed message from the final checkpoint (Conventional Commits per `.agents/skills/conventional-commits/SKILL.md`). No approval wait.
2. **Immediately** advance the active feature ledger per `.agents/skills/workflow-checkpoint/SKILL.md` step **15** — real hash, **Completed work** row, next commit `Active`, refresh **Active work**.
3. Commit ledger-only changes in a separate `docs(…)` commit. No second reviewer pass for ledger-only advancement.
4. Report both hashes, gate evidence, final review verdict (including any NITPICK), and loop round count.

## Mandatory reads

Same as `$workflow-build` plus `.agents/skills/workflow-checkpoint/SKILL.md` and `.agents/skills/workflow-fix/SKILL.md` for loop phases.

**Coding standards:** Read `.agents/skills/coding-standards/SKILL.md`, `references/universal.md`, and `references/testing.md` when the commit adds or changes tests. Load matching stack references when `ARCHITECTURE.md` or touched files apply.

**Minimalism:** Run the decision ladder in `.agents/skills/minimalism/SKILL.md` during build and fix phases.

## Recallium

Read `.agents/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project. Apply **Call resilience** (re-auth + retries) on MCP failures.

**Search before:** build — unfamiliar conventions or constraints for this commit; each checkpoint — context relevant to the staged outcome.
**Search with:** `search_memories` → `expand_memories` as needed.

**Save after:** build or fix — only when the work surfaced durable progress or constraints not already in the ledger; checkpoint — per checkpoint Recallium triggers (not ledger mirrors).
**Save with:** `store_memory` — one concise memory; prefer new snapshots over editing old ones.

Skip save when unsure.

## Role boundary

| Command | Scope |
| --- | --- |
| `$workflow-build` | Implement active commit; stop for manual `$workflow-checkpoint` |
| `$workflow-build-loop` | Implement active commit + automated checkpoint/fix loop + auto-commit on success |
| `$workflow-fix` | Correct delegated review findings; stop for manual `$workflow-checkpoint` unless under `$workflow-build-loop` |
| `$workflow-checkpoint` | Stage, review, present; commit only when developer approves |

Do not use `$workflow-build-loop` for `$workflow-finish-feature`, documentation reconciliation, or multi-commit feature runs.

## After completion

**Success:** Report content hash, ledger hash, loop rounds used, final verdict (NITPICK listed if any), and next active commit from the ledger.

**Failure:** Report that the 5-fix cap was reached, the final verdict, what remains unfixed, and that nothing was committed. Tell the developer to `$workflow-fix` manually (narrow or broad) and `$workflow-checkpoint` when ready.

Do not start the next planned commit in the same command invocation unless the developer explicitly asks to continue.
