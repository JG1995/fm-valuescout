---
name: workflow-docs-review
description: Reconcile durable documentation with implemented repository state
---

Reconcile durable documentation with implemented state for:

The developer-supplied implemented change when present; otherwise, the active feature when finishing via `$workflow-finish-feature`.

## Role in the workflow

| When | Who invokes | Preconditions |
| --- | --- | --- |
| **Feature completion** | `$workflow-finish-feature` (step 5) | Implementation-complete; tests and gate run; **feature-complete review** clear (Blocking: No, or developer explicitly approved with findings recorded) |
| **Architecture milestone** | Developer directly | Significant implemented structure change — e.g. first app landing after `$workflow-stack`, layer refactor — without a full `$workflow-finish-feature` pass |
| **Not here** | `$workflow-build`, `$workflow-checkpoint` | Intrinsic docs only per commit — see `.agents/skills/workflow-build/SKILL.md` |

Broad wiki reconciliation belongs here or in `$workflow-finish-feature`, not after every commit.

## Documentation Steward dispatch

Dispatch a foreground named `documentation-steward` Codex agent. If named-agent dispatch is unavailable, use a generic documentation-only agent with the same boundary. Do not edit the worktree concurrently.

Do not run it in the background or edit the worktree concurrently. After it returns, inspect the complete documentation diff and run `./scripts/dev check`.

## Mandatory reads (manual fallback or parent command)

Read `AGENTS.md`, `.agents/WORKFLOW.md`, `.wiki/INDEX.md`, the **technical-writing** skill (`.agents/skills/technical-writing/SKILL.md`), the active feature ledger when one exists, the linked **planned spec** when present, relevant `CONCEPT.md` bullets, and implementation evidence:

- **Feature completion** — `git diff <base>...HEAD` for the feature (same base as `$workflow-finish-feature`), plus the feature-complete **Review verdict** when invoked from there
- **Other scope** — branch diff, recent commits, or files named by the developer

## Recallium

Read `.agents/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** reconciliation — prior decisions affecting documentation not captured in wiki or Git.
**Search with:** `search_memories` → `expand_memories` as needed.

**Save after:** rare — prefer wiki for durable truth. Save only non-duplicative context that explains reconciliation choices.
**Save with:** `store_memory` — one concise memory; update existing when possible.

Skip save when unsure.

## Boundaries

You may edit **documentation only** — `.wiki/**/*.md`, root project Markdown (`README.md`, `AGENTS.md`, `CONTRIBUTING.md` when warranted), and feature ledgers (including moves to `features/completed/`).

Do not modify implementation, tests, scripts, CI, workspace configuration, agent definitions, or command templates. Do not stage, unstage, commit, push, or rewrite history. The developer runs **`$workflow-checkpoint`** to commit documentation reconciliation.

## Manual fallback

Comparison base or focus:

Use the developer-supplied comparison base or focus when present. Otherwise, use the active feature ledger, feature branch diff since merge base, or the implemented change described.

Before editing, produce a **Documentation Impact Map**.

Then reconcile documentation under the ownership rules in `.wiki/INDEX.md`: correct stale implemented-state claims, preserve planned-versus-implemented distinctions, and remove only documentation made obsolete by the implemented change.

Report factual updates, files changed, documents reviewed but unchanged, and remaining documentation risks.
