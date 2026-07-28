---
name: minimalism
description: Read this before every implementation decision — how much code to add. Pair with coding-standards for how that code should look and behave.
---

# Intentional Minimalism

## Overview

This project is a hobbyist solo-dev effort. Every abstraction, dependency, and pattern you add is something you alone must maintain. The ladder helps you stop before adding complexity that serves no immediate need. It does not ask you to lower structural quality — only to skip the paranoid padding around it.

For naming, structure, comments, tests, and formatting, read `.cursor/skills/coding-standards/SKILL.md` and `references/universal.md`.

This skill gives you a structured protocol for making implementation choices. Before you reach for a library, a pattern, or even a function, run the ladder. Stop at the first rung that holds.

These rules bias toward caution over speed. For trivial tasks, use judgment.

## Decision ladder

Start at rung one. Descend only when the current rung does not solve the problem.

1. **YAGNI.** Does this need to exist? Could you remove something instead of adding something? Challenge every "should". Only "must" survives.

2. **Standard library.** Does the language or runtime already ship with this? Check before you import anything new.

3. **Native platform.** Does the browser, OS, or platform already provide this? The platform is free. Use it.

4. **Existing dependency.** Does a package already in the project provide this? Do not add a new dependency when one you already have covers the need.

5. **One line.** Can one clear line of code solve it? If it can, do not write a function, a class, or an abstraction. If the one line is unreadable — nested ternary, chained regex, three or more conditions — drop to rung six.

6. **Minimum code that works.** Write the shortest implementation that passes all tests. No extension points. No configurability. No hooks for future needs. Add those when the need arrives.

The ladder is a reflex, not a research project. Each rung takes seconds. If you spend minutes debating whether a rung applies, drop to the next one and move on.

Ask yourself: would a senior engineer call this overcomplicated? If yes, simplify. If you wrote 200 lines and it could be 50, rewrite it.

## Tradeoff comments

When you accept a shortcut on purpose, name the limit and the upgrade trigger in a structured comment:

```
# ponytail: <what was skipped or simplified>
# Upgrade to <what to build instead> if <measurable trigger condition>
```

The ponytail is not a TODO or a permission slip for bugs. It names a ceiling. The upgrade trigger is measurable. These comments are grep-able: `grep -r "ponytail:" src/` produces a debt ledger.

Good examples:

```
# ponytail: O(n²) dedup scan
# Upgrade to hash-set dedup if n exceeds 10,000 items

# ponytail: fixed 30s polling loop
# Upgrade to websocket push if latency matters or server load increases
```

## Safety carve-outs

These domains are never subject to the ladder. Always invest full rigour:

- Input validation at trust boundaries
- Error handling that prevents data loss
- Security controls
- Accessibility
- Hardware calibration
- Anything the user explicitly asked for

## Review vocabulary

Use these tags to flag code that should be simpler:

| Tag | When to use |
|---|---|
| `delete:` | Dead code, unused flexibility. Remove it. |
| `stdlib:` | Custom code that duplicates the standard library. |
| `native:` | Code that reimplements a platform feature. |
| `yagni:` | Abstraction with one caller, config nobody sets. |
| `shrink:` | Same logic, fewer lines possible. |

Format: `L<line>: <tag> <what was found>. <what to replace it with>.`

End each review with a net line: `Net: -23 lines possible (4 findings).`

## Honesty boundaries

- Do not claim per-repo line savings. The unbuilt version is imaginary. There is no baseline.
- Do not claim a performance improvement without before-and-after measurements.
- Do not claim 100% test coverage from line coverage alone.
- Do not call a change a bug fix unless you confirmed the old behaviour was wrong.

## Output discipline

When you present completed work, show the code first. Then at most three lines: what was skipped and when to add it. Do not write a paragraph describing what the code does — the code says that. Do not list every function added — the diff shows that.

If the explanation is longer than the code, delete the explanation.

Exceptions: commit messages (Conventional Commits per `.cursor/skills/conventional-commits/SKILL.md` — explain *why* in the body when the reason is not obvious from the diff), security decisions, architectural notes in durable docs, and explicit user requests for detail.
