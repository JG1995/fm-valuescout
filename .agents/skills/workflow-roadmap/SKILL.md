---
name: workflow-roadmap
description: Order MVP features for development — dependency-aware sequence from planned specs or CONCEPT bullets; write TODO only after approval
---

## Mandatory reads

1. Read `AGENTS.md` and `.wiki/INDEX.md`.
2. Read `.wiki/CONCEPT.md` — MVP scope, principles, success narrative, and boundaries.
3. Read `.wiki/ARCHITECTURE.md` §1.1 when filled — target stack constrains sequencing (e.g. offline-first, auth model, monolith vs services).
4. Read every spec in `.wiki/features/planned/*.md` when present.
5. Inspect repository evidence read-only when implementation exists (manifests, active ledgers, implemented modules).
6. Scan `.agents/skills/` for skills whose description matches sequencing or architecture concerns; read each matching `SKILL.md`.

## Input modes — always produce a roadmap

`$workflow-roadmap` must **always** output a **dependency-aware** sequence — foundations before dependents, refactor risks explicit. Never stop because specs are incomplete. Without planned specs, dependencies are inferred from CONCEPT, principles, and §1.1 stack; confidence is lower but ordering logic is the same.

| Input available | How to treat each MVP feature |
| --- | --- |
| **Planned spec** (`features/planned/<slug>.md`) | Use spec detail for dependencies and refactor risk. Confidence **high** unless spec has open questions. |
| **CONCEPT bullet only** (no planned spec) | Infer dependencies from the bullet, product principles, success narrative, and §1.1 stack. State assumptions explicitly. Confidence **medium** or **low**. |
| **Vague bullet** (name only, no behaviour) | Place using generic engineering heuristics (foundation before features, auth before permissions, data model before reports, etc.). Confidence **low**. |

Rules for bullet-only features:

- **Do not invent** detailed user stories, screens, or acceptance criteria — infer only what the bullet and CONCEPT context reasonably imply.
- **Do** still assign an order, dependency edges, and a best-guess refactor risk — label it as inferred, not evidenced.
- **Do** say what spec detail would increase confidence and might change the order.

When every MVP item is a CONCEPT bullet, the whole roadmap is a **provisional best guess** — say that plainly in TL;DR and overall confidence.

## Request overrides

Apply developer-supplied constraints. If none are supplied, base the sequence on wiki specs and CONCEPT.md.

## Role boundary

| Command | Scope |
| --- | --- |
| `$workflow-roadmap` | **Cross-feature** delivery order — what to build before what, and refactor risks if order is wrong |
| `$workflow-plan-feature` | **One feature** — PRs, commits, delivery plan in active ledger |
| `$workflow-stack` | **Stack** — languages, frameworks, hosting |

Do not create `features/active/` ledgers or mark commits `Active`.

When cross-feature dependencies are unclear, read `.wiki/ARCHITECTURE.md`, scan `.agents/skills/` for matching skills, inspect read-only evidence across features, and search Recallium per **## Recallium** below. If sequencing still cannot be justified, **ask the developer** — do not guess order.

## Recallium

Read `.agents/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** sequencing analysis — prior sequencing or dependency decisions not in TODO, specs, or wiki.
**Search with:** `search_memories` → `expand_memories` as needed; `recallium` / `session_recap` when resuming roadmap work across sessions.

**Save after:** explicit approval only — when sequencing rationale is not already clear from TODO and planned specs.
**Save with:** `store_memory` — one concise memory; update existing when possible.

Skip save when unsure.

## External context

Use Context7 MCP when stack-specific ordering constraints need current framework facts.

## Analysis focus (refactor avoidance)

For each MVP feature, determine:

- **Foundations it needs** — data model, auth, routing, shared UI shell, API contracts, jobs, etc.
- **What it forces into the architecture** — schemas, permissions, state patterns, extension points
- **Refactor risk if built too late** — concrete "if you build X after Y, you will likely rewrite Z"
- **Refactor risk if built too early** — premature abstraction, wrong seam, unused generality

Identify:

1. **MVP spine** — thinnest end-to-end path across features (not within one feature).
2. **Dependency graph** — hard dependencies vs soft preferences.
3. **Recommended build order** — numbered, with one-line rationale per position.
4. **Parallel tracks** — what can run concurrently after a given milestone.
5. **Gating unknowns** — questions that need optional **`$workflow-spike`** (runtime experiment) or developer input before committing to order for a feature.

## Chat output format

Present in chat before any wiki writes:

```md
## MVP development roadmap

### TL;DR
<Recommended order in one sentence. First feature to `$workflow-plan-feature`. State overall confidence: high | medium | low — and whether the plan is mostly evidenced or mostly inferred.>

### Overall confidence
<One short paragraph: what was well-specified vs bullet-only; how much reordering to expect after speccing.>

### Dependency graph
<ASCII or Mermaid — every MVP feature as a node. Mark inferred edges with `(inferred)` when not supported by a spec.>

### Recommended build order

| Order | Feature | Source | Confidence | Why this position | If built later, likely refactor |
| --- | --- | --- | --- | --- | --- |
| 1 | … | spec \| CONCEPT bullet | high \| medium \| low | … | … |

**Source:** `spec` = planned file exists; `CONCEPT bullet` = MVP list only.

### MVP spine
<Cross-feature walking skeleton. Note inferred parts.>

### Parallel tracks
<Optional — what can branch after order N.>

### Spec gaps (would sharpen this plan)
<Features still bullet-only — what to add to `features/planned/` before treating order as firm. Not a blocker; sequencing already given above.>

### Gating unknowns
<Questions that need optional `$workflow-spike` or developer input before this order is safe.>

### Plan next
Run `$workflow-plan-feature` on <feature> because <reason>. Note if planning should start from bullet only or from a planned spec.
```

Record **chosen order and rationale summary** in wiki after approval. Do not copy rejected alternative sequences or lengthy rejection prose into wiki files.

## Read-only until approved

While producing the roadmap:

- Do not edit production code, tests, scripts, CI, or `.codex/` configuration.
- Do not stage, commit, or push.
- Do not write wiki files until the developer explicitly approves the sequence.

End with:

1. **Unresolved questions** that would reorder the plan.
2. **Proposed wiki update** — fill **Development sequence (approved proposal)** in `.wiki/TODO.md`.
3. **Explicit stop** — wait for approval before any file writes.

## After explicit approval only

When the developer clearly approves the sequence:

1. Fill **Development sequence (approved proposal)** in `.wiki/TODO.md` — ordered table (with source/confidence if bullet-only items exist), dependency graph, MVP spine, parallel notes, and **Plan next** only. Chosen order only — no rejected alternatives. If the plan was mostly inferred, note **provisional — revisit after speccing** in **Plan next** or spine line.
2. Do not move features to **Active** or create active ledgers — that happens when the developer runs `$workflow-plan-feature` on the first feature.

Do not start `$workflow-build` unless the developer asks.
