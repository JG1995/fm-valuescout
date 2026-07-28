# Cursor Workflow Guide

This project uses a test-driven, AI-assisted development workflow in Cursor. It combines the `./scripts/dev` command surface with Cursor commands, agents, skills, and MCP tools.

This template targets **React + Tauri v2 desktop projects** with a thin-frontend / thick-backend scaffold — React in the WebView, Rust for computation and SQLite persistence via IPC. Forks adjust stack details via `/stack` when they diverge from the template defaults.

---

## Before you start

1. **Open in Cursor.** `AGENTS.md` loads automatically. Project rules in `.cursor/rules/` apply every session (`session-start`, `workflow-phase`, `recallium`). Per `session-start`, agents read `.wiki/INDEX.md`, `.wiki/CONCEPT.md`, `.wiki/ARCHITECTURE.md`, then an active feature ledger in `.wiki/features/active/` or `.wiki/TODO.md`; they also check `.cursor/skills/` for a matching skill. Re-read architecture and ledger after `/summarize` or compaction.
2. **Recallium project name.** In `AGENTS.md` § Recallium project, replace `[REPLACE_WITH_RECALLIUM_PROJECT_NAME]` with your kebab-case project id (often the workspace folder name). Agents use that exact string on every Recallium call.
3. **MCP servers.** Project MCP config lives in `.cursor/mcp.json` (Recallium, Context7). Reload the window after changes. Ensure Recallium is enabled for this workspace (and in `~/.cursor/mcp.json` if you use a global copy).
4. **Git hooks.** Husky installs on `pnpm install` and runs `./scripts/dev check-fast` on commit (plus `check-rust` when `src-tauri/` is staged). CI runs the full `./scripts/dev check`.
5. **Verify the gate.** Run `pnpm install`, then `./scripts/dev format`, `./scripts/dev check-fast`, and `./scripts/dev test`. Run `./scripts/dev check` before merge.

Never put credentials in repository files — repository files must not contain credentials.

> **`/plan` vs `/plan-feature`:** Cursor's `/plan` is built-in Plan mode. This project uses **`/plan-feature`** for per-feature delivery plans (PRs and commits).

---

## The development loop

1. **Describe your intent** or pick the next feature from the development sequence.
2. **`/plan-feature`** — PR and commit breakpoints, high-level work descriptions.
3. **Review the plan.** Adjust PR/commit boundaries if needed.
4. **`/build`** — implement the active commit (default: one commit, then stop for checkpoint).
5. **`/checkpoint`** — stage, gate, reviewer verdict, present for approval.
6. **`/fix`** — when review blocks, address delegated findings, then checkpoint again.
7. **Approve.** Agent creates one atomic local commit after your approval.
8. **Reassess.** Activate the next commit in the delivery plan.
9. **`/finish-feature`** — when every commit in the delivery plan is done: full tests, feature-complete review, then documentation reconciliation.

For trivial changes (doc fix, rename), describe the change — the agent follows the loop internally without invoking each command.

### First-time setup (before the build loop)

The walking skeleton is already built. Follow [README — Forking this template](../README.md#forking-this-template) for prerequisites, rename table, Playwright, and editor extensions. Then:

1. Replace the Recallium `project_name` placeholder in `AGENTS.md` § Recallium project when you use Recallium (see **Before you start**).
2. Fill `.wiki/CONCEPT.md` (especially MVP scope).
3. **`/stack`** — only when you change the default stack; skip when you keep template defaults.
4. Add specs in `.wiki/features/planned/<slug>.md` when you can — optional for `/roadmap`; CONCEPT bullets alone suffice for a provisional sequence.
5. **`/roadmap`** — approve development sequence in `.wiki/TODO.md`.
6. **`/plan-feature`** on the feature named in **Plan next** — then the normal build loop.

---

## Slash commands

Invoke these from the chat command palette in Cursor (type `/`):

| Command | When to use |
| --- | --- |
| `/stack` | CONCEPT filled — recommend stack and target architecture from wiki notes; write wiki only after approval |
| `/roadmap` | Dependency-aware MVP build order (grounded in planned specs when present; inferred from CONCEPT when not); write TODO sequence only after approval |
| `/plan-feature` | Next feature from the sequence — PR/commit plan (trunk-based, atomic commits, Conventional Commits) |
| `/build` | Default: one active commit (RED/GREEN), ledger update, stop for `/checkpoint`. Opt-in: full feature one commit at a time |
| `/fix` | Delegated CRITICAL/HIGH (or specified) findings — project skills apply; stop for `/checkpoint` |
| `/checkpoint` | Stage, reviewer verdict, present for approval; commit only when approved |
| `/review` | Read-only review — commit scope (staged) or feature-complete (whole feature diff) |
| `/docs-review` | Reconcile durable documentation with implemented state |
| `/finish-feature` | Feature complete — full tests, feature review, then docs reconciliation |

### Optional — not in the main loop

| Command | When to use |
| --- | --- |
| `/spike` | One technical yes/no blocked until you **run** a small disposable experiment — after wiki, skills, and Context7 are insufficient. Not for stack comparisons or routine research |
| `/security-audit` | Read-only pass for secrets, auth, trust boundaries, and common AI footguns — before deploy or after sensitive features. Not at every checkpoint |

Command definitions live in `.cursor/commands/`. Cursor loads them from this directory.

---

## Specialist agents

Two project agents live in `.cursor/agents/`. Each pins a **model** in frontmatter (update when you change fleet defaults):

| Agent | Model | Purpose | When to dispatch |
| --- | --- | --- | --- |
| `reviewer` | Grok 4.5 High | Read-only code review | Every non-trivial staged diff; feature-complete pass at `/finish-feature` |
| `documentation-steward` | Composer 2.5 | Documentation reconciliation | After feature-complete review clears at `/finish-feature`, or architecture changes |

Structural planning uses **`/plan-feature`** and **`.wiki/ARCHITECTURE.md`**, plus per-project skills in `.cursor/skills/` — not a dedicated planning agent.

Cursor's built-in `explore` subagent handles fast codebase search automatically or via Task — no project file.

Dispatch project agents explicitly via the Task tool (`subagent_type: "reviewer"` or `"documentation-steward"`) or by asking Cursor to use the named agent. Do **not** pass Task `model` — the agent frontmatter pins the model. The main session is the **worker** — routine implementation and validation. No automatic role routing.

Manual fallback: read the agent file and follow its instructions in the current session, or use the matching command (`/review`, `/docs-review`).

---

## MCP tools

Project MCP servers in `.cursor/mcp.json`:

| Server | Purpose |
| --- | --- |
| Recallium | Search institutional memory before non-obvious decisions; save sparingly per `recallium.mdc` and `recallium-usage` skill |
| Context7 | Current library documentation (`resolve-library-id`, `query-docs`) |

Built-in Cursor tools: `WebSearch`, `WebFetch` for bounded external research.

---

## Skills

Reusable guidance in `.cursor/skills/`. The `session-start` rule requires checking for a matching skill before tasks. The list below is not exhaustive — browse `.cursor/skills/` for the full set. Commonly used skills:

- `conventional-commits` — commit message format at checkpoint
- `coding-standards` — universal and React stack-specific code style (`references/universal.md` + stack refs as they land)
- `debug` — root-cause investigation before fixes
- `minimalism` — decision ladder for how much code to add; pairs with coding-standards
- `project-strategy` — stack and architecture advice (used by `/stack`)
- `recallium-usage` — memory policy; each command has a **## Recallium** section with phase hooks
- `security-audit` — optional read-only security pass (`/security-audit`); add matching refs when your stack supplies them

---

## Further reading

- [AGENTS.md](../AGENTS.md) — standing development contract
- [Wiki index](../.wiki/INDEX.md) — durable documentation map
- [Contributing](../CONTRIBUTING.md) — commit conventions and hooks
