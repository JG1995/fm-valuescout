---
name: workflow-build
description: Implement the active commit with its assigned model profile and implementation packet — RED/GREEN, validation, handoff, then checkpoint
---

## Execution mode

**Default — one commit.** Unless the developer explicitly opts in below, implement only the **active commit** from the active feature ledger (or a trivial work contract if no ledger). Do not start the next commit, do not batch commits, and do not run `$workflow-checkpoint` inside this command unless the developer asks.

**Opt-in — full feature.** Only when the developer clearly requests it in chat (e.g. "work through the whole feature", "all commits in the plan"). Otherwise, use the default one-commit mode.

In full-feature mode, still implement **one atomic commit at a time** in plan order. After each commit: run the review checkpoint cycle (`$workflow-checkpoint` or equivalent), wait for approval, update the ledger, then continue to the next planned commit until the feature delivery plan is complete or the developer stops you.

Full-feature mode does **not** skip review, squash commits, or merge PRs without explicit approval.

## Mandatory reads

Read `AGENTS.md`, `.agents/WORKFLOW.md`, `.wiki/INDEX.md`, the active feature ledger when one exists, and relevant implementation and tests.

For a planned commit, read its implementation packet, execution metadata, validation contract, escalation conditions, and relevant feature invariants before inspecting implementation. Verify each named repository pattern before copying it.

**Coding standards:** Read `.agents/skills/coding-standards/SKILL.md`, `references/universal.md`, and `references/testing.md` when this commit adds or changes tests. Load matching files from `coding-standards/references/` when `ARCHITECTURE.md` or the touched stack applies. Scan `.agents/skills/` for other matching skills (UI patterns, domain conventions) and read each matching `SKILL.md`.

**Minimalism:** Run the decision ladder in `.agents/skills/minimalism/SKILL.md` before adding dependencies, files, or abstractions.

## Recallium

Read `.agents/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** editing — unfamiliar conventions, earlier decisions, or non-obvious constraints for this commit; escalation questions not answered by wiki or skills.
**Search with:** `search_memories` → `expand_memories` as needed; `recallium` / `session_recap` when resuming this commit across sessions.

**Save after:** implementation — only when the commit surfaced durable progress or constraints not already in the ledger.
**Save with:** `store_memory` — one concise memory; update existing when possible.

Skip save when unsure.

## Before editing

1. Restate the active commit's **work** description, governing invariants, implementation packet, assigned profile, and explicit exclusions from the ledger.
2. Produce an Impact Map: changed behaviour, likely implementation location, existing protection, missing test, and expected failure mechanism.
3. When the commit needs tests (see `coding-standards/references/testing.md`), run the **test quality gate** on the planned RED test before writing it: what failure it prevents, whether wrong implementation would fail, and whether it exercises real behaviour.
4. Identify the smallest meaningful failing test that passes the gate — or document why tests are skipped.
5. Identify required contract, integration, migration, smoke, and mutation/perturbation checks.
6. Apply the decision ladder from `.agents/skills/minimalism/SKILL.md` and coding standards from `.agents/skills/coding-standards/`. State which ladder rung you stopped at and why.
7. Treat the main session as the assigned implementation context. Assume its model and effort match the ledger because the developer selects them before invoking the workflow. Do not inspect or infer the runtime profile, and do not dispatch implementation work for model routing. If the developer explicitly says the profile is wrong, stop before editing and report the profile required by the ledger.
8. Escalate before implementation when persistence, schema, migration, authentication, concurrency, security, or a public API decision is unresolved. Read `.wiki/ARCHITECTURE.md`, scan `.agents/skills/` for matching skills, search Recallium per **## Recallium**. If the question needs a **runtime probe**, use optional **`$workflow-spike`**; if still blocked, **ask the developer** — do not implement on assumptions.

## Follow the plan — or say why not

The delivery plan is the default contract. Implement what the active commit describes.

If repository evidence shows the plan is **wrong, incomplete, or not feasible** as written:

- You may implement what is **necessary** to deliver a correct, trunk-safe outcome instead.
- **Before finishing**, tell the developer plainly:
  - what in the plan could not be followed;
  - what you did instead and why;
  - how the remaining plan should change.
- **Update the active ledger** in the same work session (before `$workflow-checkpoint`): record the deviation under **Discoveries and replanning**, adjust affected commits or PRs, and note the rationale. The checkpoint commit should include that ledger update when the deviation affects delivery shape.

Do not silently drift from the plan. Do not pretend the original plan still applies when it does not.

Apply the active packet's stop conditions. Stop and replan instead of inventing a new architecture when a Known fact is false, a required seam is absent, an invariant or exclusion cannot hold, a public or persisted contract changes unexpectedly, meaningful validation cannot be built, later-PR work is required, or a cross-feature dependency appears.

## Implementation

Use RED → GREEN → REFACTOR when `references/testing.md` says test-first applies:

- **RED:** write and run the smallest test that passes the **test quality gate**. Confirm failure is missing or wrong behaviour, not setup or syntax.
- **GREEN:** make the smallest coherent implementation and run the targeted test. Match coding standards and architectural boundaries from wiki and skills.
- **REFACTOR:** improve structure only while green, then rerun targeted tests.

Then run `./scripts/dev format` on files you touched (or the full project), affected existing tests, and `./scripts/dev check`. Run `./scripts/dev smoke` when browser flows change. `./scripts/dev mutate` exits 69 until mutation tooling is wired — report as unsupported, never as passed.

Use Context7 MCP (`resolve-library-id`, then `query-docs`) for current library documentation rather than guessing from training data.

## Documentation during build

- **Per commit:** update only documentation **intrinsic** to the change (e.g. a new command surface, config key, or contract the commit introduces). Do not run broad wiki reconciliation here.
- **Per feature:** durable documentation reconciliation happens at feature completion via `$workflow-finish-feature` and `$workflow-docs-review`, not after every commit.

## After implementation (default one-commit mode)

1. Report RED/GREEN evidence, gate results, and any discoveries affecting the remaining plan.
2. Provide the implementation handoff required by `.agents/WORKFLOW.md`: files changed, behavior, tests, commands, unresolved uncertainty, packet deviations, and escalation or replanning status.
3. Update the ledger: confirm active commit outcome matches plan or record deviation (see above). Do not mark the commit completed until `$workflow-checkpoint` commits — use `Completed — hash pending checkpoint commit` only when staging for checkpoint.
4. **Stop.** Tell the developer to run **`$workflow-checkpoint`** for review and the local commit. Review after each commit is the default — do not commit without that pass unless the developer explicitly overrides.

Do not stage, commit, push, amend, rebase, squash, or rewrite history in `$workflow-build` unless the developer explicitly asks you to checkpoint in this turn.
