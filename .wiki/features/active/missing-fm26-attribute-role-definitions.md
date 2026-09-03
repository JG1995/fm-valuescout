# Complete missing FM26 attribute role definitions

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 814e5b886e462518beeb69bd05dcbe5475874d90a8c8b9ace64d4444c2d99fa8

## Intent

Complete the FM26 In Possession / Out of Possession attribute role catalog so that every Moneyball presentation role maps to a real attribute role. The Rust scoring engine currently ships 68 roles; 11 generic out-of-possession (OOP) presentation rows have `attribute_role_id: null` and render as General Profile placeholders (`PlayerRoleScore` with `null` scores) while being absent from General Search/Planner catalogs — they are General Profile placeholders and absent from General Search/Planner catalogs, not placeholders across all surfaces. This feature adds those 11 missing generic OOP definitions with the FM-approved primary/secondary attribute bands, applies the recorded Channel Midfielder eligibility correction (`channel_midfielder_ip` gains `MC`), and maps the existing 88-entry Moneyball presentation definitions to the expanded attribute catalog. After a normal winning Load Data materializes the new compact scores at the new model version the new roles score like every other role; the migration itself preserves raw snapshot data and does not backfill scores.

## User-visible behavior

- **General attribute roles:** 11 new OOP roles exist in the closed scoring catalog and in General search/profile/planner surfaces:
  - Goalkeeper (OOP) — `GK` — Primary: Aerial Reach, Command of Area, Communication, Handling, Reflexes, Concentration, Positioning, Agility. Secondary: One On Ones, Rushing Out, Anticipation, Decisions.
  - Centre-Back (OOP) — `DC` — Primary: Heading, Marking, Tackling, Anticipation, Positioning, Jumping Reach, Strength. Secondary: Aggression, Bravery, Composure, Concentration, Decisions, Pace.
  - Wide Centre-Back (OOP) — `DC` — Primary: Heading, Marking, Tackling, Anticipation, Positioning, Jumping Reach, Strength. Secondary: Aggression, Bravery, Concentration, Decisions, Acceleration, Agility, Pace.
  - Full-Back (OOP) — `DL`, `DR` — Primary: Marking, Tackling, Anticipation, Positioning, Teamwork, Acceleration. Secondary: Aggression, Concentration, Decisions, Work Rate, Agility, Pace, Stamina.
  - Wing-Back (OOP) — `DL`, `DR`, `WBL`, `WBR` — Primary: Marking, Tackling, Anticipation, Positioning, Teamwork, Acceleration, Stamina. Secondary: Aggression, Concentration, Decisions, Work Rate, Agility, Pace.
  - Defensive Midfielder (OOP) — `DM` — Primary: Tackling, Anticipation, Decisions, Positioning, Teamwork, Work Rate. Secondary: Marking, Aggression, Concentration, Stamina, Strength.
  - Central Midfielder (OOP) — `MC` — Primary: Tackling, Decisions, Teamwork, Work Rate. Secondary: Marking, Aggression, Anticipation, Concentration, Positioning, Stamina.
  - Wide Midfielder (OOP) — `ML`, `MR` — Primary: Decisions, Teamwork, Work Rate, Acceleration. Secondary: Marking, Aggression, Anticipation, Off The Ball, Agility, Pace, Stamina.
  - Attacking Midfielder (OOP) — `AMC` — Primary: Anticipation, Decisions, Work Rate. Secondary: Marking, Aggression, Off The Ball, Teamwork, Stamina.
  - Winger (OOP) — `AML`, `AMR` — Primary: Anticipation, Decisions, Teamwork, Work Rate, Acceleration. Secondary: Marking, Aggression, Off The Ball, Agility, Pace, Stamina.
  - Centre Forward (OOP) — `ST` — Primary: Anticipation, Decisions, Work Rate. Secondary: Marking, Aggression, Off The Ball, Teamwork, Stamina.
- **Channel Midfielder correction:** `channel_midfielder_ip` supports both `AMC` and `MC` (`position_tags: ["AMC", "MC"]`) with the same definition; existing consumers that filter by position tags treat it as eligible from either base position.
- **Moneyball presentation mapping:** The 88-entry built-in Moneyball catalog keeps its metrics/version (v1) and family counts, but the 11 previously null `attribute_role_id` entries now point to the new attribute roles:
  `amc_attacking_midfielder_oop` → `attacking_midfielder_oop`, `dc_centre_back_oop` → `centre_back_oop`, `dc_wide_centre_back_oop` → `wide_centre_back_oop`, `mc_central_midfielder_oop` → `central_midfielder_oop`, `dm_defensive_midfielder_oop` → `defensive_midfielder_oop`, `dl_dr_full_back_oop` → `full_back_oop`, `gk_traditional_goalkeeper_oop` → `goalkeeper_oop`, `st_centre_forward_oop` → `centre_forward_oop`, `ml_mr_wide_midfielder_oop` → `wide_midfielder_oop`, `wbl_wbr_wing_back_oop` → `wing_back_oop`, `aml_amr_winger_oop` → `winger_oop`. All 88 presentation rows now have a non-null `attribute_role_id` and their `PlayerRoleScore` placeholders become real Current/Potential scores after supported data is materialized at the new model version. The generic `wing_back_oop` retains `DL/DR/WBL/WBR` tags while presentation `wbl_wbr_wing_back_oop` remains `WBL/WBR`.
- **General profile/Search/Planner exposure:** Existing General profile placeholders for those 11 presentation IDs become numeric scores after a normal Load Data materializes the new compact scores at model version 2. General Search can display, filter, and sort by the new `role.*` and `potential_role.*` metrics, and the Planner/tactic optimizer can consume the new OOP roles once the save's effective current snapshot has been materialized at model version 2. No new visual design or user workflow is added.

## Invariants

- The closed player attribute catalog grows from 68 to 79 roles (68 current + 68 potential → 79 + 79 compact columns). Catalog IDs remain safe snake_case (`^[a-z][a-z0-9_]*$`) and are the sole source of SQL identifiers via `player_metrics::compact` validation; WebView input never becomes an identifier. `channel_midfielder_ip` position_tags become `["AMC","MC"]`.
- Scoring keeps the existing 75/25 primary/secondary band formula, equal weight within each band, `/20×100` scaling, rounding, and null semantics (any required attribute null or missing → null score). Projection keeps the existing CA-to-PA visible-attribute projection, age-29 identity boundary, and Rust ownership.
- `player_role_metrics` remains current-only, one row per player in the effective current snapshot, with nullable 0–100 columns. Missing source attributes produce null, never zero. Historical snapshots keep raw player/staff facts only; no projected attributes or compact rows are backfilled for historical snapshots. Compact rows carry explicit `score_model_version` + `projection_model_version` per row.
- Migration v40 adds only the 22 nullable compact role columns (`goalkeeper_oop`, `potential_goalkeeper_oop`, … `centre_forward_oop`, `potential_centre_forward_oop`), each `INTEGER CHECK (col IS NULL OR col BETWEEN 0 AND 100)`. No snapshot-level column, no `UPDATE`/`DELETE`/`backfill` of existing rows. Existing rows keep `score_model_version = 1` and gain 11 new nullable columns set to `NULL`.
- `SCORE_MODEL_VERSION` moves from 1 to 2; `PROJECTION_MODEL_VERSION` stays 2. Existing readers already validate against the global checked-in versions: any current- or potential-role read that requires compact state requires `score_model_version = 2` (and for potential also `projection_model_version = 2`). As a direct compatibility consequence, existing v1 rows in `player_role_metrics` remain physically stored but fail these existing version checks and read as incomplete until a normal Load Data ingests and materializes the effective current snapshot at version 2. No new compatibility subsystem or lifecycle invariant is introduced; normal snapshot ingest/materialization is the existing recovery path.
- The 88-entry Moneyball presentation catalog stays at `BUILTIN_ROLE_CATALOG_VERSION = 1` and retains its position-family counts (attacking_midfielder 9, central_defender 12, central_midfielder 10, defensive_midfielder 10, full_back 8, goalkeeper 6, striker 10, wide_midfielder 7, wing_back 7, winger 9) and five-metric per-definition contract. All 88 `attribute_role_id` values are now non-null and validated against `all_roles()`. Tactic coverage moves from 103/111 (8 uncovered) to 119/129 (10 uncovered); exact uncovered ten are pinned in validation.
- Frontend General Search/Planner role exposure follows the existing closed-catalog contracts: metric IDs are `role.<id>` and `potential_role.<id>` mapped through `ROLE_CATALOG` mirrors (frontend `ROLE_CATALOG` has only `id`/`label`; no `phase`/`positionTags` fields). Search/Squad sorts keep null-last with UID tie-breaker; Moneyball and Club DNA ownership does not change. Dynamic columns, Squad, and Planner eligibility/options remain Rust-owned; frontend mirrors are display only.
- No new visual design, interaction model, or release packaging is introduced by this feature.

## Non-goals

- New visual design, layout, or user workflow for General, Moneyball, Shortlist, Planner, or profiles.
- Changes to the 75/25 formula, null semantics, current/potential projection inputs, or Club DNA / Moneyball percentile scoring.
- Historical Player Profile timeline, historical Moneyball seasons, cross-snapshot comparison, or analytics beyond the current-snapshot contract.
- Backfilling existing retained snapshots without a Load Data, opening or migrating the legacy `app.db`, WAL/pool/Rayon or other concurrency infrastructure, additional per-role indexes without representative failure, or new dependencies.
- BepInEx bootstrap, bridge DLL build-before-copy, or other backlog items in `.wiki/BACKLOG.md`.
- Compatibility subsystem, special promotion behavior, boost changes, or a strict 'Load Data is sole recovery across every lifecycle path' invariant beyond the normal version-mismatch consequence described above.

## Current-state map

- Relevant components: `src-tauri/src/features/scoring/catalog.rs` (68 `RoleDefinition`s, `DUMP_ATTRIBUTE_KEYS`, `label_to_dump_key`), `src-tauri/src/features/scoring/score.rs` (75/25), `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` (88 definitions, 11 null `attribute_role_id`), `src-tauri/src/features/moneyball/role_catalog.rs` (`builtin_catalog`, validation, `EXPECTED_FAMILY_COUNTS`, tactic compound-key coverage 103/111 with 8 uncovered), `src-tauri/src/features/player/query.rs` (`map_moneyball` via `attribute_role_id` → null placeholders), `src-tauri/src/features/player_metrics/compact.rs` (`SCORE_MODEL_VERSION = 1`, `PROJECTION_MODEL_VERSION = 2`, `player_current_column`/`player_potential_column`, `player_metrics_join`, `assert_snapshot_complete`, `assert_read_models_complete`), `src-tauri/src/features/player_metrics/potential_scores.rs` (one-projection-per-player writer), `src-tauri/src/db/migrations.rs` (v38 `player_role_metrics` 68+68 and `staff_role_metrics` 21, v39 drop of normalized tables, `latest_version = 39`), `src-tauri/src/features/search/{query,filter}.rs` + `src-tauri/src/features/planner/*` + `src/utils/role-catalog.ts` (68 entries, `id`/`label` only) + `src/utils/moneyball-role-catalog.ts` (88) + tactic lanes (`src/utils/tactic-ids.ts` stale 103/111 comment, `src-tauri/src/features/planner/tactic.rs`), existing ingest `snapshot/ingest.rs` + snapshot service already use `all_roles()` and `compact::SCORE_MODEL_VERSION` for current+potential materialization at the effective current snapshot.
- Data model: `player_role_metrics(snapshot_id, uid, score_model_version, projection_model_version, 68 current, 68 potential)` with FK to `players` and 0–100 checks; `players.potential_attributes_json` + `potential_projection_model_version` hold the single projected visible map; `player_moneyball_stats` snapshot-owned. Historical snapshots retain raw facts; only the effective current snapshot has compact rows.
- Persistence and migrations: `rusqlite` + `PRAGMA user_version` registry; `player_role_metrics` immutable v38 inventory (140 columns total inc. snapshot_id/uid/versions); architecture requires later catalog changes to add a new migration and model version rather than edit v38 (ADR-0028).
- Existing behavioral assumptions: General profile Pitch filters roles by `position_tags` (familiarity ≥ 15) and shows `—` for unmapped Moneyball presentation roles; Search/Squad resolver maps `role.*`/`potential_role.*` through closed-catalog lookup and safe snake_case; null-last ordering; tactic `get_planner_tactic_options` and optimizer use catalog `position_tags`; `channel_midfielder_ip` currently `["AMC"]` only. Frontend `ROLE_CATALOG` is `id`/`label` only.
- Architectural seams: ADR-0028 (compact current-snapshot metrics, fresh `app-v2.db`, one-transaction publication, no historical derived rows), ADR-0025 (selective index-driven sorts), scoring `combine_role_scores` with lane weights; Tauri `Channel` progress for Load Data; route `/players/$uid` with `tab` param and `view` default.
- Project validation commands: `./scripts/dev check` (full gate), `./scripts/dev check-rust`, `./scripts/dev check-app`, `./scripts/dev test`, `./scripts/dev smoke`; planning validators `ledger_state.py` and `delivery_state.py`.
- Primary risks: catalog/schema/model-version drift across Rust migrations, compact helpers, Moneyball catalog, and frontend mirrors; lifecycle reads after global version bump (existing version checks fail until normal materialization); Moneyball mapping to wrong attribute role; tactic coverage miscount; stale tests asserting 68.

## Feature architecture

- **Rust attribute catalog (source of truth):** `src-tauri/src/features/scoring/catalog.rs` defines the 79 `RoleDefinition`s (68 existing + 11 new OOP generics). Each new OOP definition uses the JAY-31 band lists transcribed to dump PascalCase keys. `channel_midfielder_ip` expands `position_tags` to `["AMC", "MC"]` while keeping its `display_name`, `phase`, and band lists. A single `all_roles()` remains the closed roster; catalog tests assert 79 ids, unique ids, non-empty primary from `DUMP_ATTRIBUTE_KEYS`, disjoint primary/secondary, and correct phase suffix. New ids: `goalkeeper_oop`, `centre_back_oop`, `wide_centre_back_oop`, `full_back_oop`, `wing_back_oop` (`DL/DR/WBL/WBR`), `defensive_midfielder_oop`, `central_midfielder_oop`, `wide_midfielder_oop`, `attacking_midfielder_oop`, `winger_oop`, `centre_forward_oop`. JAY-31 band transcription (PascalCase via `label_to_dump_key`): goalkeeper_oop primary `AerialReach, CommandOfArea, Communication, Handling, Reflexes, Concentration, Positioning, Agility` secondary `OneOnOnes, RushingOut, Anticipation, Decisions`; centre_back_oop primary `Heading, Marking, Tackling, Anticipation, Positioning, JumpingReach, Strength` secondary `Aggression, Bravery, Composure, Concentration, Decisions, Pace`; wide_centre_back_oop same primary secondary `Aggression, Bravery, Concentration, Decisions, Acceleration, Agility, Pace`; full_back_oop primary `Marking, Tackling, Anticipation, Positioning, Teamwork, Acceleration` secondary `Aggression, Concentration, Decisions, WorkRate, Agility, Pace, Stamina`; wing_back_oop primary `Marking, Tackling, Anticipation, Positioning, Teamwork, Acceleration, Stamina` secondary `Aggression, Concentration, Decisions, WorkRate, Agility, Pace`; defensive_midfielder_oop primary `Tackling, Anticipation, Decisions, Positioning, Teamwork, WorkRate` secondary `Marking, Aggression, Concentration, Stamina, Strength`; central_midfielder_oop primary `Tackling, Decisions, Teamwork, WorkRate` secondary `Marking, Aggression, Anticipation, Concentration, Positioning, Stamina`; wide_midfielder_oop primary `Decisions, Teamwork, WorkRate, Acceleration` secondary `Marking, Aggression, Anticipation, OffTheBall, Agility, Pace, Stamina`; attacking_midfielder_oop primary `Anticipation, Decisions, WorkRate` secondary `Marking, Aggression, OffTheBall, Teamwork, Stamina`; winger_oop primary `Anticipation, Decisions, Teamwork, WorkRate, Acceleration` secondary `Marking, Aggression, OffTheBall, Agility, Pace, Stamina`; centre_forward_oop primary `Anticipation, Decisions, WorkRate` secondary `Marking, Aggression, OffTheBall, Teamwork, Stamina`.
- **Compact metrics and global version:** `src-tauri/src/db/migrations.rs` adds migration v40 `expand_compact_role_metrics_for_generic_oop` with exactly 22 `ALTER TABLE player_role_metrics ADD COLUMN <role_id>` + matching `potential_<role_id>` for the 11 new ids, each `INTEGER CHECK (col IS NULL OR col BETWEEN 0 AND 100)`. No update/delete/backfill of existing rows. Existing `player_role_metrics` rows keep `score_model_version = 1` and 11 new columns `NULL`. `src-tauri/src/features/player_metrics/compact.rs` bumps `SCORE_MODEL_VERSION` from `1` to `2` (`PROJECTION_MODEL_VERSION` stays 2), and `player_potential_column`/`player_current_column` validation reflects 79 roles. Runtime mapping derives SQL identifiers only through closed-catalog lookup + safe snake_case; contract tests compare the complete 79-role mapping with the exact migrated schema and global model version. Later scoring work must add v41+.
- **Ingest/materialization:** No new lifecycle code is planned. The existing ingest path already iterates `all_roles()` to prepare 79 current + 79 potential scores and persists them to the effective current snapshot via `compact::persist_rows_borrowed` within the normal single final transaction. Existing version checks (`assert_snapshot_complete`, `assert_read_models_complete`, `player_metrics_join` predicates) already enforce `SCORE_MODEL_VERSION = 2`. After v40, pre-existing v1 rows fail those checks until a normal Load Data materializes 79+79 at version 2; this is stated as a compatibility consequence, not a new feature.
- **Moneyball presentation mapping:** `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` fills the 11 null `attribute_role_id` values with the mapping table above. No metric keys, weights, or version are changed. `src-tauri/src/features/moneyball/role_catalog.rs` validation (known attribute roles, duplicate compound keys, family counts 88, tactic compound-key coverage) remains green; `src-tauri/src/features/player/query.rs` now maps those 11 presentation IDs to real Current/Potential scores when the underlying compact row is at version 2. Tactic coverage becomes 119/129 with 10 uncovered `(attribute_role_id, position_tag)` combos: the prior 8 (`holding_wing_back_oop+DL`, `holding_wing_back_oop+DR`, `pressing_wing_back_oop+DL`, `pressing_wing_back_oop+DR`, `box_to_box_midfielder_ip+MC`, `box_to_box_playmaker_ip+MC`, `deep_lying_playmaker_ip+MC`, `second_striker_ip+ST`) plus `wing_back_oop+DL`, `wing_back_oop+DR` (generic retains DL/DR/WBL/WBR while presentation remains WBL/WBR).
- **Frontend mirrors (display only):** `src/utils/role-catalog.ts` expands `ROLE_CATALOG` from 68 to 79 entries with the same `id`/`label` as the Rust catalog (labels like `Goalkeeper (OOP)`, `Centre-Back (OOP)`, etc., matching `catalog.rs` `display_name` + phase; no `phase`/`positionTags` fields are added). `src/utils/moneyball-role-catalog.ts` retains 88 entries; no identifier change is needed there because Moneyball search/profile consumers read the Rust-mapped scores. `src/utils/player-metrics.ts` extends `ROLE_FAMILY_BY_ID` with the 11 new entries so `satisfies Record<RoleId, PlayerMetricRoleFamily>` holds. `src/utils/moneyball-search-metrics.ts` and Search/Planner stores continue to resolve `role.<id>` through the updated `ROLE_CATALOG`. Tests that asserted 68 now assert 79; any hard-coded `ROLE_CATALOG.length` snapshot is updated. No new UI.
- **No UI workflow change:** General placeholders become scores after materialization at version 2; General Search filters/sorts and Planner lanes work with the new `role.*` ids under the existing closed-catalog contracts; Moneyball views keep their cohort percentiles and derived role scores separate.

## Uncertainty register

### Known

- JAY-31's primary/secondary lists are approved authority; no external FM verification is required for plan acceptance.
- Current catalog is 68 roles; the 11 generic OOP presentation IDs with null `attribute_role_id` are enumerated above; `channel_midfielder_ip` currently `["AMC"]` only.
- Migration v39 is the tip; `SCORE_MODEL_VERSION = 1`, `PROJECTION_MODEL_VERSION = 2`; `player_role_metrics` has 68+68+versions; Moneyball v1 has 88 entries with family counts as recorded.
- Frontend mirrors `src/utils/role-catalog.ts` (68, `id`/`label` only) and `src/utils/moneyball-role-catalog.ts` (88) and tactic lanes consume `all_roles()` via Rust.

### Assumptions

- The 11 role display names, phases, and position families in Intent are the intended public contracts; internal `role_id` snake_case names above are the implementation identifiers that satisfy `require_safe_snake_case` and the `_ip`/`_oop` phase suffix contract.
- Adding 22 nullable `player_role_metrics` columns in v40 and bumping `SCORE_MODEL_VERSION` to 2 without a per-snapshot column is the minimal change; physical v1 rows remain intact but are rejected by the global version gate until a normal Load Data rebuilds 79 at version 2.

### Decisions

- Implement all 11 OOP generics with the JAY-31 bands and correct `channel_midfielder_ip` to `["AMC","MC"]` in this feature.
- Map the 11 Moneyball presentation definitions to the new `attribute_role_id`s in the same feature; keep Moneyball catalog version 1 and metric weights unchanged; retain generic `wing_back_oop` `DL/DR/WBL/WBR` while presentation remains `WBL/WBR`, yielding 119/129 coverage with 10 uncovered combos enumerated above.
- Enforce version via existing global `SCORE_MODEL_VERSION` checks: migration adds 22 nullable columns with no backfill; `SCORE_MODEL_VERSION` 1→2 makes existing compact rows fail existing completeness checks until normal Load Data materializes the effective current snapshot at version 2 (compatibility consequence, not a new lifecycle feature).
- Use one PR with atomic catalog+schema boundary; Moneyball mapping and frontend mirrors follow in the same PR because they depend on the same model version.
- Frontend `ROLE_CATALOG` remains `id`/`label` only; Search mirror is frontend catalog/dynamic columns; Planner eligibility/options are Rust-owned; Profile mapping is backend JSON/query; counts are Rust tests plus tactic-ids comment—frontend has no 77/11 assertion.

### Unknowns

- Exact final v40 `ALTER TABLE` ordering after migrations.rs conventions and `cargo fmt` output — verify against head and format before landing.

### Risks

- Catalog/schema/mirror drift (79 vs 68 or 88) producing undecodable identifiers, missing columns, or wrong family/phase validation.
- If migration or `SCORE_MODEL_VERSION` bump is mis-ordered, the version gate may not align with the new columns.
- Moneyball mapping to the wrong generic OOP role or wrong position tags causes tactic compound-key collisions or incorrect profile scores.
- Frontend `ROLE_CATALOG` ordering or label divergence from Rust catalog causing sort/filter or score badge mismatches.
- Tests that pin 68, 11 unmapped, or 103/111 tactic coverage failing after the catalog grows.

## Walking skeleton

Add `goalkeeper_oop` and the `channel_midfielder_ip` tag correction with migration v40's 22 nullable columns. After `SCORE_MODEL_VERSION` bumps to 2, a normal Load Data that ingests a valid dump prepares 79+79 scores and persists them for the effective current snapshot; existing Search/Planner/profile reads that already resolve `role.*` through the closed catalog then return real `goalkeeper_oop` scores without additional code changes. Seed one snapshot with known attributes; assert deterministic `goalkeeper_oop` scoring via existing ingest and resolver paths, and assert that a pre-existing v1 compact row fails the existing version check until that Load Data.

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

**Purpose:** Ship the 11 generic OOP attribute roles, channel midfielder correction, migration v40 with global version, Moneyball mapping, and frontend mirrors in one independently mergeable trunk change. One PR keeps the catalog, persistence, and presentation mapping reviewable as one coherent contract.

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

- `.wiki/features/active/missing-fm26-attribute-role-definitions.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — active feature state — only if consistency requires; otherwise unchanged.
- `.wiki/decisions/0027-scoped-potential-read-validation.md` — only if planning review warrants; otherwise unchanged (reverts to main unless evidence requires a durable decision).
- `.wiki/decisions/0028-compact-current-snapshot-metrics.md` — only if planning review warrants; otherwise unchanged (existing text already requires a new migration and model version for catalog changes).

**Behavior and data flow:**

- Move planning truth into one reviewed active ledger before implementation; record the exact delivery sequence before code.

**Ordered implementation steps:**

1. Verify the active branch is `feature/missing-attribute-roles` and base is `main` without changing Git state.
2. Confirm the worktree contains the reviewed planning paths: the active ledger and — only if warranted — amended decision records and `TODO.md` consistency edits (no implementation, tests, or architecture current-state docs).
3. Run the ledger classifier and `git diff --check` for the exact planning diff.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning/decision documents only. The ledger classifier and `git diff --check` prove structural consistency.

**Patterns to verify:**

- The active-ledger template, TODO/BACKLOG ownership rules, ADR format, and `decisions/README.md` index conventions.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, plan scope, packet order, or reviewed decisions beyond the explicitly allowed paths. Do not edit `ARCHITECTURE.md` (deferred until implementation makes it true).

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/missing-fm26-attribute-role-definitions.md && git diff --check -- .wiki/features/active/missing-fm26-attribute-role-definitions.md .wiki/TODO.md .wiki/decisions/0028-compact-current-snapshot-metrics.md .wiki/decisions/0027-scoped-potential-read-validation.md .wiki/INDEX.md`

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files.

#### Commit 2 — docs(scoring): simplify missing role delivery plan

**Status:** Completed

**Provisional commit:** `docs(scoring): simplify missing role delivery plan`

**Work:** Replace the overbuilt plan with the minimal JAY-31 contract. Restore ADR-0027 and ADR-0028 to their main/pre-Commit-1 state unless current repository evidence proves JAY-31 needs a new durable decision.

**Size assessment:** Planning-only; no implementation code.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation. No ARCHITECTURE, BACKLOG, or implementation edits.

**Implementation packet:**

- Overwrite the active ledger and revert ADRs to the lean contract owned by this commit.

**Files and responsibilities:**

- `.wiki/features/active/missing-fm26-attribute-role-definitions.md` — replace overbuilt compatibility/promotion/boost language with the lean contract: migration v40 22 nullable columns only, global `SCORE_MODEL_VERSION` 1→2 with existing version checks as the compatibility consequence (no new subsystem), Moneyball mapping unchanged, frontend mirrors unchanged. Renumber lean Rust implementation to Commit 3 and Moneyball/frontend to Commits 4–5; set Delivery fingerprint to `pending plan review`; update Active work to this commit; record developer-approved reason in Discoveries/replanning.
- `.wiki/decisions/0028-compact-current-snapshot-metrics.md` — revert to `main` (pre-Commit-1) text; its existing text already states that later catalog changes require a new migration and model version, which is sufficient for v40. Amend only if implementation evidence proves a new durable decision is required.
- `.wiki/decisions/0027-scoped-potential-read-validation.md` — revert to `main` text; no new per-snapshot provenance or lifecycle rule is added.
- `.wiki/TODO.md` — only if consistency truly requires it (likely no change).

**Behavior and data flow:**

- Correct planning truth so implementation packets are executable against the lean contract before any code lands.

**Ordered implementation steps:**

1. Verify the active branch is `feature/missing-attribute-roles` and base is `main` without changing Git state.
2. Overwrite the active ledger with the lean contract and renumbered packets as described; set Delivery fingerprint to `pending plan review`; update Active work to Commit 2 and add the bounded-replan note under Discoveries/replanning.
3. Revert ADR-0028 and ADR-0027 to their `main` versions (exact file contents from `main`); amend only if repository evidence proves a new durable decision is required for JAY-31.
4. Run ledger classifier and `git diff --check` for the exact planning diff (allowed paths only).

**Tests and proof:**

- Not applicable — planning-only correction. Classifier and `git diff --check` prove structural consistency.

**Patterns to verify:**

- Active-ledger schema 2, TODO/BACKLOG ownership, ADR format, decision index conventions.

**Constraints and non-goals:**

- Do not edit implementation, tests, executable scripts, CI, `ARCHITECTURE.md`, `INDEX.md` (unless required), `BACKLOG.md`, or Git state. Do not use the `recovery: overbuilt JAY-31 commit 2` stash as project truth.

**Dependencies and sequencing:**

- Depends on Commit 1 (Git ref `90df8821a7fdef3035cf7dce5efe56eb6a7222cc`). Must land before Commits 3–5.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/missing-fm26-attribute-role-definitions.md && git diff --check -- .wiki/features/active/missing-fm26-attribute-role-definitions.md .wiki/decisions/0028-compact-current-snapshot-metrics.md .wiki/decisions/0027-scoped-potential-read-validation.md .wiki/TODO.md`

**Stop conditions:** Stop on classifier error, unreviewed path, or branch mismatch.

**Review mandate:** Verify the staged diff owns only the corrected ledger + ADRs (+ TODO if needed), removes all overbuilt compatibility/promotion/boost/legacy-partition language, records the lean 22-column + global version contract as a compatibility consequence, and contains no implementation or ARCHITECTURE changes.

#### Commit 3 — Expand Rust catalog and compact metrics with global version

**Status:** Completed

**Provisional commit:** `feat(scoring): add FM26 OOP roles with version bump`

**Work:** Add the 11 approved generic OOP `RoleDefinition`s, correct `channel_midfielder_ip`, add migration v40 with 22 nullable compact columns, bump `SCORE_MODEL_VERSION` to 2.

**Size assessment:** ~295–330 counted non-test production lines (11 `RoleDefinition` structs ~115, `channel_midfielder_ip` tag correction, 22 `ALTER TABLE ADD COLUMN` checks, `SCORE_MODEL_VERSION` 1→2 + compact helper updates, catalog/compact/migration production code plus mechanically required interim comment/test updates counted as production lines). Exceeds 200 soft target due to atomic catalog+schema+model-version trunk-safe boundary — catalog, migration v40, and global version must land together to keep `check` green; splitting would leave undecodable identifiers or failing version gates.

**Out of scope:**

- Moneyball JSON mapping and frontend mirrors (owned by later commits). No edits to boost commands/services, snapshot promotion/service, or special provenance.

**Implementation packet:**

- Make the new catalog and model contract available; rely on existing generic `all_roles()` ingest and version-gated read paths for materialization.

**Files and responsibilities:**

- `src-tauri/src/features/scoring/catalog.rs` — append 11 `RoleDefinition`s (grouped near their IP counterparts or at a clearly commented OOP generics section). Each entry uses `phase: RolePhase::OutOfPossession`, the position tags from Intent (including `wing_back_oop` as `&["DL","DR","WBL","WBR"]`), the JAY-31 primary/secondary bands mapped via `label_to_dump_key` to dump PascalCase, and `role_id` values `goalkeeper_oop`, `centre_back_oop`, `wide_centre_back_oop`, `full_back_oop`, `wing_back_oop`, `defensive_midfielder_oop`, `central_midfielder_oop`, `wide_midfielder_oop`, `attacking_midfielder_oop`, `winger_oop`, `centre_forward_oop`. Correct `channel_midfielder_ip` `position_tags` from `&["AMC"]` to `&["AMC", "MC"]`. Keep `DUMP_ATTRIBUTE_KEYS` unchanged.
- `src-tauri/src/db/migrations.rs` — add `MIGRATION_V40_SQL` with 22 `ALTER TABLE player_role_metrics ADD COLUMN` statements (`goalkeeper_oop`, `potential_goalkeeper_oop`, … `centre_forward_oop`, `potential_centre_forward_oop`) each `INTEGER CHECK (col IS NULL OR col BETWEEN 0 AND 100)`. No snapshot column, no `UPDATE`/`DELETE` backfill. Register as migration version 40. Update `latest_version` and migration inventory tests.
- `src-tauri/src/features/player_metrics/compact.rs` — bump `SCORE_MODEL_VERSION` from `1` to `2`; update `runtime_player_catalog_maps_once_to_the_checked_in_compact_schema` to assert 79 and schema length 162 (140 + 22), update `player_metrics_join` version predicate expectations. `PROJECTION_MODEL_VERSION` remains 2. No new provenance helpers.
- `src-tauri/src/features/moneyball/role_catalog.rs` (test/comment only) and `src/utils/tactic-ids.ts` (comment/docs only) — interim update so `check` stays green before Moneyball mapping: adjust the `tactic_compound_key` coverage test and header comment from 103/111 (8 uncovered) to the mechanically correct interim value after the 11 new roles are added but before Moneyball mapping fills the 11 nulls (the interim uncovered set is the prior 8 plus the 17 new `(role_id, position_tag)` combos). Update exactly so the test passes at this commit; final 119/129 is proven in Commit 4. These are not new production paths.
- `src-tauri/src/features/player/query.rs`, `src-tauri/src/features/snapshot/ingest.rs`, `src-tauri/src/features/planner/tactic.rs`, and any other hard-coded `68` length assertions (test/comment/docs only) — update to 79 where they pin catalog size or compact vector length so `./scripts/dev check` stays green. Rely on existing generic `all_roles()` ingest and resolver paths; do not add boost/promotion or fallback logic. No new production paths beyond `catalog.rs`, `migrations.rs`, `compact.rs` in this commit.

**Behavior and data flow:**

- First launch after update runs migrations under the existing `apply` model. v40 adds 22 nullable columns; existing current rows retain physical scores but with `score_model_version = 1` they fail the existing global version gate. No row is backfilled. A normal Load Data that becomes effective current prepares 79+79 scores via `all_roles()` and persists them at version 2; reads then succeed. Before that Load Data, compact-dependent reads fail the existing completeness check as before — this is the compatibility consequence, not a new feature.

**Ordered implementation steps:**

1. Add RED catalog tests that expect 79 roles and the 11 new `role_id`s and `channel_midfielder_ip` containing both `AMC` and `MC`; verify fail on 68 baseline.
2. Insert the 11 `RoleDefinition`s with exact JAY-31 bands and the corrected channel midfielder tags.
3. Add RED schema tests that apply migrations 1–39 then v40 and assert 162-column `player_role_metrics`, 22 new nullable columns, existing rows keep `score_model_version = 1` and new columns `NULL`.
4. Implement v40 SQL and register migration; bump `SCORE_MODEL_VERSION` to 2.
5. Apply mechanically required interim count/comment updates so `check` stays green; verify no stale 68 assertion remains.
6. Run migration + compact + catalog tests to GREEN; run `./scripts/dev check`.

**Tests and proof:**

- Catalog: RED fails for 68; GREEN proves 79 unique ids, non-empty primary from `DUMP_ATTRIBUTE_KEYS`, disjoint bands, valid `_oop`/`_ip` suffix, correct `position_tags` including `wing_back_oop` 4, and a table-driven exact JAY-31 contract test that pins for each new role the exact `(role_id, display_name, phase=OutOfPossession, position_tags, primary, secondary)` in JAY-31 order (PascalCase) plus pin that `channel_midfielder_ip` changes only `position_tags`.
- Migration/schema: RED shows v40 columns absent; GREEN proves 79+79 inventory, 11 new columns `NULL` on legacy rows, `score_model_version` remains 1 physically, and contract test asserts 79-role mapping matches 162-column migrated schema with `SCORE_MODEL_VERSION = 2`.
- Normal ingest + Search/Planner (Commit 3 only): seed snapshot with known attributes; normal ingest deterministically persists 79+79 at version 2. Search resolver/catalog behavior (`role.*`/`potential_role.*` closed-catalog lookup via the existing generic `all_roles()` path, Rust/backend-only) succeeds for a new role (e.g., `role.goalkeeper_oop` / `potential_role.goalkeeper_oop`); frontend `ROLE_CATALOG` mirror resolution proof remains solely in Commit 5. Planner options proof: `src-tauri/src/features/planner/tactic.rs::exposes_phase_compatible_roles_and_placements_from_catalog` updated to expose one new OOP role and Channel Midfielder at `MC` (in addition to `AMC`). No mapped General Profile proof in this commit — that belongs to Commit 4. Existing catalog/search/planner tests updated minimally where they pinned 68.

**Patterns to verify:**

- Existing `COMPACT_ROLE_METRICS` immutable DDL pattern, migration test shape, `compact::test_support::read_row`/`count_rows`, version-gated `player_metrics_join`, `assert_read_models_complete` current vs potential flags.

**Constraints and non-goals:**

- Do not add snapshot provenance columns, legacy partitions, per-snapshot provenance, or score backfill/update/delete; do not open or change `app.db`; do not change 75/25 formula, null semantics, or projection; do not add per-role indexes; do not edit boost/promotion/service code.

**Dependencies and sequencing:**

- Depends on Commit 2. Must land before Commits 4–5 because Moneyball mapping and frontend reads assume 79-role catalog + version 2 columns.

**Validation:** `./scripts/dev check`

**Stop conditions:** Stop if any new column name fails `require_safe_snake_case`, if SQLite rejects 162-column width, or if a future catalog change would require editing v40 DDL instead of adding v41.

**Review mandate:** Verify v40 is additive and idempotent, existing rows' scores remain physically stored with new columns `NULL`, `SCORE_MODEL_VERSION = 2` aligns with migrated schema, and `check` is green with interim coverage counts; no boost/promotion/provenance edits exist.

#### Commit 4 — Map Moneyball presentation roles to the new attribute roles and pin tactic coverage

**Status:** Completed

**Provisional commit:** `feat(moneyball): map generic OOP presentation roles`

**Work:** Wire the 11 generic OOP Moneyball presentation definitions to the new attribute `role_id`s and pin deterministic tactic coverage to 119/129 with ten uncovered combos.

**Size assessment:** ~20 changed non-test implementation lines (JSON) + comment/test updates. Within soft target.

**Out of scope:**

- Frontend `ROLE_CATALOG` mirrors (Commit 5) and compact schema.

**Implementation packet:**

- Close the 11 placeholder nulls so `features/moneyball` and `features/player` presentation mapping returns real General scores after materialized version-2 data exists; keep presentation `WBL/WBR` while generic retains `DL/DR/WBL/WBR`.

**Files and responsibilities:**

- `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` — set `attribute_role_id` for the 11 ids from Intent to the new attribute ids (`amc_attacking_midfielder_oop` → `attacking_midfielder_oop`, etc., including `wbl_wbr_wing_back_oop` → `wing_back_oop`); keep `version: 1`, family counts, metric keys, weights, inverted flags unchanged.
- `src-tauri/src/features/moneyball/role_catalog.rs` (test/comment only) — update `maps_only_known_attribute_roles_and_preserves_unmapped_generic_roles` expectation from 77 mapped / 11 unmapped to 88 mapped / 0 unmapped; update `tactic_compound_key_is_unique_and_covers_...` to `covers_119_of_129_with_10_uncovered`, assert total 129, mapped 119, uncovered 10 with exact expected set: `("holding_wing_back_oop","DL")`, `("holding_wing_back_oop","DR")`, `("pressing_wing_back_oop","DL")`, `("pressing_wing_back_oop","DR")`, `("box_to_box_midfielder_ip","MC")`, `("box_to_box_playmaker_ip","MC")`, `("deep_lying_playmaker_ip","MC")`, `("second_striker_ip","ST")`, `("wing_back_oop","DL")`, `("wing_back_oop","DR")`. No new production path beyond the JSON mapping.
- `src-tauri/src/features/player/query.rs` (test only — `load_role_scores` is existing code) — no production code change beyond the catalog it already joins; the mapped checks now find values for the 11 former placeholders when compact row is at version 2 (proof ownership as below).
- `src/utils/tactic-ids.ts` (comment/docs only) — update header comment from interim to 119/129 (10 uncovered) and list the ten `NULL -> "—"` combos as above; no runtime logic change.

**Behavior and data flow:**

- `builtin_catalog()` loads 88 definitions, validates each `attribute_role_id` against the 79-role `all_roles()`, and exposes the presentation-to-attribute mapping. `get_player` now returns `PlayerRoleScore` for those 11 presentation IDs when the surrounding `player_role_metrics` row is at version 2. Moneyball validation and tactic compound-key uniqueness remain enforced.

**Ordered implementation steps:**

1. Add RED Moneyball catalog test that expects 88 mapped and 119/129 coverage; verify fail on interim baseline. Add RED for `player/query.rs::presents_duplicate_moneyball_roles_from_one_attribute_score_and_keeps_unmapped_null` expecting mapped scores (fail on 11 placeholders).
2. Fill the 11 `attribute_role_id` values in `builtin_role_definitions_v1.json`.
3. Update `role_catalog.rs` tests and `tactic-ids.ts` comment to 119/129 with exact expected set.
4. Update `src-tauri/src/features/player/query.rs` test `presents_duplicate_moneyball_roles_from_one_attribute_score_and_keeps_unmapped_null` (or renamed analogue) from 11 placeholders to mapped current/potential scores for one new OOP role; run Moneyball catalog + `player/query.rs` tests to GREEN; run `./scripts/dev check`.

**Tests and proof:**

- Table-driven exact mapping test pins all 11 `(presentation_id, attribute_role_id)` pairs exactly as Intent — each `attribute_role_id` in `all_roles()`; any missing, swapped, or extra mapping fails.
- `moneyball/role_catalog.rs` tests: GREEN proves all 88 `attribute_role_id`s non-null and in `all_roles()`, unmapped 0, duplicate `(attribute_role_id, position_tag)` unique, family counts unchanged, total 129, mapped 119, uncovered exactly ten as listed.
- General Profile placeholder-to-score proof owned here: update existing `src-tauri/src/features/player/query.rs::presents_duplicate_moneyball_roles_from_one_attribute_score_and_keeps_unmapped_null` (or its renamed analogue) from asserting 11 placeholder nulls to asserting mapped current and potential scores for at least one new OOP role (e.g., `centre_back_oop` duplicate via `centre_back_oop` vs a specialized variant) when the underlying compact row is at version 2, while legacy v1 rows are asserted rejected/incomplete via the existing global version gate (incomplete-current-potential-snapshot error) until a normal Load Data materializes version 2 — this commit owns the Profile placeholder-to-score proof, not Commit 3.

**Patterns to verify:**

- Existing `validate_builtin_catalog` shape, `expected_position_tags`/`expected_position_prefix`, and the General `get_player → PlayerRoleScore` mapping pattern.

**Constraints and non-goals:**

- Do not change Moneyball metric keys, weights, directions, or catalog version; do not merge wide-position definitions; do not change `wing_back_oop` position tags beyond `DL/DR/WBL/WBR`.

**Dependencies and sequencing:**

- Depends on Commit 3 (79-role catalog + version 2 columns must exist to validate new attribute ids).

**Validation:** `./scripts/dev check`

**Stop conditions:** Stop if any `attribute_role_id` not in 79-role closed catalog or if Moneyball validation expects version bump — approved constraint keeps `BUILTIN_ROLE_CATALOG_VERSION = 1`.

**Review mandate:** Verify the 11 mappings exactly as Intent, no metric/weight change, family counts/phase/id invariants preserved, tactic coverage pinned 119/129 with exact ten, `wing_back_oop` tags `DL/DR/WBL/WBR` while presentation `WBL/WBR`, and `check` green.

#### Commit 5 — Mirror expanded attribute catalog in frontend (display only)

**Status:** Active

**Provisional commit:** `feat(frontend): mirror expanded attribute catalog`

**Work:** Make the frontend General Search/Planner catalogs reflect the 79-role attribute catalog so the closed-catalog contracts stay coherent (display only).

**Size assessment:** ~30 changed non-test implementation lines (11 entries + 11 family mappings). Within soft target.

**Out of scope:**

- Moneyball scoring logic, Rust scoring formula, and visual design.

**Implementation packet:**

- Keep the frontend `ROLE_CATALOG` and derived search metrics in lockstep with the Rust closed catalog; Planner eligibility/options remain Rust-owned.

**Files and responsibilities:**

- `src/utils/role-catalog.ts` — expand `ROLE_CATALOG` from 68 to 79 entries: add `goalkeeper_oop`, `centre_back_oop`, `wide_centre_back_oop`, `full_back_oop`, `wing_back_oop`, `defensive_midfielder_oop`, `central_midfielder_oop`, `wide_midfielder_oop`, `attacking_midfielder_oop`, `winger_oop`, `centre_forward_oop` with labels `Goalkeeper (OOP)`, etc., ordered to mirror `catalog.rs`; each entry has only `id`/`label` (no `phase`/`positionTags`). Update any `ROLE_CATALOG.length` snapshot that pins 68.
- `src/utils/moneyball-role-catalog.ts` — retain 88 entries; no id/phase change.
- `src/utils/player-metrics.ts` — extend `ROLE_FAMILY_BY_ID: Record<RoleId, PlayerMetricRoleFamily>` with all 11 new entries so the `satisfies Record<RoleId, ...>` contract stays total and `tsc -b` passes. Exact mapping: `goalkeeper_oop` → `Goalkeepers`, `centre_back_oop` → `Central defense`, `wide_centre_back_oop` → `Central defense`, `full_back_oop` → `Full-back and wing-back`, `wing_back_oop` → `Full-back and wing-back`, `defensive_midfielder_oop` → `Defensive midfield`, `central_midfielder_oop` → `Central midfield`, `wide_midfielder_oop` → `Wide midfield and wings`, `winger_oop` → `Wide midfield and wings` (matching generic `winger_ip` → `Wide midfield and wings`), `attacking_midfielder_oop` → `Attacking midfield`, `centre_forward_oop` → `Forwards`. Do not alter existing specialized winger OOP families (`inside_outlet_winger_oop`, `tracking_winger_oop`, `wide_outlet_winger_oop` remain `Forwards`).
- Tests: `src/utils/player-metrics.test.ts` (pin `wide_midfielder_oop` → `Wide midfield and wings` and generic `winger_oop` → `Wide midfield and wings` matching `winger_ip`; keep specialized `inside_outlet_winger_oop`/`tracking_winger_oop`/`wide_outlet_winger_oop` at `Forwards`), `src/features/search/utils/dynamic-columns.test.ts` (update `ROLE_CATALOG` length 68→79), `src/utils/moneyball-role-catalog.test.ts` (remains 88), `src/utils/moneyball-search-metrics.test.ts` — update length expectations where applicable; frontend has no 77/11 mapping assertion.

**Behavior and data flow:**

- General Search metric picker, filter strip, table columns, and sort keys expose the 11 new `role.goalkeeper_oop` … `role.centre_forward_oop` (and potential counterparts) through the same closed-catalog resolver. Planner tactic lanes discover the new OOP roles via Rust `all_roles()` and compact columns.

**Ordered implementation steps:**

1. Add RED frontend tests: `ROLE_CATALOG` expects 79 entries with 11 new `id`s (fail on 68) and `ROLE_FAMILY_BY_ID` compile fails when 11 entries are missing (`tsc -b` error).
2. Expand `ROLE_CATALOG` and add all 11 `ROLE_FAMILY_BY_ID` entries with exact categories above; verify `tsc -b` passes only after the 11 entries are present (including `winger_oop` → `Wide midfield and wings`).
3. Run focused frontend proof: `./scripts/dev test src/utils/player-metrics.test.ts src/features/search/utils/dynamic-columns.test.ts src/utils/moneyball-role-catalog.test.ts src/utils/moneyball-search-metrics.test.ts src/utils/tactic-ids.test.ts` (proves `scripts/dev test` forwards paths via `pnpm exec vitest run "$@"`), then `./scripts/dev check` for the full gate.

**Tests and proof:**

- Modify `src/utils/role-catalog*` tests: RED must fail for missing ids/length; GREEN proves 79 `ROLE_CATALOG` entries with unique `id`s and labels matching Rust `display_name` + `(OOP)`, `MONEYBALL_ROLE_CATALOG` remains 88. For `src/utils/player-metrics.ts`: RED shows `tsc -b` fails; GREEN proves all 11 entries exist with exact categories above and `satisfies Record<RoleId, PlayerMetricRoleFamily>` holds. `src/utils/player-metrics.test.ts` pins generic `winger_oop` → `Wide midfield and wings` (matching `winger_ip`) and `wide_midfielder_oop` → `Wide midfield and wings`, while specialized winger OOP families remain `Forwards`.

**Patterns to verify:**

- Existing `ROLE_CATALOG` const shape (`id`/`label` only), `moneyball-role-catalog.ts` `role()` helper and `orderedPositions`, and the search/planner dynamic-column contracts.

**Constraints and non-goals:**

- Do not change Moneyball metric catalogs, design tokens, or visual layout. Keep labels consistent with `catalog.rs` display names. Do not add frontend `phase`/`positionTags`.

**Dependencies and sequencing:**

- Depends on Commit 3 (canonical Rust ids must be frozen before mirroring).

**Validation:** `./scripts/dev test src/utils/player-metrics.test.ts src/features/search/utils/dynamic-columns.test.ts src/utils/moneyball-role-catalog.test.ts src/utils/moneyball-search-metrics.test.ts src/utils/tactic-ids.test.ts && ./scripts/dev check`

**Stop conditions:** Stop if frontend catalog order diverges from Rust in a way that changes cohort/sort semantics, or if a test pins snapshot output that must be renewed — request developer approval for snapshot updates.

**Review mandate:** Verify no stale 68-length assertion remains, 11 new frontend entries match Rust ids/labels exactly with `id`/`label` only (including `winger_oop` → `Wide midfield and wings`), no Moneyball presentation count change, and `check` green.

## Active work

**PR:** PR 1 — Complete missing FM26 attribute role definitions

**Commit:** Mirror expanded attribute catalog in frontend (display only)

### RED or removal proof

Update frontend catalog and family tests to expect 79 roles and all 11 new IDs; confirm the 68-role baseline and incomplete family map fail.

### Expected outcome

General Search exposes all 79 attribute roles, the frontend family map is total for the new IDs, and Moneyball presentation remains at 88 entries.

### Explicit exclusions

Rust scoring or persistence, Moneyball metric catalogs, visual design, phase or position-tag mirrors, and unrelated refactors.

## Discoveries and replanning

- Bounded replan at planning HEAD `90df8821a7fdef3035cf7dce5efe56eb6a7222cc`: the prior ledger and ADR amendments contained an overbuilt compatibility subsystem (per-snapshot provenance, `LEGACY_V1_ROLE_IDS` partition, promotion clear-only and boost preflight machinery, 68-role partial-readability and 'sole recovery path' invariants) that JAY-31 does not request. That overbuilt implementation is isolated in stash `recovery: overbuilt JAY-31 commit 2` and is not design input. Decision: restore ADR-0027 and ADR-0028 to their `main` state (ADR-0028 already requires a new migration and model version for catalog changes, which suffices for v40); reframe Commit 3 to the minimal catalog+22-column+version-bump integration that relies on existing `all_roles()` ingest and existing version-gated read paths; remove detailed boost/promotion/provenance instructions from active packets and state the v1-row version failure as a compatibility consequence rather than a new lifecycle feature. Delivery fingerprint reset to `pending plan review`.
- Historical note: earlier complex designs for per-snapshot provenance and legacy partitions were considered and rejected; no detailed instructions are preserved in active scope.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Complete missing FM26 attribute role definitions | Commit 1 — Record the approved feature plan | 90df8821a7fdef3035cf7dce5efe56eb6a7222cc | Recorded the accepted schema-2 ledger, TODO activation, and ADR-0027/0028 amendments. | `ledger_state.py`; `git diff --cached --check` — passed. | Not applicable | Clear | 0 | None |
| PR 1 — Complete missing FM26 attribute role definitions | Commit 2 — docs(scoring): simplify missing role delivery plan | 88782a46425c7106599fd70ab66d5c01f7a6eb38 | Restored ADR-0027/0028 to `main` and reduced delivery to the 22-column migration, global score-model version bump, Moneyball mapping, and frontend mirrors. | `ledger_state.py`; `delivery_state.py`; `git diff --cached --check` — passed. | Not applicable | Clear | 1 | Corrected two proof statements: v1 rows fail the existing version gate, and frontend mirror proof remains in Commit 5. |
| PR 1 — Complete missing FM26 attribute role definitions | Commit 3 — Expand Rust catalog and compact metrics with global version | 526367b240e871b60bbb39cbd88d263e084266b0 | Added the exact 11 generic OOP definitions, Channel Midfielder MC eligibility, migration v40 with 22 nullable checked columns, 79-role compact persistence, and global score-model version 2 gates. | Focused Rust suites; `./scripts/dev check` — 756 passed, 2 ignored. | Pass | Clear | 1 | Interim coverage is 104/129 rather than the planned estimate 103/128 because the existing Moneyball catalog already maps `channel_midfielder_ip` at MC; final 119/129 remains unchanged. |
| PR 1 — Complete missing FM26 attribute role definitions | Commit 4 — Map Moneyball presentation roles to the new attribute roles and pin tactic coverage | Pending record | Mapped all 11 former Moneyball placeholders to the new generic OOP attribute roles and pinned exact 88/88 mapping plus 119/129 tactic coverage. | Focused Moneyball and Player Profile tests; `./scripts/dev check` — 756 passed, 2 ignored. | Pass | Clear | 0 | Accepted one cosmetic stale test-name nitpick; assertions and supported behavior are exact. |

## Final validation

- `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/missing-fm26-attribute-role-definitions.md`
- `git diff --check -- .wiki/features/active/missing-fm26-attribute-role-definitions.md` (active ledger only; ADRs remain exact `main` matches and are not edited)
- `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/missing-fm26-attribute-role-definitions.md /home/jonas/projects/fm-valuescout` to confirm the recorded Delivery fingerprint, then rerun `ledger_state.py`.
- `./scripts/dev test && ./scripts/dev check` on the final implementation range (separate commands: `test` runs Vitest, `check` runs Biome/TypeScript/secretlint/Rust fmt/clippy/test; do not claim `check` runs Vitest). Commit 5 validation must first run focused `./scripts/dev test src/utils/player-metrics.test.ts src/features/search/utils/dynamic-columns.test.ts src/utils/moneyball-role-catalog.test.ts src/utils/moneyball-search-metrics.test.ts src/utils/tactic-ids.test.ts` then `./scripts/dev check`.

## Documentation impact

Planning corrects durable decision ownership before implementation. ADRs 0027 and 0028 are reverted to their `main` text and remain exact `main` matches — no ADR edit in this correction pass; no durable decision amendment is needed for this feature because ADR-0028 already governs catalog changes via a new migration and model version. After implementation makes the new architecture true, documentation close-out must update `.wiki/ARCHITECTURE.md` to document migration v40 (22 nullable compact columns), 79 current + 79 potential role columns (162-column `player_role_metrics`), `SCORE_MODEL_VERSION=2` (and `PROJECTION_MODEL_VERSION=2`), all 79-role consumers (catalog, compact, ingest/materialization, Search resolver, Planner options/tactic lanes, frontend mirrors), and exact tactic coverage 119/129 with the ten intentional uncovered `(attribute_role_id, position_tag)` combinations enumerated in Intent/Commit 4. No `ARCHITECTURE.md`, `CONCEPT.md`, or `DESIGN.md` change is authored by this planning commit. At feature close-out, archive the completed ledger and clear TODO; no new ADR/BACKLOG/CONCEPT/DESIGN is added.
