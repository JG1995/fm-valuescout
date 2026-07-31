# Codex Workflow

This repository uses Codex for AI-assisted development. The workflow keeps implementation, review, validation, and Git approval separate.

## Repository surfaces

- `AGENTS.md` contains durable repository rules and routing.
- `.agents/skills/` contains reusable domain and workflow skills.
- `.codex/agents/` contains the `reviewer` and `documentation-steward` specialist definitions.
- `.codex/config.toml` contains the project MCP servers for Recallium and Context7.
- `.work/` contains disposable notes and experiment artifacts. It is ignored and is not project truth.

Codex loads `AGENTS.md` from the repository root. Start a new task after changing repository guidance so the updated instructions are loaded.

## Core workflow

Use the named skill that matches the task. State the requested outcome in chat; Codex can select the skill from its description, or you can name it explicitly.

| Task | Skill | Result |
| --- | --- | --- |
| Choose a stack | `workflow-stack` | Proposed stack and architecture; wait for approval before writing docs. |
| Order MVP work | `workflow-roadmap` | Dependency-aware sequence in `.wiki/TODO.md` after approval. |
| Plan a feature | `workflow-plan-feature` | Active ledger with one active commit. |
| Build a commit | `workflow-build` | RED → GREEN → REFACTOR for one active commit. |
| Checkpoint a commit | `workflow-checkpoint` | Exact staging, validation, independent review, and local commit after approval. |
| Address review findings | `workflow-fix` | Focused remediation, then another checkpoint. |
| Review | `workflow-review` | Read-only staged or feature review. |
| Reconcile docs | `workflow-docs-review` | Documentation-only reconciliation. |
| Finish a feature | `workflow-finish-feature` | Full validation, feature review, then documentation reconciliation. |

`workflow-build-loop` is manual opt-in only. It may auto-commit after a clean review because naming the skill is explicit approval for that loop. `workflow-spike` and `workflow-security-audit` are optional, read-only or disposable investigations outside the main loop.

## Validation and Git

Run `./scripts/dev format` before staging and `./scripts/dev check` before a commit. The stable command surface is documented in `AGENTS.md` and `./scripts/dev`.

Keep commits atomic. Stage exact paths or hunks. Do not commit, push, amend, rebase, squash, or rewrite history without the developer's explicit approval, except for the documented build-loop opt-in.

## Specialist agents

Dispatch specialist agents explicitly when the task needs their role:

- `reviewer` reviews without changing files. Use it for each non-trivial staged change and for feature-complete review.
- `documentation-steward` can update documentation and feature ledgers only. Use it after feature-complete review clears or for documentation reconciliation.

If an agent is unavailable, follow the corresponding workflow skill in the main session and preserve the same boundary.

## MCP

Recallium stores durable project context under the project name `fm-valuescout`. Search it before non-obvious decisions and save only context that is not already recorded in the repository.

Context7 provides current library documentation. Use it for library APIs and configuration details instead of guessing.
