---
name: codebase-memory
description: Explore this repository with the codebase-memory-mcp one-shot CLI. Use for architecture discovery, semantic code search, call-path tracing, change-impact analysis, or unfamiliar behavior that would otherwise require several rg and file-read passes. Prefer rg for exact text, known symbols, or known files.
---

# Codebase Memory

Use the local Codebase Memory index for relational or conceptual codebase questions. Invoke its supported CLI directly; do not add or depend on an MCP server entry.

## Select the search path

- Use `rg` or `rg --files` for exact text, filenames, and known symbols.
- Use Codebase Memory when the question depends on relationships, architecture, semantic similarity, call paths, or blast radius.
- Confirm important conclusions in the source before editing. The index accelerates discovery; source and tests remain authoritative.

## Prepare the index

Run commands from the repository root.

```bash
codebase-memory-mcp --version
codebase-memory-mcp cli list_projects
```

If this repository is absent, index it:

```bash
codebase-memory-mcp cli index_repository --repo-path "$PWD" --mode full --persistence false
```

Use the project name returned by `list_projects` for later commands. Refresh the index before a high-confidence analysis when the reported index status is stale:

```bash
codebase-memory-mcp cli index_status --project <project-name>
codebase-memory-mcp cli index_repository --repo-path "$PWD" --mode full --persistence false
```

## Query the graph

Inspect a tool's generated flags before its first use:

```bash
codebase-memory-mcp cli <tool> --help
```

Choose the narrowest useful query:

| Need | Tool |
| --- | --- |
| Repository structure and hotspots | `get_architecture` |
| Symbols by type, name, file, or concept | `search_graph` |
| Callers and callees | `trace_path` |
| Literal search with indexed context | `search_code` |
| Source for a discovered symbol | `get_code_snippet` |
| Impact of working-tree changes | `detect_changes` |
| Custom read-only graph traversal | `query_graph` |

Examples:

```bash
codebase-memory-mcp cli get_architecture --project <project-name> --aspects overview
codebase-memory-mcp cli search_graph --project <project-name> --name-pattern '.*Snapshot.*'
codebase-memory-mcp cli search_graph --project <project-name> --semantic-query '["snapshot","ingest"]'
codebase-memory-mcp cli trace_path --project <project-name> --function-name ingest_dump_file --direction both
codebase-memory-mcp cli detect_changes --project <project-name>
```

Keep stdout machine-readable. Send any filtering through `jq` only after inspecting the unfiltered result shape.

## Boundaries

- Do not run `install`; repository setup intentionally keeps Codebase Memory out of Codex MCP configuration.
- Do not run `delete_project` or mutate ADRs unless the developer explicitly requests that operation.
- Do not enable the daemon, UI, watchers, hooks, or automatic agent configuration.
- If the binary is missing or a query fails, report that state and continue with `rg` and direct source inspection.
