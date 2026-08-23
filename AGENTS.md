# Development contract

This file contains FM ValueScout's standing repository contract. Detailed procedures belong in installed skills. Project facts belong in `.wiki/`. Repository commands, tests, and CI own hard validation.

## Project scope

FM ValueScout is a solo-maintained hobby project. Prefer direct solutions, deletion, and consolidation. Structural quality still matters: keep the architecture clean, names clear, and logic correct. Avoid defensive complexity for failures that the supported contract does not require.

Apply these calibrations:

- **Correctness.** Code must satisfy its stated contract. It does not need to survive every conceivable input.
- **Tests.** Protect critical paths, trust boundaries, data-loss risks, and realistic regressions. Skip duplicate tests for straightforward behavior already proved at a stronger seam.
- **Review.** Give every non-trivial change a fresh reviewer pass. Use review to find structural mistakes, not to add ceremony.
- **Abstraction.** Prefer direct code until a current need justifies an abstraction.
- **Documentation.** Write what a maintainer needs after a long break, not a development diary.
- **Ceremony.** Keep a practice when it improves structure or protects a demonstrated risk. Drop it when it only guards against a theoretical failure.

Calibrate reusable prompts and skills to this scope. Safety, security, accessibility, trust boundaries, and data-loss prevention still require full rigor.

## Repository memory

Use the repository as the complete source of project knowledge. Do not depend on chat history, an external memory service, or an untracked index for unique facts.

For broad, architectural, risky, or multi-commit work:

1. Read `.wiki/INDEX.md` for document ownership.
2. Read the relevant current-state document and active feature ledger.
3. Inspect implementation, tests, validation configuration, and focused Git history when needed.

For ordinary work, inspect the relevant code and tests before changing them. Read the installed `project-context` skill before a non-obvious decision, when resuming work, or when deciding where durable knowledge belongs. Use the installed `code-intelligence` skill when architecture, symbol relationships, impact, diagnostics, or review evidence can materially change the work; verify its results against source and deterministic checks.

Keep guidance in the narrowest owner:

- `.wiki/CONCEPT.md` owns product purpose and boundaries.
- `.wiki/ARCHITECTURE.md` owns implemented structure, data flow, persistence, trust boundaries, and operational constraints.
- `.wiki/DESIGN.md` owns visual and interaction conventions.
- `.wiki/TODO.md` owns committed or imminent feature work; `.wiki/BACKLOG.md` owns deferred work.
- `.wiki/features/active/` owns current multi-commit intent, delivery plans, discoveries, and deviations.
- Installed `workflow-*` skills own reusable procedures; `workflow-core` owns their shared lifecycle and authority rules.
- Installed non-workflow skills own reusable task, stack, writing, and security guidance.
- `.work/` holds temporary evidence only and is not project truth.

Project documents govern project facts and constraints. Skills govern procedure and provide defaults where project guidance is silent. Code, tests, and configuration describe executable behavior. Current-state documents take precedence over plans and historical records; ADRs explain accepted rationale but do not override current behavior.

Update the narrowest owner in the change that makes the information true. Create an ADR only for a durable consequential decision with meaningful alternatives. Treat regression tests as the primary record of ordinary bugs. Add a debug report only for a confirmed reusable failure pattern or diagnostic procedure that code and tests do not explain. Remove temporary `.work/` evidence during cleanup.

## Work classification

- **Trivial:** make one focused change, run the fast gate, review the diff, and ask before committing.
- **Behavioral:** state a concise work contract and impact map, use RED → GREEN → REFACTOR for changed behavior or contract-removal proof for deletion, and run affected validation.
- **Feature:** explicitly invoke `/skill:workflow-plan-feature`, accept one schema 2 ledger, then run `/skill:workflow-deliver-feature <ledger>`.

Workflow skills are explicit opt-ins. Never invoke one only because an ordinary request resembles its task. Plans are provisional; reassess remaining work as repository facts change.

For unresolved persistence, schema, migration, authentication, concurrency, security, public API, safety-critical, or architecture decisions, inspect `.wiki/ARCHITECTURE.md`, relevant ADRs and debug reports, active and completed feature records, implementation, tests, matching skills, and focused Git history. If evidence remains insufficient, ask the developer. Do not guess and do not implement.

## Feature delivery

One accepted ledger owns feature intent, PRs, commit packets, validation, publication, and close-out. Releases are separate, explicit work.

`/skill:workflow-deliver-feature` is the normal execution path. One explicit invocation uses the ledger's validated Delivery fingerprint to:

1. activate each recorded branch;
2. implement, validate, independently review, and commit every packet;
3. publish, monitor, repair, and merge every PR;
4. synchronize the base before dependent work;
5. run feature validation and documentation close-out before the final merge.

The workflow stops for changed authority, replanning, a developer decision, exhausted correction limits, failed required validation, missing approval, conflicts, or a stale PR head. It does not stop at ordinary commit, PR, or close-out boundaries.

The narrower workflow skills remain available for manual recovery and isolated review. Do not require them between normal delivery phases.

## Specialists

The main Pi session is the supervisor. Workflow skills launch the exact globally installed PI_SETUP roles as direct subagents.

- One writer may edit the active worktree.
- Workers cannot stage, commit, switch branches, push, or mutate GitHub.
- Every non-trivial implementation and correction receives a fresh read-only reviewer pass.
- Any subagent may delegate a bounded subtask. The delegating agent verifies the result and cannot widen workflow authority.
- Role boundaries are prompt instructions, not a sandbox.
- The `documentation-steward` may edit only explicitly approved documentation paths. It must not change implementation, tests, executable scripts, CI, agent definitions, command templates, or Git state.

## Commands and validation

Use the stable `./scripts/dev` surface instead of stack-native commands:

```bash
./scripts/dev test [target...]
./scripts/dev check-fast
./scripts/dev check
./scripts/dev check-app
./scripts/dev check-rust
./scripts/dev bridge-test
./scripts/dev format [paths...]
./scripts/dev secrets [--staged]
./scripts/dev smoke
./scripts/dev mutate <target...>
./scripts/dev bridge-install
./scripts/dev package-windows
./scripts/dev release-metadata [latest-tag|none] [release-intent]
```

`check` is the full commit gate: Biome, TypeScript, secretlint, Rust format, Clippy, and Rust tests. `check-fast` is the pre-commit frontend and staged-secret path; it does not replace `check`. `check-app` runs the frontend CI gate. `check-rust` runs the Rust gate. `bridge-test` requires the .NET 6 SDK. Install Chromium once with `pnpm exec playwright install chromium`, then use `smoke` for the Playwright product suite.

`format` applies Biome fixes and `cargo fmt` before staging. `mutate` is unsupported until mutation tooling is configured and must never be reported as passed. `bridge-install` builds and installs `FmDataBridge.dll` using `FM_BRIDGE_PLUGINS`, `FM_STEAM_ROOT`, or the WSL Steam default. `package-windows` runs only on Windows, creates one unsigned x64 NSIS validation artifact plus checksum under `.release/windows/<version>/`, and does not publish.

CI selects frontend, browser, Rust, bridge, and CI checks from changed paths. The required `check` status aggregates applicable results. It does not validate release metadata or package Windows installers. The Release workflow starts only after an explicit release-preparation change reaches `main`; it waits for that exact `check` before packaging and publishing.

For every human-authored pull request, use `.pi/skills/create-pr/SKILL.md` with `.github/pull_request_template.md`. Ordinary pull requests do not classify or prepare a release. Use `.pi/skills/create-release/SKILL.md` only when the developer explicitly requests a release. `release-metadata` validates prepared version and changelog state without writing files or calling GitHub.

For changed behavior:

1. write the smallest meaningful test and confirm it fails for the expected missing or wrong behavior rather than setup or syntax;
2. implement the smallest coherent change that makes the proof pass;
3. refactor only while the focused proof stays green;
4. add proportionate negative, boundary, or failure coverage where a realistic regression warrants it; and
5. run affected tests and `./scripts/dev check`.

For intentional removal, remove obsolete implementation, tests, fixtures, mocks, snapshots, helpers, and compatibility paths together. Prove surviving behavior. Add an absence test only when observable reintroduction is plausible.

Tests must protect supported behavior and fail for a realistic wrong implementation. Do not weaken, delete, skip, or broadly rewrite tests merely to make a gate pass. Commands and tests provide evidence; prompts and confidence do not.

After two failed correction attempts for the same bounded defect, stop and reassess. Replan sooner when a known fact, invariant, architectural seam, persisted or public contract, validation contract, PR boundary, or cross-feature dependency changes.

## Design and execution

### Think first

State material assumptions before implementation. When a request has more than one plausible interpretation, present the options instead of choosing silently. If the simplest approach does not satisfy the request, say so. Ask when repository evidence cannot resolve a consequential ambiguity.

### Decision ladder

Apply this order and stop at the first sufficient option:

1. Delete or avoid the change when it is not needed.
2. Use the standard library or native platform.
3. Reuse an existing dependency.
4. Use one clear line when it stays readable.
5. Write the minimum code that satisfies the current contract.

Do not add extension points, configurability, or abstractions for hypothetical needs. Full rigor remains mandatory for input validation at trust boundaries, error handling that prevents data loss, security controls, accessibility, hardware calibration, and anything the developer explicitly requests.

When you deliberately accept a meaningful shortcut, record its limit and measurable upgrade trigger:

```text
# ponytail: <what was skipped or simplified>
# Upgrade to <what to build instead> if <measurable trigger condition>
```

A ponytail is not a TODO or permission for a defect.

### Scope discipline

Touch only what the requested outcome requires. Every changed line must trace to that outcome.

- Do not improve adjacent code, comments, or formatting.
- Match current repository style and the closest sound analogue.
- Remove imports, variables, helpers, or tests made obsolete by the change.
- Do not remove unrelated pre-existing dead code; report it instead.
- Do not refactor code that the requested outcome does not require.
- Keep each change coherent and revertible.

### Goal and proof

Turn non-trivial work into a verifiable goal. State a short plan with a proof for each step. Protect behavior at the seam where a plausible regression is observable. A stronger existing test can make a duplicate test unnecessary, but every security-critical path needs direct protection.

Preserve useful error context without exposing secrets. Validate untrusted input at its boundary. Update documentation only when a change alters a documented contract, command, architecture, operational rule, or feature state.

### Output and honesty

Present completed work concisely: lead with changed paths or the resulting behavior, then give validation and any remaining gap. Do not narrate code that the diff already explains.

Do not claim line savings against code that never existed, performance gains without before-and-after measurements, complete coverage from line coverage alone, or a bug fix without confirming that the prior behavior was wrong.

These rules apply to edits of `AGENTS.md` itself. Before adding or removing guidance, ask whether the change reduces ambiguity, preserves repository-specific policy, and will remain accurate.

## Attachments

When the developer attaches a screenshot or image, use its mounted `/mnt/...` path. Translate a supplied Windows drive path to its WSL mount when possible, for example `C:\path\image.png` to `/mnt/c/path/image.png`. If no mounted path is available, ask for the file again.

## Git and delivery authority

Commit and PR titles use Conventional Commits: `type(scope): imperative description`, under 72 characters, with no trailing period. Package and release versions use Semantic Versioning without a `v` prefix; Git tags may use `v`.

Do not commit, push, create or update PRs, merge, create branches, synchronize remotes, rewrite history, or create releases without explicit authority.

The normal exception is an explicit `/skill:workflow-deliver-feature` invocation with a valid recorded Delivery fingerprint. That fingerprint covers exact PR authority fields and commit packets. It grants only the recorded branches, reviewed local commits, non-force pushes, exact PRs, bounded reviewed CI repairs, verified-head merges with the recorded method, and fast-forward-only base synchronization. It does not authorize a release.

Any authority or packet change invalidates the grant. It never permits amend, rebase, force-push, protection bypass, self-approval, branch deletion, unrelated work, another ledger, or a second release.

Stage exact files or hunks. Never use `git add .` or `git commit -a`. Before every commit, inspect status, the complete staged diff and stat, and run `git diff --cached --check` plus recorded validation. Report documentation impact, reviewer findings, risks, and the proposed commit message. Manual work waits for explicit developer approval before committing.

## Project-owned releases

PI_SETUP delivery ends after the final PR merges and the base is synchronized. A feature merge never publishes by default.

Only an explicit `/skill:create-release` invocation can prepare a release. It inspects all changes since the latest published tag, proposes the SemVer scope, updates the version owners and changelog, increments `release-preparation.json`, and opens a dedicated release PR. The Release workflow creates the tag and GitHub release only after that PR merges and the exact `main` Check passes. Do not infer a version, tag, provider action, or deployment.

## Security

Pi packages and tools execute with the user's operating-system privileges. Project trust and role prompts are controls, not an operating-system sandbox.

Do not read or print secret values, private keys, production dumps, local authentication stores, or unrelated personal data. Keep credentials out of repository files, ledgers, templates, prompts, and logs.
