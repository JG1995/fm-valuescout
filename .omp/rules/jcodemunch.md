---
name: use-jcodemunch-mcp
description: Use when searching, navigating, analyzing, or refactoring code in an indexed repository.
alwaysApply: true
---

# jCodemunch MCP Usage Rule

## When to Use
- Finding symbols, functions, classes, or methods
- Searching for references or call sites
- Understanding architecture, dependencies, or call hierarchies
- Analyzing code quality, complexity, or hotspots
- Detecting dead code or untested symbols
- Planning refactors or assessing blast radius

## Required Workflow
1. Before any manual file search or grep, check if a jCodemunch tool exists for the task.
2. Prefer `mcp__jcodemunch_search_symbols` over regex search for symbols.
3. Prefer `mcp__jcodemunch_find_references` over manual grep or `lsp references` for usages.
4. Use `mcp__jcodemunch_get_call_hierarchy` or `get_impact_preview` before deleting or renaming symbols.
5. Use `mcp__jcodemunch_get_repo_health`, `get_hotspots`, `get_symbol_complexity` for analysis.
6. Use `mcp__jcodemunch_get_dependency_graph` instead of manual import tracing.

## Forbidden Alternatives
- **NEVER** use `search` (regex) for symbol discovery when `search_symbols` is available.
- **NEVER** use `bash` with `grep`/`rg` for finding references.
- **NEVER** use `lsp references` when `mcp__jcodemunch_find_references` is available.
- **NEVER** use `ast_grep` when a jCodemunch structural query would suffice.
- **NEVER** use `find` for file listing when `get_file_tree` exists.

## Fallback Policy
Only fall back to manual search when jCodemunch returns no results or the repository is not indexed. State the fallback reason explicitly.
