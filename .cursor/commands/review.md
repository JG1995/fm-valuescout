---
description: Review staged changes or a complete feature — functional bugs, rules and architecture, project skills
---

## Specialist dispatch

When the Task tool is available, dispatch the `reviewer` agent.

**Model:** Always use the pin in `.cursor/agents/reviewer.md` frontmatter. **Never** pass Task `model`. No exceptions.

    Task({
      subagent_type: "reviewer",
      // do not pass model — reviewer.md frontmatter owns it
      prompt: "Review following .cursor/agents/reviewer.md and AGENTS.md. Search Recallium per .cursor/rules/recallium.mdc when wiki/Git do not explain prior decisions relevant to the review. Mode: ${ARGUMENTS:-commit review on staged changes}. Read .cursor/skills/coding-standards/SKILL.md, coding-standards/references/universal.md, and coding-standards/references/testing.md when diff includes tests, plus matching stack refs. Read other matching .cursor/skills/. Return the full Review verdict with CRITICAL/HIGH/MEDIUM/NITPICK tiers. Do not fix anything — report only.",
      description: "Review changes",
    })

## Recallium

Read `.cursor/skills/recallium-usage/SKILL.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** judging findings — prior decisions or context behind the diff not in wiki, ledger, or spec.
**Search with:** `search_memories` → `expand_memories` as needed.

**Save:** do not save — report only. Durable context belongs in wiki, ledger, or checkpoint/finish-feature saves.

## Review focus

${ARGUMENTS:-Commit review — staged changes (`git diff --cached`). Judge active commit scope only.}

## Modes

| Invocation | Mode | Scope |
| --- | --- | --- |
| `/review` (default) | **commit review** | `git diff --cached`; active commit in ledger |
| `/review feature-complete for <feature> since <base>` | **feature-complete** | `git diff <base>...HEAD`; full ledger, planned spec, intent |

For **feature-complete** reviews, include comparison base and feature name in `${ARGUMENTS}` or the Task prompt. See `.cursor/commands/finish-feature.md` for the full validation sequence — `/review` alone does not run tests or documentation reconciliation.

Follow `.cursor/agents/reviewer.md` in full. **Report only** — do not edit code or fix findings.

Use read-only inspection commands only. Do not edit, write, stage, unstage, commit, or push.

**Skills:** Read `.cursor/skills/coding-standards/SKILL.md` and `coding-standards/references/universal.md` plus matching stack refs. Scan `.cursor/skills/` for other matching skills before judging conventions or structure.

**Priorities:** Functional bugs and architecture/rules first; obvious safety only — not exhaustive hardening.

**Severity:** Use the reviewer output contract — **CRITICAL** and **HIGH** block commit (commit mode) or merge readiness and `/finish-feature` doc reconciliation (feature-complete mode) unless the developer explicitly approves proceeding.

Return the full **Review verdict** structure from `.cursor/agents/reviewer.md`.
