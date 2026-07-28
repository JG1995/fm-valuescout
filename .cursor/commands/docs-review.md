---
description: Reconcile durable documentation with implemented repository state
---

Reconcile durable documentation with implemented state for:

${ARGUMENTS:-The active feature when finishing via `/finish-feature`, or the implemented change described in arguments.}

## Role in the workflow

| When | Who invokes | Preconditions |
| --- | --- | --- |
| **Feature completion** | `/finish-feature` (step 5) | Implementation-complete; tests and gate run; **feature-complete review** clear (Blocking: No, or developer explicitly approved with findings recorded) |
| **Architecture milestone** | Developer directly | Significant implemented structure change — e.g. first app landing after `/stack`, layer refactor — without a full `/finish-feature` pass |
| **Not here** | `/build`, `/checkpoint` | Intrinsic docs only per commit — see `.cursor/commands/build.md` |

Broad wiki reconciliation belongs here or in `/finish-feature`, not after every commit.

## Documentation Steward dispatch

When the Task tool is available, dispatch the `documentation-steward` agent in the foreground:

    Task({
      subagent_type: "documentation-steward",
      prompt: "Reconcile documentation following .cursor/agents/documentation-steward.md and AGENTS.md. Search Recallium per .cursor/rules/recallium.mdc when prior decisions affect reconciliation and are not in wiki or Git. Context: ${ARGUMENTS:-feature completion for the active ledger}. Use feature branch diff or implementation evidence — not staged-only unless the scope is a single pending commit. When feature completion: condense and move the active ledger to features/completed/; update ARCHITECTURE.md, TODO.md, and INDEX-owned docs per ownership rules.",
      description: "Reconcile documentation",
    })

Do not run it in the background or edit the worktree concurrently. After it returns, inspect the complete documentation diff and run `./scripts/dev check`.

## Mandatory reads (manual fallback or parent command)

Read `AGENTS.md`, `.wiki/INDEX.md`, the **technical-writing** skill (`.cursor/skills/technical-writing/SKILL.md`), the active feature ledger when one exists, the linked **planned spec** when present, relevant `CONCEPT.md` bullets, and implementation evidence:

- **Feature completion** — `git diff <base>...HEAD` for the feature (same base as `/finish-feature`), plus the feature-complete **Review verdict** when invoked from there
- **Other scope** — branch diff, recent commits, or files named in `${ARGUMENTS}`

## Recallium

Read `.cursor/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** reconciliation — prior decisions affecting documentation not captured in wiki or Git.
**Search with:** `search_memories` → `expand_memories` as needed.

**Save after:** rare — prefer wiki for durable truth. Save only non-duplicative context that explains reconciliation choices.
**Save with:** `store_memory` — one concise memory; update existing when possible.

Skip save when unsure.

## Boundaries

You may edit **documentation only** — `.wiki/**/*.md`, root project Markdown (`README.md`, `AGENTS.md`, `CONTRIBUTING.md` when warranted), and feature ledgers (including moves to `features/completed/`).

Do not modify implementation, tests, scripts, CI, Cursor configuration, agent definitions, or command templates. Do not stage, unstage, commit, push, or rewrite history. The developer runs **`/checkpoint`** to commit documentation reconciliation.

## Manual fallback

Comparison base or focus:

${ARGUMENTS:-Use the active feature ledger, feature branch diff since merge base, or the implemented change described.}

Before editing, produce a **Documentation Impact Map** (see `.cursor/agents/documentation-steward.md`).

Then follow the documentation-steward agent instructions in full.

Report factual updates, files changed, documents reviewed but unchanged, and remaining documentation risks.
