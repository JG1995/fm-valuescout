# Codex Workflow

This repository uses Codex for AI-assisted development. The workflow keeps implementation, review, validation, documentation, and Git approval separate.

## Repository surfaces

- `AGENTS.md` contains standing repository rules and project-specific validation commands.
- The installed `workflow-core` skill contains shared planning, routing, review, escalation, and PR-boundary policy.
- Other installed `workflow-*` skills provide explicit workflow commands and load `workflow-core` before continuing.
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
| Build an active PR | `$workflow-build-feature-loop` | Reviewed build loops and local commits through the next stopping boundary |
| Checkpoint | `$workflow-checkpoint` | Exact staging, validation, independent review, and approved local commit |
| Address findings | `$workflow-fix` | Focused remediation followed by another checkpoint |
| Review | `$workflow-review` | Read-only staged or feature review |
| Reconcile docs | `$workflow-docs-review` | Documentation-only reconciliation |
| Finish a feature | `$workflow-finish-feature` | Exact-scope validation, feature review, and documentation reconciliation |

The loop variants are manual opt-ins. Their documented local commit permissions come from `AGENTS.md`; they do not authorize pushes, merges, or history rewrites. For a durable active-PR run, select the ledger's `Build-feature-loop profile` and enter `/goal $workflow-build-feature-loop`. The skill stops at publication or implementation completion and never runs feature close-out.

## Validation and Git

Run `./scripts/dev format` before staging and `./scripts/dev check` before a commit. The stable command surface is documented in `AGENTS.md` and `./scripts/dev`.

Keep commits atomic and stage exact paths or hunks. Follow the approval rules in `AGENTS.md`.

## Specialist agents

Dispatch specialists explicitly:

- `reviewer` performs the default fresh-context read-only commit review at Sol Medium.
- `documentation-steward` changes documentation and feature-ledger state only at Luna Max.

The main session plans established feature work. Delegate planning only when the developer explicitly requests it. Use a generic reviewer when the required profile differs from the named agent's pinned profile. If independent review is required but unavailable, report the missing capability instead of replacing it with self-review.

Live workflow routing uses only Luna Max, Terra xhigh, Terra Max, Sol Medium, Sol High, Sol xhigh, or Sol Max. The active ledger records per-commit implementation and review profiles plus the feature review profile. Completed feature history keeps the profiles that were actually used.

## Optional MCP tools

Context7 provides current library documentation. Use it for library APIs and configuration details instead of guessing.

Repowise provides advisory architecture, symbol relationships, rationale, code health, risk, dead-code, and impacted-test evidence. Register its MCP server globally on the developer workstation and initialize each project locally. Use `$repowise` for selection, freshness, and verification rules, and fall back to exact search and direct source inspection when it is unavailable.

Use other project-specific MCP tools only when the project documents and configures them. Never put machine-specific binary paths or credentials in repository files.
