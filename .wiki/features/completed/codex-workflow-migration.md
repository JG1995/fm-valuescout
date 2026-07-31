# Codex workflow migration

## Intent

Replace the repository's Cursor-specific development workflow with a Codex-native workflow. Preserve delivery discipline, validation, feature ledgers, review isolation, and local-commit approval.

## Delivered behavior

- Repository guidance uses `AGENTS.md`, `.agents/skills/`, `.codex/agents/`, and `.codex/config.toml`.
- The named workflow remains available for planning, test-first implementation, checkpoint review, fixes, feature completion, spikes, and security audits.
- `reviewer` is read-only and uses `gpt-5.6-terra` with `xhigh` reasoning. `documentation-steward` edits documentation only and uses `gpt-5.6-terra` with `medium` reasoning.
- `.work/` holds ignored disposable artifacts. Recallium and Context7 remain project MCP servers.
- `./scripts/dev check` covers code quality only. CI runs frontend tests, browser smoke, the Windows bridge suite, and the production build as explicit product checks.

## Final architecture

```text
AGENTS.md                         repository rules and routing
.agents/skills/                   reusable workflow and domain skills
.codex/agents/                    reviewer and documentation-steward roles
.codex/config.toml                Recallium and Context7 configuration
.work/                            ignored disposable work
scripts/dev + CI                  product validation and code-quality checks
```

- Codex guidance is configuration and documentation. It is reviewed with the change, not verified by fixed inventories, text markers, or copied CI-YAML tests.
- `check` runs Biome, TypeScript, secretlint, and Rust format, lint, and tests. Browser smoke and bridge tests are separate CI jobs because they validate product behavior.

## Important decisions

- Keep repository-owned `workflow-*` skills instead of Cursor slash commands or user-level custom prompts.
- Do not add Codex hooks. Existing scripts, Git hooks, and CI own deterministic enforcement.
- Remove the retired Cursor surface after Codex migration rather than maintain dual configuration.
- Focus validation on product behavior and code quality. Do not maintain workflow self-tests for skill counts, agent prose, or CI YAML wording.

## Migration and operational implications

- Trust the repository in Codex before using project MCP servers.
- Use `workflow-build-loop` only through its explicit manual opt-in; otherwise local commits require approval.
- Bridge unit tests require the .NET 6 SDK and run in Windows CI. Full FM attachment remains a manual Windows check.

## Validation

- `./scripts/dev test` — 91 Vitest tests passed.
- `./scripts/dev check` — Biome, TypeScript, secretlint, and Rust quality tests passed (153 Rust tests; 2 intentional performance tests ignored).
- `./scripts/dev smoke` — 6 Playwright product smoke tests passed.
- `./scripts/dev bridge-test` — 114 C# bridge tests passed; 1 Windows-only test skipped where not applicable.
- Feature-complete reviewer verdict: pass, with no blocking findings.

**Delivery commits:** `8c8d7ac`, `187c635`, `eaba444`, `a8cb615`, `73b5dad`, `495edea`, `a327cc9`, `50f86a2`, `fff1268`.

## Follow-up

- Consider directory-specific `AGENTS.md` files only if future repository complexity needs them.
- Consider a reusable Codex plugin only after this project workflow proves stable across use.
