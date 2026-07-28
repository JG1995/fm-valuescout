---
description: Optional — read-only security audit for trust boundaries, secrets, auth, and common AI footguns
---

Run a read-only security audit for:

${ARGUMENTS:-scoped audit — active feature diff or staged changes. Pass `full` for repository root, or a path or comparison base (e.g. `feature-complete since main`).}

## Purpose

`/security-audit` finds security issues that `/review` and the `reviewer` agent do not cover. The reviewer checks functional bugs and architecture. This command checks trust boundaries, secrets, authorization, injection, and patterns that AI assistants often introduce.

| Command | Scope | Focus |
| --- | --- | --- |
| `/review` | Staged commit or feature | Functional bugs, rules, architecture |
| `/security-audit` | Scoped or full codebase | Security reachability and exploitability |

## Role in the workflow

**Not part of the core loop.** You do not run `/security-audit` at every checkpoint.

Core loop: `/plan-feature` → `/build` → `/checkpoint` → … → `/finish-feature`.

`/security-audit` is an **optional** pass. Typical triggers:

- Before first deploy or MVP release
- After auth, payments, multi-tenant, or sensitive data features
- On demand: `/security-audit feature-complete since main`
- When the developer asks "is this safe?" or "audit for vulnerabilities"

After findings, the developer delegates fixes via `/fix` or manual edits. Do not auto-fix inside this command.

## Recallium

Read `.cursor/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** audit — prior security decisions, incidents, or constraints not in wiki or Git.
**Search with:** `search_memories` → `expand_memories` as needed.

**Save:** do not save — report only. Durable security decisions belong in wiki, ADRs, or explicit developer-requested Recallium saves.

## Mandatory reads

1. `AGENTS.md` and `.wiki/INDEX.md`.
2. **`.cursor/skills/security-audit/SKILL.md`** — follow the full process and output contract; load bundled references from its table (`universal`, `testing`, `react`, `vite`, `rust`, `tauri`) when recon matches.
3. **`.cursor/skills/security-audit/references/universal.md`** — load on every audit before stack refs.
4. **`.wiki/ARCHITECTURE.md`** when auditing implementation — auth model, layers, and trust boundaries.
5. Active feature ledger and planned spec when auditing a feature scope.
6. Scan `.cursor/skills/` for skills whose description matches security, the stack, auth, payments, database, or mobile. Read matching `SKILL.md` files and any bundled references when recon finds the technology.

## Modes

| Invocation | Mode | Scope |
| --- | --- | --- |
| `/security-audit` (default) | **scoped** | Active feature `git diff <base>...HEAD`, or `git diff --cached` when auditing staged work |
| `/security-audit full` | **full** | Repository root |
| `/security-audit <path>` | **path** | Directory or file under the repo |
| `/security-audit feature-complete since <base>` | **scoped** | `git diff <base>...HEAD` for the feature branch |

Establish the comparison base from ledger, merge base with trunk, or developer arguments. Ask before proceeding if ambiguous.

## Procedure

Follow `.cursor/skills/security-audit/SKILL.md`:

1. **Scope** — confirm target path or diff exists.
2. **Recon** — map stack, entry points, sinks, trust boundaries; show surface map for large scopes.
3. **Select checks** — load universal and matching stack refs only when recon supports them.
4. **Deep pass** — trace source to sink; default single pass; optional parallel read-only Task subagents when four or more check clusters apply.
5. **Report** — full output contract with confirmed vs suspected findings.

Use read-only inspection commands only. Do not edit, write, stage, unstage, commit, or push.

**Severity:** Critical and High **confirmed** findings block release or deploy until fixed or explicitly approved by the developer.

Return the full security audit structure from the skill output contract.
