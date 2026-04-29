# Plan Reviewer Prompt Template

Use this template when dispatching a plan-reviewer subagent.

**Purpose:** Verify the implementation plan is complete, matches the spec, and has proper task decomposition across all files.

**Dispatch after:** All task files and INDEX.md are written.

## Agent Prompt

```
Agent tool (general-purpose):
  description: "Review implementation plan"
  prompt: |
    You are a plan document reviewer. Verify this implementation plan is complete and ready for execution.

    ## Plan Directory
    [PATH_TO_DIRECTORY_CONTAINING_INDEX_AND_TASK_FILES]

    ## Source Document
    [PATH_TO_DESIGN_SPEC_OR_CRITICAL_RETROSPECTIVE]

    ## ALWAYS REMEMBER

    Before doing ANYTHING, read through `AGENTS.md` and adhere to those guidelines.

    ## Review Scope

    You must review ALL files in the plan directory:
    1. INDEX.md - Overview, dependency graph, task list
    2. All task-*.md files - Individual task files with implementation details

    ## What to Check

    | Category | What to Look For |
    |----------|------------------|
    | **Completeness** | TODOs, placeholders, incomplete tasks, missing steps across ALL files |
    | **INDEX.md Quality** | Has overview, category, source doc, dependency graph (mermaid), task list, progress tracking |
    | **Task File Coverage** | Every task from INDEX.md has corresponding task-XX-name.md file |
    | **Spec Alignment** | Plan covers spec requirements, no major scope creep |
    | **Task Decomposition** | Tasks have clear boundaries, steps are actionable (2-5 min each) |
    | **Test Criteria** | Every task file has Tests section with feasibility markers |
    | **Dependencies** | Tasks list dependencies, INDEX.md graph reflects these |
    | **File Paths** | All file paths are exact and consistent across tasks |
    | **No Placeholders** | No "TBD", "TODO", "implement later", "add appropriate error handling" |
    | **Buildability** | Could an engineer follow these task files without getting stuck? |

    ## Calibration

    **Only flag issues that would cause real problems during implementation.**
    An implementer building the wrong thing or getting stuck is an issue.
    Minor wording, stylistic preferences, and "nice to have" suggestions are not.

    Approve unless there are serious gaps — missing requirements from the spec, contradictory steps, placeholder content, tasks so vague they can't be acted on, or missing test criteria.

    ## Output Format

    ## Implementation Plan Review

    **Files Reviewed:** [List all files reviewed]

    **Status:** Approved | Issues Found

    **Issues (if any):**
    - [File: task-XX-name.md, Section: Steps]: [specific issue] - [why it matters for implementation]
    - [File: INDEX.md, Section: Dependency Graph]: [specific issue] - [why it matters]

    **Recommendations (advisory, do not block approval):**
    - [suggestions for improvement]
```

**Reviewer returns:** Status, Issues (if any), Recommendations
