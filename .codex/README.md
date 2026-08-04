# Codex Workflow

This repository uses Codex for AI-assisted development. The workflow keeps implementation, review, validation, documentation, and Git approval separate.

## Repository surfaces

- `AGENTS.md` contains standing repository rules and project-specific validation commands.
- `.codex/WORKFLOW.md` contains planning, routing, review, escalation, and PR-boundary policy.
- Installed global skills provide reusable guidance and explicit workflows.
- `.codex/agents/` contains the `reviewer` and `documentation-steward` specialist definitions.
- `.codex/config.toml` contains project MCP and shell-environment configuration.
- `.wiki/` contains project-owned current state, feature records, ADRs, and reusable debug knowledge.
- `.work/` contains ignored disposable evidence and experiments.

Start a new Codex task after changing repository guidance so the new instructions load.

## Explicit workflows

Workflow skills never activate from task similarity. Select one through `/skills` or mention `$workflow-<name>`.

| Task | Skill | Result |
| --- | --- | --- |
| Choose a stack | `$workflow-stack` | Proposed stack and target architecture |
| Order work | `$workflow-roadmap` | Dependency-aware project sequence |
| Plan a feature | `$workflow-plan-feature` | Active ledger with PRs, commit packets, profiles, and validation |
| Build one commit | `$workflow-build` | Active commit implemented through RED, GREEN, and REFACTOR |
| Checkpoint | `$workflow-checkpoint` | Exact staging, validation, independent review, and approved local commit |
| Address findings | `$workflow-fix` | Focused remediation followed by another checkpoint |
| Review | `$workflow-review` | Read-only staged or feature review |
| Reconcile docs | `$workflow-docs-review` | Documentation-only reconciliation |
| Finish a feature | `$workflow-finish-feature` | Exact-scope validation, feature review, and documentation reconciliation |

The loop variants are manual opt-ins. Their documented local commit permissions come from `AGENTS.md`; they do not authorize pushes, merges, or history rewrites.

## Validation and Git

Run `./scripts/dev format` before staging and `./scripts/dev check` before a commit. The stable command surface is documented in `AGENTS.md` and `./scripts/dev`.

Keep commits atomic and stage exact paths or hunks. Follow the approval rules in `AGENTS.md`.

## Specialist agents

Dispatch specialists explicitly:

- `reviewer` performs the default fresh-context read-only commit review.
- `documentation-steward` changes documentation and feature-ledger state only.

The main session plans established feature work. Delegate planning only when the developer explicitly requests it. Use a generic reviewer when the required profile differs from the named agent's pinned profile. If independent review is required but unavailable, report the missing capability instead of replacing it with self-review.

## Optional MCP tools

Context7 provides current library documentation. Use it for library APIs and configuration details instead of guessing.

Codebase Memory provides advisory architecture, call-path, semantic-search, data-flow, and change-impact queries. Register `codebase-memory-mcp` globally on the developer workstation. Use `$codebase-memory` for selection and verification rules, and fall back to exact search and direct source inspection when it is unavailable.

Use other project-specific MCP tools only when the project documents and configures them. Never put machine-specific binary paths or credentials in repository files.
