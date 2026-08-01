# Codex Workflow

This repository uses Codex for AI-assisted development. The workflow keeps implementation, review, validation, and Git approval separate.

## Repository surfaces

- `AGENTS.md` contains durable repository rules and routing.
- `.agents/WORKFLOW.md` contains the canonical planning, model-routing, review, escalation, and replanning policy.
- `.agents/skills/` contains reusable domain and workflow skills.
- `.codex/agents/` contains the `planner`, default `reviewer`, and `documentation-steward` specialist definitions.
- `.codex/config.toml` contains the project MCP servers for Recallium and Context7.
- `.work/` contains disposable notes and experiment artifacts. It is ignored and is not project truth.

Codex loads `AGENTS.md` from the repository root. Start a new task after changing repository guidance so the updated instructions are loaded.

## Core workflow

Use the named skill that matches the task. State the requested outcome in chat; Codex can select the skill from its description, or you can name it explicitly.

| Task | Skill | Result |
| --- | --- | --- |
| Choose a stack | `workflow-stack` | Proposed stack and architecture; wait for approval before writing docs. |
| Order MVP work | `workflow-roadmap` | Dependency-aware sequence in `.wiki/TODO.md` after approval. |
| Plan a feature | `workflow-plan-feature` | Active ledger with packets and separate implementation/review profiles for every commit. |
| Build a commit | `workflow-build` | Assigned implementer follows the active packet through RED → GREEN → REFACTOR. |
| Checkpoint a commit | `workflow-checkpoint` | Exact staging, validation, independent review, and local commit after approval. |
| Address review findings | `workflow-fix` | Focused remediation, then another checkpoint. |
| Review | `workflow-review` | Read-only staged or feature review. |
| Reconcile docs | `workflow-docs-review` | Documentation-only reconciliation. |
| Finish a feature | `workflow-finish-feature` | Full validation, feature review, then documentation reconciliation. |

`workflow-build-loop` is manual opt-in only. It may auto-commit after a clean review because naming the skill is explicit approval for that loop. `workflow-spike` and `workflow-security-audit` are optional, read-only or disposable investigations outside the main loop.

## Validation and Git

Run `./scripts/dev format` before staging and `./scripts/dev check` before a commit. The stable command surface is documented in `AGENTS.md` and `./scripts/dev`.

Keep commits atomic. Stage exact paths or hunks. Do not commit, push, amend, rebase, squash, or rewrite history without the developer's explicit approval, except for the documented build-loop opt-in.

The active ledger selects work and model profiles. Capability Demand selects Luna, Terra, or Sol. Effort Demand selects reasoning effort. Review Demand independently selects the reviewer. Read `.agents/WORKFLOW.md` for scoring, hard floors, evidence requirements, and escalation.

## Specialist agents

Dispatch specialist agents explicitly when the task needs their role:

- `planner` uses Sol High by default and writes feature ledgers without implementing product code.
- `reviewer` is the default Terra xhigh read-only reviewer. Use it only when that profile matches the ledger; otherwise dispatch a generic read-only reviewer with the assigned model and the same contract.
- `documentation-steward` can update documentation and feature ledgers only. Use it after feature-complete review clears or for documentation reconciliation.

Each named definition pins its default model and reasoning effort. `planner` uses `gpt-5.6-sol` with `high`; `reviewer` uses `gpt-5.6-terra` with `xhigh`; `documentation-steward` uses `gpt-5.6-terra` with `medium`. Do not override a pinned role. Use a generic agent when a ledger assigns another profile.

If an agent is unavailable, follow the corresponding workflow skill in the main session and preserve the same boundary.

Implementation and review must use separate contexts. Start review from the original commit contract, packet, diff, and validation results. Do not lead with the implementer's reasoning or self-review.

## MCP

Recallium stores durable project context under the project name `fm-valuescout`. Search it before non-obvious decisions and save only context that is not already recorded in the repository.

Context7 provides current library documentation. Use it for library APIs and configuration details instead of guessing.
