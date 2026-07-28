# FM26 memory read

## Status

Active

## Intent

Extract a full CONCEPT MVP player dump from a running Football Manager 26 (Windows Steam) session via a BepInEx IL2CPP C# bridge, orchestrated by the Tauri Rust backend through a file protocol. This is the live-data foundation for snapshot ingest and every later MVP feature.

Planning input (not authority): [memory-read-initial-notes.md](../../notes/memory-read-initial-notes.md). Reference shape: [FMSuperScout](https://github.com/mavarobli/FMSuperScout) (no license — study publicly, reimplement independently).

## User-visible behavior

- With FM26 running and a save loaded, and the bridge plugin installed, the user can trigger a data scan from the app (in-app only — no in-game hotkey in this feature).
- The app shows whether the bridge is present/ready and whether FM appears available for a scan.
- A successful scan writes a dump file under the app’s bridge data directory with the CONCEPT MVP player field set.
- Unsupported or unknown FM builds fail closed with a clear compatibility error; a failed scan must not replace a prior good dump.
- Manual plugin install (copy DLL into BepInEx plugins) is acceptable for MVP. In-app install/remove is deferred unless it stays a small, low-risk add-on.

## Invariants

- Read-only toward FM memory — never write game memory.
- Prefer safe process reads (`ReadProcessMemory`) so bad addresses fail without crashing FM.
- Memory-layout knowledge stays in the bridge (versioned layouts); the WebView never owns offsets.
- Full player databases do not cross Tauri IPC as one payload — dump stays on disk for feature 2 ingest.
- Windows Steam FM26 only for memory reading in this feature.
- Scanning work stays off Unity’s main thread.

## Non-goals

- SQLite import, snapshot replace-on-success, or Load Data persistence semantics (roadmap feature 2)
- Role scoring, player search UI, profiles, squad planner/optimizer
- In-game hotkey (e.g. F9)
- macOS/Linux FM, Epic/Game Pass path detection beyond Steam if it blocks MVP
- Polished BepInEx installer wizard, auto-update of the mod layer
- Copying SuperScout source or shipping their DLL

## Current-state map

- Relevant components: walking-skeleton `health` feature only; no bridge or memory-read code
- Data model: none for players yet
- Persistence and migrations: SQLite demo table only — unused by this feature
- Existing behavioral assumptions: thin frontend / thick Rust; IPC via `tauri-client`
- Architectural seams: add `bridge/` (.NET) beside `src-tauri/`; Rust feature module for protocol + IPC; thin React status/trigger UI
- Tests and validation: Vitest + mockIPC, Playwright stub smoke, `cargo test`; no .NET in gate yet
- Primary risks: first attach/offsets on the developer’s FM26 build; WSL vs Windows host split for verification

## Feature architecture (this feature)

```text
FM26 (Windows, Steam) + BepInEx 6 IL2CPP
  └── bridge/ (C#) — status, request poll, safe memory scan, dump + diagnostics

%LOCALAPPDATA%\fm-valuescout\fm-bridge\   (exact dirname finalized in PR 1)
  ├── request.json (or flag)   ← Rust writes
  ├── status.json              ← bridge writes
  ├── dump.json                ← bridge writes (streamed)
  └── diagnostics.txt          ← bridge writes

Tauri
  ├── Rust features/memory-read — paths, request, status watch, dump presence/validation
  └── React features/memory-read — status + trigger scan (no player table)
```

Dump schema is the contract handed to feature 2. Rust may validate dump shape; it does not import into SQLite here.

## Uncertainty register

### Known

- SuperScout proves BepInEx + in-process RPM + file protocol works on FM26 Windows
- CONCEPT MVP field list (see Objective in notes / CONCEPT in-scope memory read)
- Developer runs FM on Windows host Steam; day-to-day coding may be WSL
- No spike — feature commits start immediately
- C# bridge + Rust protocol chosen over Rust-only external reader

### Assumptions

- A known-compatible BepInEx 6 IL2CPP build for FM26 is installable on the Steam path
- Offsets can be pinned for the developer’s current FM26 minor (repin process documented later)
- Manual DLL drop into `BepInEx/plugins` is enough for personal MVP use
- Bridge data directory under LocalAppData is shared correctly between Windows FM and the Tauri app when both run on Windows

### Decisions

- Architecture: C# BepInEx plugin + Rust file protocol + minimal Tauri UI
- Trigger: in-app only
- Scope end: validated dump + protocol; ingest is the next feature
- Platform for memory read: Windows Steam only
- Field depth: full CONCEPT MVP extraction set before calling this feature done
- Install: manual first; in-app install only if cheap and not a security swamp

### Unknowns

- Exact FM26 build string and offset set on first successful dump (resolve during PR 2+)
- Exact request/status JSON shapes (finalize in PR 1; keep small and versioned)
- Whether CI gains a Windows `dotnet test` job (only if tests run without machine-local FM interop)

### Risks

- Patch breaks offsets → version fail-closed + diagnostics/repin notes
- False-positive person objects → multi-field validation + UID dedupe
- Large dumps → stream JSON; never ship full dump over IPC
- Licensing/attribution → independent structure; document research provenance in wiki/notes when offsets stabilize

## Walking skeleton

Plugin loads in FM → `status.json` visible to Rust/UI → user triggers scan in app → dump on disk with verified UID/CA/PA for known players → expand fields through later PRs → freeze dump contract for ingest.

## Delivery plan

### PR 1 — Bridge bootstrap and status protocol

**Status:** Active

**Provisional PR title:** `feat(memory-read): add BepInEx bridge bootstrap and status protocol`

**Purpose:** Prove the plugin loads and the app can observe bridge readiness without scanning memory yet.

**Depends on:** Nothing.

**Merge to trunk when:** Bridge toolchain docs and ignores are in place; plugin builds on a Windows/.NET setup; Rust parses status fixtures; UI shows status with mockIPC; `./scripts/dev check` stays green on Linux CI without requiring FM.

#### Commit 1 — Bridge toolchain and repo prerequisites

**Status:** Active

**Work:**
- Document machine prerequisites for building the bridge: .NET 6 SDK, Windows host, Steam FM26, BepInEx 6 IL2CPP installed into the FM folder, and that interop assemblies are generated on first FM launch with BepInEx (not vendored in git).
- Add repo hygiene so C# build output never lands in reviews: `.gitignore` for `bridge/**/bin/`, `bridge/**/obj/`, and any local props that hold absolute Steam/BepInEx paths (e.g. `Directory.Build.user.props`, `*.user.props`).
- Add a committed **example** override file (e.g. `bridge/Directory.Build.props.example`) showing how to point `BepInExCore` / `InteropDir` at the local Steam FM26 tree; real machine paths stay untracked.
- Pin or document the intended SDK (e.g. `bridge/global.json` targeting the .NET 6 feature band, or an explicit note if we intentionally float) so `dotnet build` is reproducible across machines.
- Ensure frontend tooling ignores the bridge tree where needed (Biome / Vitest / secretlint ignore as appropriate) so Linux `./scripts/dev check` does not require `dotnet` and does not try to parse `.cs` as JS/TS.
- Optional one-liner in root README or CONTRIBUTING pointing at `bridge/README.md` for “building the FM plugin” — keep it short; full install steps can land with the scaffold commit.

**Out of scope for this commit:**
- Creating the plugin project, `BasePlugin`, or `status.json` writer
- Vendoring BepInEx, FM interop DLLs, or game assemblies into the repo
- Wiring `dotnet` into Linux CI as a hard gate
- Rust/UI work

**Validation:** Ignore rules verified (a local `bin/`/`obj/` or user props file is not tracked); example props file is copy-paste usable; `./scripts/dev check` still passes on Linux with no .NET SDK installed.

**Provisional commit:** `chore(bridge): add .NET toolchain prerequisites and ignores`

#### Commit 2 — Scaffold C# bridge and status writer

**Status:** Pending

**Work:**
- Add a top-level `bridge/` .NET 6 class-library project targeting BepInEx 6 Unity IL2CPP (same plugin host SuperScout uses). Wire project references to BepInEx core + FM interop assemblies via the local override pattern from commit 1.
- Implement a minimal `BasePlugin` that loads inside `fm.exe`, creates the bridge data directory under `%LOCALAPPDATA%\fm-valuescout\fm-bridge\` (exact name locked here), and writes a **versioned** `status.json` on load and on idle ticks if needed.
- Status payload at minimum: protocol/schema version, plugin version, process/load state (`idle`), timestamps, and cheap module presence signals if available without scanning (e.g. whether `game_plugin.dll` was located). Prefer a small typed status model + serializer over ad-hoc string writes.
- Expand `bridge/README.md` with the manual loop: build the plugin, copy the DLL into `BepInEx/plugins`, first-launch interop generation if not already done, confirm `status.json` appeared under LocalAppData.

**Out of scope for this commit:**
- Memory scanning, dumps, request polling
- Rust/UI integration
- In-app installer

**Validation:** `dotnet build` succeeds on a Windows machine with BepInEx/interop paths configured from the example props; status serialization unit test (or equivalent) covers the status shape; after manual install + FM launch, `status.json` exists and looks sane; Linux check remains green.

**Provisional commit:** `feat(bridge): scaffold BepInEx plugin status writer`

#### Commit 3 — Rust bridge paths and status IPC

**Status:** Pending

**Work:**
- Add backend feature module `src-tauri/src/features/memory-read/` following existing `health` layout (`commands.rs` / `service.rs` / types as needed). Register commands in `lib.rs` and ACL capabilities only for what this commit exposes.
- Resolve the bridge data directory the same way the plugin does (LocalAppData + `fm-valuescout/fm-bridge` on Windows). On non-Windows, return a clear “unsupported platform” / missing-bridge status rather than inventing paths — memory read is Windows-only.
- Implement read + parse of `status.json` into a bounded DTO. Missing file, empty file, or schema mismatch must map to explicit error kinds the UI can show (not a panic).
- Expose IPC command `get_bridge_status` (name finalizable here) that returns the DTO or structured error. No dump bytes in the response.
- Add Rust unit tests with temp directories and fixture `status.json` files (happy path, missing, corrupt/unsupported version).

**Out of scope for this commit:**
- Writing request files or watching dumps
- Frontend UI (types-only share is fine if it reduces churn)
- Plugin install detection beyond “status file readable”

**Validation:** `cargo test` covers fixtures; staged Rust paths pass `./scripts/dev check-rust` / full check as usual.

**Provisional commit:** `feat(memory-read): expose bridge status over IPC`

#### Commit 4 — UI bridge status panel

**Status:** Pending

**Work:**
- Add `src/features/memory-read/` with `api/` query options calling `get_bridge_status` through `tauri-client`, plus a small presentational panel (ready / missing / error / unsupported platform).
- Wire the panel into an existing route (home or a thin “Data” area) without introducing a player browser. Copy should tell the user that FM must be running with the plugin installed and point at the manual install note from the bridge README.
- Handle Query error/empty states per DESIGN patterns already used by `health`. Stub the new IPC command in Playwright smoke if the home route invokes it on load.

**Out of scope for this commit:**
- Scan / Load Data trigger button
- SQLite or search UI
- Progress UI for dumps

**Validation:** Vitest + mockIPC for ready/missing/error; smoke green with stub; visual check in `pnpm tauri dev` on Windows when status file exists.

**Provisional commit:** `feat(memory-read): show bridge status in UI`

### PR 2 — Request protocol and CA/PA dump

**Status:** Pending

**Provisional PR title:** `feat(memory-read): request scans and dump player CA PA`

**Purpose:** End-to-end walking skeleton: in-app trigger → bridge scan → dump file with UID/CA/PA; manual verify against known players.

**Depends on:** PR 1 merged.

#### Commit 1 — Safe memory reader and region scan

**Status:** Pending

**Work:**
- Introduce a memory-access abstraction (`IMemoryReader` or equivalent) with a production Windows implementation that uses `ReadProcessMemory` / `VirtualQuery` against the current process (same safety model as SuperScout: bad addresses fail the read, they do not hard-crash via raw pointer deref).
- Enumerate candidate heap regions: committed, private, read/write pages suitable for person-object scanning. Record region count for later diagnostics.
- Locate and record base/end for `game_plugin.dll` and `GameAssembly.dll`; optionally cache module images for later vtable/meta reads (can land here or in the next commit if splitting keeps the diff small).
- Ship a fake/in-memory reader implementation used only in tests so region logic and read helpers are exercised without launching FM.
- Keep all of this behind clear module folders under `bridge/` (e.g. Memory/) without player-field decoding yet.

**Out of scope for this commit:**
- Player decoding, dump format, request polling, Tauri changes

**Validation:** `dotnet test` with fake memory covers region filters and read edge cases (short read, out-of-range); no FM required.

**Provisional commit:** `feat(bridge): add safe memory reader and region scan`

#### Commit 2 — Versioned layout stub and CA/PA candidate dump

**Status:** Pending

**Work:**
- Add a versioned memory-layout registry keyed by FM major/minor (or build string from `game_plugin` / status). Unsupported versions refuse to scan and write a clear status error + diagnostics hint.
- Pin an initial layout for the developer’s current Steam FM26 build (class offsets + `PLAO_CA` / `PLAO_PA` / UID fields as needed). Document that pins are provisional and will need repin after patches.
- Implement person-candidate discovery: scan aligned pointer-sized values in regions, resolve class metadata/vtable range, accept candidates that look like players (UID + CA/PA sanity), dedupe by UID.
- Write a **streamed** dump file (e.g. `dump.json`) with document metadata (schema version, bridge version, FM version, timestamps, player count) and minimal player records: UID, CA, PA only. Never overwrite a previous good dump with an empty/failed result — write to a temp/sidecar then replace only on success (exact strategy chosen here and reused later).
- Emit basic `diagnostics.txt` (region counts, match counts per class offset, sample UIDs) enough to start a repin if the dump is empty or nonsense.

**Out of scope for this commit:**
- Names, attributes, contracts, clubs
- Rust request writer / UI trigger (next commit) — dump may be forceable via a temporary bridge-side trigger or status flag for manual testing if needed

**Validation:** Manual: load a known save on Windows, produce a dump, verify several known players’ UID/CA/PA against FM; unit tests for dump metadata and “do not clobber good dump on failure”; diagnostics readable after a failed version check.

**Provisional commit:** `feat(bridge): dump player UID CA PA candidates`

#### Commit 3 — In-app scan request and completion watch

**Status:** Pending

**Work:**
- Bridge: background poll for a request file in the bridge directory (ignore stale requests older than a short TTL, same class of bug SuperScout fixed). On valid request, run the CA/PA dump path off the Unity main thread; update `status.json` through phases (`idle` → `scanning` → `ready` / `failed`) with progress fields if cheap (candidates found, elapsed).
- Rust: write the request file from an IPC mutation (e.g. `request_player_dump`); poll/watch status (and dump mtime/presence) until terminal state or timeout; return bounded progress/result DTOs — never the full dump body over IPC.
- Preserve prior good dump on failure (enforce the replace-only-on-success rule from the bridge side; Rust should treat missing/failed dumps as errors without deleting files).
- UI: add an explicit trigger control (wording can say “Load Data” or “Scan” — product Load Data persistence still belongs to feature 2; this only requests a fresh dump). Show busy/success/error from status. Keep the player table out.

**Out of scope for this commit:**
- Full field extraction beyond UID/CA/PA
- SQLite import or snapshot retention policy beyond dump files on disk

**Validation:** Rust tests with temp dirs simulating status transitions and stale requests; Vitest for trigger + busy/error; manual Windows E2E: click trigger → dump refreshes → status returns to ready.

**Provisional commit:** `feat(memory-read): trigger dump from app via file protocol`

### PR 3 — Player identity and attributes

**Status:** Pending

**Provisional PR title:** `feat(memory-read): extract player identity and attributes`

**Purpose:** Expand dump to identity and attribute fields required by CONCEPT MVP scouting.

**Depends on:** PR 2 merged.

#### Commit 1 — Names, DOB, nationality, height, foot, positions

**Status:** Pending

**Work:**
- Extend player extraction beyond UID/CA/PA: display name (handle non-ASCII), date of birth / age inputs as present in memory, nationality (single or multi if the layout exposes it), height, preferred foot, and natural positions.
- Keep decoding behind dedicated readers (name/nation/person facet) so later attribute/contract work does not tangle with string/date helpers. Bump dump schema version when the player object shape changes.
- Reject or skip candidates that pass CA/PA checks but fail identity sanity (empty names, impossible DOB) rather than emitting garbage rows — log counts in diagnostics.
- Spot-check strategy: a short list of known players in the developer’s save (including at least one non-ASCII name if available).

**Out of scope for this commit:**
- Visible/hidden/personality attribute blocks
- Contracts, clubs, loans

**Validation:** Manual identity/position checks vs FM for the spot-check set; decoder unit tests with byte fixtures where practical; schema version bump covered by a Rust or bridge fixture if the dump shape is already parsed anywhere.

**Provisional commit:** `feat(bridge): extract player identity and positions`

#### Commit 2 — Visible, hidden, and personality attributes

**Status:** Pending

**Work:**
- Extract the three CONCEPT attribute groups into each dumped player: visible (technical/mental/physical as FM stores them), hidden, and personality. Document encoding quirks in bridge diagnostics or `bridge/` notes (e.g. attributes stored scaled ×5 — decode to the 1–20 scale the rest of the app will expect).
- Ensure attribute arrays/maps in the dump are stable and named consistently so feature 2 ingest and later role scoring do not reverse-engineer ad-hoc keys.
- Extend diagnostics with a few sample attribute snapshots for known players to speed patch verification.

**Out of scope for this commit:**
- Contracts and clubs
- Role score computation (roadmap feature 3)

**Validation:** Manual attribute spot-checks vs FM UI (including a hidden/personality field the UI shows under attributes/personality); dump schema remains streamable and not enormous beyond necessity.

**Provisional commit:** `feat(bridge): extract visible hidden and personality attributes`

### PR 4 — Contracts, clubs, loans, and dump contract freeze

**Status:** Pending

**Provisional PR title:** `feat(memory-read): extract contracts clubs and freeze dump contract`

**Purpose:** Complete CONCEPT MVP field set; freeze dump schema for feature 2 ingest; harden version/diagnostics.

**Depends on:** PR 3 merged.

#### Commit 1 — Contracts, wages, transfer status, value, reputation

**Status:** Pending

**Work:**
- Follow contract pointers from person objects; extract weekly wage, contract expiry, transfer status/listing flags, market value (FM’s value where readable), and player reputation into the dump.
- Define null/missing behavior for free agents or incomplete contract blocks so ingest does not see sentinel junk (document in dump schema comments or bridge README).
- Keep replace-only-on-success dump writing; bump schema version for the new fields.

**Out of scope for this commit:**
- Club/loan/division resolution
- SQLite

**Validation:** Manual checks on sample players (expiring contract, transfer-listed, high/low wage, free agent if available); fixture tests for contract decoder edge cases where bytes can be faked.

**Provisional commit:** `feat(bridge): extract contracts wages and transfer fields`

#### Commit 2 — Clubs, loans, division, team level, game date

**Status:** Pending

**Work:**
- Resolve **current club** vs **parent club**, loan in/out, division, and team level (senior/reserve/youth as the layout allows). Apply deterministic rules when a player appears in multiple team structures so dumps are stable across scans.
- Add in-game date (and source tag, e.g. `memory`) to dump metadata for later growth/snapshot features — even though ingest is feature 2, the date belongs in the dump contract now.
- Expand the manual verification set: loaned-in, loaned-out, reserve, youth, no club. Update diagnostics when club resolution fails for a large share of players.

**Out of scope for this commit:**
- SQLite ingest or search UI
- Asking-price heuristics or non-memory-derived estimates

**Validation:** Manual loan/club cases above; dump metadata includes a plausible game date matching FM’s save date.

**Provisional commit:** `feat(bridge): resolve clubs loans and game date`

#### Commit 3 — Dump schema freeze and handoff docs

**Status:** Pending

**Work:**
- Freeze a versioned dump document contract (required fields, types, null rules, schema version) that feature 2 will import. Prefer a short durable note under `.wiki/` or `bridge/` that describes the file layout and status/request protocol — not a second architecture essay.
- Add a Rust helper that validates “dump looks ingestible” (schema version, required top-level keys, non-zero players or explicit empty-save marker) without importing into SQLite. Expose only if useful for UI/IPC (“last dump OK”); otherwise keep it internal for tests.
- Confirm CONCEPT MVP field coverage against the ledger intent; note any intentional gaps in Discoveries. Optionally file BACKLOG for in-app BepInEx/plugin install if still deferred.
- Architecture current-state touch only if code already landed enough to describe the bridge boundary (otherwise leave for `/finish-feature` / docs reconciliation).

**Out of scope for this commit:**
- SQLite migrations/importer (feature 2)
- In-app BepInEx installer (unless already trivial)
- Player search UI

**Validation:** Golden fixture dump passes Rust validation; one full manual dump on a real save spot-checks the CONCEPT field list; `./scripts/dev check` + `./scripts/dev test` green; no player tables in SQLite migrations from this feature.

**Provisional commit:** `feat(memory-read): freeze dump schema for snapshot ingest`

## Active work

**PR:** PR 1 — Bridge bootstrap and status protocol

**Commit:** Commit 1 — Bridge toolchain and repo prerequisites

### RED test (active commit)

Prefer a small contract assertion where practical (e.g. example props file exists and documents required properties, or a script/check that Biome does not include `bridge/**/*.cs`). If the commit is pure gitignore + docs with no executable contract, treat it as trivial per testing guidance — the next commit’s status serialization test is the first behavioral RED.

### Expected outcome

Repo is ready for a C# bridge project: .NET/BepInEx prerequisites documented, local path overrides exemplified, build artifacts and user props ignored, Linux gate does not require `dotnet` or FM assemblies.

### Explicit exclusions

No plugin project yet, no status writer, no vendored BepInEx/FM DLLs, no Rust/UI, no memory scan.

## Discoveries and replanning

- Planning answers (2026-07-28): C# + Rust protocol confirmed; skip spike; Windows Steam only; full CONCEPT fields; in-app trigger only; manual install OK; feature ends at dump/protocol before ingest.
- 2026-07-28: Expanded each delivery-plan commit’s Work/Validation detail for clearer `/build` handoff (still high-level; no implementation code in the ledger).
- 2026-07-28: Inserted PR 1 commit 1 as toolchain/repo prerequisites (`chore(bridge)`); scaffold status writer is now commit 2. BepInEx/FM interop remain machine-local — not vendored.
- 2026-07-28: Recorded [ADR-0016](../../decisions/0016-csharp-bepinex-fm26-bridge.md); deferred in-app install to [BACKLOG.md](../../BACKLOG.md).

## Completed work

| PR | Commit | Hash | Notes |
| --- | --- | --- | --- |
| — | — | — | — |

## Final validation

- Manual: install plugin, load save, trigger scan from app, spot-check CONCEPT fields vs FM for the notes’ verification set (known players, loans, non-ASCII names, etc.)
- Automated: bridge unit tests with fakes; Rust protocol/validation tests; frontend mockIPC; full `./scripts/dev check` + `./scripts/dev test`
- Confirm no SQLite player schema landed in this feature

## Documentation impact

- Accepted stack decision: [ADR-0016](../../decisions/0016-csharp-bepinex-fm26-bridge.md)
- Update [ARCHITECTURE.md](../../ARCHITECTURE.md) current-state sections when bridge + protocol are implemented (ADR link already recorded in §9)
- Feature 2 plans against the frozen dump schema from PR 4
- Deferred install UX: [BACKLOG.md](../../BACKLOG.md) — in-app BepInEx / FM bridge install and remove
