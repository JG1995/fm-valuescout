---
name: conventional-commits
description: Read this before you write, review, or propose a commit message. If you are about to type git commit or draft text for a commit, read this skill first.
---

# Conventional Commits

## Overview

Every commit message must follow the Conventional Commits 1.0.0 format. This skill tells you how to choose the right type, scope, and wording for any change you make in this project. Apply it whenever you prepare a checkpoint or review a staged diff.

The commit message is part of the evidence. A good message tells a reader what the repo looks like after the commit and why that state matters. It is not a diary entry about what you did.

The description names the outcome (`what`). When the reason is not obvious from the diff, put `why` in the optional body — trade-offs, rejected alternatives, or the constraint that forced the change. Skip the body when the description and diff already make the reason clear.

## Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

The type is required. The scope is optional. A `!` before the colon marks a breaking change. The description is required and must follow the colon and space.

Examples:

```
feat(import): add CSV file format support
fix(parser): handle empty lines in input
refactor(storage): extract connection pool into separate module
```

## Choosing a type

| Type | Use when the commit... |
|---|---|
| `feat` | adds a new user-visible capability |
| `fix` | corrects incorrect behaviour (a bug) |
| `refactor` | changes structure without changing behaviour |
| `test` | adds, fixes, or characterizes tests only, with no production code change |
| `docs` | changes only documentation |
| `chore` | changes tooling, config, dependencies, or scripts with no production impact |
| `perf` | improves performance without adding a feature or fixing a bug |
| `style` | changes formatting, whitespace, or lint rules with no behavioural change |
| `ci` | changes CI configuration or workflow files |
| `build` | changes the build system or package manager |

When in doubt between two types, pick the one that best describes what a reader sees *after* the commit, not what you did to produce it. If the commit only touches tests, use `test`. If it adds a feature and tests together, use `feat`.

A characterization test written before a refactoring is `test`. An enabling refactor that preserves all existing behaviour is `refactor`.

## Writing the description

The description must:

- Use imperative mood. "Add", "Fix", "Remove", "Extract", "Replace". Never "Added", "Fixes", "Removing", "Extracted", or "Replaces".
- Be a complete phrase that completes "This commit will..." For example: "This commit will add CSV file format support." The description in the commit is `add CSV file format support`.
- Be specific enough to distinguish the commit from all others. "Fix bug" is too vague. "Fix null pointer when project name is empty" is specific.
- Stay under 72 characters. If you need more, use the body.
- Contain no issue tracker references. Put those in footers.

The description names the outcome, not the effort. Compare:

| Weak | Better |
|---|---|
| `refactor: start working on extraction` | `refactor: extract connection pool into separate module` |
| `feat: add new feature` | `feat: add project export to JSON format` |
| `fix: fix stuff` | `fix: prevent crash when project file is missing` |

## Using scope

Scope is a noun that names the part of the codebase the commit affects. Examples from this project: `storage`, `parser`, `import`, `ui`, `schema`, `migration`, `config`.

Use scope when it helps a reader find the commit in a long history. Skip scope when the change touches many areas or the scope adds no information. `feat: add CSV export` may be better than `feat(*): add CSV export` or `feat(ui,storage,import): add CSV export`.

Keep scope short. One word where possible.

## Marking breaking changes

A commit that changes existing behaviour in a way that requires action from consumers must be marked as breaking. Use one of two forms:

```
refactor(storage)!: rename connection pool environment variables
feat: drop support for Node 18

BREAKING CHANGE: the minimum required Node version is now 20.
```

Use `!` for simple breaking changes where the description alone explains the impact. Use the `BREAKING CHANGE:` footer when the breaking change needs more explanation than fits in 72 characters.

Do not mark additive changes as breaking. Adding a new feature or new API is `feat`, not breaking.

## Body and footers

Add a body only when the reader needs context that the description cannot hold. Good reasons for a body:

- Explaining *why* a change was made when the reason is not obvious from the diff.
- Describing the trade-off or alternative approaches considered.
- Providing migration instructions.

Keep the body brief. One or two paragraphs at most.

Use footers for structured metadata:

```
Reviewed-by: Alice
Refs: #123
```

`BREAKING CHANGE:` is a footer when used in the footer section.

## What to avoid

- **Vague descriptions.** "Update code", "Fix issue", "Improve stuff". These waste the commit log.
- **Multiple unrelated changes in one commit.** If the message needs "and" in the description, split the commit.
- **Descriptions that describe your work instead of the result.** "Start working on X", "WIP", "Try different approach".
- **Passive voice in the description.** "Project name is added to config" → "Add project name to config".
- **Issue numbers in the description.** Use footers instead: `Refs: #123`.
- **Period at the end of the description.** The description is not a sentence.
- **Tense switching.** Always imperative, always now.

## Examples for this project

```
feat(import): add CSV file format support
```

A new feature. Reader knows exactly what capability was added and where.

```
fix(parser): handle empty lines in input
```

A bug fix. Reader knows the location and the specific edge case.

```
refactor(storage): extract connection pool into separate module
```

Behaviour-preserving change. Reader knows what structural change was made and where.

```
test(storage): characterize legacy player persistence
```

A characterization test (common in this workflow). Reader knows the area and the intent.

```
chore: upgrade axios from 0.27 to 1.6
```

Dependency change with no production code impact beyond the upgrade itself.
