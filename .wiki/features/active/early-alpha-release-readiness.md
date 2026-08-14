# Early Alpha Release Readiness

## Status

Active

## Intent

Prepare FM ValueScout for a deliberately narrow, unsigned Windows early-alpha release that the author can dogfood and an accidental public-repository visitor can understand and install at their own risk. The release path must package the bridge from the same source revision as the desktop app, fail before publication when release identity or artifacts are wrong, preserve the existing local-data contract, and automatically publish a prepared prerelease only after its exact `main` revision passes the required checks.

## User-visible behavior

- The Dashboard no longer shows the template health status, demo SQLite value, or simulated-error controls.
- A Windows x64 early-alpha installer contains a real `FmDataBridge.dll` built from the tagged source, not the tracked development placeholder.
- Release builds retain local operational logs without sending telemetry or player data to an external service.
- The README explains the supported Windows, Steam FM26, BepInEx, database-scope, installation, update, uninstall, recovery, diagnostics, and support boundaries.
- Every successful `Check` run for a push to `main` evaluates release metadata for that exact commit. A newer prepared version produces one automatically published unsigned Windows prerelease; an unchanged version exits without creating or mutating a release.
- The matching dated `CHANGELOG.md` section becomes the complete GitHub release body without separately maintained release-note copy.
- Release notes and documentation warn that FM patches can break compatibility, the installer can trigger SmartScreen, and the two player-boost actions remain exact-build, optional operations.

## Invariants

- A release job must fail before bundling when the bridge artifact is missing, is not a managed Windows DLL, comes from a different version, or is still the placeholder.
- The app package, Tauri bundle, Rust crate and lock entry, bridge project, Git tag, changelog section, release title, and release metadata must use one matching SemVer prerelease identity. `src-tauri/tauri.conf.json` remains the canonical app version unless implementation evidence requires replanning.
- Release packaging must build the bridge and desktop app from the same checked-out commit with locked dependency resolution.
- Every human-authored pull request must use one repo-local PR-creation procedure and the repository template. The template must record exactly one `none`, `patch`, `minor`, or `major` application-release intent. `none` is a normal decision that defers publication and leaves version owners and dated changelog sections unchanged; `patch` or `minor` prepares a release from the complete unreleased change set since the latest published tag. The procedure may apply an unambiguous routine decision, but an ambiguous or `major` decision must stop for maintainer input. An eligible Dependabot-only patch pull request has a predeclared `none` intent, leaves app versions and `CHANGELOG.md` unchanged, and ships with the next prepared application release. CI must not otherwise infer release intent or commit directly to `main`.
- `CHANGELOG.md` must preserve an `Unreleased` section and use Keep a Changelog headings. A `none` pull request may add relevant user-visible notes to `Unreleased` but must not create a dated release section. A release-bearing pull request must include one dated version section for the complete intended release, and GitHub must use that section as the exact release notes.
- GitHub must publish a prerelease only after the required `Check` workflow succeeds for the same `main` SHA. The privileged job may use a draft as temporary asset-staging state, but it must verify the complete release and set `draft=false` before succeeding.
- The release evaluator must run after every successful push check on `main`. An unchanged version is a successful no-op; a lower version, inconsistent version owner, missing changelog section, non-increasing release identity, or mismatched existing tag, release, or staging draft must fail closed.
- The release workflow must produce only the supported Windows x64 installer. It must not imply macOS or Linux support.
- Normal product checks remain required. Release packaging does not replace frontend, browser, Rust, bridge, secret, or manual installed-app validation.
- Existing app databases must migrate forward without losing saves, snapshots, Planner state, Academy state, or CSV enrichment. The obsolete `demo_value` table is the only persisted table this feature may remove.
- Migration history remains append-only. Existing migration versions are not rewritten or renumbered.
- Production diagnostics must stay local and must not routinely record player rows, dump contents, memory addresses, or other bulky or sensitive data.
- The existing bridge trust boundary remains unchanged: BepInEx must already exist, the app installs only `FmDataBridge.dll`, reads remain fail-closed for unsupported layouts, and the two approved write actions remain exact-build and action-specific.
- Core use remains offline. This feature adds no account, analytics, remote logging, cloud data, or background network requirement.

## Non-goals

- Publishing any artifact from an unverified pull-request or unchecked `main` revision.
- Windows code signing, Microsoft Store distribution, or removal of SmartScreen warnings.
- Automatic updates or the Tauri updater plugin.
- macOS, Linux, non-Steam FM26, other FM editions, women's-database, or combined-database support.
- Installing or updating BepInEx itself.
- Automated database backup, selective Planner or Academy export, downgrade support, or migration rollback.
- Automated real-WebView testing with `tauri-driver`.
- External crash reporting, telemetry, analytics, or a support service-level agreement.
- General bridge protocol changes, new memory-write operations, or wider FM build support.
- Player gender investigation or filtering.
- Repository-wide dependency modernization beyond release build reproducibility, currently known High advisory fixes, and update monitoring.
- An application version bump or installer release for each auto-merged dependency patch.
- Automatic merge of dependency major or minor updates, pre-`1.0.0` packages, GitHub Actions, NuGet packages, or a pull request that does not satisfy every auto-merge guard.
- CI-side semantic interpretation of source changes, unreviewed generated changelog prose, CI-authored version commits, or a release for every documentation and tooling push.

## Current-state map

- Relevant components:
  - `.github/workflows/release.yml` builds unsigned Windows, Linux, and macOS draft assets on `v*` tags, marks them as non-prereleases, uses a fixed release body, and has never completed a release run.
  - `.github/workflows/check.yml` path-selects frontend, browser, Rust, and Windows bridge checks and exposes the strict required `check` status.
  - `scripts/dev` is the stable command surface. It can test and install the bridge but has no release-package command.
  - `src-tauri/tauri.conf.json` bundles `src-tauri/resources/FmDataBridge.dll` for every target and currently declares version `0.1.0`.
  - `src-tauri/resources/FmDataBridge.dll` is a 102-byte text placeholder. `bridge/README.md` requires a manual Release build and copy before Windows packaging.
  - `bridge/FmDataBridge.csproj` declares version `0.1.0`, targets .NET 6, and resolves BepInEx packages through wildcard versions. The last local restore selected `BepInEx.Unity.IL2CPP` `6.0.0-be.785` and `BepInEx.PluginInfoProps` `2.1.0`; no NuGet lock file is tracked.
  - `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, and `bridge/FmDataBridge.csproj` currently agree on `0.1.0`, but no command validates them as one release identity.
  - No `CHANGELOG.md`, release-metadata validator, or repository-owned PR-creation procedure with a release-intent checkpoint exists. The repository template contains only What, Why, and Notes.
  - `src-tauri/src/features/memory_read/install.rs` resolves and copies the bundled resource but treats any regular file at the resource path as installable.
  - `src/features/health/`, `src-tauri/src/features/health/`, `src/app/routes/index.tsx`, and `src-tauri/src/lib.rs` still expose the original template health and demo-value path.
  - Migration v1 creates `demo_value`; the registry currently ends at v21. The table is isolated from product persistence.
  - `src-tauri/src/lib.rs` enables `tauri-plugin-log` only for debug builds. Release builds have no durable app log.
  - `app.db` lives under Tauri's app data directory. Migrations are transactional, but no application backup or restore workflow exists. A representative database reached 7.11 GB.
  - The root README still describes a walking skeleton and template-fork workflow rather than an end-user alpha contract.
- Data model:
  - Product data remains in Rust-owned SQLite. This feature changes no save, snapshot, player, Planner, Academy, boost, or enrichment contract.
  - The only schema change is a forward migration that drops the obsolete `demo_value` table after existing databases reach the new version.
- Persistence and migrations:
  - `PRAGMA user_version` applies ordered transactions. Migration tests cover fresh and populated upgrades.
  - Manual closed-app backup and restore documentation is the early-alpha recovery floor. Automatic full-database copies are deferred because the database can be several gigabytes.
- Existing behavioral assumptions:
  - The supported runtime is Windows Steam FM26 with BepInEx 6 IL2CPP already installed.
  - Normal app-generated scans use the men's database scope. Player boosts advertise support only for the exact live-validated FM26.3.2 build.
  - Browser smoke uses IPC stubs and cannot prove the Tauri WebView, SQLite, Steam install path, or live FM bridge.
- Architectural seams:
  - React owns release-facing copy and removal of the template panel.
  - Rust owns migration safety, local diagnostics, and bridge-resource validation.
  - C# owns the plugin binary and embedded bridge version.
  - `scripts/dev` owns deterministic local and CI packaging commands.
  - GitHub Actions orchestrates checks, release-metadata evaluation, Windows packaging, staged release creation, automatic prerelease publication, checksums, and artifact retention without duplicating local validation or packaging rules.
- Project validation commands:
  - `./scripts/dev test [target...]`
  - `./scripts/dev check`
  - `./scripts/dev check-app`
  - `./scripts/dev check-rust`
  - `./scripts/dev bridge-test`
  - `./scripts/dev smoke`
  - `./scripts/dev secrets [--staged]`
  - Planned Windows-only command: `./scripts/dev package-windows`
  - Planned read-only release command: `./scripts/dev release-metadata [latest-tag] [release-intent]`
- GitHub publication state:
  - Repository: `JG1995/fm-valuescout`, public, default branch `main`.
  - Merge method: squash only; merged branches are deleted.
  - Protection: strict required `check`, linear history, no force pushes, no branch deletion, and no required approval for the solo-developer scope.
  - Repository auto-merge is disabled. Enabling it is a manual setting required by the planned bounded Dependabot workflow.
  - No tags or releases exist.
  - Secret scanning and push protection are enabled. Dependabot alerts and security updates are disabled.
- Primary risks:
  - Shipping the placeholder or a bridge built from stale source.
  - Allowing version owners or release notes to drift while packaging remains green.
  - A tag/version mismatch creating an incorrectly named or upgrade-ordered installer.
  - Releasing an unchecked `main` SHA or mutating an existing release after an unrelated same-version push.
  - A workflow change that passes Linux checks but fails on the first Windows bundle.
  - Treating stub browser smoke as proof of a packaged Windows/FM integration.
  - Logging paths, dump content, or player data while improving diagnostics.
  - Dropping the demo table by rewriting history instead of migrating existing databases safely.

## Feature architecture

The feature uses one release-metadata seam, one local release-build seam, and one GitHub orchestration seam.

`.agents/skills/create-pr/SKILL.md` becomes the single human pull-request entrypoint. It formalizes the existing “create a PR using the repository template” procedure instead of adding a separate release-only path. The skill inspects the branch diff and the complete unreleased change set since the latest published tag, obtains or selects exactly one release intent (`none`, `patch`, `minor`, or `major`), prepares and validates version and changelog files when needed, fills every repository-template section, verifies that no pull request already exists for the branch, pushes the committed branch, and creates the requested draft pull request. Repository checkpoint and Git-authority rules still govern any local release-preparation commit. The skill must not merge, tag, publish a GitHub release, or bypass a stop condition.

`none` is an explicit way to use the same procedure without publishing an application version from that pull request. It leaves durable version owners and dated changelog sections unchanged; relevant user-visible notes can remain under `Unreleased`. A later `patch` or `minor` decision evaluates all changes since the latest published tag, not only the current pull request, so deferred changes appear in the next release notes. An ambiguous compatibility boundary or `major` intent stops for maintainer input. During the early-alpha line, `patch` advances the patch component, `minor` advances the minor component and resets patch, and `major` requires an explicit decision to enter a `1.0.0-alpha.1` compatibility line. Each new numeric core starts at `alpha.1`.

`./scripts/dev release-metadata [latest-tag] [release-intent]` will be a read-only, prerelease-aware validator. It will calculate the expected next version from an explicit intent, require every durable version owner to agree, compare versions by SemVer precedence, extract the exact dated `CHANGELOG.md` section, and emit machine-readable release metadata. It must not edit files, write a changelog, create a commit, create a tag, or call GitHub.

`scripts/dev package-windows` will own the Windows package preparation and build contract. It will restore locked bridge dependencies, build the Release bridge from the checked-out source, validate the DLL and release identity, make that generated artifact the Tauri bundle resource without committing a generated production DLL, build one Windows x64 NSIS installer, and place the installer plus checksum in a predictable artifact directory. The command must fail closed and must not publish anything.

The required Check workflow will validate release metadata and exercise the Windows package command when release inputs change, then upload the candidate as a workflow artifact. The Release workflow will follow every successful `Check` run caused by a push to `main`, check out its exact verified SHA, and run the same metadata and package commands. An unchanged current version is a successful no-op. For a newer version, the only job with `contents: write` creates or repairs a matching temporary draft targeted at that SHA, uploads the installer and checksum, sets the title and exact extracted changelog section, verifies the assembled release, and publishes it as a prerelease before succeeding. An exact retry is idempotent; a different SHA or identity must never retarget or overwrite an existing draft, tag, or published release.

Dependency maintenance uses a separate privileged metadata workflow. It can enable native auto-merge for verified Dependabot-only pnpm and Cargo patch updates where every old and new dependency version is stable and at least `1.0.0`; if a later event makes metadata ineligible, it revokes an existing auto-merge request. The strict required `check` runs the same evaluator read-only for matching Dependabot pull requests, so an ineligible changed revision cannot become mergeable before revocation runs. The privileged workflow runs from the trusted default branch with `pull_request_target`, never checks out or executes pull-request code, and does not bypass `check`. GitHub Actions, NuGet, pre-`1.0.0`, prerelease, minor, major, mixed, malformed, and maintainer-modified updates remain unmerged. An eligible merge has application release intent `none`; the verified-main release evaluator sees the unchanged app version and exits without publishing.

The health scaffold is removed across React, Rust, IPC registration, tests, and current-state documentation. Migration v1 remains historical; a new migration drops `demo_value` for both existing and fresh databases. No replacement health endpoint or dashboard status panel is added.

Release diagnostics use the existing Tauri/Rust logging stack and write bounded local files in release builds. External reporting remains absent. End-user documentation supplies the data and log locations, manual backup and restore procedure, supported environment, unsigned-installer warning, bridge setup, known limitations, and private security-reporting path.

## Uncertainty register

### Known

- The current bundled bridge resource is a placeholder, so the existing tag workflow cannot produce a functional FM integration.
- The existing product and architecture documents define Windows Steam FM26 as the supported bridge runtime.
- The current release workflow produces multiple unsupported platform artifacts and marks its draft as a normal release.
- The health feature is template scaffolding with no product dependency.
- The existing browser suite does not exercise the packaged Tauri or live FM path.
- GitHub permits squash merges only and requires the strict `check` status on `main`.

### Assumptions

- The first candidate uses app version `0.1.0-alpha.1` and Git tag `v0.1.0-alpha.1`; implementation must verify that the selected NSIS path preserves this identity.
- The first release has no prior tag boundary. Its changelog therefore summarizes the current dogfoodable product rather than reconstructing every development commit as a separate entry.
- One unsigned NSIS installer is enough for the author's first dogfood release.
- Manual GitHub download and reinstall is enough; automatic updates remain deferred.
- A documented, tested closed-app copy of `app.db` is enough for the first alpha. No automatic backup or selective export is required.
- Local Rust/Tauri operational logs plus visible UI errors are enough. Capturing every React exception in an external service is not required.
- The first release-bearing pull request is not merged until its Check artifact passes the manual Windows/FM acceptance checklist; merging then allows the verified `main` workflow to publish automatically.
- Release intent remains a semantic judgment over the complete user-visible change set. The repo-local skill handles unambiguous routine decisions; Conventional Commit titles provide evidence but do not decide the release alone.

### Decisions

- Remove the health scaffold rather than rename, hide, or retain it as a diagnostics feature.
- Preserve migration history and add a forward drop-table migration.
- Keep one PR with fine-grained commits. The changes share one release-readiness acceptance boundary and do not need a risky foundation merged separately.
- Build and validate the bridge from source during packaging. Do not treat a tracked generated DLL as the release source of truth.
- Pin the currently validated BepInEx dependencies and commit NuGet lock files before release packaging depends on them.
- Keep the canonical version in Tauri configuration and fail when other manifests, the bridge, or tag disagree.
- Use Keep a Changelog with one preserved `Unreleased` section. Use the exact matching dated version section as GitHub release notes.
- Adapt the reviewed `fm-youth-tracker` release pattern: prepare release intent and changelog before merge, then evaluate and automatically publish from the verified `main` SHA after its successful Check run.
- Run release evaluation after every successful `main` push check, but create a release only when the approved application version is newer than the existing release boundary.
- Make the repo-local `create-pr` skill the only human PR procedure. It owns template completion and pre-merge release preparation for both `none` and release-bearing pull requests; deterministic validation remains in repository scripts. CI validates metadata, packages the app, and publishes the prepared release; it never decides the release intent, rewrites changelog prose, commits a bump, or pushes to `main`.
- Produce one Windows x64 NSIS candidate. Remove macOS and Linux release assets until those platforms have an implemented and tested product contract.
- Publish GitHub prereleases automatically from verified `main`; use draft state only as temporary fail-closed staging within the privileged publication job.
- Enable Dependabot alerts and security-update pull requests. Offer routine version-update pull requests only for pnpm and Cargo patch updates after a 14-day cooldown. Automatically squash-merge a Dependabot-only pnpm or Cargo patch pull request only when every changed dependency moves between stable `1.x` or newer versions, Dependabot metadata and commit verification pass, no maintainer change exists, and the strict up-to-date `check` passes.
- Disable routine GitHub Actions and NuGet version-update pull requests. Do not auto-merge their security updates. BepInEx and workflow actions cross higher-risk compatibility and repository-permission boundaries.
- Treat an eligible auto-merged dependency patch as application release intent `none`. Do not change FM ValueScout versions or `CHANGELOG.md`; include the dependency state in the next prepared application release.
- Keep signing, updater, BepInEx bootstrap, automated backup, telemetry, and real-WebView automation outside this feature.
- Enable local release logging without widening Tauri capabilities or adding a remote endpoint.
- Use GitHub private vulnerability reporting rather than publishing a personal email address when the repository setting is available.

### Unknowns

- The final stable Windows runner output path and artifact name until `package-windows` completes its first CI run.
- Whether the first live run exposes any GitHub-specific draft, tag, or asset behavior that requires tightening the exact-SHA retry checks without changing the automatic-publication contract.
- Whether the first downloaded candidate will trigger an environment-specific SmartScreen or antivirus flow beyond the documented unsigned-installer warning.
- Whether a supported FM26 session will be available during feature close-out for the fresh integrated boost validation that the completed boost record still lists as a gap.
- Whether GitHub private vulnerability reporting and immutable releases are enabled; both require a repository-setting check before the first public publication.

### Risks

- Release-specific resource routing could accidentally mutate or depend on the tracked placeholder. Packaging must use an isolated generated artifact and verify it before Tauri starts.
- Custom NuGet sources and .NET 6 can make locked restores fragile. Stop rather than silently widening versions or changing the target framework.
- `scripts/dev` is a churn-heavy shared command surface. Keep the new packaging command isolated from normal check and install behavior.
- Migration code is a high-impact shared file. Test a populated v21 upgrade and a fresh database; do not infer safety from `DROP TABLE IF EXISTS` alone.
- A successful candidate build proves packaging, not FM attach or player-boost correctness. Manual installed-app evidence remains mandatory.
- The database can be several gigabytes. Recovery documentation must not promise a cheap or automatic backup that the product does not provide.
- A run can fail after creating its temporary draft, and a later same-version push can occur after publication. Idempotency must distinguish an exact retry from a different SHA and must never retarget or overwrite the existing draft, tag, or release.
- Dependabot's cooldown reduces exposure to brand-new routine dependency releases but is not a security guarantee and does not delay security-update pull requests. Eligible security patches can therefore auto-merge as soon as the required checks pass.
- SemVer metadata cannot prove compatibility. The auto-merge workflow must reject pre-`1.0.0`, prerelease, minor, major, mixed, unverifiable, maintainer-modified, GitHub Actions, and NuGet updates even when the test suite is green.
- A privileged `pull_request_target` workflow can expose write authority to untrusted pull-request code. The auto-merge workflow must use only the trusted default-branch workflow, fetch verified Dependabot metadata, never check out or execute the pull-request revision, and grant only the permissions needed to enable native auto-merge.

## Walking skeleton

Commit 4 is the thinnest release path: from one Windows command, build the current bridge, reject placeholder or version mismatch, bundle one installer, and emit a checksum without publishing. Commits 1–3 remove the known template and dependency hazards before that path becomes active; later commits add diagnostics, the public contract, versioned changelog metadata, and verified-main GitHub orchestration.

## Delivery plan

### PR 1 — Prepare Windows early alpha distribution

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/early-alpha-release-readiness`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** Squash

**Required checks:** Strict required `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Build-feature-loop profile:** Terra xhigh — Windows packaging, a forward migration, local diagnostics, and GitHub release orchestration require material cross-toolchain judgment, but the target contract is settled and no existing product data changes.

**Provisional PR title:** `feat(release): prepare Windows early alpha distribution`

**Purpose:** Deliver one reviewable release-readiness boundary: remove template residue, make build inputs reproducible, create a verified Windows package path, add local diagnostics and honest operating documentation, make the standard template-based PR procedure own SemVer and changelog preparation, and make every verified `main` push publish a prepared prerelease or exit as an unchanged-version no-op.

**Depends on:** Current protected `main` at or after `0c9c10e41b59941d08b90a5f493283836d149830`; existing Tauri, Rust, C# bridge, in-app bridge install, and project validation surfaces.

**Merge to trunk when:** Every commit has cleared its checkpoint review; the prepared `0.1.0-alpha.1` identity matches every version owner and dated changelog section; the strict `check` status includes successful metadata validation and a Windows candidate package for the PR; the candidate contains the source-built bridge and checksum; the downloaded artifact passes the installation, recovery, and live-FM checklist; documentation matches the implemented scope; and feature close-out reports no blocking finding. Because the verified `main` workflow publishes automatically, the initial release-bearing PR must not merge before that acceptance evidence exists.

#### Commit 1 — Remove the template health scaffold

**Status:** Completed

**Provisional commit:** `fix(health): remove template health scaffold`

**Work:** Remove the Dashboard health/demo UI, frontend health module, health IPC commands and registration, Rust health module, dedicated health tests and stubs, and architecture references that describe the walking skeleton as current product behavior. Add migration v22 to drop `demo_value` while retaining migration v1 as historical upgrade input.

**Out of scope:**

- Replacement diagnostics UI or an About page.
- Changes to bridge status, snapshot status, error boundaries, or product tables.
- Any migration change beyond removing `demo_value`.

**Implementation packet:**

- Owners and files: `src/features/health/**`, `src/app/routes/index.tsx`, related Dashboard and testing IPC setup, `src-tauri/src/features/health/**`, `src-tauri/src/features/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/db/migrations.rs`, `.wiki/ARCHITECTURE.md`.
- Existing patterns to verify: additive migrations and populated-upgrade tests in `src-tauri/src/db/migrations.rs`; independent Dashboard panels and route loader tests in `src/app/routes/index.test.tsx`; explicit Tauri command registration in `src-tauri/src/lib.rs`.
- Constraints and invariants: preserve migration v1; v22 removes only `demo_value`; fresh and populated databases retain all product tables and `user_version`; no replacement health abstraction.
- Dependencies and ordering: first commit; later diagnostics work must not reuse or recreate the health scaffold.

**Implementation profile:** Terra xhigh — the deletion is conceptually simple, but safe removal crosses React, IPC registration, Rust modules, test stubs, architecture documentation, and an existing-database migration.

**Review profile:** Sol High — review must independently verify the cross-layer removal and populated-database safety rather than treating deleted UI as the complete change.

**Validation:**

- RED: add or update a migration test that opens a populated v21 database and proves the obsolete table still exists before v22, then expects v22 to remove it while product rows remain.
- `./scripts/dev test src/app/routes/index.test.tsx`
- `./scripts/dev check-rust`
- `./scripts/dev smoke`
- `./scripts/dev check`
- Expected evidence: no production `health`, `get_status`, `get_demo_value`, `set_demo_value`, or `demo_value` reference remains except historical migration-v1 text and the v22 upgrade test.

**Stop conditions:** Stop and replan if any non-template product path reads the health module or demo table, if removing the route loader changes unrelated Dashboard suspense behavior, or if the migration cannot distinguish obsolete demo data from user-owned data.

**Review mandate:**

- Confirm the Dashboard and loader no longer depend on health queries or simulated errors.
- Confirm all registered health commands and Rust module exports are removed.
- Confirm migration v1 remains intact and v22 is append-only.
- Confirm populated v21 upgrades preserve saves, snapshots, Planner, Academy, and enrichment rows.
- Confirm test and IPC stubs contain no callable production health path.
- Confirm architecture prose stops describing the health demo as current state without documenting planned release behavior as already implemented.

#### Commit 2 — Lock release build inputs

**Status:** Completed

**Provisional commit:** `chore(deps): lock release build inputs`

**Work:** Replace wildcard BepInEx package references with the currently validated exact versions, enable and commit NuGet lock files for bridge and bridge tests, require locked restore for release-relevant builds, and refresh the pnpm lockfile so the known High build-tool advisories use fixed transitive versions without adding product dependencies.

**Out of scope:**

- Moving the bridge off .NET 6.
- Changing BepInEx, IL2CPP, memory layouts, protocol behavior, or plugin APIs.
- Broad dependency upgrades unrelated to the known advisories or reproducible release inputs.

**Implementation packet:**

- Owners and files: `bridge/FmDataBridge.csproj`, `bridge/Tests/FmDataBridge.Tests.csproj`, bridge lock files, `package.json` only if a declared version must move, and `pnpm-lock.yaml`.
- Existing patterns to verify: `bridge/global.json`, current Windows bridge CI, current resolved BepInEx versions, and `pnpm install --frozen-lockfile` in GitHub workflows.
- Constraints and invariants: keep current bridge behavior and .NET 6 compatibility; do not add NuGet sources; locked restore must work on a clean Windows runner; production dependency graph must not grow.
- Dependencies and ordering: follows health removal; package command in Commit 4 depends on these locks.

**Implementation profile:** Terra xhigh — exact dependency selection is known, but custom NuGet feeds, prerelease BepInEx packages, .NET 6, and two lock graphs require careful restore validation.

**Review profile:** Sol High — release build inputs execute in CI and inside the game process, so the review must verify provenance, pinning, and unchanged bridge behavior.

**Validation:**

- `./scripts/dev bridge-test`
- `./scripts/dev check`
- Clean Windows CI restore succeeds in locked mode.
- Expected evidence: no wildcard `PackageReference` remains; committed NuGet lock files resolve `BepInEx.Unity.IL2CPP` `6.0.0-be.785` and `BepInEx.PluginInfoProps` `2.1.0`; frozen pnpm installation succeeds; the three previously observed High build-tool advisories no longer resolve to their vulnerable versions.

**Stop conditions:** Stop and request a developer decision if the exact BepInEx versions are no longer available from the existing feeds, locked restore requires changing the target framework, or an advisory fix requires a product dependency or breaking toolchain upgrade.

**Review mandate:**

- Confirm every release-executed NuGet package has an exact resolved version.
- Confirm bridge and test locks agree where they share dependencies.
- Confirm no machine-local FM or interop assembly path enters a lock file.
- Confirm the bridge test suite proves behavior stayed unchanged.
- Confirm pnpm remediation is limited to the known advisory path and declared compatible ranges.

#### Commit 3 — Automate guarded dependency patches

**Status:** Completed

**Provisional commit:** `ci(deps): automate guarded dependency patches`

**Work:** Add a conservative Dependabot configuration and a metadata-only auto-merge workflow. Offer grouped routine version updates only for pnpm and Cargo patch releases after a 14-day cooldown. Enable native squash auto-merge only when every dependency in a verified Dependabot-only pull request moves between normal SemVer versions with major version `1` or newer, the highest update type is patch, no maintainer change exists, and the strict up-to-date `check` succeeds. The same policy can merge an eligible security patch immediately because GitHub exempts security updates from cooldown. Disable routine GitHub Actions and NuGet version updates; leave their security updates and every other ineligible pull request for explicit review. Record the repository settings required to enable Dependabot alerts, security updates, and GitHub auto-merge.

**Out of scope:**

- Automatic merge of minor, major, pre-`1.0.0`, prerelease, GitHub Actions, NuGet, unverifiable, mixed-policy, or maintainer-modified updates.
- Automatic approval, bypass of branch protection, a personal access token, a GitHub App credential, or release authority.
- Renovate, third-party dependency bots, or paid GitHub security products.
- Repository-wide action or dependency upgrades in this commit.

**Implementation packet:**

- Owners and files: `.github/dependabot.yml`, `.github/workflows/dependabot-automerge.yml`, a small tested policy evaluator under `scripts/`, `scripts/dev` for its command route, and the active ledger discovery record if GitHub rejects or limits an ecosystem.
- Existing patterns to verify: package-manager directories, Cargo manifest location, NuGet solution directory, current GitHub Actions paths, strict `check` branch protection, squash-only merges, GitHub native auto-merge, and verified outputs from `dependabot/fetch-metadata`.
- Constraints and invariants: use `cooldown.default-days: 14` for pnpm and Cargo routine version updates; run weekly with a low open-PR limit and patch-only compatible grouping; set GitHub Actions and NuGet `open-pull-requests-limit: 0`; preserve security-update pull requests; use the highest update type and inspect every entry in `updated-dependencies-json`; require normal SemVer with both versions at major `1` or newer; reject maintainer changes. The strict `check` must fail closed using the same evaluator for an ineligible Dependabot pull request. Use `pull_request_target` only for a workflow stored on the trusted default branch. Check the Dependabot author, repository, base branch, and verified metadata. Pin the metadata action to a reviewed commit SHA. Never check out, install, build, or execute pull-request code in the privileged job. Grant only `contents: write` and `pull-requests: write`; enable with `gh pr merge --auto --squash` so native auto-merge waits for branch protection, and revoke a previously enabled request when later metadata is ineligible.
- Dependencies and ordering: follows locked inputs so monitoring starts from a reproducible baseline. Commit 8 must treat the resulting same-version `main` push as a release no-op; the dependency state ships with the next prepared application release.

**Implementation profile:** Terra xhigh — the policy is narrow, but safe automation crosses untrusted pull-request metadata, SemVer classification, action pinning, token permissions, strict branch protection, and release no-op behavior.

**Review profile:** Sol High — a false positive can merge unreviewed executable input into `main`, while a false negative only leaves a dependency pull request open.

**Validation:**

- RED: policy tests first reject minor, major, pre-`1.0.0`, prerelease, mixed, malformed, GitHub Actions, NuGet, non-Dependabot, wrong-repository, wrong-base, and maintainer-modified metadata.
- Tests accept only pnpm or Cargo pull requests where every dependency is a stable patch update between major version `1` or newer, and prove that the highest grouped update type cannot hide an ineligible entry.
- `./scripts/dev test scripts/`
- `./scripts/dev check`
- Inspect the workflow statically and in GitHub: the privileged job never checks out or runs pull-request code; metadata verification stays enabled; the action reference is immutable; permissions are limited; and only `gh pr merge --auto --squash` can mutate pull-request state.
- Enable repository auto-merge. Exercise one eligible fixture through the live workflow and require native auto-merge to wait for the strict up-to-date `check`; exercise one ineligible fixture and require it to remain unmerged.
- After merge, GitHub recognizes every configured ecosystem without errors; routine pnpm and Cargo releases younger than 14 days are withheld; routine minor and major updates are not offered; routine Actions and NuGet pull requests are disabled; alerts and security updates remain enabled; and an eligible dependency merge causes a same-version application-release no-op.

**Stop conditions:** Stop and request a developer decision if GitHub rejects the 14-day cooldown or patch-only policy, verified metadata cannot classify every grouped dependency, the built-in token cannot enable native auto-merge with the stated permissions, or the bridge's custom NuGet feeds require credentials for alerts or security updates. Do not add a personal token, GitHub App, registry secret, feed token, or alternate registry during this commit.

**Review mandate:**

- Confirm pnpm and Cargo routine updates are weekly, patch-only, delayed for 14 days, grouped compatibly, and limited to a low number of open pull requests.
- Confirm routine GitHub Actions and NuGet updates are disabled without suppressing alerts or security-update pull requests.
- Confirm the policy rejects every update outside the stable `1.x`-or-newer pnpm/Cargo patch boundary and rejects any non-Dependabot or maintainer-modified pull request.
- Confirm `pull_request_target` never consumes or executes pull-request-controlled files, commands, expressions, or artifacts.
- Confirm native auto-merge cannot complete until the strict, up-to-date `check` passes and uses the repository's squash-only merge method.
- Confirm a later maintainer push or metadata-verification failure revokes any earlier native auto-merge request before the changed revision can merge.
- Confirm eligible dependency merges have release intent `none`, do not edit application versions or `CHANGELOG.md`, and cannot create a GitHub release.
- Confirm no credential, local path, or private package source is committed.

#### Commit 4 — Package the Windows bridge from source

**Status:** Active

**Provisional commit:** `build(release): package the bridge from source`

**Work:** Add `./scripts/dev package-windows` as the non-publishing Windows release-build surface. It builds the locked Release bridge, verifies the generated managed DLL and matching version, routes that artifact into a release-specific Tauri bundle without treating the tracked placeholder as production input, validates the canonical prerelease identity, builds one Windows x64 NSIS installer, and emits the installer plus SHA-256 checksum in a predictable artifact directory.

**Out of scope:**

- GitHub release creation, signing, updater artifacts, MSI, macOS, or Linux bundles.
- Changing bridge runtime behavior or in-app BepInEx installation.
- Committing the generated release DLL or installer.

**Implementation packet:**

- Owners and files: `scripts/dev`, `src-tauri/tauri.conf.json`, a release-specific Tauri config or equivalent isolated resource mapping, bridge and app manifests, `.gitignore` only if generated release output needs an explicit rule, and focused build validation support.
- Existing patterns to verify: `bridge-install` path and build behavior, `BUNDLED_PLUGIN_DLL_RESOURCE` in `install.rs`, Tauri resource layout, current versions in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `bridge/FmDataBridge.csproj`.
- Constraints and invariants: canonical version match; source-built DLL only; no tracked generated binary; command is Windows-only and non-publishing; normal `check` remains cross-platform; one NSIS artifact and checksum.
- Dependencies and ordering: depends on Commit 2's locked restore; GitHub workflows in Commit 8 must call this command rather than reimplement it.

**Implementation profile:** Terra xhigh — the contract is fixed, but artifact routing crosses Bash/Windows, .NET, Tauri resource bundling, SemVer, and deterministic output handling.

**Review profile:** Sol High — review must trace source commit to bridge DLL to Tauri resource to installer and verify every failure mode blocks packaging.

**Validation:**

- RED: the release-package validation must reject the current 102-byte placeholder and a deliberate manifest-version mismatch before invoking Tauri.
- `./scripts/dev bridge-test`
- `./scripts/dev check`
- On a Windows host: `./scripts/dev package-windows`
- Expected evidence: one NSIS installer and matching SHA-256 file; extracted or installed resources contain a managed `FmDataBridge.dll` with the release version; the tracked placeholder is not modified or accepted.

**Stop conditions:** Stop and replan if a clean release build requires committing the generated DLL, if Tauri cannot route an isolated generated resource with the current configuration model, if NSIS cannot preserve the SemVer prerelease identity, or if version verification needs an unreviewed executable or network service.

**Review mandate:**

- Confirm the generated bridge and app come from the same checkout.
- Confirm placeholder, missing, malformed, or version-mismatched DLLs fail before Tauri packaging.
- Confirm the release build does not mutate a tracked resource or leave a dirty worktree.
- Confirm one Windows x64 NSIS artifact is produced and no unsupported platform bundle remains.
- Confirm checksum generation covers the exact uploaded installer.
- Confirm the package command never creates a tag, release, or remote mutation.

#### Commit 5 — Retain local release logs

**Status:** Pending

**Provisional commit:** `feat(diagnostics): retain local release logs`

**Work:** Enable bounded file logging for release builds through the existing Tauri/Rust logging stack, include app and schema identity needed for diagnosis, and document the log location and retention behavior. Keep debug console behavior useful without adding external reporting.

**Out of scope:**

- Sentry or another external crash service.
- Telemetry, analytics, automatic uploads, or a diagnostics UI.
- Logging player rows, dump contents, memory addresses, or full local paths from sensitive operations.

**Implementation packet:**

- Owners and files: `src-tauri/src/lib.rs`, existing logging dependency configuration, focused Rust tests where pure configuration can be tested, and release diagnostics documentation owned by Commit 6.
- Existing patterns to verify: current debug-only `tauri-plugin-log` registration; safe bridge status and diagnostics redaction; Tauri app-data and log-directory resolution.
- Constraints and invariants: local files only; bounded size/retention; startup and migration failures remain diagnosable; no new Tauri capability or WebView-native privilege; no routine sensitive data.
- Dependencies and ordering: package command exists before final installed-build proof; documentation follows in Commit 6.

**Implementation profile:** Luna Max — the existing logging plugin and local-only contract keep the implementation bounded once current Tauri configuration is verified.

**Review profile:** Sol High — logs cross a privacy and release-diagnostics boundary, and release-only behavior needs careful configuration review despite the small diff.

**Validation:**

- `./scripts/dev check-rust`
- `./scripts/dev check`
- Packaged Windows candidate starts and creates a bounded local log containing app identity and startup/migration events.
- Inspection of the log from Load Data and an expected failure shows no player rows, dump bodies, memory addresses, or unredacted sensitive diagnostic payloads.

**Stop conditions:** Stop and replan if persistent logs require a remote service, a broad WebView capability, or an unbounded file target, or if existing log calls routinely emit sensitive data that cannot be scoped safely in this commit.

**Review mandate:**

- Confirm release builds register the file target and debug builds retain useful behavior.
- Confirm file size and retention are bounded.
- Confirm startup, migration, bridge orchestration, and package identity are diagnosable.
- Confirm no remote endpoint, telemetry, or new WebView permission appears.
- Confirm sensitive bridge and player data are not added to routine logs.

#### Commit 6 — Define the early alpha operating contract

**Status:** Pending

**Provisional commit:** `docs(release): define the early alpha contract`

**Work:** Rewrite the README around the product and early-alpha status, add end-user installation/update/uninstall/recovery/troubleshooting instructions, add the unofficial-project and unsigned-installer warnings, document the exact supported runtime and known limitations, add `SECURITY.md` for private vulnerability reporting, and add a maintainer release runbook covering the universal template-based PR procedure, candidate acceptance, automatic publication, no-op behavior, failed staging-draft recovery, and emergency release withdrawal. Commit 7 owns the PR-time version-selection and changelog mechanics in that runbook.

**Out of scope:**

- Legal advice or a claim that memory reading complies with every FM licence or anti-cheat policy.
- Promising support for untested platforms, game builds, database scopes, or automatic upgrades.
- Implementing capabilities that the documentation describes.

**Implementation packet:**

- Owners and files: `README.md`, `SECURITY.md`, `bridge/README.md`, `.wiki/notes/` release runbook, and current-state architecture sections only where prior commits made release behavior true.
- Existing patterns to verify: `.wiki/INDEX.md` ownership rules, CONCEPT scope and EULA risk assumptions, bridge prerequisites and live-validation evidence, architecture distribution/test limits, and GitHub private reporting settings.
- Constraints and invariants: state only implemented or explicitly manual behavior; distinguish author dogfood from public support; document men's database and exact-build boost limits; use no personal email address; preserve developer setup instructions behind the product introduction.
- Dependencies and ordering: follows implemented package and diagnostics behavior so documentation can describe current truth.

**Implementation profile:** Luna Max — the source facts and target audience are established; the work is careful reconciliation rather than architecture design.

**Review profile:** Sol Medium — review must verify factual consistency, usable recovery steps, and absence of unsupported promises.

**Validation:**

- `./scripts/dev check`
- `git diff --check`
- Follow the documented install, log-location, closed-app backup, restore, update, and uninstall steps against a Windows candidate and confirm each expected result.

**Stop conditions:** Stop for a developer decision if the documentation would need a personal contact address, a legal conclusion, a support promise, or a tested-platform claim that repository or runtime evidence cannot establish.

**Review mandate:**

- Confirm the README leads with FM ValueScout rather than template/fork guidance.
- Confirm supported OS, FM edition/build boundary, Steam, BepInEx, men's database, unsigned installer, and manual update are explicit.
- Confirm backup, restore, diagnostics, uninstall, and issue-report instructions are executable and honest.
- Confirm boost risks and the fresh integrated-validation requirement are visible.
- Confirm SECURITY uses a private GitHub route without publishing personal contact data.
- Confirm current-state docs describe only implemented release behavior.

#### Commit 7 — Prepare every pull request for release evaluation

**Status:** Pending

**Provisional commit:** `build(release): prepare pull request release metadata`

**Work:** Add a Keep a Changelog file, a repo-local `.agents/skills/create-pr/SKILL.md`, a Release intent section in the repository PR template, and a read-only `./scripts/dev release-metadata [latest-tag] [release-intent]` command with focused tests. The skill becomes the one template-based procedure for every human-authored pull request. It accepts or selects `none`, `patch`, `minor`, or `major`; leaves version owners and dated changelog sections unchanged for `none`; evaluates the complete unreleased change set since the latest published tag and updates the changelog and durable version owners for an unambiguous `patch` or `minor`; validates the result; and stops for ambiguous compatibility or `major` decisions. After release preparation is committed under the normal checkpoint rules, it performs the existing duplicate check, push, and template-complete draft PR creation flow. The command supports full SemVer prerelease precedence, calculates the expected version from the selected intent, validates all durable version owners, extracts the exact matching dated changelog section, and emits machine-readable metadata for CI. Prepare the initial `0.1.0-alpha.1` version across the app, Rust lock entry, Tauri bundle, and bridge.

**Out of scope:**

- Deciding release intent from one Conventional Commit title or file-path heuristic instead of the complete user-visible diff.
- A second PR command or separate release-only pull-request type.
- Generating changelog prose, editing files, staging, committing, tagging, or calling GitHub from the metadata command.
- Adding a SemVer runtime dependency when a small tested parser can satisfy the prerelease contract.
- Stable `1.0.0` or updater metadata.

**Implementation packet:**

- Owners and files: `CHANGELOG.md`, `.agents/skills/create-pr/SKILL.md`, `scripts/dev`, a small release-metadata module and focused Vitest suite under `scripts/`, `package.json`, `src-tauri/Cargo.toml`, the root `app` entry in `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `bridge/FmDataBridge.csproj`, `.github/pull_request_template.md`, `AGENTS.md` only for a short route into the repo-local skill, and the Commit 6 release runbook for operator behavior and recovery.
- Existing patterns to verify: the repository's established “create a PR using the template” behavior, the private `fm-youth-tracker/.pi/prompts/create-pr.md`, its `scripts/release-metadata.ts` and tests, current `scripts/dev` routing, squash-only Conventional Commit titles, Tauri version ownership, Cargo locked resolution, generated `MyPluginInfo.PLUGIN_VERSION`, and Keep a Changelog headings.
- Constraints and invariants: one human PR procedure and one repository template; exactly one recorded release intent; `none` leaves version owners and dated changelog sections unchanged; later release-bearing preparation covers all changes since the latest published tag; `src-tauri/tauri.conf.json` remains canonical; all five durable version files agree; only the root Cargo package version may change in `Cargo.lock`; SemVer accepts prerelease identifiers and rejects invalid or non-increasing identities; `Unreleased` remains present; the version section is exact release-note content; the command is deterministic and read-only; the skill follows normal checkpoint authority, may push and create the requested PR only after the branch is committed and validated, and must not tag, merge, publish a GitHub release, or bypass a stop condition.
- Dependencies and ordering: follows the public operating contract so the same runbook can define release preparation; the verified-main workflow in Commit 8 consumes this command and metadata format.

**Implementation profile:** Terra xhigh — the code is bounded, but full prerelease ordering, multiple language manifests, Cargo lock preservation, initial-release behavior, and an agent-assisted semantic contract require careful cross-toolchain validation.

**Review profile:** Sol High — a false positive can authorize the wrong version or notes, while a false negative can block every main-branch release.

**Validation:**

- RED: tests first reject mismatched version owners, missing or duplicate changelog sections, invalid prerelease syntax, lower/equal versions marked release-bearing, and incorrect `patch`/`minor`/`major` calculations.
- Tests prove `0.1.0-alpha.1` is newer than no release boundary, `alpha.2` outranks `alpha.1`, an unchanged version returns `releaseRequired: false`, and the extracted release notes equal the complete dated changelog section.
- Exercise the repo-local skill against representative `none`, patch, minor, ambiguous, and major fixtures. All cases use the same template and publication procedure; `none` leaves version owners and dated changelog sections unchanged; patch and minor cover the complete unreleased range; ambiguous and major cases stop without publishing a pull request.
- Run the metadata command against the repository with no latest tag and the selected initial-release intent; require `version=0.1.0-alpha.1`, `tag=v0.1.0-alpha.1`, `releaseRequired=true`, and non-empty exact notes.
- `./scripts/dev bridge-test`
- `./scripts/dev check`
- Expected evidence: every owner reports `0.1.0-alpha.1`; only the root `app` entry changes in `Cargo.lock`; `CHANGELOG.md` retains `Unreleased`; the command leaves the worktree byte-for-byte unchanged.

**Stop conditions:** Stop and request a developer decision if Tauri or MSBuild rejects the prerelease identity, the bridge's runtime plugin version cannot match the app without generated source, calculating the next alpha version needs a policy beyond the recorded intent rules, or Cargo changes any dependency while refreshing the root package version.

**Review mandate:**

- Confirm every human pull request uses the same skill and template, records exactly one release intent, and does not require a separate release-only PR procedure.
- Confirm `none` defers release without changing version owners or dated sections, while a later patch or minor includes the complete unreleased change set since the latest published tag.
- Confirm routine unambiguous outcomes need no separate release-classification prompt, while ambiguous or major outcomes stop for maintainer input.
- Confirm the next-version calculation follows the recorded alpha SemVer policy and handles all precedence cases used by the workflow.
- Confirm every durable version owner and only the root Cargo lock entry moves together.
- Confirm changelog extraction returns one exact dated section and cannot bleed into adjacent versions.
- Confirm malformed, missing, duplicated, stale, or lower metadata fails closed.
- Confirm the command has no write, Git, network, or GitHub side effect.

#### Commit 8 — Publish prereleases from verified main

**Status:** Pending

**Provisional commit:** `ci(release): publish prereleases from verified main`

**Work:** Replace the tag-triggered multi-platform release matrix with a Windows x64 workflow that follows each successful `Check` run caused by a push to `main`. Check out the exact verified SHA, evaluate release metadata, exit successfully when the version did not advance, and otherwise reuse `package-windows` in the privileged final job. That job creates or repairs only a matching temporary draft, uploads the installer and checksum, sets the version title and exact changelog section, verifies the assembled release, and publishes it with `prerelease=true` and `draft=false` before succeeding. Add release-metadata validation and the candidate-package job to the required Check path for release-input changes.

**Out of scope:**

- Signing, updater metadata, immutable-release configuration, store upload, or unsupported platforms.
- CI-generated version commits, changelog prose, release-intent inference, or direct pushes to `main`.
- Bypassing strict branch protection or granting write permission to pull-request, metadata, or build jobs.

**Implementation packet:**

- Owners and files: `.github/workflows/release.yml`, `.github/workflows/check.yml`, the Commit 6 release runbook, and `CHANGELOG.md` only for the release-automation entry in the still-unpublished initial version section.
- Existing patterns to verify: the reviewed `fm-youth-tracker` `workflow_run` handoff, current path filtering and aggregate `check`, Windows bridge job, GitHub release REST/CLI draft and prerelease semantics, exact `workflow_run.head_sha`, explicit workflow permissions, and the candidate artifact path from Commit 4.
- Constraints and invariants: run only after a successful Check push run on `main`; check out that exact SHA; read-only by default; `contents: write` only in the final staged-publication job; one Windows installer plus checksum; exact changelog notes; no unsupported matrix; publish only after all assets and metadata are assembled and verified; an existing draft may be repaired only for the same version and SHA; an existing published tag/release makes an exact retry a no-op and any mismatch a failure; action references are pinned to reviewed commit SHAs where third-party actions remain.
- Dependencies and ordering: final implementation commit; depends on deterministic packaging, release metadata, local diagnostics, and the release runbook.

**Implementation profile:** Terra xhigh — the workflow crosses event provenance, exact-SHA checkout, release idempotency, Windows packaging, artifact transfer, SemVer metadata, and GitHub permissions.

**Review profile:** Sol High — a mistake can publish or overwrite the wrong artifact, target an unchecked commit, leak write authority, or make every `main` push fail.

**Validation:**

- `./scripts/dev test`
- `./scripts/dev bridge-test`
- `./scripts/dev smoke`
- `./scripts/dev check`
- GitHub strict `check` passes with release metadata and the Windows candidate-package job included for this PR.
- Exercise workflow conditions against fixtures or expressions for failed Check, pull-request Check, successful non-`main` Check, unchanged version, newer version, failed temporary draft, mismatched existing draft or published release, and exact retry.
- Download the candidate workflow artifact, verify its SHA-256, install it, confirm the bundled bridge is a managed DLL with the matching version, confirm a local release log is created, and complete the non-FM recovery checklist.
- After merge, confirm the Release run checks out the Check run's exact `head_sha`, creates and publishes one `v0.1.0-alpha.1` prerelease, and uses the exact matching changelog section as its complete body.

**Stop conditions:** Stop and replan if `workflow_run` cannot unambiguously bind the successful Check to the pushed `main` SHA, if a read-only job can mutate releases, if an existing draft, tag, or release can be retargeted or overwritten from a different SHA, if publication can occur before complete asset and metadata verification, if the candidate and release jobs do not use the same package command, or if the downloaded installer does not contain the verified bridge.

**Review mandate:**

- Confirm the Release workflow handles only successful push-origin Check runs on `main` and checks out `workflow_run.head_sha`.
- Confirm unchanged metadata is a successful no-op and invalid or stale metadata fails before packaging.
- Confirm all earlier jobs are read-only and only the final staged-publication job has `contents: write`.
- Confirm the workflow has one Windows x64 target and no unsupported platform assets.
- Confirm the candidate and main release paths call the same tested metadata and package commands.
- Confirm version, target SHA, bridge version, release title, tag identity, artifact name, checksum, and exact changelog notes agree.
- Confirm an exact retry is idempotent and a mismatched existing draft, tag, or release fails without mutation.
- Confirm successful completion leaves one published prerelease (`draft=false`, `prerelease=true`) and no unrelated release is mutated.
- Confirm third-party action references on the release path are immutable or justified.

## Active work

**PR:** PR 1 — Prepare Windows early alpha distribution

**Commit:** Commit 6 — Define the early alpha operating contract

### RED proof

The README must replace the template-fork framing with executable early-alpha operating guidance without promising unimplemented support, recovery, or legal conclusions.

### Expected outcome

The README, security route, and maintainer runbook accurately describe the supported Windows early-alpha install, update, recovery, diagnostics, and release operations.

### Explicit exclusions

- Legal advice, personal contact details, or a support-level promise.
- Claims for untested platforms, game builds, database scopes, automatic updates, or recovery behavior.
- Implementing capabilities only because they are documented.

## Discoveries and replanning

- Planning confirmed on 2026-08-14 that the bundled `FmDataBridge.dll` is a 102-byte text placeholder and the release workflow does not build the bridge. This is a release blocker and is owned by Commits 2 and 4.
- GitHub inspection on 2026-08-14 confirmed squash-only merges, strict up-to-date required `check`, no required approval, no tags or releases, enabled secret scanning and push protection, and disabled repository auto-merge, Dependabot alerts, and Dependabot security updates.
- Inspection of `JG1995/fm-youth-tracker/.pi/prompts/create-pr.md`, its release-metadata validator and tests, changelog, and verified-main Release workflow on 2026-08-14 confirmed a reusable split between pre-merge release intent and post-Check release creation. This ledger adapts that split for prerelease SemVer, one Windows artifact, exact changelog notes, and automatic publication after verified `main`.
- The reference workflow publishes automatically and supports stable SemVer only. This feature keeps its automatic-publication boundary, adds `0.1.0-alpha.1` ordering, and moves installed-candidate acceptance before the initial release-bearing PR merges.
- GitHub documentation reviewed on 2026-08-14 confirms that `cooldown.default-days` can delay routine version updates by 14 days, cooldown does not apply to security updates, Dependabot metadata reports the highest grouped SemVer change and every updated dependency, and native auto-merge waits for required branch protection. The revised plan automatically squash-merges only verified stable pnpm/Cargo patches after `check`, disables routine Actions and NuGet updates, and leaves every other dependency pull request unmerged.
- The pinned `dependabot/fetch-metadata` v3.1.0 source reports pnpm projects as `npm_and_yarn` in its metadata, while Dependabot configuration still requires `npm`. Commit 3 preserves that boundary so eligible pnpm patches are classified rather than accidentally rejected.
- Sol High review found that GitHub retains auto-merge after a maintainer with write permission pushes to an eligible Dependabot pull request. Commit 3 therefore re-evaluates later events and revokes an existing auto-merge request when metadata verification or policy eligibility fails; its read-only policy job is part of the strict `check` aggregate so an ineligible changed revision cannot win a workflow scheduling race.
- Official Codex documentation reviewed on 2026-08-14 confirms that `.agents/skills` is the repository-scoped skill location. The existing template-based PR behavior is not currently repository-owned, so this feature will formalize it as `.agents/skills/create-pr/SKILL.md` and make release intent one field in that universal procedure. Deterministic metadata validation and release publication remain in repository scripts and GitHub Actions.
- Repowise was synchronized to `0c9c10e41b59941d08b90a5f493283836d149830`. It identifies `src-tauri/src/db/migrations.rs` and `scripts/dev` as high-churn/high-impact surfaces; deterministic migration and Windows packaging evidence remains authoritative.
- NuGet serializes an exact `Version="[x]"` constraint in `packages.lock.json` as the canonical requested range `[x, x]`. Commit 2 uses those exact constraints for both validated BepInEx packages; the bridge test first rejected the prior open-ended locks, then passed after regeneration with locked restore enabled.
- The local Windows host has only the .NET 9 SDK, while `bridge/global.json` pins .NET 6. The existing Windows CI job installs .NET 6 before its locked bridge test; that clean-Windows proof remains for the first push of this commit.
- Sol High review of Commit 4 required the root `app` lock entry to be part of release identity validation and the Tauri Cargo invocation to use `--locked`. The package command now isolates and clears version-scoped ignored build and artifact directories; the first native Windows package run remains required evidence.
- No planned feature spec existed to promote. This ledger is the sole active source of feature intent.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Remove the template health scaffold | Pending record | Completed | Passed — no blocking findings | None |
| PR 1 | Commit 2 — Lock release build inputs | Pending record | Completed | Passed — no retained findings | Clean Windows CI restore awaits the first push |
| PR 1 | Commit 3 — Automate guarded dependency patches | Pending record | Completed | Passed — no retained findings after two corrective review rounds | Live GitHub settings and fixture validation await the first push |
| PR 1 | Commit 4 — Package the Windows bridge from source | Pending record | Completed | Passed — no retained findings after corrective review | Native Windows package and extraction evidence await the first push |
| PR 1 | Commit 5 — Retain local release logs | Pending record | Completed | Passed — no retained findings | Packaged Windows log creation, rotation, and privacy inspection await the first push |

## Final validation

**Feature review profile:** Sol xhigh — final review must trace the cross-commit source-to-installer chain, migration safety, local diagnostics privacy, Dependabot and release permission boundaries, automatic-publication fail-closed behavior, and manual installed-app evidence. A missed interaction could merge unsafe executable input, ship a nonfunctional bridge, expose unintended data, or create a misleading public artifact.

Before feature review:

- `./scripts/dev format`
- `./scripts/dev test`
- `./scripts/dev bridge-test`
- `./scripts/dev smoke`
- `./scripts/dev check`
- `./scripts/dev secrets`
- On Windows: `./scripts/dev package-windows`
- Run `./scripts/dev release-metadata` against the latest release boundary and confirm `0.1.0-alpha.1`, `v0.1.0-alpha.1`, `releaseRequired: true`, matching version owners, and exact changelog notes.
- Confirm the required GitHub `check` passes, including release metadata and the candidate-package job for the final PR.
- Download the candidate artifact from GitHub, verify its checksum, install and launch it, confirm its app/bridge versions, confirm local logs, and execute the documented backup/restore/update/uninstall checks.
- With BepInEx present, install or update the bundled bridge, restart FM26, and confirm bridge version/status and one complete men's-database Load Data flow on the supported build.
- Exercise Search, profile, configurable tables, CSV import, Planner, Academy, restart persistence, and prior-data preservation from the installed candidate.
- Back up the FM save before testing both player-boost actions. Confirm the exact-build guard, live readback, SQLite reconciliation, and recovery copy from the installed candidate. If a supported FM session is unavailable, record the accepted gap and do not merge the initial release-bearing PR, because merge success will publish the prerelease automatically.
- Enable repository auto-merge, Dependabot alerts, and Dependabot security updates. Confirm routine pnpm/Cargo updates are patch-only and use the 14-day cooldown; routine Actions and NuGet updates remain disabled; only stable `1.x`-or-newer pnpm/Cargo patches can enable native squash auto-merge; and all other dependency pull requests remain unmerged. Verify private vulnerability reporting and decide whether to enable immutable releases before the first public publication. Repository settings are manual; the Dependabot policy and metadata-only workflow are owned by Commit 3.
- Use the universal `create-pr` procedure for the initial release-bearing branch and confirm its template records the selected intent. Merge it only after its version, changelog, and candidate acceptance remain current against `main`. Confirm the successful Check run triggers release evaluation for the exact merge SHA and automatically publishes one unsigned Windows prerelease with the exact changelog section.
- Confirm a later successful same-version documentation-only push to `main` exits without creating or mutating a release, and exercise the documented recovery path for a matching temporary draft without publishing any mismatched artifact.

## Documentation impact

Complete during reconciliation. Expected owners are `README.md`, `SECURITY.md`, `CHANGELOG.md`, `AGENTS.md`, `.agents/skills/create-pr/SKILL.md`, `.github/pull_request_template.md`, `bridge/README.md`, `.wiki/ARCHITECTURE.md`, `.wiki/notes/`, `.wiki/TODO.md`, and the completed feature record. `CONCEPT.md` should need no scope change unless implementation widens the supported release contract.
