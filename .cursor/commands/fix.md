---
description: Address delegated review findings — same project skills as build; then checkpoint again (or continue the loop under /build-loop)
---

Fix **delegated review findings** on the current commit. This command does not advance the delivery plan to the next commit.

## What to fix

${ARGUMENTS:-Address all **CRITICAL**, **HIGH**, and **MEDIUM** findings from the most recent **Review verdict** in this conversation. If no verdict exists, ask the developer what to fix.}

The developer may delegate narrowly (e.g. one HIGH item, or CRITICAL/HIGH only). Fix **only** what is delegated — no drive-by cleanup, refactors, or scope expansion.

**Default delegation** includes **MEDIUM** as well as CRITICAL and HIGH. MEDIUM does not make **Blocking: Yes** in manual `/checkpoint` — the developer may still approve a commit with MEDIUM findings. `/build-loop` auto-fixes MEDIUM before it commits.

If a finding is incorrect, say so and do not "fix" it by over-engineering.

**NITPICK under `/build-loop`:** When the verdict is NITPICK-only, `/build-loop` does not invoke `/fix` — do not fix NITPICK items in that case. When the verdict mixes CRITICAL, HIGH, and/or MEDIUM with NITPICK, include the NITPICK items in the same fix pass. Outside `/build-loop`, NITPICK remains optional unless the developer delegates it.

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
| `/build-loop` | Same as `/build`, then automated checkpoint/fix (manual opt-in only) |
| `/fix` | Correct **delegated review findings** on the current commit |
| `/checkpoint` | Stage, review, present; commit only when developer approves (unless `/build-loop` Phase 3) |

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
3. **Stop** for manual workflow — tell the developer to run **`/checkpoint`** again. Do not stage, commit, or dispatch reviewer fixes automatically.

**When invoked under `/build-loop`:** do **not** stop or wait for the developer. Continue into the next automated `/checkpoint` pass in the same command invocation. `/build-loop` owns the loop; only exit when the loop contract says so (clean verdict, fix cap reached, or unrecoverable blocker).

`/fix` never commits. Only **`/build-loop` Phase 3** auto-commits on loop success. Do not stage, commit, push, amend, rebase, squash, or rewrite history from `/fix` unless the developer explicitly asks outside an active `/build-loop` run.
