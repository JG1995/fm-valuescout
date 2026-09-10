# Local FM graphics

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** d64905ed4105624b208f870340d825ff0b2616dae9256a9cbe4b2e8fcdfe5f18

## Intent

Deliver Linear JAY-63. FM ValueScout will index one user-selected local FM26 graphics root and use the local mappings to show player portraits and UID-backed club logos. It will not download, copy, bundle, manage, or redistribute graphics packs.

## User-visible behavior

- Settings detects a conventional FM26 Documents or OneDrive graphics root when possible, or starts a Rust-owned native folder choice. Users can clear and manually rescan one app-wide root.
- Settings reports safe aggregate scan diagnostics. A missing or unreadable root does not affect Load Data or other features.
- Player Profile, Search, and Squad show a local player portrait and current-club logo in their existing fixed identity seams when a valid UID mapping exists. Missing graphics remain the normal person or shield fallback.
- My Club shows the selected managed-club logo only when its exact stored club UID resolves. Club names remain visible text identity.
- The index rebuilds after a root change, after **Rescan graphics**, and lazily on the first resolve after restart. There is no watcher or persisted generated index.

## Invariants

- Local packs only. Player portraits are the only person images. Do not add Staff portraits.
- Portraits resolve only by player UID. Club images resolve only by extracted club UID. Never match a name, team UID, or a React-supplied path.
- New v9 dumps contain nullable player `currentClubUid` and `parentClubUid`, staff `clubUid`, and manager `clubUid`. Rust rejects v8 and older dumps with the existing update-and-rescan message. Existing SQLite rows remain readable with null IDs.
- The bridge reads a positive `u32` only at `club + ObjectUidOffset`; it never stores a team UID as a club UID.
- React sends only graphics kind and positive UID to resolve an image. Rust owns folder choice, root validation, discovery, parsing, file containment, image reads, MIME detection, and result bounds. No command returns a filesystem path.
- Rust opens the selected root once with ambient authority. Every later directory and file traversal or open uses that root capability or a descendant capability. It opens each descendant component from its already-open parent capability, including every later source-resolution component. The engine retains only root-relative capability-owned mapping identity; it never stores canonical or absolute paths.
- Discovery is recursive, bounded, no-follow, and independent of discovery order. It skips symlinked directories, Windows junctions/reparse points, and non-regular files. The streaming parser has no external entity or network behavior. It accepts no arbitrary XML path or source escape.
- Bounds are 32 directory levels, 10,000 `config.xml` files, 1,000,000 directory entries across the whole scan, 8 MiB per config, 500,000 accepted mappings across indexes, and 8 MiB per returned image. Capability discovery streams under the fail-closed global entry budget. It retains at most the lexically smallest 10,000 root-relative config identities in a bounded max-heap, accounts for config overflow/truncation, then globally sorts that retained set before parsing. A config-count overflow may process only those globally first 10,000 configs; entry-budget truncation discards every candidate index. Invalid records and read failures produce categorized aggregate diagnostics.
- Global root-relative lexical config-path order and first valid mapping determine duplicate precedence after the retained-set sort; arbitrary discovery order cannot change installed precedence. A club `logo` beats `icon` at resolution. Extensionless sources probe PNG, JPEG/JPG, then WebP in that order.
- Root change and rescan replace the whole runtime generation and clear positive and negative runtime results atomically. Frontend Query keys include that returned generation, and root-change/rescan success removes old generation keys before new results can render.
- Slots stay fixed while requests are pending, missing, or invalid. Existing names remain visible and accessible.
- No graphics-pack asset, private local path, live dump, UID sample, or real FM data enters the repository.

## Non-goals

- Staff imagery, remote images, name matching, arbitrary WebView filesystem reads, watcher support, persisted generated indexes, or full FM pack-priority emulation.
- Pack installation, copying, download, synchronization, management, or redistribution.
- Changes to managed-club membership, player/staff search behavior, scoring, snapshots, bridge writes, releases, versions, or package publication.

## Current-state map

- PR 1 is the completed, merged club-identity foundation. `bridge/Layouts/Fm263Layout.cs` sets `ObjectUidOffset = 0x0C`; `ContractClubReader`, `SquadClubIndex`, `HumanManagerSelector`, and `CapADumpPipeline` own the club-address and selected-current/parent seams. `BridgeProtocol`, `DumpDocument`, `DumpWriter`, and `bridge/DUMP_SCHEMA.md` specify schema v9.
- `src-tauri/src/features/memory_read/dump_validation.rs` accepts v9 and rejects v8 and older. `snapshot/ingest.rs` prepares outside the database mutex and publishes snapshots atomically. `db/migrations.rs` ends at v45. Player raw current/parent-club UID fields, staff club UID, snapshot manager club UID, and managed-club `club_uid` are nullable; legacy stored rows remain readable with null IDs.
- `managed_club/service.rs` lists exact effective-current `{ clubName, clubUid }` options, validates and persists the submitted name/UID pair, and returns the nullable UID with `Unconfigured`, `Available`, or `Missing` status. Legacy name-only selections retain a null UID without rebinding. Current cohort SQL remains name based.
- `player/query.rs`, `search/query.rs`, and `planner/squad.rs` own the Player Detail, Search, and Squad read projections. Staff has no in-scope presentation consumer.
- `player-identity.tsx` has a player-initial and club-monogram rail. `search-results-panel.tsx` and `squad-overview-panel.tsx` each own the dense player identity cell. `ManagedClubSelector` owns the My Club context seam. `src/app/routes/settings.tsx` owns Settings composition.
- `tauri-plugin-dialog` and `dialog:allow-open` exist. CSV import is a picker analogue, but it receives a React path and is not acceptable for this root trust boundary. `src-tauri/Cargo.toml` sets `rust-version = "1.77.2"`; the merged lock has `quick-xml` 0.41.0 indirectly but no capability-filesystem crate.
- `.github/workflows/check.yml` runs Rust checks only on Linux. Its existing Windows job runs bridge tests, so it cannot prove the Rust Windows junction/reparse path.
- Supported validation is `./scripts/dev bridge-test`, `./scripts/dev test`, `./scripts/dev check`, `./scripts/dev smoke`, `./scripts/dev inspect-ui`, and `./scripts/dev bridge-install`. Browser inspection is Chromium-stub evidence only.

## Feature architecture

1. Completed PR 1 advanced the dump contract to v9, added nullable raw persistence in v44, and bound new managed-club selections to the exact current-snapshot club UID in v45. It preserves legacy null rows and text-led membership queries.
2. PR 2 first adds a pure Rust graphics engine. It opens the trusted selected root once, then uses capability filesystem handles and root-relative mapping identity for every traversal and image open. Each descendant component opens from its already-open parent capability, including later source resolution. It produces immutable person and club indexes plus safe diagnostics, and resolves one bounded image result without SQLite, Tauri state, commands, or React.
3. The next packet owns v46 root persistence, Documents/OneDrive candidate discovery and picker start selection, the zero-argument Rust picker, `GraphicsRuntime`, command registration, root generations, and a bounded result cache. It exposes safe non-path status and kind-plus-UID resolution only.
4. Settings owns typed IPC and Query lifecycle. Presentation stays organized by UI seam: Profile projects its player club UID; Search and Squad project their table club UIDs together; My Club consumes its already stored selected UID.

## Uncertainty register

### Known

- The FM26 club-address path, merged schema-v9/v44/v45 identity foundation, dialog, Query, route, and identity seams above exist in source.
- `DESIGN.md` currently says bundled-only/no portraits-or-crests because no source exists. Delivery must reconcile that contradiction; it is not optional documentation polish.
- The protected-main rule, GitHub PR template, squash merge, and release exclusion are recorded in the existing ledger and repository contract.

### Decisions

- Keep two PRs. PR 1 is the independently mergeable schema and persistence boundary. PR 2 depends on its squash merge being reachable from synchronized `main` and contains the local graphics feature.
- Persist only the configured root in SQLite. Keep indexes and image bytes in memory only.
- Use the Rust native folder picker and narrow commands. Do not widen the existing WebView dialog authority for a caller-supplied root.
- Retain `quick-xml` 0.41 as the direct streaming parser, reusing the merged lock resolution. Add only `cap-std` 3.4.6 and `cap-fs-ext` 3.4.6 as direct filesystem dependencies. `cap-std::fs::Dir::open_ambient_dir` provides the one ambient root open; `cap-fs-ext` supplies `follow(FollowSymlinks::No)` and Windows `maybe_dir(true)` for no-follow child opens. Do not add `cap-primitives` directly: Cargo resolves it transitively at 3.4.6. The cap-std 3.4.6 workspace declares Rust 1.63, which is compatible with this crate's 1.77.2 MSRV, and it replaces the suggested vulnerable 3.4.5 line. Update `Cargo.lock` with the exact resolved capability crate set. Do not change the MSRV or toolchain configuration.
- Add a `rust-windows` Windows Rust `./scripts/dev check-rust` job to the existing Check workflow with the existing Rust job's checkout, pinned toolchain, and Rust-cache pattern. Set `shell: bash` for that command. Add `rust-windows` to `check.needs`, add `RUST_WINDOWS: ${{ needs.rust-windows.result }}` to the required-result environment, and include `"$RUST_WINDOWS"` in the aggregation loop. Keep it with this trust-boundary packet because Linux cannot prove Windows junction/reparse behavior.
- Cache at most 256 available and 256 missing results per graphics kind. Evict least-recently-used entries within each class; replace the complete cache on every accepted root generation.
- Do not create an ADR. These choices apply the existing Rust-owned SQLite, filesystem, and IPC architecture.

### Unknowns and accepted gaps

- The installed dialog crate must expose a Rust-owned directory picker without a capability change. Stop if it cannot.
- Live FM26.3.2 must still verify `club + 0x0C` for senior, reserve/youth, and loan current/parent cases. Real-pack scale and scan time are also unknown. Both are disclosed post-merge manual checks, not automated merge gates.

### Risks

- A team UID, stale schema field, or partial ingest would misidentify a logo or lose identity. Keep the protocol writer, validator, prepared ingest, migration, and persistence together.
- Malformed local packs can escape containment or consume work. The pure engine must prove every bound and deterministic precedence on a temporary filesystem. It must reject an entry-budget-truncated candidate before installation and prove no-follow Windows junction/reparse and intermediate-directory replacement behavior on Windows.
- A capability crate or CI job that cannot provide no-follow child directory/file opens on Windows would invalidate the approved architecture. Keep the dependency surface to `cap-std` plus `cap-fs-ext`; `cap-primitives` remains transitive.
- Overlapping scans and stale Query data can render an old positive or missing result. Generation replacement and generation-keyed invalidation are required.
- Profile and dense-table changes can shift geometry. Keep each UI seam atomic and inspect populated routes.

## Walking skeleton

PR 1 first records the plan, then stores a nullable extracted current-club UID in a v9 snapshot and saves an exact UID with a new managed-club selection. PR 2 then indexes a synthetic nested tree, configures it through Settings, resolves one player portrait, and renders it in the Profile rail. Search/Squad and club-logo seams extend that proven resolver without changing its trust boundary.

## Delivery plan

### PR 1 — Persist FM club identity

**Status:** Merged

**PR ref:** <https://github.com/JG1995/fm-valuescout/pull/140>

**Merge ref:** 1f94a491a535adfb924f86db1e7e7f751ba067f4

**Branch:** `feature/local-fm-club-identity`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub protected `main` requires strict status `check`

**Feature close-out:** Not required

**CI repair rounds:** 0

**Provisional PR title:** `feat(memory-read): persist FM club identities`

**Purpose:** Merge the versioned nullable club-identity foundation before a local filesystem runtime or image UI depends on it.

**Depends on:** None.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(graphics): record local FM graphics plan`

**Work:** Record the reviewed JAY-63 plan and its already-correct Active TODO entry before implementation.

**Size assessment:** No implementation code. This is maximally atomic because it records the accepted delivery authority and no behavior.

**Distinct outcome:** One classifier-valid schema-2 plan is the active implementation contract.

**Out of scope:** Implementation, tests, executable configuration, BACKLOG, ADRs, the two unrelated dirty completed-feature records, and all Git mutations beyond the later authorized commit.

**Implementation packet:** Preserve the accepted plan and record only its ledger and matching TODO entry before implementation.

**Files and responsibilities:**

- `.wiki/features/active/local-fm-graphics.md` — intent, two PRs, ten commit packets, authority, risks, and validation.
- `.wiki/TODO.md` — one Active JAY-63 link.

**Behavior and data flow:** Planning truth moves into the active ledger before code changes.

**Ordered implementation steps:**

1. Verify the recorded branch and base without changing Git state.
2. Run the ledger classifier against the unstaged active ledger and confirm the reviewed ledger is runnable.
3. Confirm only the reviewed ledger and JAY-63 TODO entry belong to the planning artifact; no planned spec, ADR, or BACKLOG change exists.
4. Under later fingerprint-authorized delivery authority, stage exactly `.wiki/features/active/local-fm-graphics.md` and `.wiki/TODO.md`.
5. Inspect the complete cached stat and diff, run `git diff --cached --check`, then obtain checkpoint review before the later commit.

**Tests and proof:** Not applicable — independently reviewed planning documents only. The pre-stage classifier, exact cached inspection, cached whitespace check, and checkpoint review prove the record.

**Patterns to verify:** The schema-2 active-ledger template and TODO ownership rule.

**Constraints and non-goals:** Do not change plan scope, implementation, delivery authority, or unrelated dirty records. Do not use plain `git diff --check` as a substitute for the cached check; the ledger is initially untracked.

**Dependencies and sequencing:** Cleared independent plan review, developer acceptance, a recorded fingerprint, and activation of the recorded branch precede the later staging and commit.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/local-fm-graphics.md` before staging; then `git diff --cached --stat`; complete `git diff --cached`; and `git diff --cached --check`.

**Stop conditions:** Stop for a classifier failure, uncleared review, changed branch/base, unreviewed planning path, an inexact staged set, a cached whitespace failure, or either unrelated dirty completed ledger in the proposed commit.

**Review mandate:** Confirm the pre-stage classifier result, one active commit, the approved two-PR/ten-commit structure, exact cached ledger/TODO diff and stat, cached whitespace result, checkpoint review, no implied authority before a fingerprint, and no unrelated path.

#### Commit 2 — Upgrade dumps to schema v9 and persist nullable club UIDs

**Status:** Completed

**Provisional commit:** `feat(memory-read): persist nullable club UIDs`

**Work:** Make v9 the only ingestible new dump and atomically retain nullable player, staff, and manager club-object UIDs through prepared ingest and SQLite.

**Size assessment:** Above the soft target. This is maximally atomic: bridge producer, v9 documentation and fixtures, Rust validator, prepared ingest, v44, and raw persistence form one contract; splitting publishes a rejected dump or silently discards a validated field.

**Distinct outcome:** A v9 dump stores nullable club identity faithfully, while v8 remains stale and prior SQLite snapshots read null.

**Out of scope:** Read DTOs and TypeScript types/mocks; graphics runtime/UI; managed-club binding; changed cohort SQL; all staff-facing UI.

**Implementation packet:**

**Ordered implementation steps:** Add fake-memory and canonical v9 fixture RED proof; update producer/writer; migrate every valid-path fixture consumer to v9 while retaining v8 only for stale rejection; update validation and prepared ingest; add v44 and transactional persistence; run bridge then Rust gates.

**Tests and proof:** Add only fake-memory and synthetic v9 fixture coverage for positive/null, loan, staff, manager, stale schema, migration, and atomic ingest parity. Fake-memory tests distinguish current and parent loan IDs and null failures. Make `golden_dump_v9.json` the canonical fixture for every current valid-path consumer in snapshot, player, search, planner, staff, academy, CSV, Moneyball, and memory-read tests; retain `golden_dump_v8.json` only for explicit stale-schema negative validation. Rust tests prove v9 acceptance, explicit v8 rejection, v44 null preservation, raw insert parity, and canonical-v9 valid paths across the named consumers.

**Patterns to verify:** `NationReader` positive UID validation, `ContractClubReader` overflow helpers, stale-fixture validation, and existing additive migration tests.

**Constraints and non-goals:** Store raw identity only; do not add query DTOs, mocks, UI, compatibility ingest, or name/team inference.

**Dependencies and sequencing:** Follows Commit 1. Commit 3 uses the stored player current UID, and all later player-facing projections depend on this merged contract.

- Extend `ContractClubLink`, `SquadAssignment`, and `HumanManagerClubAssignment` with a nullable positive club UID read only from `club + ObjectUidOffset`. `CapADumpPipeline` emits player `currentClubUid` from selected squad club and `parentClubUid` from the contract club; staff emits its contract `clubUid`; manager emits its selected graph-or-contract `clubUid`. Retain all current name, loan, and manager-choice rules.
- Advance `BridgeProtocol.DumpSchemaVersion`, dump models/writer, `bridge/DUMP_SCHEMA.md`, and v9 golden/stale fixtures together. Create `golden_dump_v9.json` as the canonical valid fixture, migrate every current valid-path consumer in snapshot, player, search, planner, staff, academy, CSV, Moneyball, and memory-read tests to it, and retain `golden_dump_v8.json` only for explicit stale-schema rejection. Rust validates positive-or-null UID fields and rejects v8 and older with the existing update-and-rescan message.
- Add v44 nullable columns for player current/parent UID, staff UID, and snapshot manager UID. Thread fields through raw preparation, scoring preparation, and the existing atomic insert. Existing rows receive null.
- Do not add downstream DTO fields merely because storage exists. The later presentation packets introduce each player-facing projection at its consumer; staff and manager UIDs have no in-scope consumer.

**Files and responsibilities:**

- `bridge/Extraction/{ContractClubReader,SquadClubIndex,HumanManagerSelector,StaffReader}.cs` and `bridge/Scanning/CapADumpPipeline.cs` — source only club-object UIDs at existing address seams.
- `bridge/{Models/DumpDocument.cs,Protocol/BridgeProtocol.cs,Output/DumpWriter.cs,DUMP_SCHEMA.md}` and bridge fake-memory/writer tests — v9 producer and frozen contract.
- `src-tauri/src/features/memory_read/dump_validation.rs` and `src-tauri/src/features/memory_read/fixtures/{golden_dump_v8.json,golden_dump_v9.json}` — canonical v9 validation; explicit stale v8 rejection only.
- Current valid-path fixture consumers in `src-tauri/src/features/{snapshot,player,search,planner,staff,academy,csv_import,moneyball,memory_read}/` — migrate to `golden_dump_v9.json`; retain v8 references only in the stale-validation test.
- `src-tauri/src/features/snapshot/ingest.rs` and `src-tauri/src/db/migrations.rs` — prepared values, v44, atomic persistence, and migration/ingest proof.

**Behavior and data flow:** Person/contract/team/club and selected-squad/club reads produce nullable JSON UIDs; Rust validates them, prepares outside the DB lock, and inserts them with the retained snapshot transaction.

**Validation:** `./scripts/dev bridge-test`; `./scripts/dev check-rust`; `./scripts/dev check`.

**Stop conditions:** Stop and replan if a UID cannot be read from a club object, a required field lacks an atomic persistence owner, a v44 upgrade changes legacy data, a current valid-path fixture consumer remains on v8, or a proposed consumer needs a changed membership rule.

**Review mandate:** Trace every UID to `club + 0x0C`; check current/parent and manager selection parity; verify v9 is canonical for every named valid-path fixture consumer and v8 is stale-negative-only; compare writer, schema docs, fixture, validator, prepare, migration, and insert; reject speculative DTO exposure and any staff-facing UI.

#### Commit 3 — Bind managed-club selections to club UIDs

**Status:** Completed

**Provisional commit:** `feat(managed-club): bind selections to FM UIDs`

**Work:** Keep the text-led managed-club picker while saving the selected effective-current option's exact nullable club UID.

**Size assessment:** Within the soft target. The v45 migration, pair-validation command contract, typed picker option, and existing selection UI are one maximally atomic exact-selection outcome; either half alone cannot prevent later name lookup.

**Distinct outcome:** New saved selections carry an exact UID; legacy name-only selections remain readable and deliberately unresolved for logos.

**Out of scope:** Rebinding legacy rows by name, changing cohort membership SQL, logo rendering, player/staff DTOs, or Staff images.

**Implementation packet:**

**Ordered implementation steps:** Add v45 and mismatched-pair RED proof; return typed options/status; validate and save the exact pair; update picker and doubles; run frontend and full gates.

**Tests and proof:** Prove a valid pair persists, a mismatched pair rejects, and a legacy row retains its displayed name plus null identity without rebinding.

**Patterns to verify:** Existing `ManagedClubSelector` controlled combobox, effective-current query boundary, and v29-style migration preservation.

**Constraints and non-goals:** Do not rebind legacy state by name, alter cohorts, render images, or expose player/staff DTOs.

**Dependencies and sequencing:** Requires Commit 2. PR 2 Commit 6 consumes its status UID.

- Add v45 nullable `managed_club_settings.club_uid`. List typed `{ clubName, clubUid }` options from the effective current snapshot. Preserve existing display/search behavior.
- Change set/status contracts to include the selected UID. Rust validates the submitted name-and-UID pair against the effective current snapshot before write. It retains the name-based queries and missing state; it never recovers a legacy ID by name.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — v45 and legacy-null migration proof.
- `src-tauri/src/features/managed_club/{service.rs,commands.rs}` — typed options/status, exact-pair validation, persistence, and service tests.
- `src/features/managed-club/{api,types,components/managed-club-selector.tsx}` plus existing IPC mocks and My Club tests — typed picker values with unchanged text interaction.

**Behavior and data flow:** The effective snapshot provides a name/UID option; React returns that pair; Rust validates and persists it; later My Club presentation reads the stored UID while current cohorts still compare saved name.

**Validation:** `./scripts/dev test`; `./scripts/dev check`.

**Stop conditions:** Stop if the current source cannot provide a stable exact option, pair validation would change cohort membership, or legacy recovery requires name matching.

**Review mandate:** Check v45 preservation, exact effective-current validation, unchanged picker text/accessibility, absent name-derived identity, and unchanged membership SQL.

### PR 2 — Add local FM graphics

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/local-fm-graphics`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub protected `main` requires strict status `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(graphics): add local FM graphics`

**Purpose:** Add the local filesystem trust boundary, its Settings control plane, and separately reviewable presentation seams after the identity foundation merges.

**Depends on:** PR 1 merged by squash, with its immutable merge ref reachable from synchronized `main`.

#### Commit 1 — Build the bounded local graphics index

**Status:** Completed

**Provisional commit:** `feat(graphics): index local FM graphics`

**Work:** Replace the invalidated `std::fs` scanner with a pure Rust, temporary-filesystem-testable capability index and bounded image resolver.

**Size assessment:** Above the soft target because the one trust-boundary outcome needs a capability traversal, root-relative mapping identity, global entry budget, streaming XML parsing, immutable accepted indexes, deterministic precedence, bounded image validation, and its Windows CI proof. Splitting scanner, resolver, or Windows validation would leave a partial containment contract on trunk.

**Distinct outcome:** An accepted complete scan from one trusted capability root produces immutable deterministic indexes and a bounded available-or-missing image result; an entry-budget-truncated scan installs no index.

**Out of scope:** SQLite, root persistence, folder picker, managed app state, registered commands, cache, React, Query, Settings, and all presentation.

**Implementation packet:**

**Ordered implementation steps:**

1. Add temporary-tree RED tests for accepted scans, global entry-budget rejection with no installed index, globally root-relative duplicate precedence, and capability no-follow behavior. Add an adversarial ordering tree where per-directory DFS differs from full root-relative ordering, such as `a/config.xml` before sibling `a.b/config.xml` in DFS even though `a.b/config.xml` sorts first because `.` precedes `/`. Add Windows-only junction/reparse and intermediate-directory replacement proof before production code; make that proof run in the new Windows Rust CI job.
2. Replace the staged `std::fs` scanner. Retain direct `quick-xml` 0.41, add direct `cap-std` 3.4.6 and `cap-fs-ext` 3.4.6, and update `Cargo.lock`. Verify the lock has the patched 3.4.6 `cap-std`, `cap-fs-ext`, and transitive `cap-primitives` set, with no direct `cap-primitives` manifest dependency.
3. At the engine entry point, open the selected root exactly once with `Dir::open_ambient_dir`. Pass only the resulting root or descendant directory capability through discovery and resolution. Replace stored canonical/absolute `PathBuf` values with validated root-relative capability mapping identity.
4. Stream capability discovery under the remaining global entry budget. Open each descendant directory or file one component at a time from its already-open parent capability with `follow(FollowSymlinks::No)`; use `maybe_dir(true)` for Windows directory handles. Descend only through the returned child capability. For each discovered `config.xml`, retain its root-relative identity in a bounded max-heap of the lexically smallest 10,000 identities and count overflow/truncation. Reject skipped symlinks, junctions, reparse points, non-regular files, and all failed child opens.
5. If the global entry budget is reached, report truncation and discard every candidate index. Otherwise, globally sort the retained config identities before parsing. A config-count overflow may parse only the globally first 10,000 retained configs under the existing contract. Apply first-valid duplicate precedence from that global order, so arbitrary discovery order cannot alter an installed index.
6. Stream only record attributes from globally ordered retained configs. Retain the existing relative-source, image signature, MIME, extension-probe, config-byte, mapping, depth, and image-result bounds under component-by-component capability opens, including later source resolution. Return only bounded bytes/MIME or normal missing.
7. Add the `rust-windows` job with `shell: bash` for `./scripts/dev check-rust`; add it to `check.needs`, add `RUST_WINDOWS` from `needs.rust-windows.result` to the aggregation environment, and add `"$RUST_WINDOWS"` to its loop so that result gates `check`. Run the recorded Rust gates and inspect the lockfile, workflow, and full diff.

**Tests and proof:** Temporary-tree tests add accepted nested configs, reordered or multiline attributes, root-relative containment, global entry-budget reach that reports truncation and exposes no index, remaining bounds, MIME and size result validation, deterministic duplicates, logo-over-icon resolution, extension probing, and no image preload. Add a duplicate-precedence test where DFS/per-directory order differs from full root-relative lexical order (`a/config.xml` versus `a.b/config.xml`) and assert the globally first config wins. Add no-follow child-directory/file proof plus an intermediate-directory replacement proof. On Windows, create a junction/reparse test tree and prove traversal and resolution reject it without opening an external target; run that test through the new Windows Rust CI job. Fixtures remain synthetic names and image signatures only.

**Patterns to verify:** `tempfile` test style, existing Rust trust-boundary error mapping, and the Check workflow's pinned Rust toolchain/cache job pattern. Do not reuse the React CSV picker or its path authority.

**Constraints and non-goals:** Pure engine only: no SQLite, picker, runtime, commands, cache, managed state, React, or image UI. Ambient authority is permitted once for the selected root only. Do not use `std::fs` traversal, canonical absolute-path containment, platform flag constants, direct `cap-primitives`, or a partial index after entry-budget truncation.

**Dependencies and sequencing:** PR 1's merge ref `1f94a491a535adfb924f86db1e7e7f751ba067f4` is reachable from this branch. This packet establishes the engine and the required Windows proof for Commit 2; no other packet may bypass it. It requires the manifest, lockfile, and Check workflow changes in the same atomic outcome.

- Retain `quick-xml` 0.41 directly and add only `cap-std` plus `cap-fs-ext` at 3.4.6. Let the lockfile provide transitive `cap-primitives` 3.4.6; do not add it to `Cargo.toml`.
- Build a pure `features/graphics` engine that opens its trusted selected root once, recursively discovers `config.xml` only through capability handles, and retains each accepted source as root-relative capability-owned identity. Stream discovery under the entry budget, keep only the lexically smallest 10,000 config identities in a bounded max-heap, then globally sort that set before parsing. Never store or return a canonical or absolute path.
- Enforce all stated depth, config, global-directory-entry, config-byte, mapping, and result-byte limits. A config-count overflow may process the globally first 10,000 configs; entry-budget reach is a truncated failed candidate that installs no indexes. Reject absolute or escaping source text, symlinks, Windows junctions/reparse points, non-regular files, unsupported files, unreadable files, malformed input, and invalid MIME/image inputs.
- Return immutable person and club indexes plus safe aggregate diagnostics after the globally ordered retained parse, then resolve one kind/positive UID through component-by-component retained capability identity to bounded data/MIME or normal missing. Use only synthetic temporary files and signatures in tests.

**Files and responsibilities:**

- `src-tauri/Cargo.toml` — direct `quick-xml` 0.41, `cap-std` 3.4.6, and `cap-fs-ext` 3.4.6 dependencies; no direct `cap-primitives`.
- `src-tauri/Cargo.lock` — locked direct and transitive capability resolution, including patched 3.4.6 capability crates.
- `src-tauri/src/features/graphics/{mod.rs,index.rs}` and colocated tests — pure capability-root discovery, root-relative mapping identity, sorted bounded traversal, parser, accepted-index install, image validation, result DTO, and synthetic filesystem proof.
- `src-tauri/src/features/mod.rs` — graphics module registration only; do not register commands or managed state.
- `.github/workflows/check.yml` — `rust-windows` quality job with `shell: bash` for `./scripts/dev check-rust`, plus its explicit `check.needs`, `RUST_WINDOWS` environment, and aggregation-loop entries.

**Behavior and data flow:** The engine receives the selected root internally, opens it once with ambient authority, and retains the resulting root capability. Capability discovery streams under the global entry budget, retains the lexically smallest root-relative config identities in a bounded max-heap, and globally sorts them before parsing; discovery order cannot affect installed precedence. Every traversal and later source-resolution component opens only from its already-open parent capability. An entry-complete scan atomically yields immutable indexes and safe diagnostics; entry-budget truncation yields diagnostics without an installed candidate. One positive UID lookup returns bounded bytes/MIME or missing.

**Validation:** `./scripts/dev check-rust`; `./scripts/dev check`; the Linux Rust job and the new `rust-windows` job, with `shell: bash` for `./scripts/dev check-rust`, must pass in GitHub Actions; inspect `Cargo.toml`, `Cargo.lock`, and `.github/workflows/check.yml` to confirm the direct and transitive dependency set, `rust-windows` in `check.needs`, `RUST_WINDOWS` in the aggregation environment and loop, and Check aggregation; `git diff --check`.

**Stop conditions:** Stop and replan if `cap-std` plus `cap-fs-ext` 3.4.6 cannot provide Windows no-follow child directory and file opens, a child or intermediate replacement can escape its parent capability, the root must be reopened with ambient authority, mapping identity needs a canonical or absolute path, entry-budget truncation can install any partial index, Cargo cannot resolve the documented patched set under Rust 1.77.2, or a Windows Rust CI job cannot execute the required proof.

**Review mandate:** Verify (1) only one ambient root open occurs and no `std::fs` traversal/canonical absolute path remains; (2) every descendant directory, file, and later source-resolution component opens from its already-open parent capability and is no-follow, including Windows `maybe_dir` behavior; (3) no direct `cap-primitives` exists and the lock resolves the 3.4.6 patched set under the recorded MSRV; (4) discovery streams under the global 1,000,000-entry budget, the max-heap retains only the lexically smallest 10,000 root-relative configs, entry-budget truncation installs nothing, and retained configs sort globally before parsing; (5) arbitrary discovery order cannot change first-valid precedence, including the DFS-versus-root-relative adversarial test; (6) all other parser, source, mapping, and image bounds remain enforced; (7) synthetic tests cover intermediate replacement and Windows junction/reparse rejection; and (8) `rust-windows` uses `shell: bash` for `./scripts/dev check-rust` and gates Check through explicit `needs`, environment, and aggregation-loop entries, with no SQLite, Tauri, or React scope.

#### Commit 2 — Manage the configured graphics runtime and IPC

**Status:** Active

**Provisional commit:** `feat(graphics): manage graphics runtime and IPC`

**Work:** Persist the selected root; own conventional Documents/OneDrive candidate discovery and picker start selection; and expose the pure engine through a managed generation runtime and narrow commands.

**Size assessment:** Above the soft target. v46, native choice, candidate discovery and start selection, runtime state, command registration, generation semantics, and bounded cache are one maximally atomic safe control plane; separating them would persist an unusable root or expose an unsafe resolver.

**Distinct outcome:** The app deterministically discovers a conventional candidate, safely stores only a user-chosen root, and serves safe status or one kind-plus-UID result from its current generation without ever accepting or returning a path.

**Out of scope:** Settings UI/Query keys, React, presentation, watcher support, persisted indexes, and all arbitrary caller paths.

**Implementation packet:**

**Ordered implementation steps:** Add v46/runtime RED tests; implement fixed-order Documents/OneDrive candidate discovery, its absence behavior, and Rust-owned picker start selection; add root storage and a zero-argument picker; install generation/cache behavior; register commands/state; prove stale scans and safe serialization; run Rust gates.

**Tests and proof:** Rust tests prove Documents-first candidate precedence, candidate absence, picker starting-directory selection, cache capacity and least-recent eviction, root replacement clearing available and missing entries, stale scan discard, no DB-mutex scan, and DTO serialization with neither requested nor returned paths.

**Patterns to verify:** Existing `lib.rs` managed-state/command registration and snapshot's prepare-outside-mutex pattern.

**Constraints and non-goals:** Candidate discovery, precedence, absence behavior, and picker starting-directory selection remain Rust-only. No Settings/React detection, caller path, watcher, persisted index, or presentation belongs here.

**Dependencies and sequencing:** Requires the pure engine in Commit 1. Settings in Commit 3 consumes only this narrow contract.

- Add v46 singleton app-wide root persistence. In Rust, test a fixed Documents-first then OneDrive candidate list for `Sports Interactive/Football Manager 2026/graphics`; the first readable directory wins. When no candidate exists, report only safe absent state and keep configuration unchanged. Implement a Rust-owned zero-argument native directory picker that starts at the detected candidate, otherwise the first existing candidate parent in that same order, otherwise the native default. Provide status, choose, clear, rescan, and positive-UID resolution commands. Register the module, managed `GraphicsRuntime`, and commands in `features/mod.rs` and `lib.rs`.
- `GraphicsRuntime` owns the current root, candidate state, generation, immutable indexes, safe summary, and per-kind LRU caches: at most 256 available and 256 missing results for each kind. Status exposes only non-path candidate state; no response retains a filesystem path.
- On choose/root replacement and rescan, reserve a monotonically increasing generation before scan work. Build outside the DB mutex; install only if the completed target generation is still current. An older completion is discarded. On accepted replacement, atomically swap root/index/summary/generation and empty every result cache. Lazy restart scan follows the same generation rule.
- Root change/rescan completion reports the current generation. The resolver accepts only kind and positive UID and returns available bounded data/MIME or normal missing from that generation.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — v46 root row and upgrade tests.
- `src-tauri/src/features/graphics/{runtime.rs,service.rs,commands.rs,mod.rs}` — candidate discovery, picker start, persistence, generation, safe non-path status, cache, and commands over Commit 1 engine.
- `src-tauri/src/{features/mod.rs,lib.rs}` — module, managed state, and registration.
- Colocated Rust tests — migration, Documents-before-OneDrive selection and absence, picker start, no-path DTO serialization, LRU capacity/eviction, replacement, and stale-scan completion protection.

**Behavior and data flow:** Rust determines the conventional candidate and native picker start without exposing either path; a zero-argument picker selects a root under Rust control; v46 stores it; a scan installs the matching generation; status reports only safe aggregate and candidate state; callers resolve only `(kind, uid)`.

**Validation:** `./scripts/dev check-rust`; `./scripts/dev check`.

**Stop conditions:** Stop if the Rust dialog API needs a new capability, the fixed candidate order or safe absence behavior cannot be implemented, picker start needs a caller path, a command needs a caller path, file work holds the DB mutex, replacement cannot be atomic at the runtime boundary, or stale completion can install older indexes.

**Review mandate:** Verify Documents-before-OneDrive candidate precedence, absence, and Rust-only picker start; v46 stores only the root; picker has no path parameter; each command is registered and narrow; generation/install/cache semantics handle overlap; positive and negative LRU limits/eviction are exact; no index/image bytes persist and no filesystem path crosses IPC.

#### Commit 3 — Add the Graphics Settings section

**Status:** Pending

**Provisional commit:** `feat(settings): configure local graphics`

**Work:** Compose the safe graphics commands into typed frontend APIs, generation-aware Query lifecycle, and the Settings Graphics section.

**Size assessment:** Within the soft target unless route doubles require mechanical expansion. Typed API/keys, generation invalidation, Settings actions, and their route evidence are one maximally atomic user control seam; UI without invalidation leaves stale image states visible.

**Distinct outcome:** Users can display safe runtime status, choose, clear, or rescan a root through pathless commands, and no old available or missing graphics Query result can remain rendered after the generation changes.

**Out of scope:** Candidate discovery, picker starting-directory selection, new Rust scanner/runtime behavior, caller paths, image presentation, Player/Profile/Search/Squad/My Club changes, and watcher support.

**Implementation packet:**

**Ordered implementation steps:** Add Query lifecycle RED tests; add typed APIs/keys; compose Settings from runtime-provided safe state and pathless commands; update mocks/stub; run UI evidence and gates.

**Tests and proof:** Route tests prove choose, clear, rescan, safe runtime status, pathless invocation, diagnostics, and no displayed path. Query tests prove a prior available and prior missing result are absent after a new generation, not rendered while the new key is pending. Smoke and `inspect-ui` prove the section at supported viewports.

**Patterns to verify:** Existing typed invoke/query options, Settings section fallbacks, mutation invalidation, and IPC stub conventions.

**Constraints and non-goals:** Commit 3 displays only safe runtime state and invokes pathless commands; it performs no candidate detection or picker-start selection. Do not add frontend path authority, scanner/runtime work, watcher, or presentation seams.

**Dependencies and sequencing:** Requires Commit 2's status, mutation, and resolve contract. It precedes every graphics presenter.

- Add `src/features/graphics` typed fetchers, mutations, status/result types, and Query keys. Keys for image results include `generation`, `kind`, and `uid`.
- On choose, clear, or rescan success, first remove all image-result queries for the prior generation, then invalidate graphics status and route-owned consumers. Components receive the returned/new status generation and must render pending/fallback until a query for that generation settles; neither old available data nor old missing data may be selected as placeholder data.
- Add the Graphics Settings section to `src/app/routes/settings.tsx`. Display only the runtime-provided safe candidate/selected status, never a local path. Invoke the zero-argument choose, clear, and rescan commands; provide bounded diagnostics, cancellation/error, and generation-aware feedback. Do not detect candidates or select a picker start in React.
- Update typed IPC mocks, route tests, smoke stub, and Settings inspection evidence. Do not add a frontend dialog invocation.

**Files and responsibilities:**

- `src/features/graphics/{api,types,components}` — typed commands, keys, mutations, status panel, and generation-safe result lifecycle.
- `src/app/routes/settings.tsx` and `src/app/routes/settings.test.tsx` — section composition and observed actions.
- `src/testing/setup.ts`, a graphics IPC mock, `e2e/tauri-ipc-stub.ts`, `e2e/smoke.spec.ts` — narrow command doubles and browser-stub route proof.

**Behavior and data flow:** Settings reads safe runtime status and invokes a pathless Rust command; returned generation removes old result cache entries before future image consumers query the new generation. React neither discovers a candidate nor chooses a picker start.

**Validation:** `./scripts/dev test`; `./scripts/dev check`; `./scripts/dev smoke`; `./scripts/dev inspect-ui /settings 1280 800`; `./scripts/dev inspect-ui /settings 1600 900`; `./scripts/dev inspect-ui /settings 3440 1440`.

**Stop conditions:** Stop if runtime-provided candidate state would expose a path, any Settings action needs React path authority, Query cannot remove both old positive and negative data before render, or mock/stub contracts diverge from the command DTOs.

**Review mandate:** Verify only safe runtime state reaches UI; Commit 3 performs no detection or picker-start selection; no WebView picker/path API exists; all three actions report clear states; generation-keyed invalidation removes old available and missing data; mock, route, smoke, and inspected Settings evidence agree.

#### Commit 4 — Render Player Profile identity graphics

**Status:** Pending

**Provisional commit:** `feat(profile): render local identity graphics`

**Work:** Project the Player Detail current-club UID and render its portrait plus current-club crest in the existing profile rail seam.

**Size assessment:** Within the soft target. Player Detail projection and the one rail's two adjacent identity graphics are maximally atomic: separate commits would add an unused projection or an incomplete rail identity contract.

**Distinct outcome:** A Player Profile rail renders its UID-backed portrait and current-club crest, or the existing stable person/shield fallbacks.

**Out of scope:** Search/Squad table DTOs or UI, managed-club logo, staff UID exposure, changes to resolver/runtime, name matching, or layout changes outside the rail.

**Implementation packet:**

**Ordered implementation steps:** Add profile UID/fallback RED tests; introduce only Player Detail projection; compose rail portrait/crest slots; update doubles; inspect and run gates.

**Tests and proof:** Component and route tests prove exact portrait/player and crest/club UID requests, available states, and stable pending, null, missing, and error rail geometry. Existing profile text/name tests remain the text-first guard.

**Patterns to verify:** `PlayerIdentityRail` placeholder dimensions, route composition, Query result-state use, and Design image-alt rules.

**Constraints and non-goals:** Profile only; no Search/Squad/My Club, staff/manager DTO, name lookup, resolver alteration, or cross-feature import.

**Dependencies and sequencing:** Requires PR 1 Commit 2 and PR 2 Commits 2–3. Table work remains later.

- Extend only `player/query.rs`, player command DTO serialization, Player Detail frontend type, and profile mocks with nullable `currentClubUid`; do not add staff or manager DTO fields.
- Add/reuse the graphics presenter through route composition so `PlayerIdentityRail` receives explicit portrait and crest slots. Portrait requests use player `uid`; crest requests use `currentClubUid`; the adjacent text remains identity.
- Preserve rail dimensions and existing monogram fallbacks during pending, null, missing, or error results.

**Files and responsibilities:**

- `src-tauri/src/features/player/{query.rs,commands.rs}` and `src/features/player-profile/{types,api,components/player-identity.tsx}` — one consumer-owned player projection and rail composition.
- `src/app/routes/players.$uid.tsx`, its route tests, graphics mocks/setup, and browser stub/smoke evidence — route wiring and observable fallbacks.

**Behavior and data flow:** Player Detail supplies nullable club UID; the rail independently queries portrait(player UID) and clubLogo(club UID) for its current graphics generation; it keeps stable fallback slots until available data arrives.

**Validation:** `./scripts/dev test`; `./scripts/dev check`; `./scripts/dev smoke`; `./scripts/dev inspect-ui /players/42?section=overview 1280 800`; `./scripts/dev inspect-ui /players/42?section=overview 1600 900`; `./scripts/dev inspect-ui /players/42?section=overview 3440 1440`.

**Stop conditions:** Stop if the rail cannot keep dimensions, current club UID is unavailable without name matching, or composition would import graphics across a feature boundary instead of through the route.

**Review mandate:** Verify Player Detail is the only new projection; requests use numeric UIDs; portrait and crest share the intended rail seam; fallbacks and accessible names remain correct; no staff UI, arbitrary path, or layout shift appears.

#### Commit 5 — Render player-table identity graphics

**Status:** Pending

**Provisional commit:** `feat(players): render local table identity graphics`

**Work:** Add player portrait and current-club mark behavior to the shared dense identity treatment in Search and Squad together.

**Size assessment:** Above the soft target only if the two table projection paths are mechanically large. This is maximally atomic because Search and Squad are the one shared dense player-table behavior: splitting creates inconsistent visible identity semantics and leaves one required table seam unproved.

**Distinct outcome:** Both Search and Squad rows request player portraits and exact current-club marks while retaining their fixed two-slot identity geometry and fallbacks.

**Out of scope:** Profile, My Club managed-club logo, staff images, generic scanner/runtime changes, name matching, table sizing/virtualization redesign, and any new resolver command.

**Implementation packet:**

**Ordered implementation steps:** Add Search/Squad exact-UID and fallback RED tests; add both consumer projections; compose paired dense-cell marks; update doubles; inspect both tables and run gates.

**Tests and proof:** Search and Squad tests prove both image kinds use exact UIDs, name-plus-null does not trigger lookup, missing/error retains slots, and rows retain height and navigation. Browser smoke and inspection prove dense-table containment.

**Patterns to verify:** Search and Squad identity cells, virtualized table keys, route composition, and fixed two-line row rules.

**Constraints and non-goals:** Keep Search and Squad together; no profile/My Club/staff UI, resolver change, generic image-kind split, or table redesign.

**Dependencies and sequencing:** Requires PR 1 Commit 2 and PR 2 Commits 2–3. The Profile presentation packet is not a code dependency.

- Add nullable `currentClubUid` to only Search and Squad Rust query/command DTOs, their frontend types, IPC mocks, and route fixtures. Do not expose staff or manager UID fields.
- Compose graphics at the route/table boundary into `PlayerIdentityCell` and `SquadIdentityCell`; each visible row requests `personPortrait(player.uid)` and `clubLogo(currentClubUid)`. Preserve row keys, virtualization, name/club text, existing navigation, and fixed mark slots.
- Treat pending, null, missing, invalid, and generation replacement as normal fallback states. Do not add one image-kind-specific table abstraction that separates the two adjacent marks.

**Files and responsibilities:**

- `src-tauri/src/features/{search,planner}/` query and command owners — table-specific current-club UID projections.
- `src/features/{search,squad}/types` and `{search-results-panel,squad-overview-panel}.tsx` — dense identity slots and typed rows.
- Search/Squad route tests, graphics mocks/setup, `e2e/tauri-ipc-stub.ts`, and `e2e/smoke.spec.ts` — exact requests and no-regression table evidence.

**Behavior and data flow:** The page DTO supplies row player and nullable club IDs; each dense identity cell queries the current graphics generation and shows its stable portrait/crest or existing fallback without filesystem access.

**Validation:** `./scripts/dev test`; `./scripts/dev check`; `./scripts/dev smoke`; `./scripts/dev inspect-ui /search 1280 800`; `./scripts/dev inspect-ui /search 1600 900`; `./scripts/dev inspect-ui /search 3440 1440`; `./scripts/dev inspect-ui /my-club 1280 800`; `./scripts/dev inspect-ui /my-club 1600 900`; `./scripts/dev inspect-ui /my-club 3440 1440`.

**Stop conditions:** Stop and replan if source evidence proves Search and Squad do not share dense-table behavior, virtualized rows cannot bound visible requests, a table needs name matching, or slots change row height.

**Review mandate:** Verify the two tables land together; each projection is consumer-owned; both kinds use exact IDs and current generation; fixed rows, virtualization, text identity, navigation, and fallbacks survive; no staff image or resolver expansion occurs.

#### Commit 6 — Render the managed-club logo

**Status:** Pending

**Provisional commit:** `feat(managed-club): render local club logo`

**Work:** Render the selected managed-club logo from its already stored exact UID in the My Club context seam.

**Size assessment:** Within the soft target. The existing exact selected UID, its single context presenter, and fallback proof form one maximally atomic outcome; no other surface needs this selection state.

**Distinct outcome:** My Club displays a local logo for an available selected UID and a shield fallback for null, missing, or stale selection without altering club membership behavior.

**Out of scope:** New managed-club persistence or DTO fields, name recovery, selector search changes, player/profile/table graphics, Staff images, and membership SQL changes.

**Implementation packet:**

**Ordered implementation steps:** Add exact-selected-UID RED tests; compose the existing status UID into one My Club mark; update the mock/stub; inspect and run gates.

**Tests and proof:** Tests cover valid UID rendering, null legacy selection, missing fallback, and a selected name that cannot cause a lookup. Existing selection/cohort tests remain the guard for unchanged behavior.

**Patterns to verify:** `ManagedClubSelector` semantics, existing selected-status type, and adjacent-text image accessibility.

**Constraints and non-goals:** Do not add persistence/DTOs, rebind legacy names, change selector search/cohorts, or add player/staff UI.

**Dependencies and sequencing:** Requires PR 1 Commit 3 and PR 2 Commits 2–3.

- Consume the `clubUid` delivered by PR 1 Commit 3 in the My Club route/selector context. Request only `clubLogo(clubUid)` through the current graphics generation; retain the selected club name as text.
- Show a decorative or correctly labelled mark according to adjacent text. Preserve the picker behavior and its missing-selection message.

**Files and responsibilities:**

- `src/features/managed-club/components/managed-club-selector.tsx` and My Club route composition — selected-UID logo slot.
- Managed-club route/component tests, graphics IPC mock, and browser stub/smoke — exact UID and fallback evidence.

**Behavior and data flow:** Persisted managed-club status supplies nullable exact UID; the context presenter resolves the current-generation logo or keeps the shield; it does not send club name to graphics IPC.

**Validation:** `./scripts/dev test`; `./scripts/dev check`; `./scripts/dev smoke`; `./scripts/dev inspect-ui /my-club 1280 800`; `./scripts/dev inspect-ui /my-club 1600 900`; `./scripts/dev inspect-ui /my-club 3440 1440`.

**Stop conditions:** Stop if the logo needs a name-derived UID, a legacy selection is silently rebound, or the selector/cohort contract would change.

**Review mandate:** Trace the mark to persisted `clubUid`; verify null/missing shield behavior, adjacent text accessibility, unchanged picker/cohort SQL, no staff image, and no expanded graphics command.

#### Commit 7 — Reconcile local graphics delivery records

**Status:** Pending

**Provisional commit:** `docs(graphics): reconcile local graphics delivery`

**Work:** Complete feature validation and review, reconcile contradictory durable documentation, transition TODO, and archive the ledger before final publication.

**Size assessment:** No production code. These delivery records are maximally atomic because they state the verified complete feature and archive its authority/evidence together.

**Distinct outcome:** Current-state docs accurately describe delivered local graphics; JAY-63 leaves Active and its complete schema-2 record is archived.

**Out of scope:** Runtime behavior, tests, releases, invented merge data, unrelated completed records, and claims for unrun manual verification.

**Implementation packet:**

**Ordered implementation steps:**

1. Run final validation and feature review; reconcile Architecture and mandatory Design facts.
2. While the ledger remains at `.wiki/features/active/local-fm-graphics.md`, run the ledger and delivery classifiers.
3. Transition TODO and move the ledger to `.wiki/features/completed/local-fm-graphics.md`.
4. Inspect the documentation diff. Run every supported subsequent ledger or delivery validation against the completed path; do not target the active path after the move.

**Tests and proof:** Not applicable for new runtime behavior. Final command evidence, feature review, documentation inspection, the active-path ledger and delivery classifiers before the move, and completed-path subsequent classifiers prove record accuracy.

**Patterns to verify:** `.wiki/INDEX.md` ownership, recent schema-2 archives, and the no-release policy.

**Constraints and non-goals:** Do not change application behavior, invent publication/live-test facts, touch unrelated completed ledgers, omit Design reconciliation, or run a validator against the active path after moving the ledger.

**Dependencies and sequencing:** All implementation packets, merged and synchronized PR 1, final validation, and clear feature review are required.

- Reconcile `.wiki/ARCHITECTURE.md` with v9/v44-v46, root persistence, Rust-owned trust boundary, generations/cache, narrow IPC, UID data flow, the new `rust-windows` Rust job, and Check aggregation after implementation proves them.
- Reconcile `.wiki/DESIGN.md` as mandatory work: replace its bundled-only/no-image wording with delivered local-only portrait/logo, fixed-slot, fallback, and text-first rules. Do not leave it contradictory.
- Move JAY-63 from Active to Completed in `.wiki/TODO.md`; move this ledger to `.wiki/features/completed/local-fm-graphics.md`; retain PR authority, fingerprints, completed rows, and final validation. Do not write a final self-merge ref into its own content.

**Files and responsibilities:**

- `.wiki/{ARCHITECTURE.md,DESIGN.md,TODO.md}` — proven implemented facts and lifecycle state.
- `.wiki/features/active/local-fm-graphics.md` → `.wiki/features/completed/local-fm-graphics.md` — archived plan and delivery evidence.

**Behavior and data flow:** Documentation records verified behavior only; it changes no application data or runtime behavior.

**Validation:** Run every command in **Final validation**; before moving, run `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/local-fm-graphics.md` and `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/local-fm-graphics.md /home/jonas/projects/fm-valuescout`; then move the ledger and run `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/completed/local-fm-graphics.md`; `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/completed/local-fm-graphics.md /home/jonas/projects/fm-valuescout`; and `git diff --check`. No validation command may target the active path after the move.

**Stop conditions:** Stop if docs contradict implementation, `DESIGN.md` remains bundled-only/no-image, a required final validator/review is unavailable, manual evidence is represented as passed, delivery authority changes, or an unapproved documentation path is required.

**Review mandate:** Verify Architecture and Design state only delivered behavior, TODO/archive lifecycle is correct, authority/evidence remains intact, no release is claimed, and the post-merge manual gap remains explicit.

## Active work

**PR:** PR 2 — Add local FM graphics

**Commit:** Manage the configured graphics runtime and IPC

### RED or removal proof

Add v46 and runtime tests first. Confirm they fail because root persistence, Rust-only candidate selection and folder choice, generation-safe runtime installation, narrow commands, and bounded per-kind result caches do not exist.

### Expected outcome

Rust selects safe conventional candidates and picker starts, persists one chosen root, and serves non-path status and kind-plus-positive-UID resolution through a generation-owned runtime. Root changes, rescans, and lazy restart scans replace indexes and both result-cache classes atomically, while stale scans cannot install.

### Explicit exclusions

- Settings UI, Query keys, React, and image presentation.
- Watchers, persisted indexes or image bytes, and caller-supplied paths.
- The unrelated dirty completed-ledger URL formatting changes.

## Discoveries and replanning

- This bounded replan replaces the prior `std::fs` attempt in PR 2 Commit 1. Review found its per-directory `Vec<DirEntry>` collection unbounded and its canonical-path checks unable to prevent intermediate Windows junction/reparse replacement races. The packet now requires capability handles, root-relative identity, a global 1,000,000-entry budget, fail-closed installation, and a Windows Rust CI proof.
- The approved capability choice is direct `cap-std` plus `cap-fs-ext` 3.4.6, with transitive `cap-primitives` 3.4.6. Source and crate metadata show `Dir::open_ambient_dir`, no-follow final-child options, Windows `maybe_dir`, and Rust 1.63 workspace compatibility; do not use the suggested 3.4.5 line.
- This bounded replan retains the two-PR and later-packet order. Only PR 2 Commit 1 gains its required Cargo lockfile and Check-workflow companion changes. The index plan uses bounded max-heap retention plus a global root-relative sort, not per-directory/DFS order, so discovery order cannot choose duplicate precedence.
- PR 1 Commit 2 no longer exposes speculative staff, manager, or player-facing read DTOs. Staff and manager IDs persist only. Profile, Search, and Squad introduce their own player club-UID projections when they consume them.
- `DESIGN.md` has an explicit bundled-only/no-image rule. The close-out packet must reconcile it with local-only delivered images.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Persist FM club identity | Commit 1 — Record the approved feature plan | 810a242b788986750edad38c776748e6b1b03b60 | Recorded the reviewed schema-2 JAY-63 ledger and its one Active TODO link without changing executable behavior. | `ledger_state.py` and `delivery_state.py` reported runnable with the accepted fingerprint; exact cached stat and diff were inspected; `git diff --cached --check` passed. | Not applicable | Clear | 0 | None |
| PR 1 — Persist FM club identity | Commit 2 — Upgrade dumps to schema v9 and persist nullable club UIDs | bf923ac4d599695fedf9d99d453932888927b79d | Advanced the bridge and Rust dump contract to v9, extracted exact nullable club-object UIDs, added migration v44, and retained the IDs through prepared atomic snapshot persistence while legacy rows remain null. | `./scripts/dev bridge-test` passed 219 tests with 3 skipped; `./scripts/dev check-rust` and `./scripts/dev check` passed 809 Rust tests with 2 ignored; focused manager tests, v8-reference audit, LSP diagnostics, and `git diff --cached --check` passed. | Pass | Clear | 2 | Review corrections bound manager UID to the selected graph-or-contract source, completed exact fake-memory/migration/persistence proofs, and reconciled current v9 bridge and Architecture documentation. |
| PR 1 — Persist FM club identity | Commit 3 — Bind managed-club selections to club UIDs | ade7c9f2763fb225f57572abf578aa0955e6df46 | Added migration v45 and exact effective-current managed-club name/UID options, validation, persistence, status, and typed picker flow while retaining name-based cohorts and legacy null identity. | `./scripts/dev test` passed 1,000 tests; `./scripts/dev check` passed 811 Rust tests with 2 ignored; `./scripts/dev smoke` passed 62 tests; exact mock/stub pair proofs, LSP diagnostics, and `git diff --cached --check` passed. | Pass | Accepted findings — direct populated v44-to-v45 managed-club upgrade proof remains a MEDIUM advisory for feature close-out. | 3 | Review corrections removed all name/index-derived UID behavior from Vitest and Playwright doubles, required exact option pairs, and aligned browser smoke with the IPC contract. |
| PR 2 — Add local FM graphics | Commit 1 — Build the bounded local graphics index | Pending record | Added a capability-owned, root-relative graphics index with bounded no-follow discovery, globally deterministic config precedence, fail-closed entry truncation, streaming XML parsing, and bounded lazy image resolution. Added Windows Rust CI coverage for junction and reparse behavior. | Twelve focused graphics tests passed on Linux; `./scripts/dev check-rust` and `./scripts/dev check` passed 823 Rust tests with 2 ignored; `git diff --check` and primary Rust LSP diagnostics passed. The Windows-only proof is pending GitHub CI. | Pass | Accepted findings — correction review accepted the blocking fixes; a MEDIUM discovery-iterator error advisory remains for feature close-out. | 2 | Replaced the invalidated `std::fs` scanner with direct `cap-std` and `cap-fs-ext` 3.4.6 capability traversal and added the packet-required `rust-windows` Check job. |

## Final validation

Run after all implementation packets and before feature review/publication:

1. `./scripts/dev bridge-test`
2. `./scripts/dev test`
3. `./scripts/dev check`
4. `./scripts/dev smoke`
5. `./scripts/dev inspect-ui /settings 1280 800`
6. `./scripts/dev inspect-ui /settings 1600 900`
7. `./scripts/dev inspect-ui /settings 3440 1440`
8. `./scripts/dev inspect-ui /players/42?section=overview 1280 800`
9. `./scripts/dev inspect-ui /players/42?section=overview 1600 900`
10. `./scripts/dev inspect-ui /players/42?section=overview 3440 1440`
11. `./scripts/dev inspect-ui /search 1280 800`
12. `./scripts/dev inspect-ui /search 1600 900`
13. `./scripts/dev inspect-ui /search 3440 1440`
14. `./scripts/dev inspect-ui /my-club 1280 800`
15. `./scripts/dev inspect-ui /my-club 1600 900`
16. `./scripts/dev inspect-ui /my-club 3440 1440`

Inspect each populated capture for fixed slots, fallback state, readable text, clipping, overflow, and profile/table containment. Delete `.work/ui-inspection/` during close-out. Browser captures prove only Chromium-stub UI, not native Tauri, Rust filesystem, SQLite, picker, or live FM.

**Disclosed post-merge manual verification gap:** Before the developer performs the live check, run `./scripts/dev bridge-install`, restart FM26, then test live FM26.3.2 and a real local graphics pack. Verify `club + 0x0C` for senior, reserve/youth, and loan current/parent cases; root choice; nested portrait/logo/icon mappings; duplicate diagnostics; manual rescan/cache refresh; missing fallbacks; and unchanged Load Data. This is a post-merge precondition and manual test, not an automated merge gate. Record it as not run until evidence exists.

## Documentation impact

Commit 2 updates `bridge/DUMP_SCHEMA.md` with the v9 contract. Close-out must update `.wiki/ARCHITECTURE.md` for delivered graphics behavior, the `rust-windows` Rust job, and Check aggregation; reconcile `.wiki/DESIGN.md`; transition `.wiki/TODO.md`; and archive the ledger. No release documentation is in scope.
