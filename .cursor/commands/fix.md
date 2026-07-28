---
description: Address delegated review findings — same project skills and architecture rules as build; then checkpoint again
---

Fix **delegated review findings** on the current commit. This command does not advance the delivery plan to the next commit.

## What to fix

${ARGUMENTS:-Address all **CRITICAL** and **HIGH** findings from the most recent **Review verdict** in this conversation. If no verdict exists, ask the developer what to fix.}

The developer may delegate narrowly (e.g. one HIGH item, or specific MEDIUM findings). Fix **only** what is delegated — no drive-by cleanup, refactors, or scope expansion.

If a finding is incorrect or a NITPICK, say so and do not "fix" it by over-engineering.

## Mandatory reads

1. Read `AGENTS.md` and `.wiki/INDEX.md`.
2. Read the **Review verdict** being addressed (from chat or arguments).
3. Read the **active feature ledger** when one exists — active commit, out of scope, delivery plan context.
4. Read `.wiki/ARCHITECTURE.md` when findings touch structure or layers.

**Coding standards:** Read `.cursor/skills/coding-standards/SKILL.md`, `references/universal.md`, and `references/testing.md` when fixing test-related findings. Load matching `coding-standards/references/` files when the stack or diff applies. Scan `.cursor/skills/` for other matching skills — same requirement as `/build`.

## Recallium

Read `.cursor/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** fixing — prior decisions or conventions behind delegated findings not in wiki, ledger, or verdict.
**Search with:** `search_memories` → `expand_memories` as needed.

**Save after:** fixes complete — only when remediation surfaced durable context not already in the ledger or verdict.
**Save with:** `store_memory` — one concise memory; update existing when possible.

Skip save when unsure.

## Role boundary

| Command | Scope |
| --- | --- |
| `/build` | Implement the **active commit** from the plan (new work) |
| `/fix` | Correct **delegated review findings** on the current commit |
| `/checkpoint` | Stage, review, present verdict, commit after approval |

Do not mark the active commit completed, activate the next commit, or implement planned work not required by the delegated findings.

## Before editing

1. List each delegated finding you will address (by verdict tier and title).
2. For each: state the intended correction and how you will verify it.
3. Apply the Decision Ladder from `AGENTS.md` and matching skills. Do not add abstractions beyond what the finding requires.

## Implementation

- Fix the **functional or architectural issue** described in each finding. Match project skills and wiki architecture.
- **Tests:** add or adjust tests only when a finding requires proof (missing behavioural test, wrong assertion, mock hiding behaviour). Use RED → GREEN when a new test is needed; otherwise run affected tests after the fix.
- Run `./scripts/dev format` when findings are Biome format or import-order only; otherwise run affected tests and `./scripts/dev check`. Smoke/mutate: report unsupported (status 69), never as passed.

Use Context7 MCP for library facts when a finding involves external APIs.

## After fixing

1. Report what changed per finding — map each delegated item to the correction.
2. Note any finding you did not fix and why (not delegated, invalid, or needs product decision).
3. **Stop.** Tell the developer to run **`/checkpoint`** again. Do not stage, commit, or dispatch reviewer fixes automatically.

Do not stage, commit, push, amend, rebase, squash, or rewrite history unless the developer explicitly asks.
