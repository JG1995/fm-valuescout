# Codex Workflow Migration

## Status

Active

## Intent

Replace the repository's Cursor-specific development workflow with a Codex-native workflow while preserving its delivery discipline, validation gate, feature ledgers, review isolation, and approval boundaries.

## User-visible behavior

- Contributors can use Codex project guidance, skills, specialist agents, and MCP servers without relying on Cursor configuration.
- The named workflow remains available for planning, test-first implementation, checkpoint review, fixes, feature completion, spikes, and security audits.
- Existing application behavior and product delivery order do not change.

## Invariants

- `./scripts/dev` remains the stable validation surface.
- Non-trivial work receives an independent, read-only reviewer pass.
- Local commits require explicit developer approval, except for the existing manual build-loop opt-in.
- Feature plans remain in `.wiki/features/active/` and durable project facts remain in `.wiki/`.
- Recallium uses the project name `fm-valuescout`; repository configuration contains no credentials.
- Every commit in this migration keeps the repository gate green.

## Non-goals

- Change application code, runtime behavior, persistence, schemas, or product features.
- Support Cursor and Codex indefinitely.
- Package the workflow as a reusable Codex plugin.
- Add Codex lifecycle hooks when existing scripts, Git hooks, and CI already enforce the rule.
- Replace Recallium with Codex memory.

## Current-state map

- Relevant components: `AGENTS.md`, `.cursor/`, `.wiki/`, `README.md`, `CONTRIBUTING.md`, `scripts/dev`, workflow contract scripts, and CI fixture setup.
- Data model: no application data changes.
- Persistence and migrations: no changes.
- Existing behavioral assumptions: Cursor loads commands, Markdown agents, rules, skills, and `.cursor/mcp.json`.
- Architectural seams: durable guidance in `AGENTS.md`; reusable procedures in skills; external tools in MCP; deterministic enforcement in `scripts/dev`, Git hooks, and CI.
- Tests and validation: `scripts/test-cursor-config.sh`, `scripts/test-agent-definitions.sh`, `scripts/test-ci-workflow.sh`, and `./scripts/dev check`.
- Primary risks: losing a workflow invariant during translation, leaving stale Cursor references, making specialist agents too permissive, or committing a migration step that breaks the gate.

## Feature architecture (this feature)

Codex will use four repository-owned surfaces:

- `AGENTS.md` for concise, durable repository rules and routing.
- `.agents/skills/` for reusable workflow and domain skills.
- `.codex/agents/` for specialist agent configuration.
- `.codex/config.toml` for trusted project configuration and MCP servers.

The human workflow guide lives at `.codex/README.md`. Disposable agent work uses the ignored `.work/` directory. Codex guidance is reviewed as configuration and documentation; repository checks focus on product behavior and code quality.

## Uncertainty register

### Known

- Codex reads repository skills from `.agents/skills/`.
- Codex reads project-scoped specialist agents from `.codex/agents/*.toml`.
- Codex reads trusted project MCP configuration from `.codex/config.toml`.
- Current Cursor agent model identifiers are not Codex model identifiers.

### Assumptions

- Codex becomes the only supported AI development workflow after cutover.
- Specialist agents can use explicit Codex model and reasoning pins.
- Recallium and Context7 remain the required MCP servers.

### Decisions

- Preserve the existing named workflow as Codex skills instead of reproducing Cursor slash commands.
- Pin the reviewer and documentation steward after evaluating the workflow: both use `gpt-5.6-terra`; reviewer uses `xhigh` reasoning and documentation steward uses `medium`.
- Do not add Codex hooks during this migration.
- Keep the product development sequence unchanged while this tooling initiative is active.

### Unknowns

- Whether later use shows a need for directory-specific `AGENTS.md` files.
- Whether the workflow should become a reusable plugin after it proves stable in this repository.

### Risks

- Skill descriptions can overlap and trigger the wrong workflow.
- Transitional duplication can let Cursor and Codex instructions drift before cutover.
- Project-local MCP configuration requires the repository to be trusted by Codex.

## Walking skeleton

Add a validated `.codex/config.toml` with Recallium and Context7 while the existing Cursor workflow still operates. This proves Codex can load project configuration before commands, agents, or documentation are removed.

## Delivery plan

### PR 1 — Replace Cursor workflow with Codex

**Status:** Completed

**Provisional PR title:** `chore(workflow): replace Cursor configuration with Codex`

**Purpose:** Migrate the complete repository workflow in one review surface while keeping every intermediate commit valid.

**Depends on:** Existing `scripts/dev` validation and repository documentation ownership.

#### Commit 1 — Document the Codex migration contract

**Status:** Completed — `8c8d7ac`

**Work:** Create the active migration ledger and mark the tooling initiative active without changing the product development sequence.

**Out of scope for this commit:**

- Codex configuration, skills, agents, scripts, or application code.
- A deliberately failing contract test committed to trunk.

**Validation:** Review the ledger against the approved plan, verify exactly one active feature commit after ledger advancement, and run `./scripts/dev check`.

**Provisional commit:** `docs(workflow): document Codex migration contract`

#### Commit 2 — Add Codex project configuration

**Status:** Completed — `187c635`

**Work:** Add `.codex/config.toml` with Recallium and Context7, then add contract validation for the trusted project configuration.

**Out of scope for this commit:**

- Workflow skills, specialist agents, hooks, or removal of Cursor configuration.

**Validation:** Run the new contract test RED before adding the configuration, then run it GREEN and run `./scripts/dev check`.

**Provisional commit:** `chore(workflow): add Codex project configuration`

#### Commit 3 — Port repository skills to Codex

**Status:** Completed — `eaba444`

**Work:** Copy domain skills to `.agents/skills/` with mechanical path updates. Port each Cursor command as a `workflow-*` skill with clear triggers and no Cursor argument macros.

**Out of scope for this commit:**

- Specialist agent definitions, broad documentation cutover, or Cursor removal.

**Validation:** Validate skill structure and frontmatter, exercise representative explicit skill invocation, and run `./scripts/dev check`.

**Provisional commit:** `feat(workflow): port repository skills to Codex`

#### Commit 4 — Port specialist agents to Codex

**Status:** Completed — `a8cb615`

**Work:** Add Codex TOML definitions for the reviewer and documentation steward, translate delegation instructions, and validate their scope and permissions.

**Out of scope for this commit:**

- Broad documentation cutover or removal of Cursor compatibility.

**Validation:** Verify the reviewer is read-only, verify the documentation steward's write boundary, validate TOML definitions, and run `./scripts/dev check`.

**Provisional commit:** `feat(workflow): port specialist agents to Codex`

#### Commit 5 — Make Codex the documented workflow

**Status:** Completed — `73b5dad`

**Work:** Reduce `AGENTS.md` to durable rules and routing, add the Codex workflow guide, and update contributor and wiki documentation to use Codex terminology and paths.

**Out of scope for this commit:**

- Deleting transitional Cursor files or changing application behavior.

**Validation:** Check documentation links and terminology, confirm workflow invariants remain represented, and run `./scripts/dev check`.

**Provisional commit:** `docs(workflow): make Codex the documented workflow`

#### Commit 6 — Remove Cursor compatibility

**Status:** Completed — `495edea`

**Work:** Replace Cursor-specific contract scripts and CI fixture wiring, move disposable-work references, remove `.cursor/`, and eliminate stale live references.

**Out of scope for this commit:**

- Historical notes where Cursor is part of the recorded fact, or application changes.

**Validation:** Run the full test suite and `./scripts/dev check`, then search for unintended `.cursor`, Cursor command, and unsupported model references.

**Provisional commit:** `chore(workflow): remove Cursor compatibility`

### PR 2 — Focus validation on product behavior

**Status:** Active

**Provisional PR title:** `chore(validation): focus checks on product behavior`

**Purpose:** Remove self-testing workflow contracts from the product gate, keep static and product checks, and run the C# bridge test suite in CI.

**Depends on:** PR 1 — Codex workflow migration.

#### Commit 1 — Replace workflow contracts with product checks

**Status:** Completed — hash pending checkpoint commit

**Work:** Remove fixed-inventory, fixed-wording, workflow-YAML, and dispatcher contract scripts. Keep `check` for static analysis, secret scanning, and Rust quality tests; make browser smoke explicit in CI; add `bridge-test` and a Windows CI job for the C# bridge suite.

**Out of scope for this commit:**

- Changing application behavior, product tests, or production dependencies.
- Broad documentation reconciliation beyond this ledger.

**Validation:** Run the existing frontend, Rust, browser smoke, and bridge product suites where their toolchains are available; run the full `check` gate; inspect CI workflow syntax and diff.

**Provisional commit:** `chore(validation): focus checks on product behavior`

#### Commit 2 — Document product-focused validation

**Status:** Pending

**Work:** Update contributor, architecture, bridge, and repository guidance to describe product-test ownership and remove workflow-contract terminology.

**Out of scope for this commit:**

- New product features or additional test frameworks.
- Changing Codex skills, agents, or MCP configuration.

**Validation:** Check documentation links and terminology, then run the product validation commands available locally.

**Provisional commit:** `docs(validation): document product-focused checks`

## Active work

**PR:** PR 2 — Focus validation on product behavior

**Commit:** Commit 1 — Replace workflow contracts with product checks

### Next step

Implement Commit 1, then checkpoint it before moving to the documentation commit.

### Delivered outcome

The repository uses Codex-only guidance, disposable work uses `.work/`, and no live guidance or CI fixture depends on `.cursor/`.

### Preserved exclusions

- Do not change application behavior, product documentation, Codex MCP configuration, or specialist agent definitions.
- Do not add hooks or change Git approval boundaries.

## Discoveries and replanning

- **Planned:** Add RED contract checks in the documentation commit. **Changed:** Record the RED test as the first action of commit 2. **Why:** A committed failing contract test would violate the invariant that every migration commit keeps trunk green.
- **Planned:** Preserve Cursor commands as project-local Codex custom prompts. **Changed:** Port them as `workflow-*` skills. **Why:** Codex custom prompts are user-level, while versioned repository skills are project-local and are the supported replacement.
- **Planned:** Move disposable-work references and the ignore rule with Cursor removal. **Changed:** Add the `.work/` ignore rule while porting skills. **Why:** The copied debug and spike skills write disposable artifacts there; ignoring it now preserves the scratch-work boundary during the transition.
- **Planned:** Keep Codex workflow contracts in the full gate after migration. **Changed:** Remove them in PR 2 and run product suites directly. **Why:** Fixed inventories, text markers, and CI YAML copies test development process rather than FM ValueScout behavior.

## Completed work

| PR | Commit | Hash | Notes |
| --- | --- | --- | --- |
| PR 1 | Document the Codex migration contract | `8c8d7ac` | Added the approved delivery plan and marked the tooling initiative active. |
| PR 1 | Add Codex project configuration | `187c635` | Added Recallium and Context7 project MCP configuration with gate validation. |
| PR 1 | Port repository skills to Codex | `eaba444` | Added copied domain skills, `workflow-*` skills, and their validation contract. |
| PR 1 | Port specialist agents to Codex | `a8cb615` | Added read-only reviewer and documentation-only steward definitions, workflow dispatch, and boundary validation. |
| PR 1 | Make Codex the documented workflow | `73b5dad` | Added Codex workflow guidance, contributor terminology, and documentation contract validation. |
| PR 1 | Remove Cursor compatibility | `495edea` | Removed the retired Cursor surface and replaced its contracts with Codex-only validation. |

## Final validation

At feature end.

## Documentation impact

The migration will update repository workflow guidance, contributor documentation, architecture references, contract scripts, and CI fixture setup. It does not change product documentation or application behavior.
