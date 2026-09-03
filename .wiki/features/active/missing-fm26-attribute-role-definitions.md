# Complete missing FM26 attribute role definitions

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 4c1e928abaf409282a439d7974001315cf57b48e938e553c23fcacc52ab47c5a

## Intent

Complete the FM26 In Possession / Out of Possession attribute role catalog so that every Moneyball presentation role maps to a real attribute role. The Rust scoring engine currently ships 68 roles; 11 generic out-of-possession (OOP) presentation rows have `attribute_role_id: null` and render as placeholders in General profiles, searches, and planner surfaces. This feature adds those 11 missing generic OOP definitions with the FM-approved primary/secondary attribute bands, applies the recorded Channel Midfielder eligibility correction (`channel_midfielder_ip` gains `MC`), and maps the existing 88-entry Moneyball presentation definitions to the expanded attribute catalog. After a fresh Load Data the new roles score like every other role; before that the migration preserves existing scores and shows the new roles as unavailable.

## User-visible behavior

- **General attribute roles:** 11 new OOP roles exist in the closed scoring catalog and in General search/profile/planner surfaces:
  - Goalkeeper (OOP) — `GK` — Primary: Aerial Reach, Command of Area, Communication, Handling, Reflexes, Concentration, Positioning, Agility. Secondary: One On Ones, Rushing Out, Anticipation, Decisions.
  - Centre-Back (OOP) — `DC` — Primary: Heading, Marking, Tackling, Anticipation, Positioning, Jumping Reach, Strength. Secondary: Aggression, Bravery, Composure, Concentration, Decisions, Pace.
  - Wide Centre-Back (OOP) — `DC` — Primary: same as Centre-Back. Secondary: Aggression, Bravery, Concentration, Decisions, Acceleration, Agility, Pace.
  - Full-Back (OOP) — `DL`, `DR` — Primary: Marking, Tackling, Anticipation, Positioning, Teamwork, Acceleration. Secondary: Aggression, Concentration, Decisions, Work Rate, Agility, Pace, Stamina.
  - Wing-Back (OOP) — `DL`, `DR`, `WBL`, `WBR` — Primary: Marking, Tackling, Anticipation, Positioning, Teamwork, Acceleration, Stamina. Secondary: Aggression, Concentration, Decisions, Work Rate, Agility, Pace.
  - Defensive Midfielder (OOP) — `DM` — Primary: Tackling, Anticipation, Decisions, Positioning, Teamwork, Work Rate. Secondary: Marking, Aggression, Concentration, Stamina, Strength.
  - Central Midfielder (OOP) — `MC` — Primary: Tackling, Decisions, Teamwork, Work Rate. Secondary: Marking, Aggression, Anticipation, Concentration, Positioning, Stamina.
  - Wide Midfielder (OOP) — `ML`, `MR` — Primary: Decisions, Teamwork, Work Rate, Acceleration. Secondary: Marking, Aggression, Anticipation, Off The Ball, Agility, Pace, Stamina.
  - Attacking Midfielder (OOP) — `AMC` — Primary: Anticipation, Decisions, Work Rate. Secondary: Marking, Aggression, Off The Ball, Teamwork, Stamina.
  - Winger (OOP) — `AML`, `AMR` — Primary: Anticipation, Decisions, Teamwork, Work Rate, Acceleration. Secondary: Marking, Aggression, Off The Ball, Agility, Pace, Stamina.
  - Centre Forward (OOP) — `ST` — Primary: Anticipation, Decisions, Work Rate. Secondary: Marking, Aggression, Off The Ball, Teamwork, Stamina.
- **Channel Midfielder correction:** `channel_midfielder_ip` supports both `AMC` and `MC` (`position_tags: ["AMC", "MC"]`) with the same definition; the pitch, role selector, planner lanes, and optimizer treat it as eligible from either base position.
- **Moneyball presentation mapping:** The 88-entry built-in Moneyball catalog keeps its metrics/version (v1) and family counts, but the 11 previously null `attribute_role_id` entries now point to the new attribute roles:
  `amc_attacking_midfielder_oop` → `attacking_midfielder_oop`, `dc_centre_back_oop` → `centre_back_oop`, `dc_wide_centre_back_oop` → `wide_centre_back_oop`, `mc_central_midfielder_oop` → `central_midfielder_oop`, `dm_defensive_midfielder_oop` → `defensive_midfielder_oop`, `dl_dr_full_back_oop` → `full_back_oop`, `gk_traditional_goalkeeper_oop` → `goalkeeper_oop`, `st_centre_forward_oop` → `centre_forward_oop`, `ml_mr_wide_midfielder_oop` → `wide_midfielder_oop`, `wbl_wbr_wing_back_oop` → `wing_back_oop`, `aml_amr_winger_oop` → `winger_oop`. All 88 presentation rows now have a non-null `attribute_role_id` and their `PlayerRoleScore` placeholders become real Current/Potential scores after supported data is materialized. The generic `wing_back_oop` retains `DL/DR/WBL/WBR` tags while presentation `wbl_wbr_wing_back_oop` remains `WBL/WBR`.
- **General profile/Search/Planner exposure:** Existing General profile placeholders for those 11 presentation IDs become numeric scores after a Load Data that materializes the new compact scores. General Search can display, filter, and sort by the new `role.*` and `potential_role.*` metrics, and the Planner/tactic optimizer can consume the new OOP roles once the save's effective current snapshot is at the new model version. No new visual design or user workflow is added.
- **Strict “Require Load Data” compatibility:** Raw snapshot/history rows and non-compact tables are preserved. Existing `app-v2.db` snapshots are migrated to the new schema with explicit snapshot-level score-model provenance (see Invariants/Architecture). Before a successful fresh Load Data for a given snapshot, the 11 new OOP columns remain uncomputed (null) while the 68 existing roles remain readable at legacy availability. No new-role score may become materialized for any player or snapshot through migration, promotion, or player/squad boost before that snapshot's successful fresh Load Data. Uncomputed new-column nulls are never mislabeled as computed missing-attribute nulls; reads that request a new OOP role on a legacy-availability snapshot are treated as unavailable (`—` in UI / missing-preserving). After Load Data the current snapshot's 79 current + 79 potential scores are fully materialized at the new model.

## Invariants

- The closed player attribute catalog grows from 68 to 79 roles (68 current + 68 potential → 79 + 79 compact columns). Catalog IDs remain safe snake_case (`^[a-z][a-z0-9_]*$`) and are the sole source of SQL identifiers via `player_metrics::compact` validation; WebView input never becomes an identifier. `channel_midfielder_ip` position_tags become `["AMC","MC"]`.
- Scoring keeps the existing 75/25 primary/secondary band formula, equal weight within each band, `/20×100` scaling, rounding, and null semantics (any required attribute null or missing → null score). Projection keeps the existing CA-to-PA visible-attribute projection, age-29 identity boundary, and Rust ownership.
- `player_role_metrics` remains current-only, one row per player in the effective current snapshot, with nullable 0–100 columns. Missing source attributes produce null, never zero. Historical snapshots keep raw player/staff facts only; no projected attributes or compact rows are backfilled for historical snapshots. Compact rows carry explicit `score_model_version` + `projection_model_version` per row.
- Snapshot-level score-model provenance is the authoritative lifecycle marker. Migration v40 owns the new immutable checked-in inventory (11 current + 11 potential nullable columns) and a new `snapshots.compact_score_model_version INTEGER NOT NULL DEFAULT 1 CHECK (compact_score_model_version IN (1,2))` column with `NOT NULL` + `DEFAULT 1` and does not edit v38 DDL. SQLite semantics: the `ALTER ... NOT NULL DEFAULT 1` causes every pre-v40 snapshot row to read/store `1`, any omitted insert to store `1`, and any explicit `NULL` insert to be rejected — no `UPDATE snapshots SET ...` backfill is required or performed. `DEFAULT 1` is only the migration/legacy-safe fallback; every successfully published fresh ingest must explicitly bind `compact_score_model_version = 2` in `insert_prepared_snapshot` (see Ingest below) and tests prove every published ingest row is `2` including older non-current ingests. Migration preserves 68-score compact rows (11 new columns nullable, `player_role_metrics.score_model_version` remains `1`); no snapshot row is left null and no `UPDATE player_role_metrics SET score_model_version = 2` occurs before the next Load Data for that snapshot. Compact rows remain current-only, so a non-current ingested snapshot carries provenance `2` with no derived rows until it is later promoted.
- `SCORE_MODEL_VERSION` moves from 1 to 2; `PROJECTION_MODEL_VERSION` stays 2. Readers validate against the snapshot's provenance and the exact pair: current-role reads require `score_model_version` matching the snapshot's availability (`1` for legacy 68-role availability, `2` for 79-role availability); any potential-role read requires both `score_model_version` and `projection_model_version` (potential-only Search/Squad and Profile/Planner paths enforce both). A missing compact row or wrong-version row fails before values are read; a read never writes or repairs. New OOP columns on legacy-availability snapshots remain uncomputed nulls and are not returned as computed missing-attribute nulls. Provenance `1` means exactly the checked-in `LEGACY_V1_ROLE_IDS` partition (68 ids) is computed; provenance `2` means all 79 `all_roles()` ids are computed. See partition invariant below.
- Legacy-v1 partition is an immutable checked-in inventory `LEGACY_V1_ROLE_IDS: &[&str]` of exactly the 68 pre-feature ids, owned by `src-tauri/src/features/player_metrics/compact.rs` (compact contract owner). It lists every legacy id explicitly (no derivation by array position): `goalkeeper_ip`, `ball_playing_goalkeeper_ip`, `no_nonsense_goalkeeper_ip`, `line_holding_keeper_oop`, `sweeper_keeper_oop`, `centre_back_ip`, `ball_playing_centre_back_ip`, `no_nonsense_centre_back_ip`, `wide_centre_back_ip`, `advanced_centre_back_ip`, `overlapping_centre_back_ip`, `covering_centre_back_oop`, `stopping_centre_back_oop`, `covering_wide_centre_back_oop`, `stopping_wide_centre_back_oop`, `full_back_ip`, `inside_full_back_ip`, `holding_full_back_oop`, `pressing_full_back_oop`, `inside_wing_back_ip`, `playmaking_wing_back_ip`, `wing_back_ip`, `advanced_wing_back_ip`, `holding_wing_back_oop`, `pressing_wing_back_oop`, `defensive_midfielder_ip`, `box_to_box_midfielder_ip`, `box_to_box_playmaker_ip`, `deep_lying_playmaker_ip`, `half_back_ip`, `dropping_defensive_midfielder_oop`, `pressing_defensive_midfielder_oop`, `screening_defensive_midfielder_oop`, `wide_covering_defensive_midfielder_oop`, `central_midfielder_ip`, `advanced_playmaker_ip`, `midfield_playmaker_ip`, `wide_central_midfielder_ip`, `pressing_central_midfielder_oop`, `screening_central_midfielder_oop`, `wide_covering_central_midfielder_oop`, `wide_midfielder_ip`, `tracking_wide_midfielder_oop`, `wide_outlet_wide_midfielder_oop`, `inside_winger_ip`, `playmaking_winger_ip`, `winger_ip`, `attacking_midfielder_ip`, `channel_midfielder_ip`, `free_role_ip`, `second_striker_ip`, `central_outlet_attacking_midfielder_oop`, `splitting_outlet_attacking_midfielder_oop`, `tracking_attacking_midfielder_oop`, `wide_forward_ip`, `inside_forward_ip`, `inside_outlet_winger_oop`, `tracking_winger_oop`, `wide_outlet_winger_oop`, `centre_forward_ip`, `channel_forward_ip`, `deep_lying_forward_ip`, `false_nine_ip`, `poacher_ip`, `target_forward_ip`, `central_outlet_centre_forward_oop`, `splitting_outlet_centre_forward_oop`, `tracking_centre_forward_oop`. Runtime asserts `LEGACY_V1_ROLE_IDS.len() == 68`, every entry is in `all_roles()`, ids are unique and `require_safe_snake_case`, and `LEGACY_V1_ROLE_IDS ∪ NEW_OOP_11 == all_roles()` with `all_roles().len() == 79`. Writers align columns/values by iterating `all_roles()` order: for provenance 1 the writer computes scores only for ids in the `LEGACY_V1_ROLE_IDS` set and writes `None` (uncomputed null) for the 11 new ids, so column order stays `all_roles()` order and value order stays aligned. Drift is rejected: unknown id, duplicate, wrong case, or size ≠68 fails the compact contract test; a writer that would emit 68 values without aligned `None`s fails `persist_rows_borrowed` length check. Duplication is limited to role-id strings; full `RoleDefinition` bodies are not duplicated.
- `COMPACT` preparation stays outside the `Db(Mutex<Connection>)` lock where practical; publication revalidates captured save/snapshot context and persists raw + derived state in one final transaction. Every successfully ingested snapshot receives `snapshots.compact_score_model_version = 2` in that transaction regardless of effective-current selection; compact rows are persisted only for the effective current snapshot. Promotion and single-player boost reconciliation reuse the same version-preserving helpers keyed by `LEGACY_V1_ROLE_IDS`: promotion rebuilds legacy-availability snapshots at version 1 (compute exactly the `LEGACY_V1_ROLE_IDS` 68, leave 11 new columns null uncomputed), fresh Load Data and promotion of a provenance-2 snapshot write version 2 (full 79); boost `replace_player` captures the target snapshot's `compact_score_model_version` and recomputes only the availability-appropriate columns (version 1 → recompute 68 via `LEGACY_V1_ROLE_IDS`, preserve 11 null; version 2 → recompute all 79) and never materializes new-role scores on legacy snapshots. Snapshot provenance is always non-null (`1` or `2`); a newly published snapshot cannot be inserted without explicit `2`.
- The 88-entry Moneyball presentation catalog stays at `BUILTIN_ROLE_CATALOG_VERSION = 1` and retains its position-family counts (attacking_midfielder 9, central_defender 12, central_midfielder 10, defensive_midfielder 10, full_back 8, goalkeeper 6, striker 10, wide_midfielder 7, wing_back 7, winger 9) and five-metric per-definition contract. All 88 `attribute_role_id` values are now non-null and validated against `all_roles()`. Tactic coverage moves from 103/111 (8 uncovered) to 119/129 (10 uncovered); exact uncovered ten are pinned in Architecture/validation.
- Frontend General Search/Planner role exposure follows the existing closed-catalog contracts: metric IDs are `role.<id>` and `potential_role.<id>` mapped through `ROLE_CATALOG` mirrors (frontend `ROLE_CATALOG` has only `id`/`label`; no `phase`/`positionTags` fields). Search/Squad sorts keep null-last with UID tie-breaker; Moneyball and Club DNA ownership does not change. Dynamic columns, Squad, and Planner eligibility/options remain Rust-owned; frontend mirrors are display only.
- No new visual design, interaction model, or release packaging is introduced by this feature.

## Non-goals

- New visual design, layout, or user workflow for General, Moneyball, Shortlist, Planner, or profiles.
- Changes to the 75/25 formula, null semantics, current/potential projection inputs, or Club DNA / Moneyball percentile scoring.
- Historical Player Profile timeline, historical Moneyball seasons, cross-snapshot comparison, or analytics beyond the current-snapshot contract.
- Backfilling existing retained snapshots' new OOP scores without a Load Data, opening or migrating the legacy `app.db`, WAL/pool/Rayon or other concurrency infrastructure, additional per-role indexes without representative failure, or new dependencies.
- BepInEx bootstrap, bridge DLL build-before-copy, or other backlog items in `.wiki/BACKLOG.md`.

## Current-state map

- Relevant components: `src-tauri/src/features/scoring/catalog.rs` (68 `RoleDefinition`s, `DUMP_ATTRIBUTE_KEYS`, `label_to_dump_key`), `src-tauri/src/features/scoring/score.rs` (75/25), `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` (88 definitions, 11 null `attribute_role_id`), `src-tauri/src/features/moneyball/role_catalog.rs` (`builtin_catalog`, validation, `EXPECTED_FAMILY_COUNTS`, tactic compound-key coverage 103/111 with 8 uncovered), `src-tauri/src/features/player/query.rs` (`map_moneyball` via `attribute_role_id` → null placeholders), `src-tauri/src/features/player_metrics/compact.rs` (`SCORE_MODEL_VERSION = 1`, `PROJECTION_MODEL_VERSION = 2`, `player_current_column`/`player_potential_column`, `player_metrics_join`, `assert_snapshot_complete`, `assert_read_models_complete`), `src-tauri/src/features/player_metrics/potential_scores.rs` (one-projection-per-player writer, `rebuild_snapshot`, `replace_player`, `reconcile_current_selection`), `src-tauri/src/db/migrations.rs` (v38 `player_role_metrics` 68+68 and `staff_role_metrics` 21, v39 drop of normalized tables, `latest_version = 39`), `src-tauri/src/features/search/{query,filter}.rs` + `src-tauri/src/features/planner/*` + `src/utils/role-catalog.ts` (68 entries, `id`/`label` only) + `src/utils/moneyball-role-catalog.ts` (88) + tactic lanes (`src/utils/tactic-ids.ts` stale 103/111 comment, `src-tauri/src/features/planner/tactic.rs`), `src-tauri/src/features/snapshot/{ingest,service}.rs` (promotion via `potential_scores::reconcile_current_selection`), `src-tauri/src/features/player/service.rs` (boost `replace_player`), `src-tauri/src/features/search/query.rs` `assert_read_models_complete` for current vs potential, `src-tauri/src/features/squad` and `src-tauri/src/features/player_metrics/resolver.rs`.
- Data model: `player_role_metrics(snapshot_id, uid, score_model_version, projection_model_version, 68 current, 68 potential)` with FK to `players` and 0–100 checks; `players.potential_attributes_json` + `potential_projection_model_version` hold the single projected visible map; `player_moneyball_stats` snapshot-owned. `snapshots` has no compact model version column yet; historical snapshots retain raw facts; only the effective current snapshot has compact rows.
- Persistence and migrations: `rusqlite` + `PRAGMA user_version` registry; `player_role_metrics` immutable v38 inventory (140 columns total inc. snapshot_id/uid/versions); architecture requires later catalog changes to add a new migration and model version rather than edit v38.
- Existing behavioral assumptions: General profile Pitch filters roles by `position_tags` (familiarity ≥ 15) and shows `—` for unmapped Moneyball presentation roles; Search/Squad resolver maps `role.*`/`potential_role.*` through closed-catalog lookup and safe snake_case; null-last ordering; tactic `get_planner_tactic_options` and optimizer use catalog `position_tags`; `channel_midfielder_ip` currently `["AMC"]` only. Frontend `ROLE_CATALOG` is `id`/`label` only.
- Architectural seams: ADR-0028 (compact current-snapshot metrics, fresh `app-v2.db`, one-transaction publication, no historical derived rows), ADR-0025 (selective index-driven sorts), scoring `combine_role_scores` with lane weights; Tauri `Channel` progress for Load Data; route `/players/$uid` with `tab` param and `view` default.
- Project validation commands: `./scripts/dev check` (full gate), `./scripts/dev check-rust`, `./scripts/dev check-app`, `./scripts/dev test`, `./scripts/dev smoke`; planning validators `ledger_state.py` and `delivery_state.py`.
- Primary risks: catalog/schema/model-version drift across Rust migrations, compact helpers, Moneyball catalog, and frontend mirrors; lifecycle reads after version bump (incomplete-snapshot errors vs preserved existing scores); lifecycle promotion/boost recomputing 79 roles early and mislabeling uncomputed nulls; potential-only reads missing score-model check; Moneyball mapping to wrong attribute role; tactic coverage miscount; stale tests asserting 68.

## Feature architecture

- **Rust attribute catalog (source of truth):** `src-tauri/src/features/scoring/catalog.rs` defines the 79 `RoleDefinition`s (68 existing + 11 new OOP generics). Each new OOP definition uses the JAY-31 band lists transcribed to dump PascalCase keys (e.g., `AerialReach`, `CommandOfArea`, `Communication`, `Handling`, `Reflexes`, `Concentration`, `Positioning`, `Agility` plus secondaries such as `OneOnOnes`, `RushingOut`, `Anticipation`, `Decisions`). `channel_midfielder_ip` expands `position_tags` to `["AMC", "MC"]` (canonical pitch order) while keeping its `display_name`, `phase`, and band lists. A single `all_roles()` remains the closed roster; catalog tests assert 79 ids, unique ids, non-empty primary from `DUMP_ATTRIBUTE_KEYS`, disjoint primary/secondary, and correct phase suffix. New ids: `goalkeeper_oop`, `centre_back_oop`, `wide_centre_back_oop`, `full_back_oop`, `wing_back_oop` (`DL/DR/WBL/WBR`), `defensive_midfielder_oop`, `central_midfielder_oop`, `wide_midfielder_oop`, `attacking_midfielder_oop`, `winger_oop`, `centre_forward_oop`.
- **Compact metrics and snapshot-level provenance (strict Require Load Data):** `src-tauri/src/db/migrations.rs` adds migration v40 `expand_compact_role_metrics_for_generic_oop_and_snapshot_provenance` with:
  - 22 `ALTER TABLE player_role_metrics ADD COLUMN <role_id>` + matching `potential_<role_id>` for the 11 new ids, each `INTEGER CHECK (col IS NULL OR col BETWEEN 0 AND 100)`.
  - `ALTER TABLE snapshots ADD COLUMN compact_score_model_version INTEGER NOT NULL DEFAULT 1 CHECK (compact_score_model_version IN (1,2))` — `NOT NULL` + `DEFAULT 1` causes existing rows to store/read `1`, omitted inserts to store `1`, and explicit `NULL` inserts to be rejected; no `UPDATE snapshots SET ...` backfill is required or performed. `DEFAULT 1` is the migration/legacy-safe fallback only — every successfully published fresh ingest explicitly binds `2` in `insert_prepared_snapshot` (tests prove every published ingest row is `2`, including older non-current ingests). Existing `player_role_metrics` rows keep `score_model_version = 1` and the 11 new columns remain `NULL` (uncomputed). No `UPDATE player_role_metrics SET score_model_version = 2` occurs.
  `src-tauri/src/features/player_metrics/compact.rs` bumps `SCORE_MODEL_VERSION` to 2 (projection stays 2), and `player_potential_column`/`player_current_column` validation reflects 79 roles. It also owns the immutable checked-in partition `pub const LEGACY_V1_ROLE_IDS: &[&str]` (68 ids enumerated in Invariants) with runtime assertions (`len==68`, unique, all in `all_roles()`, disjoint from new 11, `all_roles().len()==79`). Writers for provenance 1 compute exactly those 68 ids (set membership against `LEGACY_V1_ROLE_IDS`) and write `None` for the remaining 11, iterating `all_roles()` order so column/value order stays aligned. `potential_scores.rs` persists 79+79 values per player for version-2 writes; compact helpers add `snapshot_compact_version(conn, snapshot_id)` and version-preserving `rebuild_snapshot_versioned` / `replace_player_preserving_version` (both keyed by `LEGACY_V1_ROLE_IDS`) used by promotion/boost paths. `compact::assert_snapshot_complete` and `compact::assert_read_models_complete` accept the snapshot's provenance: legacy snapshots require `score_model_version = 1` and `IS NULL` on the 11 new columns (set membership via `LEGACY_V1_ROLE_IDS`); version-2 snapshots require `score_model_version = 2` with all 79 columns computed (null only when required attribute missing). Any potential-role read (Search potential filter/sort, Squad potential sort, Profile potential scores, Planner potential optimizer, tactic potential lanes) enforces `projection_model_version = 2` in addition to the snapshot's score version. Historical snapshots are not scored; promotion and `replace_player` keep one-row replacement semantics. The migration owned by this feature is the only writer that initializes `snapshots.compact_score_model_version` for pre-v40 rows; ingest (below) is the only writer that sets `2` for newly ingested snapshots; later scoring work must add v41+.
- **Ingest / promotion / boost lifecycle:**
  - **Migration:** as above, preserves raw rows, preserves 68 scores at version 1, adds nullable new columns, marks every pre-v40 snapshot provenance 1 with `NOT NULL DEFAULT 1` guaranteeing no null provenance.
  - **Ingest (`src-tauri/src/features/snapshot/ingest.rs` `publish_prepared_snapshot_canonical`):** prepares 79+79 scores outside the `Db` mutex, then in the single final transaction inserts the raw snapshot (explicit `compact_score_model_version = 2`), updates `snapshots.compact_score_model_version = 2` for the ingested `snapshot_id` regardless of whether it becomes effective current, then selects the new effective current snapshot in game-date order, clears displaced derived rows from non-current snapshots (`player_role_metrics` and `players.potential_*`), and persists fully computed rows only for the effective current snapshot with `score_model_version = 2` / `projection_model_version = 2`. If the ingested snapshot is not selected as current (older in-game date), it retains provenance 2 with raw rows only and no compact rows; the still-current snapshot keeps its own provenance (1 or 2) and its compact rows unchanged. Revalidates captured save/snapshot context before any write. This guarantees every successfully published snapshot has explicit provenance 2 and none remains null/ambiguous, while compact rows remain current-only.
  - **Promotion (`src-tauri/src/features/snapshot/service.rs` via `potential_scores::reconcile_current_selection`):** clears non-current snapshots' derived rows, and if the newly selected current snapshot has provenance 1, rebuilds its compact rows at version 1 (compute exactly the `LEGACY_V1_ROLE_IDS` 68, leave 11 new columns as `NULL` uncomputed, set `potential_attributes_json`/`projection_model_version` still 2 but compact score version 1). If provenance is 2, rebuilds at version 2 (full 79 via `all_roles()`). Never upgrades a legacy snapshot to 2 without a fresh Load Data — promotion of a provenance-1 snapshot stays at 1. Promotion of a previously ingested non-current provenance-2 snapshot now materializes its 79 derived rows at 2 on selection.
  - **Boost (`src-tauri/src/features/player/service.rs` via `potential_scores::replace_player`):** captures the target snapshot's `compact_score_model_version`; replacement recomputes only the availability-appropriate columns keyed by `LEGACY_V1_ROLE_IDS` (version 1 → recompute 68, preserve 11 `NULL` uncomputed; version 2 → recompute all 79) and persists with the snapshot's version. Does not materialize new-role scores on legacy snapshots. Squad boost iterates per-player with the same preservation.
  - **Read:** `player/query.rs` `load_role_scores`, `search/query.rs` `search_players`, squad/planner queries use `player_metrics_join` filtered by the snapshot's provenance version and enforce both `score_model_version` and `projection_model_version` when any potential role is requested. A potential-only Search/Squad sort that supplies a wrong `score_model_version` fails as incomplete rather than returning mislabeled nulls.
- **Moneyball presentation mapping:** `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` fills the 11 null `attribute_role_id` values with the mapping table above. No metric keys, weights, or version are changed. `src-tauri/src/features/moneyball/role_catalog.rs` validation (known attribute roles, duplicate compound keys, family counts 88, tactic compound-key coverage) remains green; `src-tauri/src/features/player/query.rs` now maps those 11 presentation IDs to real Current/Potential scores instead of `None`. After mapping, tactic coverage becomes 119/129 with 10 uncovered `(attribute_role_id, position_tag)` combos: the prior 8 (`holding_wing_back_oop+DL`, `holding_wing_back_oop+DR`, `pressing_wing_back_oop+DL`, `pressing_wing_back_oop+DR`, `box_to_box_midfielder_ip+MC`, `box_to_box_playmaker_ip+MC`, `deep_lying_playmaker_ip+MC`, `second_striker_ip+ST`) plus `wing_back_oop+DL`, `wing_back_oop+DR` (generic retains DL/DR/WBL/WBR while presentation remains WBL/WBR).
- **Frontend mirrors (display only):** `src/utils/role-catalog.ts` expands `ROLE_CATALOG` from 68 to 79 entries with the same `id`/`label` as the Rust catalog (labels like `Goalkeeper (OOP)`, `Centre-Back (OOP)`, etc., matching `catalog.rs` `display_name` + phase; no `phase`/`positionTags` fields are added). `src/utils/moneyball-role-catalog.ts` retains 88 entries; no identifier change is needed there because Moneyball search/profile consumers read the Rust-mapped scores. `src/utils/moneyball-search-metrics.ts` and Search/Planner stores continue to resolve `role.<id>` through the updated `ROLE_CATALOG`. Tests that asserted 68 now assert 79; any hard-coded `ROLE_CATALOG.length` snapshot is updated. Frontend has no 77/11 mapping assertion; mapping counts are Rust tests plus the `src/utils/tactic-ids.ts` header comment.
- **Strict read compatibility (Require Load Data):** After the app updates and v40 runs, existing `app-v2.db` current snapshots have version 1 rows with the 68 old scores intact and 11 new columns null uncomputed, and `snapshots.compact_score_model_version = 1` (`NOT NULL`). General Search, Profile, and Planner reads for the 68 existing roles succeed without Load Data; reads that request a new OOP role return unavailable (`—` in UI) and Search filters/sorts on a new `role.*` are treated as incomplete/unavailable rather than computed null. After the user runs Load Data, ingest prepares 79+79 scores outside the `Db` mutex, then the single final transaction marks the ingested snapshot `2`, selects the new effective current snapshot, clears displaced derived rows, and persists fully computed rows only for the effective current snapshot at `2`. If the ingested snapshot is older than current and not selected, it remains provenance 2 with no compact rows (raw only) — reads for that snapshot remain unavailable until it is later promoted, at which point promotion materializes 79 rows at 2. `load_role_scores`/`load_potential_attributes` version checks now require the snapshot's provenance version. No automatic repair or lazy recompute is added. If repository facts later show this provenance design cannot be made coherent, delivery must stop and request a developer decision rather than invent a compatibility shim.
- **No UI workflow change:** General placeholders become scores, General Search filters/sorts and Planner lanes work with the new `role.*` ids under the existing closed-catalog contracts; Moneyball views keep their cohort percentiles and derived role scores separate.
- **ADR/Architecture reconciliation (deferred):** This design changes ADR-0028's compact provenance and lifecycle from single global `SCORE_MODEL_VERSION` to per-snapshot `snapshots.compact_score_model_version` + per-row version preservation. Reconciliation must update `.wiki/ARCHITECTURE.md` and `.wiki/decisions/0028-compact-current-snapshot-metrics.md` (and related decision) to document the new migration, snapshot provenance column, ingest/promotion/boost version-preserving rules, and read validation. ADR-0027 and ADR-0028 are amended in planning while `.wiki/ARCHITECTURE.md` remains deferred until implementation makes the new behavior true.

## Uncertainty register

### Known

- JAY-31's primary/secondary lists are approved authority; no external FM verification is required for plan acceptance.
- Current catalog is 68 roles; the 11 generic OOP presentation IDs with null `attribute_role_id` are enumerated above; `channel_midfielder_ip` currently `["AMC"]` only.
- Migration v39 is the tip; `SCORE_MODEL_VERSION = 1`, `PROJECTION_MODEL_VERSION = 2`; `player_role_metrics` has 68+68+versions; Moneyball v1 has 88 entries with family counts as recorded. `snapshots` has no compact provenance column; compact is current-snapshot-only per ADR-0028.
- Frontend mirrors `src/utils/role-catalog.ts` (68, `id`/`label` only) and `src/utils/moneyball-role-catalog.ts` (88) and tactic lanes consume `all_roles()` via Rust; `src/utils/tactic-ids.ts` header comment is stale 103/111.

### Assumptions

- The 11 role display names, phases, and position families in Intent are the intended public contracts; internal `role_id` snake_case names above are the implementation identifiers that satisfy `require_safe_snake_case` and the `_ip`/`_oop` phase suffix contract.
- Adding `snapshots.compact_score_model_version INTEGER NOT NULL DEFAULT 1` in v40 and keeping existing compact rows at version 1 is the minimal truthful provenance that preserves 68 existing scores while deferring new-role materialization to Load Data; SQLite automatically stores `1` for every pre-v40 snapshot row and for any omitted insert, and rejects explicit `NULL` — no `UPDATE` backfill is needed, and `DEFAULT 1` does not prove a fresh ingest used the right value, so every new successful ingest must explicitly bind `2`; full 79 recompute for the effective current is deferred to Load Data for that snapshot (and for a previously ingested non-current provenance-2 snapshot, deferred to promotion).
- The existing `—` placeholder treatment for null role scores and null-last sort with UID tie-breaker remain the correct UX for uncomputed new OOP columns before Load Data.

### Decisions

- Implement all 11 OOP generics with the JAY-31 bands and correct `channel_midfielder_ip` to `["AMC","MC"]` in this feature.
- Map the 11 Moneyball presentation definitions to the new `attribute_role_id`s in the same feature; keep Moneyball catalog version 1 and metric weights unchanged; retain generic `wing_back_oop` `DL/DR/WBL/WBR` while presentation remains `WBL/WBR`, yielding 119/129 coverage with 10 uncovered combos enumerated above.
- Enforce strict Require Load Data via snapshot-level provenance: migration marks every pre-v40 snapshot 1 (`NOT NULL DEFAULT 1`) with no row version bump; every successful Load Data transaction marks the ingested snapshot 2 regardless of whether it becomes effective current (compact rows remain current-only); promotion rebuilds a provenance-1 current snapshot at 1 (exactly `LEGACY_V1_ROLE_IDS` 68) and promotes a provenance-2 non-current snapshot to 79 at 2, boost preserves the snapshot's availability version; any potential-role read enforces both `score_model_version` and `projection_model_version`.
- Use one PR with atomic catalog+schema+provenance+lifecycle boundary; Moneyball mapping and frontend mirrors follow in the same PR because they depend on the same model version.
- Frontend `ROLE_CATALOG` remains `id`/`label` only; Search mirror is frontend catalog/dynamic columns; Planner eligibility/options are Rust-owned; Profile mapping is backend JSON/query; counts are Rust tests plus tactic-ids comment—frontend has no 77/11 assertion.

### Unknowns

- Exact final v40 `ALTER TABLE` ordering after `sqlfmt`/`migrations.rs` conventions — verify against `migrations.rs` head and `cargo fmt` output.
- Whether any downstream consumer enumerates role ids via snapshot fixtures that pin 68 — inspect `src-tauri/tests` and frontend snapshots during implementation.

### Risks

- Catalog/schema/mirror drift (79 vs 68 or 88) producing undecodable identifiers, missing columns, or wrong family/phase validation.
- Provenance handling: if migration, ingest, promotion, or boost mis-orders the snapshot version, leaves `compact_score_model_version` null/ambiguous, or recomputes 79 roles early (including via wrong `LEGACY_V1_ROLE_IDS` membership or position-derived partition), new OOP scores materialize before Load Data or mislabel uncomputed nulls as computed. Ingest leaving a non-current snapshot null/ambiguous and later promotion choosing the wrong version are explicit failure modes.
- Potential-only read without `projection_model_version` check returning wrong-version rows.
- Moneyball mapping to the wrong generic OOP role or wrong position tags causes tactic compound-key collisions or incorrect profile scores.
- Frontend `ROLE_CATALOG` ordering or label divergence from Rust catalog causing sort/filter or score badge mismatches.
- Tests that pin 68, 11 unmapped, or 103/111 tactic coverage failing after the catalog grows.

## Walking skeleton

Commit 2 provides the atomic vertical proof for catalog/schema/provenance/lifecycle/read using the raw new `goalkeeper_oop` metric — no Moneyball presentation mapping and no frontend mirror are in Commit 2. Commit 2 adds `goalkeeper_oop` (smallest OOP definition) and the `channel_midfielder_ip` tag correction, with migration v40's 22 nullable role metric columns plus `snapshots.compact_score_model_version INTEGER NOT NULL DEFAULT 1`; pre-v40 rows obtain/read 1 through SQLite `ADD COLUMN` default semantics with no `UPDATE` backfill (row versions remain 1), and the fresh Load Data path writing `compact_score_model_version = 2` for every successfully ingested snapshot regardless of whether it becomes effective current (compact rows remain current-only). Seed one current legacy snapshot (provenance 1) and one fresh snapshot (provenance 2) with known attributes; assert legacy snapshot's compact row persists version 1 with existing scores intact and 11 new columns null uncomputed, fresh snapshot persists version 2 with deterministic new OOP score, General Search filtering/sorting by the raw `role.goalkeeper_oop` is unavailable on legacy and succeeds on fresh, and `potential_role.goalkeeper_oop` enforces both `score_model_version` and `projection_model_version`. This vertical proof lives inside the atomic Commit 2 (Commit 2's RED→GREEN) and is not a separately committable partial state. Commit 3 extends the skeleton with Moneyball presentation proof (`gk_traditional_goalkeeper_oop` → `goalkeeper_oop` yields real `PlayerRoleScore` only when the underlying compact row is version 2, otherwise placeholder null). Commit 4 extends it with frontend discovery proof (`src/utils/role-catalog.ts` mirror exposing `goalkeeper_oop` through the closed-catalog `role.*`/`potential_role.*` contract). Do not claim Moneyball or frontend presence in Commit 2.

## Delivery plan

### PR 1 — Complete missing FM26 attribute role definitions

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/missing-attribute-roles

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** check

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(scoring): complete missing FM26 attribute roles`

**Purpose:** Ship the 11 generic OOP attribute roles, channel midfielder correction, snapshot-level provenance migration/model-version, strict lifecycle (migration/ingest/promotion/boost/read), Moneyball mapping, and frontend mirrors in one independently mergeable trunk change. One PR keeps the catalog, persistence, and presentation mapping reviewable as one coherent contract.

**Depends on:** Clean `main` at planning HEAD `9db10145`; accepted plan review and Delivery fingerprint; no planned spec or earlier PR.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(scoring): record missing FM26 role plan`

**Work:** Commit the independently reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/missing-fm26-attribute-role-definitions.md` — approved feature intent, delivery plan, and packets (now including interim 104/129 tactic pin, SQLite `NOT NULL DEFAULT 1` semantics correction, and explicit 11-entry `ROLE_FAMILY_BY_ID` ownership).
- `.wiki/TODO.md` — active feature state (move this feature to Active while preserving Player gender under Next) — only if consistency requires; otherwise unchanged.
- `.wiki/decisions/0028-compact-current-snapshot-metrics.md` — amend to record per-snapshot `compact_score_model_version`, immutable `LEGACY_V1_ROLE_IDS` partition, migration/ingest/promotion/boost/read lifecycle, and strict Require Load Data rationale (deferred `ARCHITECTURE.md` reconciliation remains implementation-owned; this planning diff only amends the decision record).
- `.wiki/decisions/0027-scoped-potential-read-validation.md` — inspect actual content and explicitly reconcile its scoped-read principle vs rejected snapshot-marker alternative: status-note/amend only what is truly superseded (per-snapshot provenance now adopts a persisted marker for snapshot availability), preserving the valid scoped identifier/width principles.
- `.wiki/INDEX.md` — only if repository convention requires a decision index/ownership text change for the ADR amendment; otherwise unchanged.

**Behavior and data flow:**

- Move planning truth into one reviewed active ledger and amend the two durable decision owners so the ledger, TODO, and ADRs are coherent before implementation; record the exact delivery sequence and decision lifecycle before code.

**Ordered implementation steps:**

1. Verify the active branch is `feature/missing-attribute-roles` and base is `main` without changing Git state.
2. Confirm the worktree contains the reviewed planning paths: the active ledger, the two amended decision records, and — only if needed — `TODO.md`/`INDEX.md` consistency edits (no implementation, tests, or architecture current-state docs).
3. Run the ledger classifier and `git diff --check` for the exact planning diff (all five allowed paths).
4. Stage and inspect the exact planning diff for independent checkpoint review (decision amendments are planning documentation, not implementation).

**Tests and proof:**

- Not applicable — this commit changes planning/decision documents only. The ledger classifier and `git diff --check` prove structural consistency; the amended ADRs keep decision ownership durable before the architecture becomes true.

**Patterns to verify:**

- The active-ledger template, TODO/BACKLOG ownership rules, ADR format, and `decisions/README.md` index conventions.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, plan scope, packet order, or reviewed decisions beyond the explicitly allowed decision amendments. Do not edit `ARCHITECTURE.md` (deferred until implementation makes it true).

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation. Decision amendments are part of the planning truth and must land in this first commit before the four implementation commits.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/missing-fm26-attribute-role-definitions.md && git diff --check -- .wiki/features/active/missing-fm26-attribute-role-definitions.md .wiki/TODO.md .wiki/decisions/0028-compact-current-snapshot-metrics.md .wiki/decisions/0027-scoped-potential-read-validation.md .wiki/INDEX.md`

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files.

#### Commit 2 — Expand Rust catalog, migrate compact metrics with snapshot provenance, and enforce strict lifecycle

**Status:** Active

**Provisional commit:** `feat(scoring): add FM26 OOP roles with strict provenance`

**Work:** Add the 11 approved generic OOP `RoleDefinition`s, correct `channel_midfielder_ip`, add migration v40 with 22 nullable compact columns plus `snapshots.compact_score_model_version`, bump `SCORE_MODEL_VERSION` to 2, and enforce Require Load Data across migration, ingest, promotion, boost, and read.

**Size assessment:** ~320–380 changed non-test implementation lines (11 structs + one tag correction + 22 ALTERs + one snapshot ALTER + version bump + ingest/promotion/boost/read guards). Exceeds the 200-line soft target, but atomicity requires it: runtime catalog (79 ids) and checked-in schema/provenance must land together to remain trunk-safe—splitting leaves 79 roles with 68-column schema or vice versa, breaking `runtime_player_catalog_maps_once_to_the_checked_in_compact_schema` and every `player_metrics_join` consumer. Justification is recorded here; do not split.

**Out of scope:**

- Moneyball JSON mapping and frontend mirrors (owned by later commits).

**Implementation packet:**

- Make the new model contract and strict lifecycle readable immediately after migration so 68 existing roles remain usable before Load Data and 11 new columns stay uncomputed nulls until that snapshot's successful ingest; promotion/boost must not materialize new roles early.

**Files and responsibilities:**

- `src-tauri/src/features/scoring/catalog.rs` — append 11 `RoleDefinition`s in the existing `ROLES` array (grouped near their IP counterparts or at a clearly commented OOP generics section). Each entry uses `phase: RolePhase::OutOfPossession`, the position tags from Intent (including `wing_back_oop` as `&["DL","DR","WBL","WBR"]`), the JAY-31 primary/secondary bands mapped via `label_to_dump_key` to dump PascalCase (`AerialReach`, `CommandOfArea`, `Communication`, `Handling`, `Reflexes`, `Concentration`, `Positioning`, `Agility`, `OneOnOnes`, `RushingOut`, `Anticipation`, `Decisions`, `Heading`, `Marking`, `Tackling`, `JumpingReach`, `Strength`, `Aggression`, `Bravery`, `Composure`, `Pace`, `Acceleration`, `Teamwork`, `Positioning`, `OffTheBall`, `Stamina` as required), and `role_id` values `goalkeeper_oop`, `centre_back_oop`, `wide_centre_back_oop`, `full_back_oop`, `wing_back_oop`, `defensive_midfielder_oop`, `central_midfielder_oop`, `wide_midfielder_oop`, `attacking_midfielder_oop`, `winger_oop`, `centre_forward_oop`. Correct `channel_midfielder_ip` `position_tags` from `&["AMC"]` to `&["AMC", "MC"]`. Keep `DUMP_ATTRIBUTE_KEYS` unchanged.
- `src-tauri/src/db/migrations.rs` — add `MIGRATION_V40_SQL` / `EXPAND_COMPACT_ROLE_METRICS_FOR_GENERIC_OOP_AND_SNAPSHOT_PROVENANCE_SQL` with 22 `ALTER TABLE player_role_metrics ADD COLUMN` statements (`goalkeeper_oop`, `potential_goalkeeper_oop`, … `centre_forward_oop`, `potential_centre_forward_oop`) each `INTEGER CHECK (col IS NULL OR col BETWEEN 0 AND 100)`, plus `ALTER TABLE snapshots ADD COLUMN compact_score_model_version INTEGER NOT NULL DEFAULT 1 CHECK (compact_score_model_version IN (1,2))` — SQLite stores `1` for every pre-v40 row and for any omitted insert, and rejects explicit `NULL`; no `UPDATE` backfill is required. `DEFAULT 1` is the migration/legacy-safe fallback only — every new snapshot inserted via `insert_prepared_snapshot` explicitly binds `2`. Existing `player_role_metrics` rows keep `score_model_version = 1` with 11 new columns `NULL`. Register as migration version 40. Update `latest_version` expectations and migration inventory tests. Do not derive DDL from runtime catalogs at migration runtime.
- `src-tauri/src/features/player_metrics/compact.rs` — owns the immutable checked-in `pub const LEGACY_V1_ROLE_IDS: &[&str]` (68 ids, see Invariants) with tests `legacy_v1_partition_is_exactly_the_pre_feature_68` asserting `len==68`, unique, `require_safe_snake_case`, every id in `all_roles()`, and `LEGACY_V1 ∪ new_11 == all_roles()` with `all_roles().len()==79`. Bump `SCORE_MODEL_VERSION` from `1` to `2`; update `runtime_player_catalog_maps_once_to_the_checked_in_compact_schema` to assert 79 and schema length 162 + snapshot provenance column `NOT NULL` check, update `player_metrics_join` version predicate expectations, add `snapshot_compact_version(conn, snapshot_id)` helper and adjust `assert_snapshot_complete` / `assert_read_models_complete` to validate against `snapshots.compact_score_model_version` (including `IS NULL` check on 11 new columns for provenance 1 via set membership) and to require both `score_model_version` and `projection_model_version` for any potential-role read. `PROJECTION_MODEL_VERSION` remains 2. Add `snapshot_compact_version` unit tests and `snapshot_provenance_is_always_1_or_2` null-rejection test.
- `src-tauri/src/features/player_metrics/potential_scores.rs` — ensure `rebuild_snapshot`/`replace_player`/`reconcile_current_selection` write version-aware rows keyed by `compact::LEGACY_V1_ROLE_IDS`: ingest path writes 79+79 at version 2 for the effective current snapshot; promotion path rebuilds legacy snapshots at version 1 (compute exactly the 68 `LEGACY_V1_ROLE_IDS`, leave 11 `NULL` uncomputed) and version-2 snapshots at version 2 (full 79); `replace_player` preserves the snapshot's `compact_score_model_version` and recomputes only availability-appropriate columns (set membership), keeping one-projection-per-player semantics. Length mismatch or unknown id fails.
- `src-tauri/src/features/snapshot/ingest.rs` — in `publish_prepared_snapshot_canonical`, `insert_prepared_snapshot` explicitly binds `compact_score_model_version = 2` for the ingested row (so every successfully published fresh ingest is `2` even when non-current — `DEFAULT 1` would store `1` if omitted, explicit `NULL` is rejected; tests prove this), then select the effective current snapshot in game-date order, clear displaced derived rows, and persist `prepare_player_derived` (now 79-role) + `player_compact::persist_rows_borrowed` at `SCORE_MODEL_VERSION = 2` only for the effective current snapshot. If ingested snapshot is not current (older `game_date`), it retains provenance `2` with raw rows only and no compact rows; the still-current snapshot keeps its own provenance and rows. Guarantee: no successfully published snapshot remains with null/ambiguous provenance after commit (the `DEFAULT` is never relied on for fresh ingest).
- `src-tauri/src/features/snapshot/service.rs` — `reconcile_current_selection` version-aware rebuild; `delete_snapshot` promotion rebuild respects provenance (legacy 1 vs 2).
- `src-tauri/src/features/player/service.rs` — boost `replace_player` path preserves snapshot provenance version; add version lookup before replacement.
- `src-tauri/src/features/search/query.rs` / `src-tauri/src/features/search/filter.rs` / `src-tauri/src/features/player/query.rs` — no identifier logic change beyond closed-catalog validation, but reads must enforce snapshot provenance: `assert_read_models_complete` is called with snapshot version; any `potential_role.*` field requires both score and projection versions; `player_metrics_join` predicates match the snapshot's `compact_score_model_version`.
- `src-tauri/src/features/moneyball/role_catalog.rs` — interim tactic-coverage update: `tactic_compound_key_is_unique_and_covers_103_of_111_with_8_uncovered` → `tactic_compound_key_is_unique_and_covers_104_of_129_with_25_uncovered` with `total = 129` (68-role 111 + 17 new generic OOP combos + 1 `channel_midfielder_ip+MC`), `mapped = 104` (prior 103 + `channel_midfielder_ip+MC`), `uncovered = 25` = prior 8 + all 17 new: `("goalkeeper_oop","GK"), ("centre_back_oop","DC"), ("wide_centre_back_oop","DC"), ("full_back_oop","DL"), ("full_back_oop","DR"), ("wing_back_oop","DL"), ("wing_back_oop","DR"), ("wing_back_oop","WBL"), ("wing_back_oop","WBR"), ("defensive_midfielder_oop","DM"), ("central_midfielder_oop","MC"), ("wide_midfielder_oop","ML"), ("wide_midfielder_oop","MR"), ("attacking_midfielder_oop","AMC"), ("winger_oop","AML"), ("winger_oop","AMR"), ("centre_forward_oop","ST")` plus the prior 8 `("holding_wing_back_oop","DL"), ("holding_wing_back_oop","DR"), ("pressing_wing_back_oop","DL"), ("pressing_wing_back_oop","DR"), ("box_to_box_midfielder_ip","MC"), ("box_to_box_playmaker_ip","MC"), ("deep_lying_playmaker_ip","MC"), ("second_striker_ip","ST")`. Required so `catalog.rs` expansion leaves `./scripts/dev check` green before mapping.
- `src/utils/tactic-ids.ts` — interim header comment `104/129` with the 25 `NULL -> "—"` combos enumerated as above (derived: 8 prior + 17 new). No runtime logic change.

**Behavior and data flow:**

- First launch after update runs migrations under the existing `apply` transaction model. v40 adds 22 nullable `player_role_metrics` columns and `snapshots.compact_score_model_version INTEGER NOT NULL DEFAULT 1` — SQLite stores `1` for every pre-v40 row and for any omitted insert, rejects explicit `NULL`; no `UPDATE` backfill is performed. Existing current rows retain 68 scores and gain 11 null current + 11 null potential columns with `snapshots.compact_score_model_version = 1` (`NOT NULL`) and `player_role_metrics.score_model_version = 1`. No row is left null. General (role, position) coverage becomes `total 129 = 111 + 17 new OOP combos + 1 channel MC`, `mapped 104 = 103 + channel MC`, `uncovered 25` as enumerated above; `assert_snapshot_complete` now requires the snapshot's provenance version via `LEGACY_V1_ROLE_IDS` (provenance 1 requires `IS NULL` on the 11 new columns; provenance 2 requires all 79 computed). Before the next Load Data for that snapshot, General reads for the 68 existing roles succeed; reads that request a raw new OOP role (`goalkeeper_oop`) return unavailable (null uncomputed vs computed) via closed-catalog `role.*` resolver. Promotion of a legacy snapshot rebuilds at version 1 (exactly 68 via `LEGACY_V1_ROLE_IDS`), boost preserves version 1, ingest explicitly binds `compact_score_model_version = 2` for every ingested snapshot (compact rows only on effective current, `DEFAULT 1` never relied on), and later promotion of a non-current provenance-2 snapshot materializes 79 at 2.

**Ordered implementation steps:**

1. Add RED catalog tests that expect 79 roles and the 11 new `role_id`s and `channel_midfielder_ip` containing both `AMC` and `MC`; verify fail on 68 baseline.
2. Insert the 11 `RoleDefinition`s with exact JAY-31 bands and the corrected channel midfielder tags.
3. Add RED partition tests for `compact::LEGACY_V1_ROLE_IDS` (expect 68, unique, every id in `all_roles()`, union with new 11 is 79); verify they fail before the partition exists and that a position-derived or truncated inventory would fail.
4. Add RED schema tests that open a v39 `app-v2.db` by applying registry entries 1–39 manually in a test helper (iterating `migrations::all()` filtered to `version <= 39` and executing each `sql` with `PRAGMA user_version` bump, as in `migrates_snapshot_schema_from_every_prior_version`) or via a test-only `apply_through_version(conn, 39)` helper encapsulating the same registry filtering, then seed snapshots+compact rows at version 1, then apply remaining migrations (v40) via `migrations::apply` (or iterating `version > 39`), and assert 162-column `player_role_metrics`, `snapshots.compact_score_model_version INTEGER NOT NULL DEFAULT 1 CHECK IN (1,2)` with `NOT NULL` and SQLite semantics (existing rows read `1`, omitted insert stores `1`, explicit `NULL` rejected), no null provenance after migration, existing rows version 1 with null new columns, preserved old scores, and snapshot provenance `1` for every pre-v40 snapshot (achieved by `DEFAULT 1`, not an `UPDATE` backfill). Remove all `migrations::apply` to v39 wording — `apply` would apply v40 once registered and cannot stop at v39.
5. Add RED lifecycle tests: legacy snapshot promotion retains version 1 with 11 null uncomputed (verified via `LEGACY_V1_ROLE_IDS` set membership, not array position) and does not materialize; boost on legacy preserves version 1; ingest of a snapshot that becomes current explicitly binds provenance `2` and writes all 79 scores on the effective current (prove every successfully published fresh ingest — current or older non-current — is `2`); ingest of an older-than-current snapshot also explicitly binds `2` with raw rows only and keeps the effective current's provenance/rows unchanged; promotion of that previously ingested non-current provenance-2 snapshot then materializes 79 at version 2; potential-only Search (`potential_role.goalkeeper_oop`) with wrong `score_model_version` fails as incomplete; omitted `compact_score_model_version` on insert stores `1` (not rejected) while explicit `NULL` is rejected — fresh ingest must explicitly bind `2` and tests prove it.
6. Implement v40 SQL and register migration; bump `SCORE_MODEL_VERSION` to 2; add `LEGACY_V1_ROLE_IDS` and snapshot provenance helpers (`snapshot_compact_version`, `rebuild_snapshot_versioned`).
7. Adjust ingest to set `compact_score_model_version = 2` for every ingested snapshot in the same transaction (compact rows only for effective current); adjust promotion/boost to key by `LEGACY_V1_ROLE_IDS`; update `assert_read_models_complete` to enforce both versions for potential reads and provenance-1 `IS NULL` on new columns.
8. Run migration + compact + snapshot lifecycle + search + player query tests to GREEN (legacy promotion must compute exactly 68 via `LEGACY_V1_ROLE_IDS` and prove 79 via `all_roles()`; position-derived inference would fail the partition test).
9. Run `./scripts/dev check` (full gate) — no expected gaps.

**Tests and proof:**

- Modify `catalog.rs` tests (RED must fail for 68; GREEN proves 79 unique ids, non-empty primary from `DUMP_ATTRIBUTE_KEYS`, disjoint bands, valid `_oop`/`_ip` suffix, correct `position_tags` including `wing_back_oop` 4) and add a table-driven exact JAY-31 contract test `new_oop_roles_match_jay31_exact_bands` that pins for each of the 11 new roles the exact tuple `(role_id, display_name, phase=OutOfPossession, position_tags, primary, secondary)` in JAY-31 order transcribed to dump PascalCase, plus a pin that `channel_midfielder_ip` changes only `position_tags` to `["AMC", "MC"]` while `display_name`, `phase`, `primary`, and `secondary` remain the pre-feature values; any band-membership, ordering, casing, or transcription drift fails (protects more than generic validity/band validity).
- Add `player_metrics/compact.rs` partition tests (RED must fail before partition; GREEN proves `LEGACY_V1_ROLE_IDS.len()==68`, unique, `require_safe_snake_case`, every entry in `all_roles()`, `all_roles().len()==79`, union with the 11 new ids is exactly `all_roles()`, writer alignment via `all_roles()` order with set membership leaves 11 `NULL`, and wrong/position-derived partition fails).
- Modify `db/migrations.rs` tests: RED shows v40 columns absent and no snapshot provenance column; GREEN proves 79+79 inventory, snapshot column `INTEGER NOT NULL DEFAULT 1 CHECK IN (1,2)` with `NOT NULL` and SQLite semantics (existing rows `1`, omitted insert `1`, explicit `NULL` rejected, no `UPDATE` backfill), provenance `1` for every pre-v40 snapshot via `DEFAULT 1`, 11 new columns `NULL` on legacy rows, `score_model_version` remains 1, and that fresh ingest explicitly binds `2` (prove every successfully published fresh ingest — including older non-current — is `2`; `DEFAULT 1` is only the migration/legacy-safe fallback, omission would store `1` not fail, explicit `NULL` is rejected).
- Modify `player_metrics/compact.rs` tests: assert 79, 162-length schema plus snapshot column `NOT NULL`, version predicates with provenance via `LEGACY_V1_ROLE_IDS` (`IS NULL` on 11 new columns for provenance 1), potential-only wrong-version failure (`potential_role.goalkeeper_oop` or `potential_role.central_midfielder_oop` with `score_model_version = 1` or `projection_model_version = 1` fails), both-versions enforcement for `potential_role.*` and `tactic_potential.*`, and that ingested snapshot rows cannot remain null/ambiguous.
- Add/adjust `potential_scores.rs` / `snapshot/service.rs` / `snapshot/ingest.rs` / `player/service.rs` tests: legacy promotion recomputes exactly the 68 `LEGACY_V1_ROLE_IDS` and proves 11 new columns remain `NULL` uncomputed (not position-derived); legacy boost preserves version 1 and does not materialize; fresh ingest that becomes current materializes 79 at version 2 and sets ingested snapshot provenance 2; older-than-current ingest marks ingested snapshot provenance 2 with no compact rows while effective current keeps its own provenance/rows; later promotion of that non-current provenance-2 snapshot materializes 79 at 2; new snapshot rows cannot remain with null provenance after successful publication.
- No "expected gap" — `./scripts/dev check` must pass at this checkpoint.

**Patterns to verify:**

- Existing `COMPACT_ROLE_METRICS_V38_SQL` immutable DDL pattern, v33→v34 migration test shape, `compact::test_support::read_row`/`count_rows`, version-gated `player_metrics_join`, `assert_read_models_complete` current vs potential flags.

**Constraints and non-goals:**

- Do not recompute the 11 new scores for existing snapshots in the migration; do not open or change `app.db`; do not change 75/25 formula, null semantics, or projection; do not add per-role indexes.

**Dependencies and sequencing:**

- Depends on Commit 1. Must land before Commits 3–4 because Moneyball mapping and frontend reads assume 79-role catalog + version 2 columns + provenance.

**Validation:** `./scripts/dev check`

**Stop conditions:** Stop if any new column name fails `require_safe_snake_case`, if SQLite rejects 162-column width or snapshot ALTER, if version preservation cannot be done atomically, or if a future catalog change would require editing v40 DDL instead of adding v41.

**Review mandate:** Verify v40 is additive and idempotent, existing rows' 68 scores byte-preserved with new columns `NULL` uncomputed, every snapshot provenance is `1` (and `NOT NULL`, no null/ambiguous rows) after migration, no in-place `score_model_version = 2` update, `LEGACY_V1_ROLE_IDS` is exactly the pre-feature 68 (prove via explicit inventory test, not array position), writers align columns/values by `all_roles()` order with set membership on `LEGACY_V1_ROLE_IDS` (11 new columns `NULL` for version 1), version-preserving promotion/boost keyed by `LEGACY_V1_ROLE_IDS`, `SCORE_MODEL_VERSION = 2`, potential reads enforce both versions, ingest sets `compact_score_model_version = 2` for every successfully ingested snapshot in the same transaction (compact rows current-only) and no published row remains null, older-than-current ingest then later promotion test passes, walking skeleton vertical proof passes via raw `role.goalkeeper_oop`/`potential_role.goalkeeper_oop` (legacy unavailable, fresh real; no Moneyball/frontend claims in Commit 2), and `./scripts/dev check` green.

#### Commit 3 — Map Moneyball presentation roles to the new attribute roles and pin tactic coverage

**Status:** Pending

**Provisional commit:** `feat(moneyball): map generic OOP presentation roles`

**Work:** Wire the 11 generic OOP Moneyball presentation definitions to the new attribute `role_id`s and pin deterministic tactic coverage to 119/129 with ten uncovered combos.

**Size assessment:** ~60 changed non-test implementation lines (JSON + validation + tactic header comment) plus tests. Within soft target.

**Out of scope:**

- Frontend `ROLE_CATALOG` mirrors (Commit 4) and compact schema.

**Implementation packet:**

- Close the 11 placeholder nulls so `features/moneyball` and `features/player` presentation mapping returns real General scores after materialized data exists; keep presentation `WBL/WBR` while generic retains `DL/DR/WBL/WBR`.

**Files and responsibilities:**

- `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` — set `attribute_role_id` for the 11 ids from Intent to the new attribute ids (`amc_attacking_midfielder_oop` → `attacking_midfielder_oop`, etc., including `wbl_wbr_wing_back_oop` → `wing_back_oop`); keep `version: 1`, family counts, metric keys, weights, inverted flags unchanged.
- `src-tauri/src/features/moneyball/role_catalog.rs` — update `maps_only_known_attribute_roles_and_preserves_unmapped_generic_roles` expectation from 77 mapped / 11 unmapped to 88 mapped / 0 unmapped; update `tactic_compound_key_is_unique_and_covers_104_of_129_with_25_uncovered` (Commit 2 interim) to `covers_119_of_129_with_10_uncovered`, assert total 129, mapped 119, uncovered 10 with exact expected set: `("holding_wing_back_oop","DL")`, `("holding_wing_back_oop","DR")`, `("pressing_wing_back_oop","DL")`, `("pressing_wing_back_oop","DR")`, `("box_to_box_midfielder_ip","MC")`, `("box_to_box_playmaker_ip","MC")`, `("deep_lying_playmaker_ip","MC")`, `("second_striker_ip","ST")`, `("wing_back_oop","DL")`, `("wing_back_oop","DR")` — this is Commit 2's `104/129` plus exactly the 15 combos contributed by the 11 filled `attribute_role_id`s (all 17 new generic combos except the two `wing_back_oop+DL/DR` that remain uncovered because presentation `wbl_wbr_wing_back_oop` covers only `WBL/WBR`).
- `src-tauri/src/features/player/query.rs` (`load_role_scores`) — no code change beyond the catalog it already joins; the mapped checks `scores_by_role.get(attribute_role_id)` now find values for the 11 former placeholders when compact row provenance is 2 and new columns are computed after Load Data; before Load Data returns unavailable null.
- `src/utils/tactic-ids.ts` — update header comment from interim 104/129 (25 uncovered) to 119/129 (10 uncovered) and list the ten `NULL -> "—"` combos as above (RED began from 104/129); no runtime logic change.

**Behavior and data flow:**

- `builtin_catalog()` loads 88 definitions, validates each `attribute_role_id` against the 79-role `all_roles()`, and exposes the presentation-to-attribute mapping. `get_player` now returns `PlayerRoleScore` for those 11 presentation IDs when the surrounding `player_role_metrics` row has version matching snapshot provenance 2 and new columns are computed after Load Data; before Load Data returns `None` → UI `—` (uncomputed).

**Ordered implementation steps:**

1. Add RED Moneyball catalog test that expects 88 mapped and 119/129 coverage; verify fail on Commit 2 interim baseline (77 mapped with 11 null, 104/129 with 25 uncovered — prior 8 + all 17 new generic combos).
2. Fill the 11 `attribute_role_id` values in `builtin_role_definitions_v1.json` (the 15 tactic combinations derived from those 11 ids are enumerated in Files above; they lift `mapped` from 104→119 and reduce `uncovered` 25→10 by covering exactly those 15 new generic combos, leaving `wing_back_oop+DL/DR` + prior 8 as the final 10).
3. Update `role_catalog.rs` tests and `tactic-ids.ts` comment from interim 104/129 (25 uncovered) to 119/129 (10 uncovered) with exact expected set.
4. Run Moneyball catalog + `player/query.rs` `presents_duplicate_moneyball_roles_from_one_attribute_score_and_keeps_unmapped_null` analogue to GREEN, updated for 0 unmapped and provenance-aware reads; verify tactic compound-key uniqueness and that RED began from interim 104/129.
5. Run `./scripts/dev check`.

**Tests and proof:**

- Modify `moneyball/role_catalog.rs` tests: RED must fail for 11 nulls and interim 104/129 (proving Commit 2's 25-uncovered assignment); GREEN proves all 88 `attribute_role_id`s non-null and in `all_roles()`, unmapped 0, duplicate `(attribute_role_id, position_tag)` unique, family counts unchanged, total 129, mapped 119, uncovered exactly ten as listed (the 15 mapped combos are exactly the new generic combos minus `wing_back_oop+DL/DR`). Update `player/query.rs` tests that asserted unmapped nulls for those ids to assert mapped scores after a version-2 compact row with fixture scores and unavailable null on version-1 legacy snapshot. Verify `src/utils/tactic-ids.ts` comment matches Rust counts.

**Patterns to verify:**

- Existing `validate_builtin_catalog` shape, `expected_position_tags`/`expected_position_prefix`, and the General `get_player → PlayerRoleScore` mapping pattern.

**Constraints and non-goals:**

- Do not change Moneyball metric keys, weights, directions, or catalog version; do not merge wide-position definitions; do not change `wing_back_oop` position tags beyond `DL/DR/WBL/WBR`.

**Dependencies and sequencing:**

- Depends on Commit 2 (79-role catalog + version 2 columns + provenance must exist to validate new attribute ids).

**Validation:** `./scripts/dev check`

**Stop conditions:** Stop if any `attribute_role_id` not in 79-role closed catalog or if Moneyball validation expects version bump — approved constraint keeps `BUILTIN_ROLE_CATALOG_VERSION = 1`.

**Review mandate:** Verify the 11 mappings exactly as Intent, no metric/weight change, family counts/phase/id invariants preserved, tactic coverage pinned 119/129 with exact ten, `wing_back_oop` tags `DL/DR/WBL/WBR` while presentation `WBL/WBR`, and `./scripts/dev check` green.

#### Commit 4 — Mirror expanded attribute catalog in frontend (display only)

**Status:** Pending

**Provisional commit:** `feat(frontend): mirror expanded attribute catalog`

**Work:** Make the frontend General Search/Planner catalogs reflect the 79-role attribute catalog so the closed-catalog contracts stay coherent (display only).

**Size assessment:** ~80 changed non-test implementation lines (11 entries + label updates). Within soft target.

**Out of scope:**

- Moneyball scoring logic, Rust scoring formula, and visual design.

**Implementation packet:**

- Keep the frontend `ROLE_CATALOG` and derived search metrics in lockstep with the Rust closed catalog so filter/sort/table layout and Planner tactic lanes discover the new OOP roles; Planner eligibility/options remain Rust-owned.

**Files and responsibilities:**

- `src/utils/role-catalog.ts` — expand `ROLE_CATALOG` from 68 to 79 entries: add `goalkeeper_oop`, `centre_back_oop`, `wide_centre_back_oop`, `full_back_oop`, `wing_back_oop`, `defensive_midfielder_oop`, `central_midfielder_oop`, `wide_midfielder_oop`, `attacking_midfielder_oop`, `winger_oop`, `centre_forward_oop` with labels `Goalkeeper (OOP)`, etc., ordered to mirror `catalog.rs`; each entry has only `id`/`label` (no `phase`/`positionTags`). Update any `ROLE_CATALOG.length` snapshot that pins 68.
- `src/utils/moneyball-role-catalog.ts` — retain 88 entries; no id/phase change. If a derived label helper formats phase tags, ensure `OOP` labeling stays `OOP`.
- `src/utils/player-metrics.ts` — **required logic change**: extend `ROLE_FAMILY_BY_ID: Record<RoleId, PlayerMetricRoleFamily>` with all 11 new entries using established `PlayerMetricRoleFamily` categories so the `satisfies Record<RoleId, ...>` contract stays total and `tsc -b` passes after `ROLE_CATALOG` expands. Exact mapping (commit-owned): `goalkeeper_oop` → `Goalkeepers`, `centre_back_oop` → `Central defense`, `wide_centre_back_oop` → `Central defense`, `full_back_oop` → `Full-back and wing-back`, `wing_back_oop` → `Full-back and wing-back`, `defensive_midfielder_oop` → `Defensive midfield`, `central_midfielder_oop` → `Central midfield`, `wide_midfielder_oop` → `Wide midfield and wings`, `winger_oop` → `Forwards`, `attacking_midfielder_oop` → `Attacking midfield`, `centre_forward_oop` → `Forwards` — `winger_oop` maps to `Forwards` matching `inside_outlet_winger_oop`, `tracking_winger_oop`, and `wide_outlet_winger_oop` in `src/utils/player-metrics.ts`; it must not map to `Wide midfield and wings` (where `wide_midfielder_oop` remains). This file needs a logic change; remove any claim it does not.
- `src/utils/moneyball-search-metrics.ts`, `src/features/search/*`, `src/features/planner/*` — no identifier logic change beyond the catalog const; verify they resolve `role.<id>` through `ROLE_CATALOG` and need no hard-coded allowlist beyond the catalog const; dynamic columns are the Search mirror.
- Tests: `src/utils/role-catalog.test.ts` or equivalent, `src/utils/moneyball-role-catalog.test.ts`, `src/features/search/utils/dynamic-columns.test.ts`, `src/utils/moneyball-search-metrics.test.ts` — update length expectations from 68 to 79 where applicable; frontend has no 77/11 mapping assertion. Add RED for `player-metrics.ts` family completeness (missing 11 entries fails `satifies`/`tsc`), then GREEN after adding them.

**Behavior and data flow:**

- General Search metric picker, filter strip, table columns, and sort keys expose the 11 new `role.goalkeeper_oop` … `role.centre_forward_oop` (and potential counterparts) through the same closed-catalog resolver. Planner tactic lanes and depth/optimizer candidate queries discover the new OOP roles via Rust `all_roles()` and compact columns. Before Load Data the new metrics render `—` (uncomputed); after Load Data they render `ScoreBadge` values and participate in filtering/sorting.

**Ordered implementation steps:**

1. Add RED frontend tests: `ROLE_CATALOG` expects 79 entries with 11 new `id`s (fail on 68) and `ROLE_FAMILY_BY_ID` compile fails when 11 entries are missing (`tsc -b` / `satisfies Record<RoleId, PlayerMetricRoleFamily>` error).
2. Expand `ROLE_CATALOG` and add all 11 `ROLE_FAMILY_BY_ID` entries with exact categories above; update frontend test expectations for 79; verify moneyball catalog remains 88. Verify `tsc -b` passes only after the 11 entries are present (proving the family owner is Commit 4).
3. Run frontend tests (`./scripts/dev test`) and the app gate (`./scripts/dev check-app`) to GREEN (both must pass with the new 79-length catalog and complete family map).
4. Run `./scripts/dev check` (full gate including TypeScript) — no expected gaps.

**Tests and proof:**

- Modify `src/utils/role-catalog*` and `moneyball-role-catalog*` tests: RED must fail for missing ids/length; GREEN proves 79 `ROLE_CATALOG` entries with unique `id`s and labels matching Rust `display_name` + `(OOP)`, `MONEYBALL_ROLE_CATALOG` remains 88 (Rust owns counts). Update `dynamic-columns` tests to resolve new `role.*` ids; verify `tactic-ids.ts` comment 119/129 is the single source for coverage (frontend has no mapping count assertion). For `src/utils/player-metrics.ts`: RED shows `tsc -b` fails with `Property 'goalkeeper_oop' is missing...` and family tests fail; GREEN proves all 11 `ROLE_FAMILY_BY_ID` entries exist with exact categories above, `satisfies Record<RoleId, PlayerMetricRoleFamily>` holds, and `./scripts/dev check-app` / full `./scripts/dev check` pass, removing the false claim that this file needs no logic change.

**Patterns to verify:**

- Existing `ROLE_CATALOG` const shape (`id`/`label` only), `moneyball-role-catalog.ts` `role()` helper and `orderedPositions`, and the search/planner dynamic-column/metric-picker contracts.

**Constraints and non-goals:**

- Do not change Moneyball metric catalogs, design tokens, or visual layout. Keep labels consistent with `catalog.rs` display names. Do not add frontend `phase`/`positionTags`.

**Dependencies and sequencing:**

- Depends on Commit 2 (canonical Rust ids must be frozen before mirroring). Can land after Commit 3 or with it, but must follow the Rust catalog expansion.

**Validation:** `./scripts/dev check`

**Stop conditions:** Stop if frontend catalog order diverges from Rust in a way that changes filtered cohort/sort semantics, or if a test pins snapshot output that must be renewed — request developer approval for snapshot updates.

**Review mandate:** Verify no stale 68-length assertion remains, 11 new frontend entries match Rust ids/labels exactly with `id`/`label` only, no Moneyball presentation count change, Search/Planner dynamic columns correctly resolve new `role.*` ids through existing closed-catalog path, and `./scripts/dev check` green.

## Active work

**PR:** PR 1 — Complete missing FM26 attribute role definitions

**Commit:** Expand Rust catalog, migrate compact metrics with snapshot provenance, and enforce strict lifecycle

### RED or removal proof

Add focused failing catalog, partition, migration, lifecycle, and read-model tests for the 79-role contract. The proof must fail on the 68-role baseline and detect early materialization, wrong snapshot provenance, and incomplete potential-version validation.

### Expected outcome

Migration v40, the Rust catalog, compact persistence, ingest, promotion, boost, and reads implement strict per-snapshot score-model provenance. Legacy snapshots preserve their 68 valid scores and leave the 11 new roles uncomputed until fresh Load Data.

### Explicit exclusions

Moneyball presentation mappings, frontend catalog mirrors, scoring formula changes, projection changes, historical derived rows, new indexes, and unrelated cleanup.

## Discoveries and replanning

- Second independent review (planning-only correction): pinned trunk-safe interim tactic coverage `104/129` with 25 uncovered (prior 8 + all 17 new generic OOP combos) in Commit 2 and `119/129` with 10 uncovered after Commit 3 mapping 15 of those (leaving `wing_back_oop+DL/DR` + prior 8); corrected SQLite `ALTER ... INTEGER NOT NULL DEFAULT 1` semantics (existing rows store/read `1`, omitted inserts store `1`, explicit `NULL` rejected, no `UPDATE` backfill, explicit `bind 2` for every fresh ingest) and removed false omission-fails claims; assigned all 11 `ROLE_FAMILY_BY_ID` entries to Commit 4 with RED/GREEN `tsc` proof; amended ADR-0028/0027 for durable per-snapshot provenance, `LEGACY_V1_ROLE_IDS`, lifecycle, and scoped-read reconciliation (no `ARCHITECTURE.md` edit). Channel midfielder position-tag, immutable `LEGACY_V1_ROLE_IDS` partition, and snapshot-level provenance (`snapshots.compact_score_model_version INTEGER NOT NULL DEFAULT 1`; every pre-v40 row stores `1` via `DEFAULT`, every successful Load Data explicitly binds `2` regardless of effective-current selection with compact rows current-only) remain as above; any later evidence that strict Require Load Data cannot be made coherent requires a developer decision and a new fingerprint.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Complete missing FM26 attribute role definitions | Commit 1 — Record the approved feature plan | Pending record | Recorded the accepted schema-2 ledger, TODO activation, and ADR-0027/0028 amendments. | `ledger_state.py`; `git diff --cached --check` — passed. | Not applicable | Clear | 0 | None |

## Final validation

- `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/missing-fm26-attribute-role-definitions.md`
- `git diff --check -- .wiki/features/active/missing-fm26-attribute-role-definitions.md .wiki/TODO.md .wiki/decisions/0028-compact-current-snapshot-metrics.md .wiki/decisions/0027-scoped-potential-read-validation.md .wiki/INDEX.md`
- `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/missing-fm26-attribute-role-definitions.md /home/jonas/projects/fm-valuescout` to confirm the recorded Delivery fingerprint, then rerun `ledger_state.py`.
- Feature-level gates recorded per commit (`./scripts/dev check` at every implementation checkpoint, covering `check-rust`, `check-app`, `test`) must pass on the final implementation range; Commit 2 must pass with interim `104/129` (25 uncovered) and SQLite `NOT NULL DEFAULT 1` semantics (existing `1`, omitted `1`, explicit `NULL` rejected, explicit `bind 2` for fresh ingest); Commit 3 RED must start from that interim; Commit 4 must pass TypeScript with all 11 `ROLE_FAMILY_BY_ID` entries; no "expected gap" is allowed.

## Documentation impact

Planning corrects durable decision ownership before implementation: this feature amends `.wiki/decisions/0028-compact-current-snapshot-metrics.md` (per-snapshot `compact_score_model_version INTEGER NOT NULL DEFAULT 1` with SQLite semantics, immutable `LEGACY_V1_ROLE_IDS` 68-id partition owned by `player_metrics/compact.rs`, migration without `UPDATE` backfill, ingest explicitly binding `2` for every fresh snapshot including older non-current, promotion/boost/read version preservation, and strict Require Load Data rationale) and reconciles `.wiki/decisions/0027-scoped-potential-read-validation.md` (preserves scoped identifier/width principles, notes that the rejected snapshot-marker alternative is now adopted as the per-snapshot provenance marker only for snapshot availability). `.wiki/INDEX.md` is updated only if its decision-index convention requires it. After implementation makes the new architecture true, reconciliation must update `.wiki/ARCHITECTURE.md` to document the new column, partition owner, and lifecycle (implementation-owned); no `ARCHITECTURE.md`, `CONCEPT.md`, or `DESIGN.md` change is authored by this planning commit.
