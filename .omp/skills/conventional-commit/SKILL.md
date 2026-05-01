---
name: commit
description: "Prompt and workflow for generating conventional commit messages with multi-commit analysis. Analyzes changes since the last commit, determines logical grouping, proposes a plan for user approval, and executes the approved plan while adhering to Conventional Commits."
---

### Instructions

This file contains a prompt template for generating conventional commit messages. It provides instructions, examples, and formatting guidelines to help users write standardized, descriptive commit messages in accordance with the Conventional Commits specification.

## ALWAYS REMEMBER

Before doing ANYTHING, read through `AGENTS.md` and adhere to those guidelines if relevant. In particular: **NEVER autonomously stage, commit, or push code to version control. All commits must be explicitly approved by the human operator.**

### Workflow

This skill follows a three-phase workflow to analyze changes, propose a commit plan, and execute with user approval.

#### Phase 1 — Analyze

1. **Run `git status`** to review all changed files.
2. **Run `git diff`** (unstaged changes) and **`git diff --staged`** (already staged changes) to inspect all modifications.
3. **Group changes into logical units** based on:
   - Type of change (feat, fix, refactor, docs, chore, etc.)
   - Affected scope/module (e.g., parser, ui, api)
   - Concern separation (infrastructure vs feature work)
4. **Order groups from least risky to most risky**: `chore` → `docs` → `style` → `refactor` → `perf` → `test` → `fix` → `feat` → `BREAKING CHANGE`.

#### Phase 2 — Propose

5. **Present a structured plan to the user** showing:

   **If proposing a single commit:**

   - Type, scope, and description
   - Files to be staged
   - Proposed commit message
   - Rationale for keeping as one commit

   **If proposing multiple commits:**

   - Number of commits
   - For each commit (in order):
     - Commit number (1, 2, 3, ...)
     - Type, scope, and description
     - Files to be staged for this commit
     - Proposed commit message
   - Rationale for splitting

6. **Wait for user approval.** The user may:

   - Accept the plan as proposed
   - Request modifications to commit grouping, messages, or ordering
   - Request to merge commits into one
   - Cancel the operation

   **DO NOT proceed until user explicitly approves.**

#### Phase 3 — Execute

7. **Before staging**, run pre-commit auto-fixing hooks on the target files to prevent mid-commit formatting failures:

   ```bash
   pre-commit run --files <file1> <file2> ...
   ```

   If hooks report fixes (exit code 1), the files have been auto-corrected in the working tree. Proceed to stage the now-clean files. This prevents the common pattern of a commit failing due to trailing whitespace, end-of-file, or prettier fixes, requiring a second `git add && git commit`.

8. **For each approved commit** (in the agreed order):

   - Stage the relevant files: `git add <file1> <file2> ...`
   - Execute the commit: `git commit -m "type(scope): description"`
   - Confirm the commit was created successfully

9. **Provide a completion summary** listing all commits created with their hashes and messages.

### Decision Criteria for Splitting

**SHOULD split into separate commits when:**

- Changes span multiple types (e.g., a new feature mixed with a bug fix)
- Changes affect distinct scopes/modules with no logical coupling
- Infrastructure/tooling changes are mixed with feature work
- Documentation changes are mixed with code changes
- Splitting would create cleaner, more atomic change units

**MAY keep as a single commit when:**

- All changes are tightly coupled (e.g., a feature and its direct tests)
- Splitting would leave any intermediate commit in a broken state
- Changes are small and logically unified
- The changes are part of an indivisible unit (e.g., a rename with its necessary code updates)

### Commit Message Structure

Each commit message MUST follow this XML structure:

```xml
<commit-message>
	<type>feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert</type>
	<scope>()</scope>
	<description>A short, imperative summary of the change</description>
	<body>(optional: more detailed explanation)</body>
	<footer>(optional: e.g. BREAKING CHANGE: details, or issue references)</footer>
</commit-message>
```

### Examples

```xml
<examples>
	<example>feat(parser): add ability to parse arrays</example>
	<example>fix(ui): correct button alignment</example>
	<example>docs: update README with usage instructions</example>
	<example>refactor: improve performance of data processing</example>
	<example>chore: update dependencies</example>
	<example>feat!: send email on registration (BREAKING CHANGE: email service required)</example>
</examples>
```

### Validation

```xml
<validation>
	<type>Must be one of the allowed types. See <reference>https://www.conventionalcommits.org/en/v1.0.0/#specification</reference></type>
	<scope>Optional, but recommended for clarity.</scope>
	<description>Required. Use the imperative mood (e.g., "add", not "added").</description>
	<body>Optional. Use for additional context.</body>
	<footer>Use for breaking changes or issue references.</footer>
</validation>
```

### Reference

For the complete Conventional Commits 1.0.0 specification, including detailed examples, FAQ, and rationale, see:

- Local: `.omp/skills/commit/conventional-commits.md`
- Official: https://www.conventionalcommits.org/en/v1.0.0/
