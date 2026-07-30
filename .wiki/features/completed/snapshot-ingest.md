# Snapshot ingest + Load Data

## Intent

Persist a validated FM26 `dump.json` into SQLite as the active app save's **current** snapshot, and make **Load Data** one action: scan → validate → ingest. Prove import with a short player sanity list and a simple multi-save switcher.

## Delivered behavior

- User can create, rename, and switch between **app save slots** (not FM save files). Exactly one save is active.
- First launch creates a default save when the database has none.
- **Load Data** in the top bar (`AppTopBar`): bridge scan, schema v5 validation, then replace the **save that was active when Load Data started** with a new current snapshot in one SQLite transaction.
- After a successful load, the UI shows snapshot metadata (player count, game date/version when present, loaded time) and a **truncated / incomplete** banner when `scanTruncated` is true.
- A short **sanity list** (name, CA, club) from the current snapshot proves ingest without opening JSON files.
- Failed scan or failed ingest leaves the previous snapshot untouched.
- Clear errors for missing dump, validation failure, ingest failure, and unsupported platform (scan path). `LoadDataError` uses `phase: scan | ingest` so the UI can distinguish scan vs ingest failures.

## Final architecture

```text
React features/snapshot
  → save switcher panel (create / rename on home route)
  → AppTopBar: ActiveSaveSelect, SnapshotFreshnessChip, Load Data → invokeCommand("load_data")
  → snapshot overview + sanity list (TanStack Query)

Rust features/snapshot
  → load_data: brief Db lock → active_save_id; scan without Db lock;
    ingest_dump_file_for_save(captured save_id)
  → save CRUD IPC; get_current_snapshot; list_sanity_players
  → ingest: validate_dump_json → transactional replace current snapshot

SQLite (migration v2)
  saves (one is_active)
  snapshots (one is_current per save)
  players (FK snapshot_id; scalars + JSON attribute maps)
```

- `memory_read` owns bridge protocol and dump validation; `snapshot` owns persistence and Load Data semantics. Dump body never crosses IPC — Rust reads `dump.json` from the bridge directory.
- `request_player_dump` remains registered for tests; the **Load Data** button in `AppTopBar` calls `load_data`.

## Important decisions

- **Truncated dumps:** ingest allowed; UI always surfaces incomplete state (`scanTruncated` / `maxAccepted`).
- **Load Data:** one user action — scan then ingest; prior snapshot retained on any failure.
- **Captured save on Load Data:** `active_save_id` is read under a brief Db lock **before** the bridge scan; ingest uses `ingest_dump_file_for_save` with that id so a mid-scan save switch cannot target the wrong slot.
- **Multi-save (M1):** thin app save slots + active save; not a singleton database.
- **Snapshot history:** out of scope; schema uses `snapshots` + `snapshot_id` on players so history can be additive later ([BACKLOG.md](../../BACKLOG.md)).
- **Attributes:** JSON text columns for maps/arrays; scalars for list/search foundations; `null` in dump JSON means unknown — never coerced to 0 on ingest.

## Migration and operational implications

- Migration v2 adds `saves`, `snapshots`, and `players` with partial unique indexes (one active save, one current snapshot per save). Foreign-key enforcement is enabled on open.
- Load Data latency includes bridge scan (up to 120s default) plus ingest; UI shows busy state during the mutation.
- Ingest hard-fails closed on validation errors; scan-path soft validation in `request_player_dump` logs warnings only and does not gate ingest.

## Validation

- `./scripts/dev test`, `./scripts/dev check`, Playwright smoke (save switcher, snapshot overview, Load Data with stubbed IPC).
- `cargo test`: migration, save CRUD, ingest (golden/truncated/reject/rollback), `load_data` (scan fail, ingest rollback, captured save id).
- Manual Windows: create/switch saves, Load Data with FM running, confirm sanity list and truncated banner on capped dumps.

**Delivery commits (final hashes):** `86032cb`, `446913f`, `a77d744`, `fc068b1`, `98297c6`, `b8540fa`, `7ee4788` (see Git history for full sequence).

## Follow-up

- **Next feature:** [Role scoring engine](../../TODO.md) (order 3) — scores on ingest using FM role-relevant attributes.
- **Active:** [bridge scan performance](../active/bridge-scan-performance.md) keeps the 500-player cap through its first PR, then validates complete snapshots.
- **BACKLOG:** snapshot history per save.
- **Roadmap:** player search (order 4), profiles, squad planner, optimizer — see [TODO.md](../../TODO.md).
