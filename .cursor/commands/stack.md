---
description: Turn product notes into a recommended stack and target architecture — read wiki first, write only after approval
---

## Mandatory reads

1. Read `.cursor/skills/project-strategy/SKILL.md` and follow its workflow, hard gates, and output contracts.
2. Read `AGENTS.md` and `.wiki/INDEX.md`.
3. Read `.wiki/CONCEPT.md` as the primary product input. Read `.wiki/DESIGN.md` when UI platform or design-system choices affect the stack.
4. Inspect repository evidence read-only when present (manifests, `scripts/dev`, CI, existing `.wiki/ARCHITECTURE.md`). An unfilled template with no app stack is normal — do not pretend a stack is already chosen.

## Request overrides

Apply these after wiki reads. They refine or override product notes when the developer states new constraints:

${ARGUMENTS:-No extra constraints supplied. Base the recommendation on CONCEPT.md and inspected evidence.}

## Operating mode

Default to **pre-build strategy** when CONCEPT.md is filled but no real application stack is implemented.

Switch to **mid-build review** only when manifests, application entry points, or implemented architecture exist. Use the matching output contract from the skill.

Do not dispatch a planning subagent. `/stack` chooses stack and target architecture from product notes; `/plan-feature` plans delivery inside that agreed direction.

## Recallium

Read `.cursor/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** recommending — prior stack or architecture decisions not in wiki or ADRs.
**Search with:** `search_memories` → `expand_memories` as needed; `recallium` / `session_recap` when resuming stack work across sessions.

**Save after:** explicit approval only — when approved stack choices are not already clear from wiki and ADRs.
**Save with:** `store_memory` — one concise memory; update existing when possible.

Skip save when unsure.

## External research

Use Context7 MCP (`resolve-library-id`, `query-docs`) for current library and framework facts.

Use `WebSearch` and `WebFetch` for comparable projects and maintenance signals per the skill's external research rules.

## Stack decision format (chat only)

For **each major stack decision**, present a structured comparison in chat before the summary recommendation. Major decisions include layers such as client/UI platform, backend/runtime, data store, auth, hosting/deployment, testing approach, and other choices that would be costly to reverse.

For every such decision, use this structure:

```md
### <Decision area> (e.g. Frontend framework)

**Recommendation:** <chosen option>

| Option | Pros | Cons |
| --- | --- | --- |
| **<Option A — recommended>** | … | … |
| **<Option B>** | … | … |
| **<Option C>** | … | … |

**Why this over the others:** <short rationale tying the choice to CONCEPT.md constraints, not generic popularity.>
```

Rules:

- Always **three options** and **one recommendation** (mark the recommended row).
- Options must be plausible for this project — not strawmen.
- Pros and cons must be specific to this product, constraints, and evidence.
- Explain why the two non-recommended options were not chosen, not only why the pick wins.

This three-option comparison is **for chat only**. Do not copy rejected options or rejection rationale into wiki files.

## Read-only until approved

While producing the recommendation:

- Do not edit production code, tests, scripts, CI, or `.cursor/` configuration.
- Do not stage, commit, or push.
- Do not write wiki files, ADRs, or scratch notes until the developer explicitly approves the recommendation.

Present the skill output contract in chat (**Pre-Build Strategy** or **Mid-Build Review**), with major stack decisions using the **Stack decision format** above inside or alongside **Recommended Stack** / **Stack and Architecture Verdict**. End with:

1. **Unresolved questions** that would change the stack if answered differently.
2. **Proposed wiki updates** — fill §1.1 in `.wiki/ARCHITECTURE.md` and whether a `.wiki/decisions/` ADR is warranted.
3. **Explicit stop** — wait for approval before any file writes.

## After explicit approval only

When the developer clearly approves the recommendation:

1. Fill **§1.1 Target architecture (approved proposal)** in `.wiki/ARCHITECTURE.md` — chosen stack and direction only. Do not list rejected alternatives. Do not edit other sections to describe proposals as implemented state.
2. Create ADRs in `.wiki/decisions/` only for consequential stack choices. Each ADR documents the **accepted decision and its consequences** — not a comparison table of rejected options. Status **Accepted** or **Proposed** per skill confidence.
3. Note gaps in `.wiki/CONCEPT.md` only when approval included filling missing product facts — keep edits minimal.

Do not start `/build` or feature implementation unless the developer asks.
