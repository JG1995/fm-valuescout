---
name: workflow-spike
description: Optional — run a disposable experiment when docs and inspection cannot answer one technical yes/no (or how much) question
---

Run a **bounded technical spike** for the exact, testable question supplied by the developer. If no question is supplied, ask for one before starting.

## Purpose

A spike answers a question that **only runtime evidence** can resolve — not a literature review or stack comparison.

| Spike answers | Spike does **not** replace |
| --- | --- |
| “Will this library work in our process / auth / deploy model?” | **`$workflow-stack`** or **`project-strategy`** — compare stacks and record approved direction |
| “Does this API return what we need under our constraints?” | **`$workflow-plan-feature`** — normal delivery planning from wiki and specs |
| “Can we hit latency X with approach Y?” | **`debug`** skill — root-cause on known broken behaviour |
| “Does this migration apply cleanly on our schema?” | Ad-hoc chat + Context7 when docs alone suffice |

**Research** (“should we use Pi vs hand-rolling an LLM handler?”) starts with wiki, skills, WebSearch, and Context7. **Spike** is when you still need a **small try**: e.g. wire Pi into a scratch script and see if it connects to your MCP setup.

## Role in the workflow

**Not part of the core loop.** You do not run `$workflow-spike` every feature.

Core loop: `$workflow-plan-feature` → `$workflow-build` → `$workflow-checkpoint` → … → `$workflow-finish-feature`.

`$workflow-spike` is an **optional side path** when planning or build is blocked until one technical question is answered:

```text
plan-feature or build blocked
  → wiki + skills + Recallium + Context7 (try first)
  → still need to *try* something? → $workflow-spike
  → record conclusion → resume plan-feature or $workflow-build
```

Typical triggers:

- `$workflow-plan-feature` or `$workflow-roadmap` lists a **gating unknown** before committing to order or first commit
- `$workflow-build` escalation: structural decision cannot be resolved without a probe
- You choose to validate an integration **before** locking it into the delivery plan

After a spike, **return to the workflow** — update the ledger, then `$workflow-plan-feature` or `$workflow-build`; do not treat spike code as the implementation.

## Recallium

Read `.agents/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** running the experiment — prior experiments or decisions on this question not in wiki or ledger.
**Search with:** `search_memories` → `expand_memories` as needed.

**Save after:** spike complete — only when the conclusion is durable and not already in the ledger or wiki.
**Save with:** `store_memory` — one concise memory; update existing when possible.

Skip save when unsure.

## Before you spike — try these first

1. Read `.wiki/ARCHITECTURE.md` and relevant wiki specs.
2. Scan `.agents/skills/` for matching skills; read each `SKILL.md`.
3. Search Recallium per **## Recallium** above.
4. Use Context7 MCP (`resolve-library-id`, `query-docs`) for library API facts.
5. Use `WebSearch` / `WebFetch` for public docs and examples.

If the question is still **“we won't know until we run something”**, proceed. If it is **“which option is better?”**, stay in research — present tradeoffs in chat or wiki; spike only the tie-breaker experiment.

## Spike procedure

### 1. Contract the question

State one **testable** question. Bad: “explore LLM options.” Good: “Can we invoke Pi’s agent API from our Node service with our current env vars?”

Record:

- **Question** — single sentence
- **Why inspection failed** — what you already read and why it is insufficient
- **Success evidence** — what output proves yes, no, or a measured bound
- **Out of scope** — what this spike will not decide (product choice, full feature design)

If the question is vague, stop and ask the developer to narrow it.

### 2. Choose experiment surface

| Surface | When |
| --- | --- |
| **`.work/spikes/<date>-<slug>/`** | Default — scratch scripts, notes, throwaway configs (not production tree) |
| **Disposable git branch** | When the experiment must touch app layout but must not land on trunk |
| **Read-only only** | When a single command against existing code answers the question — no new files |

Never spike on `main` with uncommitted production changes you intend to keep mixed with experiment junk.

### 3. Run the smallest experiment

- One variable; smallest code or command that could answer the question.
- No feature polish, no “while we're here” improvements.
- No new dependencies in production manifests unless the spike question is specifically about that dependency — prefer isolated scratch.

### 4. Record results

Capture in chat and (when a feature ledger exists) under **Discoveries and replanning** or **Uncertainty register**:

- Commands run and exact outputs (trim secrets)
- **Verdict:** `supported` | `unsupported` | `conditional` (with conditions) | `still uncertain`
- Limitations — what the spike did not prove
- **Plan impact** — how this changes PR/commit order, `$workflow-roadmap` position, or blocks the next `$workflow-build`

### 5. Dispose

- **Do not** stage, commit, push, or merge spike artifacts by default.
- Delete scratch under `.work/spikes/` when done, or leave a short `NOTES.md` there if the developer wants a paper trail (not wiki truth).
- If spike code touched a disposable branch, delete the branch after recording conclusions.
- If the spike proves an approach works, **reimplement cleanly** in a normal `$workflow-build` commit — do not merge spike code wholesale.

### 6. Resume the workflow

Tell the developer explicitly:

- **Resume with:** `$workflow-plan-feature` (replan), `$workflow-roadmap` (resequence), or `$workflow-build` (active commit)
- **Blocked on human:** if verdict is `still uncertain` or the spike raised a product decision — ask the developer

Do not advance the delivery plan or mark commits complete inside `$workflow-spike`.

## Output format

```md
## Spike: <short slug>

**Question:** …
**Why not docs/inspection:** …
**Success evidence:** …
**Out of scope:** …

### Experiment
<What you ran — surface, commands, key snippets if needed>

### Result
**Verdict:** supported | unsupported | conditional | still uncertain
**Evidence:** …
**Limitations:** …

### Plan impact
<How roadmap, ledger, or next commit should change>

### Resume with
$workflow-plan-feature | $workflow-roadmap | $workflow-build | ask developer
```
