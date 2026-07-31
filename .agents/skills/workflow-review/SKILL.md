---
name: workflow-review
description: Review staged changes or a complete feature — functional bugs, rules and architecture, project skills
---

## Specialist dispatch

Dispatch a separate named `reviewer` Codex agent. If named-agent dispatch is unavailable, use a generic read-only reviewer with the same instructions. Give the reviewer the active scope and require a full Review verdict with CRITICAL, HIGH, MEDIUM, and NITPICK tiers.

## Recallium

Read `.agents/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** judging findings — prior decisions or context behind the diff not in wiki, ledger, or spec.
**Search with:** `search_memories` → `expand_memories` as needed.

**Save:** do not save — report only. Durable context belongs in wiki, ledger, or checkpoint/finish-feature saves.

## Review focus

Commit review — staged changes (`git diff --cached`). Judge active commit scope only.

## Modes

| Invocation | Mode | Scope |
| --- | --- | --- |
| `$workflow-review` (default) | **commit review** | `git diff --cached`; active commit in ledger |
| `$workflow-review feature-complete for <feature> since <base>` | **feature-complete** | `git diff <base>...HEAD`; full ledger, planned spec, intent |

For **feature-complete** reviews, include the comparison base and feature name in the request. See `.agents/skills/workflow-finish-feature/SKILL.md` for the full validation sequence — `$workflow-review` alone does not run tests or documentation reconciliation.

**Report only** — do not edit code or fix findings.

Use read-only inspection commands only. Do not edit, write, stage, unstage, commit, or push.

**Skills:** Read `.agents/skills/coding-standards/SKILL.md` and `coding-standards/references/universal.md` plus matching stack refs. Scan `.agents/skills/` for other matching skills before judging conventions or structure.

**Priorities:** Functional bugs and architecture/rules first; obvious safety only — not exhaustive hardening.

**Severity:** **CRITICAL** and **HIGH** block commit (commit mode) or merge readiness and `$workflow-finish-feature` documentation reconciliation (feature-complete mode) unless the developer explicitly approves proceeding.

Return a full **Review verdict** with Blocking status and CRITICAL, HIGH, MEDIUM, and NITPICK tiers.
