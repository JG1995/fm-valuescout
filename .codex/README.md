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
| Build a commit | `workflow-build` | Main session follows the active packet through RED → GREEN → REFACTOR. |
| Checkpoint a commit | `workflow-checkpoint` | Exact staging, validation, independent review, and local commit after approval. |
| Address review findings | `workflow-fix` | Focused remediation, then another checkpoint. |
| Review | `workflow-review` | Read-only staged or feature review. |
| Reconcile docs | `workflow-docs-review` | Documentation-only reconciliation. |
| Finish a feature | `workflow-finish-feature` | Full validation, feature review, then documentation reconciliation. |
| Finish a feature automatically | `workflow-finish-feature-loop` | Manual opt-in: Sol High feature review/fix loop, reconciliation, and local close-out commits. |
| Polish a live Tauri UI | `workflow-ui-polish` | Manual opt-in: inspect an isolated running app, make cohesive UI improvements, and present visual evidence. |

`workflow-build-loop` and `workflow-finish-feature-loop` are manual opt-ins only. They may auto-commit after their blocking review tiers clear because naming either skill is explicit approval for its documented local commits. Each loop allows at most three fix rounds. `workflow-spike` and `workflow-security-audit` are optional, read-only or disposable investigations outside the main loop.

## Validation and Git

Run `./scripts/dev format` before staging and `./scripts/dev check` before a commit. The stable command surface is documented in `AGENTS.md` and `./scripts/dev`.

Keep commits atomic. Stage exact paths or hunks. Do not commit, push, amend, rebase, squash, or rewrite history without the developer's explicit approval, except for the documented build-loop opt-in.

The active ledger selects work and model profiles. Capability Demand selects Luna, Terra, or Sol. Effort Demand selects reasoning effort. Review Demand independently selects the reviewer. Read `.agents/WORKFLOW.md` for scoring, hard floors, evidence requirements, and escalation.

## Specialist agents

Dispatch specialist agents explicitly when the task needs their role:

- `planner` uses Terra xhigh by default for established architecture and writes feature ledgers without implementing product code. Use a generic Sol planner when the canonical routing conditions require it.
- `reviewer` is the default Terra High read-only reviewer. Use it only when that profile matches the ledger; otherwise dispatch a generic read-only reviewer with the assigned model and the same contract. Feature-complete review always uses a generic Sol High reviewer.
- `documentation-steward` can update documentation and feature ledgers only. Use it after feature-complete review clears or for documentation reconciliation.

The main session implements active commits and review fixes. Assume the developer selected the implementation model and effort recorded in the ledger. Do not inspect the main session's runtime profile or dispatch a builder for model routing.

Each named definition pins its default model and reasoning effort. `planner` uses `gpt-5.6-terra` with `xhigh`; `reviewer` uses `gpt-5.6-terra` with `high`; `documentation-steward` uses `gpt-5.6-terra` with `medium`. Do not override a pinned role. Use a generic reviewer when the ledger assigns a different review profile.

If the planner or documentation steward is unavailable, follow the corresponding workflow skill in the main session and preserve the same boundary. If a reviewer is unavailable, stop and report the missing review capability; do not replace independent review with main-session self-review.

Every initial review of non-trivial work must use a separate fresh context. Start it from the original commit contract, packet, diff, and validation results. Do not lead with the implementer's reasoning or self-review. After the main session applies fixes, ask the same reviewer context to verify the corrected findings and newly exposed paths when it remains available. Dispatch another fresh reviewer when that context is unavailable or when the correction materially changes the scope, architecture, or review mandate.

## MCP

Recallium stores durable project context under the project name `fm-valuescout`. Search it before non-obvious decisions and save only context that is not already recorded in the repository.

Context7 provides current library documentation. Use it for library APIs and configuration details instead of guessing.

The pinned local `@hypothesi/tauri-mcp-cli` provides trusted live Tauri UI control. Use it only with the isolated UI-agent session described below.

### Live Tauri UI control

The project pins `@hypothesi/tauri-mcp-cli` for trusted, local UI-polish work. Start one of these application sessions in another terminal:

```bash
./scripts/dev ui-agent
./scripts/dev ui-agent --dump /absolute/path/dump.json
```

Each run creates and later removes a temporary application-data directory. The optional dump remains read-only and is ingested through the application's Rust snapshot service before the loopback bridge starts. There is no live-database mode.

Use `pnpm exec tauri-mcp driver-session start --json` to connect after the application is ready, and use the matching `status` and `stop` subcommands to manage the session. Before any broad control, confirm that status reports `identifier: app.fmvaluescout` and a `cwd` that matches this repository. If another app owns the default port, retarget the session to the port reported by the launcher and verify both values again. The CLI exposes broad trusted-development capabilities, including arbitrary WebView JavaScript. Version 0.12.0 does not dispatch application-defined commands through its advertised IPC command executor; use `window.__TAURI__.core.invoke(...)` through `webview-execute-js` when a UI-polish check needs real product IPC. Frontend console messages are available through `read-logs`, while Rust startup and migration messages remain in the launcher terminal.

Invoke `$workflow-ui-polish` only for an explicit live UI-polish request. It requires a connected isolated session, uses before/after evidence, checks both target window sizes plus keyboard and focus behavior, and leaves Git and external actions under the normal approval rules.
