# Development Contract

This file contains the standing repository contract. Detailed procedures belong in installed `workflow-*` skills. Project facts belong in `.wiki/`. Hard validation belongs in repository commands, tests, and CI.

## Project scope

This template defaults to hobbyist solo-developer scope. A derived project can override this in CONCEPT.md.

Scope changes what "careful" means. Structural quality — clean architecture, clear names, correct logic — does not change. Those are always the goal. What changes is the amount of defensive padding around each path.

A solo dev does not need null guards on every reference, fallback for every error path, or resilience against failures that exist only in theory. These layers of paranoia take more time to write than the bug they prevent would take to fix.

Specific calibrations:

- **Correctness.** The code must be correct for its stated contract. It does not need to survive every conceivable input.
- **Tests.** Cover critical paths and data-loss risks. Skip tests for code that is straightforward and unlikely to break.
- **Review.** Every non-trivial change gets a reviewer pass. Trivial changes (copy edit, button move, rename) do not. The reviewer exists to catch structural mistakes, not to enforce ceremony.
- **Abstraction.** The solo dev knows the whole codebase. Premature abstraction costs more than refactoring when the need arrives.
- **Documentation.** Write what you need after a 3-month break. Not what a new hire needs.
- **Ceremony.** If a practice takes longer than the bug it prevents, skip it. You can fix issues when they appear.

When a prompt or agent definition asks for ceremony — ADRs, ponytail comments, formal review passes, multi-layer defense-in-depth, full documentation reconciliation — calibrate it to this scope. If the practice produces cleaner structure, keep it. If it guards against a failure that will never happen in practice, drop it.

## Read order

For broad, architectural, risky, or multi-commit work:

1. Read `.wiki/INDEX.md` and relevant current-state documents.
2. Read the active feature ledger in `.wiki/features/active/`, when one exists.
3. Inspect the relevant implementation and tests.

For ordinary work, inspect the relevant code and tests before changing them.

## Classify before changing

- **Trivial:** inspect, make the focused change, run the fast gate, review.
- **Behavioural:** state a concise work contract and impact map. Use RED → GREEN → REFACTOR. Run affected validation.
- **Architectural:** explicitly invoke `$workflow-plan-feature`, identify risks and decisions, create one active feature ledger, and implement one commit at a time.

Plans are provisional. Reassess remaining commits after each one. Ask only product questions that repository evidence or a bounded technical spike cannot answer.

**Unresolved structural decisions** (persistence, schema, migration, authentication, concurrency, security, public API, or layer boundaries): read `.wiki/ARCHITECTURE.md`, relevant ADRs and debug reports, the active or completed feature records, and matching installed skills. Inspect targeted Git history when current files do not explain the rationale. If evidence is still insufficient and the question needs a **runtime probe**, the developer can explicitly invoke **`$workflow-spike`**. Otherwise, ask the developer. Do not guess and do not implement.

## Guidance layers

Keep guidance in the narrowest appropriate layer:

- `.wiki/CONCEPT.md` owns product purpose and boundaries.
- `.wiki/ARCHITECTURE.md` owns the current implemented system, including its stack and operational constraints.
- `.wiki/features/active/` owns current multi-commit feature intent and delivery plans (PRs and commits).
- The installed `workflow-core` skill owns shared lifecycle, routing, review, escalation, and PR-boundary policy.
- Other installed `workflow-*` skills own explicit commands such as `$workflow-plan-feature`, `$workflow-build`, and `$workflow-checkpoint`. Each command loads `workflow-core` before continuing.
- Installed non-workflow skills own reusable task- or stack-specific operating guidance.
- `.codex/agents/` owns specialist role prompts. It must not duplicate this contract.

Project-owned guidance governs project facts and constraints. When reusable skill guidance conflicts with an applicable project document, follow the project document. Skills govern procedure and provide defaults where project guidance is silent. Code, tests, and configuration remain authoritative for current executable behavior. Current-state documents take precedence over plans and historical records; ADRs explain accepted rationale until superseded but do not override current behavior.

Do not treat `.work/` as project truth. Do not document proposed behaviour as implemented. Use the existing wiki ownership rules rather than duplicating facts across documents.

## Development workflow

The development cycle follows a repeating loop. Invoke a `workflow-*` skill explicitly through `/skills` or by mentioning `$workflow-<name>`. Never select a workflow skill from an ordinary natural-language request.

1. **Feature plan** (`$workflow-plan-feature`) — plan one feature with PR and commit breakpoints, implementation packets, validation contracts, and separate implementation and review profiles.
2. **Build** (`$workflow-build`) — implement the active commit test-first (RED → GREEN → REFACTOR).
3. **Checkpoint** (`$workflow-checkpoint`) — stage exact changes, run the gate, present evidence and review, wait for approval, and commit locally.
4. **Fix** (`$workflow-fix`) — address only the findings the developer delegates, then checkpoint again.
5. **Reassess** — update the delivery plan and select the next commit; repeat from build until the plan is done.
6. **Finish feature** (`$workflow-finish-feature`) — when every planned commit is done, run full tests, the ledger-selected feature-complete review (Sol High for a legacy ledger), and documentation reconciliation.

`$workflow-build-feature-loop` is the explicit Goal-mode opt-in for repeating the complete `$workflow-build-loop` contract across the remaining commits in one active PR. It stops at publication, implementation completion, replanning, failed review, or a required developer decision. It never runs feature close-out.

For a trivial change, the user can describe the fix without invoking a workflow skill. Follow the applicable standing rules internally.

The loop variants are manual opt-ins only. Never suggest or run them automatically. Their documented local commit permissions come from this file; they do not authorize pushes, merges, or history rewrites.

For broad features, `$workflow-plan-feature` produces a delivery plan before the first `$workflow-build` cycle. `$workflow-stack` and `$workflow-roadmap` precede it for new projects.

Use the exact implementation and review profiles recorded in the active ledger. Review runs in a fresh context and retains a defect only when it has a violated contract, a concrete execution path, and an observable consequence. Follow the installed `workflow-core` skill and the relevant user-facing workflow skill for routing, hard floors, evidence requirements, and escalation.

## Commands and validation

```bash
./scripts/dev test [target...]
./scripts/dev check
./scripts/dev check-app
./scripts/dev bridge-test
./scripts/dev format [paths...]
./scripts/dev secrets [--staged]
./scripts/dev smoke
./scripts/dev mutate <target...>
./scripts/dev bridge-install
```

`check` is the commit gate: Biome verify (`biome check`), TypeScript, secretlint, and Rust format, lint, and tests. `check-app` runs its frontend part only for CI. `bridge-test` runs the C# bridge unit suite and requires the .NET 6 SDK. Run `pnpm exec playwright install chromium` once after install, then use `smoke` for the Playwright product suite (`e2e/smoke.spec.ts`). `format` applies Biome lint and format fixes (`biome check --write`), then `cargo fmt` in `src-tauri/` — run before staging during `$workflow-build` and `$workflow-checkpoint`; it is not part of the gate. Optional path args forward to Biome only. `secrets` runs secretlint on the full tree, or on staged files with `--staged` (no lint-staged). `mutate` is unsupported until mutation tooling is wired into `scripts/dev`. Never report an unsupported command as passed. `bridge-install` builds the C# FM plugin and copies `FmDataBridge.dll` into BepInEx plugins (see `bridge/README.md`; path via `FM_BRIDGE_PLUGINS` / `FM_STEAM_ROOT` / WSL Steam default).

`test` runs `vitest run` (full suite or forwarded args). CI selects frontend, browser, Rust, and bridge product checks from the changed paths. Its required `check` status aggregates the applicable results. Desktop installer builds run only from the release workflow.

Use stack-native commands only through the stable `scripts/dev` surface.

For non-trivial behaviour:

- Write the smallest meaningful test first and run it RED.
- Confirm RED fails for the expected missing behaviour, not setup or syntax.
- Make the smallest coherent change GREEN, then refactor only while green.
- Run affected existing tests and `./scripts/dev check`.
- Add proportionate negative, boundary, or failure coverage.
- Use deliberate perturbation or scoped mutation testing for critical logic when available.

Prompts guide the workflow. Deterministic commands and tests provide evidence. Do not weaken, delete, skip, or broadly rewrite tests merely to make a change pass.

Increase reasoning effort when the model has the right architecture but incomplete execution. After two failed correction attempts on the same bounded defect, stop and request a profile change or replan. Replan sooner when a known fact, invariant, architectural seam, persisted or public contract, validation contract, PR boundary, or cross-feature dependency changes. Use `$workflow-spike` only for a genuine runtime unknown and only when the developer explicitly invokes it. Every non-trivial staged change requires a separate fresh-context read-only reviewer pass with the ledger-assigned review profile, or the default Sol Medium reviewer when no ledger exists. Feature-complete review uses the ledger's feature review profile, or Sol High for a legacy ledger without that field.

## Design and execution

This section adds rules for how to make implementation choices, how to change existing code, and how to present results. These rules bias toward caution over speed. For trivial tasks, use judgment.

### Think first

Before you implement, state your assumptions. If the request has more than one interpretation, present the options. Do not pick one in silence. If the simplest approach is not what was asked for, say so. If anything is unclear, stop and ask. Confusion costs less than rework.

### Decision ladder

Every implementation choice follows an ordered protocol. Start at rung one. Descend only when the current rung does not solve the problem.

1. **YAGNI.** Does this need to exist? Could you remove something instead of adding something? Challenge every "should". Only "must" survives.
2. **Standard library.** Does the language or runtime already ship with this? Check before you import anything new.
3. **Native platform.** Does the browser, OS, or platform already provide this? The platform is free. Use it.
4. **Existing dependency.** Does a package already in the project provide this? Do not add a new dependency when one you already have covers the need.
5. **One line.** Can one clear line of code solve it? If it can, do not write a function, a class, or an abstraction. If the one line is unreadable — nested ternary, chained regex, three or more conditions — drop to rung six.
6. **Minimum code that works.** Write the shortest implementation that passes all tests. No extension points. No configurability. No hooks for future needs. Add those when the need arrives.

The ladder is a reflex, not a research project. Each rung takes seconds. If you spend minutes debating whether a rung applies, drop to the next one and move on.

Ask yourself: would a senior engineer call this overcomplicated? If yes, simplify. If you wrote 200 lines and it could be 50, rewrite it.

### Scope discipline

Touch only what the request requires. Every changed line must trace directly to the stated outcome.

- Do not improve adjacent code, comments, or formatting. Leave surrounding code as you found it.
- Match the existing style, even where you would do it differently.
- When your changes make an import, variable, or function unused, remove it. Do not remove pre-existing dead code unless the request includes it.
- If you notice unrelated dead code, mention it. Do not delete it.
- Do not refactor things that are not broken.

### Implementation rigour

**Tradeoff comments.** When you accept a shortcut on purpose, name the limit and the upgrade trigger in a structured comment:

```
# ponytail: <what was skipped or simplified>
# Upgrade to <what to build instead> if <measurable trigger condition>
```

The ponytail is not a TODO or a permission slip for bugs. It names a ceiling. The upgrade trigger is measurable. These comments are grep-able: `grep -r "ponytail:" src/` produces a debt ledger.

**Safety carve-outs.** These domains are never subject to the ladder. Always invest full rigour:

- Input validation at trust boundaries
- Error handling that prevents data loss
- Security controls
- Accessibility
- Hardware calibration
- Anything the user explicitly asked for

**Goal-driven execution.** Turn each task into a verifiable goal. Write the test before the code. State a brief plan with a verification check for each step:

```
1. [step] → verify: [check]
2. [step] → verify: [check]
```

Strong verification lets you loop without asking for clarification.

**Tests are not bloat.** A test is the discipline that makes minimalism safe. Protect non-trivial behavior with a runnable assertion at the seam where a plausible regression would be observable. Trivial expressions and behavior already proved by a stronger test need no duplicate test. Every security-critical path must have a test.

### Output and review

**Output discipline.** When you present completed work, show the code first. Then at most three lines: what was skipped and when to add it. Do not write a paragraph describing what the code does — the code says that. Do not list every function added — the diff shows that.

If the explanation is longer than the code, delete the explanation.

Exceptions: commit messages (use the global `conventional-commits` skill and explain *why* in the body when the reason is not obvious from the diff), security decisions, architectural notes in durable docs, and explicit user requests for detail.

**Honesty boundaries.** Do not claim per-repo line savings — the unbuilt version is imaginary, so there is no baseline. Do not claim a performance improvement without before-and-after measurements. Do not claim 100% test coverage from line coverage alone. Do not call a change a bug fix unless you confirmed the old behaviour was wrong.

**Self-referential.** These rules apply to changes in AGENTS.md itself. Before you add a section, ask: does it reduce ambiguity or create it? Will it age well? Is it covered by an existing rule already?

## Scope and Git

- Keep each active commit focused on one coherent, revertible outcome.
- Include directly related production code, tests, and documentation together.
- Do not perform unrelated cleanup.
- Stage exact files or hunks. Never use `git add .` or `git commit -a`.
- Before commit, inspect status and the complete diff, run `git diff --cached --check`, review the staged diff and stat, and report tests, gate results, documentation impact, reviewer findings, risks, and the proposed commit message.
- Wait for explicit developer approval before committing locally.
- Explicitly invoking `$workflow-build-loop` authorizes its documented single local content commit after review clears. Explicitly invoking `$workflow-build-feature-loop` authorizes one reviewed local content commit for each Active or Pending commit in its preflight active-PR scope. Explicitly invoking `$workflow-finish-feature-loop` authorizes its documented local correction and documentation commits. No other workflow invocation grants commit approval.
- Never push, amend, rebase, squash, or otherwise rewrite history without explicit approval.

## Project knowledge

The repository is the complete source of project knowledge. Do not rely on chat history, an external memory service, or an untracked index for unique facts.

- Read the installed `project-context` skill before a non-obvious decision, when resuming multi-session work, or when deciding where new durable knowledge belongs.
- Inspect the current-state owner, relevant feature records, ADRs, debug reports, implementation, tests, and targeted Git history as needed. Search narrowly before reading broadly.
- Use the installed `repowise` skill for indexed architecture discovery, symbol relationships, rationale, code health, defect risk, change impact, dead code, and coverage-backed test selection when the MCP or CLI is available. Treat its index and scores as advisory, respect stale warnings, verify conclusions against current repository evidence, and fall back to direct inspection without blocking the task.
- Update the narrowest owner in the same change that makes the information true. Current product, architecture, and design facts belong in their wiki documents. Active discoveries and deviations belong in the feature ledger.
- Create an ADR only for a consequential decision with durable effects, meaningful alternatives, and non-obvious rationale.
- Treat regression tests as the primary record of ordinary bugs. Add `.wiki/debugging/` reports only for confirmed, reusable failure patterns or diagnostic procedures that code and tests do not explain.
- Keep temporary evidence, failed hypotheses, raw logs, and experiment artifacts in `.work/`; remove them during cleanup.

## Documentation boundaries

Follow `.wiki/INDEX.md` and the installed `project-context` skill for documentation ownership. Trivial changes normally need no wiki update. Update durable documentation when externally meaningful behaviour, commands, configuration, contracts, persistent-data assumptions, or architecture changes. Multi-commit work gets one active feature ledger. Architectural or schema work updates architecture and receives an ADR only when justified. Confirmed reusable failure patterns receive a short debug report only when the regression test and commit do not explain enough. Reconcile and archive feature documentation at feature completion rather than after every minor change.

The Documentation Steward may change documentation and feature-ledger state, but must not change implementation, tests, executable scripts, CI, Codex configuration, agent definitions, command templates, or Git state.

The main session plans established feature work and implements all build and fix work. Delegate planning only when the developer explicitly requests it. Dispatch the `reviewer` and `documentation-steward` specialists explicitly. Every initial review of non-trivial work uses a separate fresh reviewer context with the assigned review profile. After a fix, reuse that reviewer context when available unless the correction materially changes the review scope or architecture; otherwise dispatch a fresh reviewer. See `.codex/README.md` for role selection and MCP details.

When you need current library API or configuration details, use Context7 MCP (`resolve-library-id`, then `query-docs`). Use web search and fetch for bounded external research. Never put credentials in repository files.
