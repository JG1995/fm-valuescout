# Snapshot ingest + Load Data

## Status

Active

## Intent

Persist a validated FM26 `dump.json` into SQLite as the active save’s **current** snapshot, and make **Load Data** one action: scan → validate → ingest. Prove the import with a small player sanity list and a simple multi-save switcher.

## User-visible behavior

- User can create, rename, and switch between **saves** (game save slots in the app). Exactly one save is active.
- First launch creates a default save if none exist.
- **Load Data** on the bridge panel: requests a bridge dump, validates schema v5, then replaces the active save’s current snapshot in one SQLite transaction.
- After a successful load, the UI shows snapshot metadata (player count, game date/version when present, loaded time) and a **truncated / incomplete** banner when `scanTruncated` is true.
- A short **sanity list** (name, CA, club) of players from the current snapshot proves ingest without opening JSON files.
- Failed scan or failed ingest leaves the previous snapshot untouched.
- Clear errors for missing dump, validation failure, ingest failure, and unsupported platform (scan path).

## Invariants

- Dump body never crosses IPC — Rust reads `dump.json` from the bridge directory.
- Ingest runs only against dumps that pass `validate_dump_json` / `validate_dump_at_bridge_directory` (schema v5).
- Replace-only-on-success: delete/replace players for a snapshot only inside a committed transaction after a full successful parse+insert.
- Players and snapshot metadata belong to a **save**; UI queries use the **active** save’s **current** snapshot only.
- Attribute JSON `null` means unknown — never coerce to 0 on ingest.
- Truncated dumps are allowed; the snapshot must store and surface `scanTruncated` / `maxAccepted`.
- Frontend never opens SQLite or arbitrary filesystem paths.

## Non-goals

- Snapshot **history** / timelines (deferred — see [BACKLOG.md](../../BACKLOG.md))
- Role scoring (order 3)
- Full player search, filters, pagination UI (order 4)
- Player profile pages
- Raising or removing the bridge 10k scan cap / full-scan performance
- Multi-window sync or cloud sync of saves
- Importing dumps from user-picked files outside the bridge directory

## Current-state map

- Relevant components: `BridgeStatusPanel` Load Data → `request_player_dump`; `memory_read` dump validation + golden fixture; SQLite `demo_value` only (migration v1)
- Data model: none for players yet
- Persistence: `%LOCALAPPDATA%\fm-valuescout\fm-bridge\dump.json`; `app.db` under app data
- Existing behavioral assumptions: Load Data = scan only; validation failures after scan are logged, not hard-failed on IPC today
- Architectural seams: Rust owns FS + DB; React presents status and triggers; dump contract frozen in `bridge/DUMP_SCHEMA.md`
- Tests: golden_dump_v5.json; cargo dump validation; Vitest bridge panel mocks
- Primary risks: large ingest blocking UI thread (use `spawn_blocking`); Load Data combining scan+ingest error UX; schema richness vs YAGNI

## Feature architecture (this feature)

```text
React (memory-read + snapshot UI)
  → save switcher / create / rename
  → Load Data → IPC load_data (or scan + ingest composed in Rust)
  → snapshot status + sanity list queries

Rust features/snapshot (new)
  → saves + active-save selection
  → ingest: read dump path via memory_read helpers → validate → transactional replace current snapshot
  → query: snapshot metadata + player summaries for active save

SQLite
  saves
  snapshots (per save; one current)
  players (FK snapshot_id; scalars + JSON for maps/arrays as needed for later search)
```

`memory_read` keeps bridge protocol and dump validation. `snapshot` owns persistence and Load Data persistence semantics. Same crate may call `memory_read::dump_validation` and dump path helpers — do not duplicate schema rules.

## Uncertainty register

### Known

- Dump schema v5 frozen; validator ready
- CONCEPT: explicit Load Data refresh; live dump is source of truth
- Solo scope; truncated dumps are primarily for development

### Assumptions

- Easy save switching = create + rename + set-active control on home (no deep settings app)
- Sanity list ≈ first N players by name or dump order (e.g. 20) — not full search
- Default save auto-created on first open is enough onboarding

### Decisions

- **Truncated dumps:** ingest allowed; always surface incomplete state in UI
- **Load Data:** one action — scan then ingest; previous snapshot retained on any failure
- **Multi-save (M1):** thin saves + active save now; not a singleton DB
- **History:** out of scope; schema uses `snapshots` + `snapshot_id` on players so history can be additive later
- **Sanity list:** in scope (name, CA, club)
- **Attributes storage:** prefer JSON columns for attribute/hidden/personality maps in v1; scalars for list/search foundations (uid, name, ca, pa, club name, etc.)

### Unknowns

- Exact column set for `players` beyond sanity + CONCEPT scalars — settle in PR 1 commit from DUMP_SCHEMA (no inventing fields)

### Risks

- Combined Load Data latency (scan up to 120s + ingest) — keep busy states clear
- Mutex/`Connection` held too long during large insert — `spawn_blocking` + short transactions
- Soft validation today on scan path vs hard-fail before ingest — ingest must hard-fail closed

## Walking skeleton

Migration creates saves/snapshots/players → ingest golden fixture into a save in cargo tests → IPC load + sanity list returns rows → home shows save switcher, Load Data success metadata, truncated banner, and a short player table.

## Delivery plan

### PR 1 — Saves schema and ingest engine

**Status:** Active

**Provisional PR title:** `feat(snapshot): add saves schema and dump ingest`

**Purpose:** Land SQLite model and transactional ingest without requiring the full Load Data UX yet. Trunk gets a tested import path from fixtures.

**Depends on:** Completed FM26 memory read (dump schema v5 + validator).

**Merge to trunk when:** Migrations apply; ingest replaces current snapshot from a validated dump file; save CRUD works via IPC or service tests; gate green.

#### Commit 1 — Migration for saves, snapshots, and players

**Status:** Active

**Work:** Add migration v2: `saves`, `snapshots` (with current-pointer or `is_current` / `saves.current_snapshot_id`), `players` keyed by `snapshot_id`. Include dump metadata columns needed for UI (`scan_truncated`, `max_accepted`, `player_count`, game/bridge version fields, timestamps). Index what sanity list and later search will need (e.g. snapshot_id, name, ca). Keep attribute maps as JSON text for v1.

**Out of scope for this commit:**
- Ingest logic
- IPC / UI
- Role scores

**Validation:** Migration tests on fresh DB; `./scripts/dev check`.

**Provisional commit:** `feat(snapshot): add saves snapshots and players migration`

#### Commit 2 — Save CRUD and active-save selection

**Status:** Pending

**Work:** Service + Tauri commands to list/create/rename saves and set the active save. Ensure a default save exists when the DB has none. Active save is the only target for ingest and queries.

**Out of scope for this commit:**
- Ingest
- React UI (stub IPC ok for later)

**Validation:** `cargo test` for create/switch/default; `./scripts/dev check`.

**Provisional commit:** `feat(snapshot): add save CRUD and active save selection`

#### Commit 3 — Transactional ingest from dump file

**Status:** Pending

**Work:** Ingest a validated dump path into the active save: create new current snapshot, insert players, commit; on failure roll back and keep prior current snapshot. Persist `scanTruncated` / `maxAccepted`. Map dump player fields into columns/JSON. Hard-fail if validation fails. Unit-test with `golden_dump_v5.json` and a truncated fixture variant.

**Out of scope for this commit:**
- Calling the bridge scan
- React sanity list UI

**Validation:** `cargo test` success, validation reject, replace-only-on-success, truncated metadata; `./scripts/dev check`.

**Provisional commit:** `feat(snapshot): ingest validated dump into active save`

### PR 2 — Load Data UX, sanity list, save switcher

**Status:** Pending

**Provisional PR title:** `feat(snapshot): wire Load Data ingest and save switcher`

**Purpose:** User-visible end-to-end: switch saves, Load Data scans and ingests, see metadata + truncated state + sanity list.

**Depends on:** PR 1 merged.

**Merge to trunk when:** Home UI supports saves and post-load proof; Vitest + gate green; docs updated for data flow.

#### Commit 1 — Load Data command composes scan and ingest

**Status:** Pending

**Work:** Rust command used by the button runs bridge dump request then ingest for the active save. Typed errors distinguish scan failure vs ingest failure. Previous snapshot retained if ingest fails after a successful scan (dump may remain on disk). Prefer `spawn_blocking` for ingest if it can block the async runtime.

**Out of scope for this commit:**
- Sanity list UI
- Save switcher UI

**Validation:** Service/command tests with mocked or fixture dump path where practical; `./scripts/dev check`.

**Provisional commit:** `feat(snapshot): compose scan and ingest for Load Data`

#### Commit 2 — Snapshot status, sanity list, and save switcher UI

**Status:** Pending

**Work:** React: active-save switcher (create / rename / switch); after load (and on open) show snapshot metadata, truncated banner, and a short player sanity table (name, CA, club). Wire queries/mutations with TanStack Query; mockIPC tests. Soften/replace scan-only success copy so Load Data means “in database.”

**Out of scope for this commit:**
- Full search/filter UI
- Snapshot history browser

**Validation:** Vitest + mockIPC; `./scripts/dev test`; `./scripts/dev check`.

**Provisional commit:** `feat(snapshot): add save switcher and ingest sanity list`

#### Commit 3 — Docs for snapshot data flow

**Status:** Pending

**Work:** Update ARCHITECTURE (ingest path, saves model), bridge/README or feature notes as needed; point TODO/completed cross-links at finish time. No ADR unless schema policy proves consequential beyond this ledger.

**Out of scope for this commit:**
- Implementation changes beyond doc fixes discovered while writing

**Validation:** Doc links resolve; `./scripts/dev check`.

**Provisional commit:** `docs(snapshot): document saves and Load Data ingest path`

## Active work

**PR:** 1 — Saves schema and ingest engine

**Commit:** Migration for saves, snapshots, and players

### RED test (active commit)

Assert a fresh DB after migrations has `saves`, `snapshots`, and `players` tables (and `user_version` ≥ 2). Catches shipping ingest code against migration v1-only schema.

### Expected outcome

Migration v2 applied on open; tables exist; no ingest/UI yet.

### Explicit exclusions

Ingest, IPC save commands, React, role scoring, history.

## Discoveries and replanning

- Product decisions (2026-07-29): truncated ingest allowed with UI; Load Data = scan+ingest; sanity list in scope; multi-save M1; snapshot history deferred to backlog.

## Completed work

| PR | Commit | Hash | Notes |
| --- | --- | --- | --- |
| — | — | — | — |

## Final validation

Full `./scripts/dev test`, `./scripts/dev check`, smoke; manual Windows: create/switch saves, Load Data with FM, confirm sanity list and truncated banner on capped dumps.

## Documentation impact

- ARCHITECTURE data flow for ingest + saves
- TODO Active / Plan next → role scoring after this feature
- BACKLOG: snapshot history (added at plan time)
- CONCEPT multi-save not previously explicit — note in completed record / CONCEPT reconciliation at finish if needed
