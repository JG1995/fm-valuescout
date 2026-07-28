---
description: Prepare and review one exact-staged atomic local commit
---

Prepare one atomic local commit for this completed work.

Requested focus:

${ARGUMENTS:-Use the completed active commit or the single coherent outcome in the working tree.}

## Mandatory reads

Read `AGENTS.md`, `.wiki/INDEX.md`, the active feature ledger when one exists, **`.cursor/skills/coding-standards/SKILL.md`**, **`coding-standards/references/universal.md`**, **`coding-standards/references/testing.md`** when the staged work includes tests, and matching stack or project references before staging or review.

## Recallium

Read `.cursor/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** presenting the checkpoint package — refresh context on decisions or constraints relevant to this commit.
**Search with:** `search_memories` → `expand_memories` as needed.

**Reconcile after gate, before the checkpoint package** — non-blocking if Recallium is unavailable; report failures and continue.

1. Search recent memories for this feature or area (constraints, roadmap, learnings).
2. Compare the staged outcome and any ledger updates against what Recallium already holds.
3. Decide and report one of:
   - **No update** — nothing durable beyond wiki, Git, or the ledger
   - **New memory** — see triggers below
4. **Prefer new memories over editing old ones.** Recallium is temporal — when constraints or roadmap change, `store_memory` a fresh snapshot and note what changed (and optionally which earlier memory it supersedes). Do not rewrite prior entries in place.
5. Use `modify_memory` only to fix a mistake in a memory created this session, or when the developer explicitly asks to correct or inactivate an entry.

**New memory at checkpoint** — only when not already in wiki, Git, or the ledger:

| Trigger | Type | Example |
| --- | --- | --- |
| Invariants or non-goals changed | `feature` | New constraints snapshot after replanning |
| Active work or plan-next changed | `progress` | Roadmap snapshot after commit lands |
| Non-obvious gotcha from this commit | `learning` or `debug` | CI flake, offset pin, protocol quirk |
| Durable choice not warranting an ADR | `decision` | Small boundary call |

Do not mirror the feature ledger or staged diff. Skip when unsure.

## Checkpoint steps

1. Inspect `git status --short` and the complete unstaged diff.
2. State the one coherent, revertible outcome and identify unrelated or accidental changes.
3. Update documentation intrinsic to this commit before staging. If the implementation diverged from the plan, update the ledger under **Discoveries and replanning** with what changed and why — include that ledger file in the staged commit when it documents the deviation.
4. When the diff includes implementation or test files, run `./scripts/dev format` (optional paths forwarded to Biome; `cargo fmt` always runs). Re-inspect the diff and `git status` — autofix may change files beyond the active focus. Include formatting fixes in this commit when they belong to touched files.
5. If an active ledger is part of this commit, mark the commit `Completed — hash pending checkpoint commit`.
6. Stage exact files or hunks only. Never use `git add .` or `git commit -a`.
7. Run and inspect `git diff --cached --check`, `git diff --cached --stat`, and the complete staged diff.
8. Run targeted tests and `./scripts/dev check`. Status 69 on smoke/mutate means unsupported, not passed.
9. Require a separate reviewer pass for non-trivial work. Dispatch via Task when available:

    Task({
      subagent_type: "reviewer",
      prompt: "Review following .cursor/agents/reviewer.md and AGENTS.md. Search Recallium per .cursor/rules/recallium.mdc when wiki/Git do not explain prior decisions relevant to the diff. Read .cursor/skills/coding-standards/SKILL.md, coding-standards/references/universal.md, and coding-standards/references/testing.md when diff includes tests, plus matching stack refs. Read other matching .cursor/skills/. Judge active commit scope only. Return the full Review verdict with CRITICAL/HIGH/MEDIUM/NITPICK tiers. Do not fix anything — report only.",
      description: "Review staged changes",
    })

    Or run `/review` in a separate read-only chat.

    **CRITICAL** and **HIGH** block commit until fixed or **explicitly approved** by the developer.

9. **Present the reviewer report to the developer** — include the full **Review verdict** (all tiers, Blocking status, plan scope note). Do **not** automatically fix CRITICAL, HIGH, or other findings in `/checkpoint`. The developer delegates fixes via **`/fix`**, manual edit, or explicit instructions — then runs `/checkpoint` again.

10. **Assess the commit message** — do not invent a new subject from scratch when the plan already has one:
    - Start from the active commit's **Provisional commit** in the feature ledger (set by `/plan-feature`).
    - Compare it to the staged diff per `.cursor/skills/conventional-commits/SKILL.md`: does it describe the actual outcome after this commit?
    - If yes: present it as **unchanged** — use this message at commit unless the developer overrides.
    - If no (plan divergence, scope shift, wrong type/scope, or message no longer matches staged work): present a **revised** message and state briefly why the provisional no longer fits. Update the ledger **Provisional commit** when the revision reflects plan change, not mere wording polish.
    - If there is no ledger or no provisional for this commit: derive one message from the staged outcome using the conventional-commits skill.

11. Present the complete checkpoint package: outcome, staged files, RED/GREEN evidence, gate results, the **Review verdict** (verbatim), documentation impact, risks, the **commit message assessment** (provisional, unchanged or revised with reason), and **Recallium** (`no update` / `new #… (type, one-line why)` / `skipped — Recallium unavailable`). If the plan diverged, summarize the deviation.

12. **Stop and wait for explicit developer approval.** If **Blocking: Yes** and the developer has not explicitly approved proceeding with blocking findings recorded, do not commit. Unstaging or editing code to address review findings without developer direction is not part of this command.

13. Only after explicit developer approval, commit locally with the approved message. Never push, amend, rebase, squash, or rewrite history.

14. Report the resulting hash.

15. **Ledger advancement (immediate follow-up commit).** When an active feature ledger tracks this work, update it right after the content commit succeeds — do not leave the update unstaged for a later ask:
    - Replace `Completed — hash pending checkpoint commit` with `Completed — \`<hash>\``.
    - Add the row to **Completed work**.
    - Mark the next planned commit `Active` (or close the PR / activate the next PR when appropriate).
    - Refresh **Active work** (RED test, expected outcome, exclusions) for that next commit.
    - Stage **only** the ledger file(s), commit immediately with a short `docs(…)` message (e.g. `docs(memory-read): record commit N hash and activate commit N+1`).
    - This second commit does **not** need a new reviewer pass or a second approval when it is ledger-only advancement from the just-approved checkpoint. Report both hashes when done.

## Fixing review findings

Addressing CRITICAL/HIGH (or any) findings is **not** automatic. Typical flow:

1. Developer reads the Review verdict.
2. Developer runs **`/fix`** (or gives explicit fix instructions / edits manually).
3. Developer runs **`/checkpoint`** again when ready.

Only fix review findings inside `/checkpoint` when the developer **explicitly asks** you to apply specific corrections in this turn.
