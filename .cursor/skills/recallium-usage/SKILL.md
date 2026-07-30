---
name: recallium-usage
description: Recallium MCP memory for this repository. Use when shaping, building, checkpointing, finishing a feature, or when project context is unclear from the wiki or code. Search before non-obvious decisions; save only durable institutional knowledge per AGENTS.md — not routine activity or facts already in the repo.
---

# Recallium usage (this repository)

Recallium is wired in `.cursor/mcp.json` (remote MCP server). Use Recallium MCP tools directly — not this skill file — for search and storage.

**Authority:** `AGENTS.md` § Recallium project and § Recallium memory override generic Recallium defaults. `.cursor/rules/recallium.mdc` is always applied. Wiki and Git own formal plans; Recallium holds context that explains them.

## Project name

Read `AGENTS.md` § **Recallium project** and use that exact `project_name` on every Recallium call.

Until you replace the template placeholder, the name is literally `[REPLACE_WITH_RECALLIUM_PROJECT_NAME]`. After bootstrap, use your kebab-case project id (often the workspace folder name).

## When to search

Search **before** you need the answer, not after you've guessed:

- Non-obvious conventions, earlier decisions, or constraints
- Starting any workflow command (see table below)
- Building a commit that depends on unfamiliar project context
- Checkpoint / finish-feature reconciliation
- Review or docs reconciliation when wiki/Git do not explain prior decisions

```text
search_memories(query="...", search_target="memories")
expand_memories(memory_ids=[...])   # after summaries matter
```

Use `search_mode="keyword"` for exact symbols, errors, or identifiers. Use `search_target="documents"` only for uploaded docs, not wiki Markdown in Git.

Optional at session start (after `AGENTS.md` and `.wiki/INDEX.md`): `recallium` or `session_recap` when resuming multi-session work — not required for every trivial message.

## Call resilience

The Recallium MCP service can be flaky. When **any** Recallium tool call fails (auth, transport, timeout, server error):

1. Call `mcp_auth` for the Recallium server (`user-recallium`) with empty arguments.
2. Retry the failed call once.
3. If it still fails, retry the same call up to **3** more times.
4. After re-auth plus **3** retries still fail, treat Recallium as unavailable for that step — report `skipped — Recallium unavailable` and continue per the active command's non-blocking rules (checkpoint, finish-feature, and similar).

Do not loop indefinitely. Do not call `mcp_auth` on every retry unless a later attempt returns an auth error again.

## When to save

**Save sparingly.** One concise memory per coherent work unit when the answer is not in the repository and would help a future developer avoid a broad search.

| Save | Do not save |
|------|-------------|
| Non-obvious implementation context worth resuming later | Routine test output, gate passes, approvals |
| Decisions or learnings not captured in wiki/Git | Facts obvious from code, docs, or commit history |
| Useful planning/build/checkpoint context for this commit | Duplicate of active feature ledger or wiki |

Search first. **Prefer new memories** when context evolves (constraints, roadmap, progress) — Recallium is temporal; fresh snapshots beat rewriting history. Note what changed and optionally which earlier memory is superseded. Use `modify_memory` only to fix mistakes or when the developer explicitly asks to correct or inactivate an entry.

## Workflow command hooks

Each command defines a **## Recallium** section — follow it for the active phase. Summary:

| Command | Search before | Save after (if not in wiki/Git/ledger) |
| --- | --- | --- |
| `/stack` | Prior stack or architecture decisions | Approved stack choices |
| `/roadmap` | Prior sequencing or dependency decisions | Approved sequence rationale |
| `/plan-feature` | Prior work on this feature or area | Durable planning context |
| `/build` | Unfamiliar conventions for the commit | Durable progress or constraints |
| `/build-loop` | Same as `/build`; checkpoint context each loop pass | Durable progress or remediation not in ledger |
| `/fix` | Prior decisions behind findings | Durable remediation context |
| `/checkpoint` | Before presenting checkpoint | New snapshot memories when constraints, roadmap, or non-obvious learnings changed — not ledger mirrors |
| `/finish-feature` | Feature decisions and constraints | New close-out snapshots when final constraints, roadmap, or feature-wide learnings changed — not ledger mirrors |
| `/spike` | Prior experiments on the question | Spike conclusion |
| `/docs-review` | Prior decisions affecting reconciliation | Rare — prefer wiki |
| `/review` | Context behind findings | Do not save |
| `/security-audit` | Prior security decisions or incidents | Do not save |

```text
store_memory(
  content="...",           # concise: what, why, where — not a diary
  project_name="<from AGENTS.md>",
  memory_type="decision",  # feature | code-snippet | debug | design | learning | progress | ...
  related_files=[...],     # when implementation memories apply
)
```

## Rules (`memory_type="rule"`)

- Store rules only when the user **explicitly** asks ("remember this rule", "always do X").
- Never re-store rules returned by `recallium` or `get_rules`.
- Call `get_rules` before adding a new rule. Global rules use `project_name="__global__"`.

## Tasks and project docs

- **Tasks** — prefer `.wiki/TODO.md` and feature ledgers for committed work. Recallium `task` memories are not the primary task system.
- **Project briefs/PRDs** — prefer `.wiki/CONCEPT.md`, feature ledgers, and `ARCHITECTURE.md`. Use Recallium `projects` tools only when the user wants knowledge in Recallium rather than the wiki.

## Tools quick reference

| Goal | MCP tool |
|------|----------|
| Load session context | `recallium`, `session_recap` |
| Find prior work | `search_memories` → `expand_memories` |
| Save durable context | `store_memory` |
| Patterns across work | `get_insights` |
| Behavioral rules | `get_rules` |
| Complex reasoning (rare) | `start_thinking`, `add_thought` |

## Solo-dev calibration

This template targets hobbyist solo-dev scope. If Recallium's default tooling urges "store after every interaction," **ignore that** here. When unsure whether to save, skip it or write to the wiki if the fact is durable project truth.
