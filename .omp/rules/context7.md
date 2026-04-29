---
name: use-context7-mcp
description: Use when looking up library documentation, API references, configuration options, or CLI tool usage.
alwaysApply: true
---

# Context7 MCP Usage Rule

## When to Use
- Any question about a library, framework, SDK, API, or cloud service
- API syntax, configuration options, version migration issues
- Debugging library-specific behavior
- Setup instructions for any tool or service

## Required Workflow
1. Resolve library ID via `mcp__context_resolve_library_id` first, unless user provides a `/org/project` ID.
2. Query documentation via `mcp__context_query_docs` with a specific, descriptive query.
3. Retry with `--research` flag if initial results are insufficient.
4. Limit to 3 attempts per question before falling back.

## Forbidden Alternatives
- **NEVER** use `web_search` for library API details when Context7 is available.
- **NEVER** answer from training data for library-specific behavior without first trying Context7.
- **NEVER** use the `ctx7` CLI when the Context7 MCP server is configured.
- **NEVER** skip the library resolution step.

## Fallback Policy
If Context7 quota is exhausted or the server is unavailable, inform the user explicitly and note that subsequent information may be outdated. Do not silently fall back to training data.
