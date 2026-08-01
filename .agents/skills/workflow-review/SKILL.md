---
name: workflow-review
description: Run a fresh-context, profile-routed review of staged changes or a complete feature with evidence-backed findings
---

## Specialist dispatch

Read `.agents/WORKFLOW.md`. Select the review profile from the invocation mode: use the active commit's profile for commit review and a generic **Sol High** reviewer for every feature-complete review. For non-trivial work without a ledger, use the named `reviewer` at its default Terra High profile. Use the named `reviewer` only when its pinned profile matches; otherwise use a generic read-only reviewer that follows `.codex/agents/reviewer.toml`.

For commit review, give the reviewer the original commit scope, relevant invariants and non-goals, implementation packet when one exists, commit-specific mandate, staged diff, validation results, and repository access. For feature-complete review, give it the complete ledger, linked planned spec, comparison base, full feature diff, commit-review summaries, final validation results, and repository access. Do not initially provide implementation reasoning or self-review. Require the full evidence-shaped Review verdict from `.codex/agents/reviewer.toml`.

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

**Priorities:** The commit-specific mandate, functional bugs, and architecture/rules first; obvious safety only — not exhaustive hardening. Retain a defect only when it has a violated contract, concrete execution path, and observable consequence. Put unsupported concerns under investigation notes.

**Severity:** **CRITICAL** and **HIGH** block commit (commit mode) or merge readiness and `$workflow-finish-feature` documentation reconciliation (feature-complete mode) unless the developer explicitly approves proceeding.

Return a full **Review verdict** with Blocking status and CRITICAL, HIGH, MEDIUM, and NITPICK tiers.
