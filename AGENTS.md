# Development contract

This file contains FM ValueScout's standing repository contract. Reusable procedures belong in the globally installed jay-claude plugin; project-specific procedures belong in `.claude/skills/`. Project facts belong in the local-only `.wiki/`. Repository commands, tests, and CI own hard validation.

## Project scope

FM ValueScout is a solo-maintained hobby project. Prefer direct solutions, deletion, and consolidation. Structural quality still matters: keep the architecture clean, names clear, and logic correct. Avoid defensive complexity for failures that the supported contract does not require.

Apply these calibrations:

- **Correctness.** Code must satisfy its stated contract. It does not need to survive every conceivable input.
- **Tests.** Protect critical paths, trust boundaries, data-loss risks, and realistic regressions. Skip duplicate tests for straightforward behavior already proved at a stronger seam.
- **Review.** Independent review is mandatory inside an explicitly invoked jay-claude workflow. Outside one, inspect your own diff and launch a reviewer only when the developer asks; that request must also cover any re-review after fixes. Use review to find structural mistakes, not to add ceremony.
- **Abstraction.** Prefer direct code until a current need justifies an abstraction.
- **Documentation.** Write what a maintainer needs after a long break, not a development diary.
- **Ceremony.** Keep a practice when it improves structure or protects a demonstrated risk. Drop it when it only guards against a theoretical failure.

Calibrate reusable prompts and skills to this scope. Safety, security, accessibility, trust boundaries, and data-loss prevention still require full rigor.

## Repository memory

Use this checkout, its tracked files plus the local-only `.wiki/`, as the complete source of project knowledge. Do not depend on chat history or an external memory service for unique facts.

`.wiki/` is excluded from Git and exists only in this checkout; a fresh clone has none, and Git history holds only the copy from before it was untracked. Never force-add it or `.work/` (`git add -f`). Tracked files may point local readers at wiki paths, but builds, tests, and CI must not depend on the wiki. jay-claude's `scripts/local_delivery.py` backs up the wiki outside the repository whenever it saves delivery state.

For broad, architectural, risky, or multi-commit work:

1. Read `.wiki/INDEX.md` for document ownership.
2. Read the relevant current-state document and, for feature work, the feature's DESIGN and IMPLEMENTATION documents.
3. Inspect implementation, tests, validation configuration, and focused Git history when needed.

For ordinary work, inspect the relevant code and tests before changing them. Read the installed `project-context` skill before a non-obvious decision, when resuming work, or when deciding where durable knowledge belongs. Use the installed `code-intelligence` skill when architecture, symbol relationships, impact, diagnostics, or review evidence can materially change the work; verify its results against source and deterministic checks.

Keep guidance in the narrowest owner:

- `.wiki/CONCEPT.md` owns product purpose and boundaries.
- `.wiki/ARCHITECTURE.md` owns implemented structure, data flow, persistence, trust boundaries, and operational constraints.
- `.wiki/DESIGN.md` owns visual and interaction conventions.
- `.wiki/TODO.md` owns committed or imminent feature work; `.wiki/BACKLOG.md` owns deferred work.
- `.wiki/features/<feature>/` holds each feature's `<FEATURE>-DESIGN.md` (requirements) and `<FEATURE>-IMPLEMENTATION.md` (PR and package plan, minimal progress, and discoveries). `.wiki/features/completed/` keeps historical records from earlier workflows only.
- The jay-claude plugin's workflow skills (`/jay-claude:<skill>`) own reusable delivery procedures; its `core` skill owns their shared lifecycle, authority, and review rules.
- The plugin's other skills own reusable task, stack, writing, and security guidance.
- Project skills in `.claude/skills/` own project-specific procedures: `create-release` and `ui-inspection`.
- `.work/` holds temporary evidence and jay-claude run state and is not project truth. Keep jay-claude snapshots, journals, and unintegrated worker results until their delivery is reconciled.

Project documents govern project facts and constraints. Skills govern procedure and provide defaults where project guidance is silent. Code, tests, and configuration describe executable behavior. Current-state documents take precedence over plans and historical records; ADRs explain accepted rationale but do not override current behavior.

Update the narrowest owner in the change that makes the information true. Create an ADR only for a durable consequential decision with meaningful alternatives. Treat regression tests as the primary record of ordinary bugs. Add a debug report only for a confirmed reusable failure pattern or diagnostic procedure that code and tests do not explain. Remove temporary `.work/` evidence during cleanup.

## Work classification

- **Trivial:** make one focused change, run the fast gate, review the diff, and ask before committing.
- **Behavioral:** state a concise work contract and impact map, use RED → GREEN → REFACTOR for changed behavior or contract-removal proof for deletion, and run affected validation.
- **Feature:** explicitly invoke `/jay-claude:design-feature <feature>` and accept the DESIGN, then `/jay-claude:plan <feature>` and accept the reviewed schema 4 ledger, then run `/jay-claude:deliver-feature .wiki/features/<feature>/<FEATURE>-IMPLEMENTATION.md`.

Workflow skills are explicit opt-ins. Never invoke one only because an ordinary request resembles its task. An IMPLEMENTATION file's presence does not activate one. Plans are provisional; reassess remaining work as repository facts change.

For unresolved persistence, schema, migration, authentication, concurrency, security, public API, safety-critical, or architecture decisions, inspect `.wiki/ARCHITECTURE.md`, relevant ADRs and debug reports, relevant feature documents, implementation, tests, matching skills, and focused Git history. If evidence remains insufficient, ask the developer. Do not guess and do not implement.

## Feature delivery

A feature's accepted DESIGN and IMPLEMENTATION ledger own its requirements, PR and package plan, and minimal progress. Releases are separate, explicit work.

`/jay-claude:deliver-feature <ledger>` is the normal execution path. One explicit invocation runs the accepted plan's waves, packages, reviews, PRs, CI repairs, and merges, continuing across PR boundaries without waiting for the developer. The plugin's `core` and `deliver-feature` skills own the procedure, authority, review points, correction limits, and stop conditions. The narrower `build`, `checkpoint`, `fix`, `finish-feature`, `sync-docs`, and `publish-pr` skills are recovery tools.

Plans for this repository record these project facts:

- **Ignored wiki.** Because `.wiki/` is excluded from Git, an accepted plan is frozen with `python3 scripts/local_delivery.py accept <ledger> <repository-root> --approved` instead of a commit, and delivery follows the plugin's ignored-wiki reference. No package write scope may include `.wiki/` or `.work/`.
- **Publication.** GitHub, `.github/pull_request_template.md`, squash merge, and the required `check` status.
- **Worker setup.** Worker worktrees contain only tracked files, and `./scripts/dev` refuses to run without `node_modules`. Each package's Validation starts with `pnpm install --frozen-lockfile --prefer-offline` as approved setup. Rust packages rebuild `src-tauri/target` in each worktree. Bridge packages that reference FM types also need the untracked `bridge/Directory.Build.user.props`.
- **Shared resources.** `smoke` and `inspect-ui` start Vite on fixed port 5173, and Playwright reuses any server already listening there outside CI. Never run them in two checkouts at once; run them serially in the integration checkout.

## Specialists

The main Claude Code session is the coordinator. jay-claude workflows launch the plugin's roles (`jay-claude:worker`, `jay-claude:reviewer`, `jay-claude:researcher`) as subagents, and the `security-audit` skill launches `jay-claude:security-auditor`. Each role's contract owns its boundaries. Role boundaries are prompt instructions, not a sandbox.

## Commands and validation

Use the stable `./scripts/dev` surface instead of stack-native commands:

```bash
./scripts/dev test [target...]
./scripts/dev check-fast
./scripts/dev check
./scripts/dev check-app
./scripts/dev check-rust [test-filter...]
./scripts/dev bridge-test
./scripts/dev format [paths...]
./scripts/dev secrets [--staged]
./scripts/dev smoke
./scripts/dev inspect-ui [route|all] [width] [height]
./scripts/dev mutate <target...>
./scripts/dev bridge-install
./scripts/dev package-windows
./scripts/dev release-metadata [latest-tag|none] [release-intent]
./scripts/dev graphics-calibration
```

`check` is the full commit gate: Biome, TypeScript, secretlint, Rust format, Clippy, and Rust tests. `check-fast` is the pre-commit frontend and staged-secret path; it does not replace `check`. `check-app` runs the frontend CI gate. `check-rust` runs the Rust gate; optional filters limit its tests to matching names. `bridge-test` requires the .NET 6 SDK. Install Chromium once with `pnpm exec playwright install chromium`, then use `smoke` for the Playwright product suite.

For material UI work, read the project `ui-inspection` skill in `.claude/skills/` rather than the generic `jay-claude:ui-inspection`, capture the affected route, and inspect the reported image. Use its complete page set only for cross-page changes. Treat every capture as disposable Chromium-stub evidence, not native application proof.

`format` applies Biome fixes and `cargo fmt` before staging. `mutate` is unsupported until mutation tooling is configured and must never be reported as passed. `bridge-install` builds and installs `FmDataBridge.dll` using `FM_BRIDGE_PLUGINS`, `FM_STEAM_ROOT`, or the WSL Steam default. `package-windows` runs only on Windows, creates one unsigned x64 NSIS validation artifact plus checksum under `.release/windows/<version>/`, and does not publish.

CI selects frontend, browser, Rust, bridge, and CI checks from changed paths. The required `check` status aggregates applicable results. It does not validate release metadata or package Windows installers. The Release workflow starts only after an explicit release-preparation change reaches `main`; it waits for that exact `check` before packaging and publishing.

Every pull request completes `.github/pull_request_template.md`; its title and body become the squash commit. Ordinary pull requests never edit release version owners, dated `CHANGELOG.md` sections, or `release-preparation.json`, and never run `release-metadata` as a classification step. Use the project `create-release` skill only when the developer explicitly requests a release. `release-metadata` validates prepared version and changelog state without writing files or calling GitHub. `graphics-calibration` is a private local command. Set `FM_VALUESCOUT_GRAPHICS_ROOT` to a readable graphics root before running it. The command runs one ignored Rust test with `--nocapture` and emits one path-free JSON report. It does not use PowerShell and works from WSL against a mounted Windows filesystem. Reports label WSL execution as WSL and mounted Windows storage as a mounted Windows filesystem; they do not describe WSL measurements as native Windows performance. Missing or unreadable roots fail without printing the supplied value.

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

The normal exception is an explicit `/jay-claude:deliver-feature` invocation for one accepted ledger, within the grant the plugin's `core` skill defines: the planned branches, reviewed package commits, non-force pushes, the planned PRs, bounded reviewed CI repairs, exact-head merges with the recorded method, and fast-forward-only base synchronization. It does not authorize a release.

Merging to `main` publishes nothing; only a merged `release-preparation.json` change starts the Release workflow. Delivery may therefore merge reviewed PRs without waiting for the developer. If merges ever deploy or publish, replace this rule with one that requires developer approval for each merge.

An accepted-plan change without reviewed renewal ends the grant. It never permits amend, rebase, force-push, protection bypass, self-approval, branch deletion, unrelated work, another ledger, or a release.

Stage exact files or hunks. Never use `git add .` or `git commit -a`. Before every commit, inspect status, the complete staged diff and stat, and run `git diff --cached --check` plus recorded validation. Report documentation impact, reviewer findings, risks, and the proposed commit message. Manual work waits for explicit developer approval before committing.

## Project-owned releases

jay-claude delivery ends after the final PR merges and the base is synchronized. A feature merge never publishes.

Only an explicit `/create-release` invocation of the project skill can prepare a release. It inspects all changes since the latest published tag, proposes the SemVer scope, updates the version owners and changelog, increments `release-preparation.json`, and opens a dedicated release PR. The Release workflow creates the tag and GitHub release only after that PR merges and the exact `main` Check passes. Do not infer a version, tag, provider action, or deployment.

## Security

Claude Code, its plugins, and their tools execute with the user's operating-system privileges. Project trust and role prompts are controls, not an operating-system sandbox.

Do not read or print secret values, private keys, production dumps, local authentication stores, or unrelated personal data. Keep credentials out of repository files, ledgers, templates, prompts, and logs.
