# Role scoring engine

## Status

Active

## Intent

Compute a 0–100 role-fit score for every player for every FM26 In Possession (IP) and Out of Possession (OOP) role, using FM-designated primary and secondary attributes. Persist scores on Load Data ingest so search, profiles, planner, and optimizer share one scoring model. Expose a pure combined IP+OOP score with caller-supplied weights (persistence and UI deferred to squad planner).

## User-visible behavior

- After a successful **Load Data**, each ingested player has per-role IP and OOP scores available in SQLite (null only when a required attribute is missing).
- A thin proof on the home sanity path shows that scores exist (one sample role score column or equivalent) — not a full role browser.
- Combined IP+OOP scoring is available as a Rust/API function with default equal weights; no weight settings UI in this feature.

## Invariants

- Score range is **0–100** (integer after rounding). Same model will drive search, profiles, planner, and optimizer.
- Per-role score uses **primary** and **secondary** attribute bands: band means first (equal weight within band), then `0.75 × primary_mean + 0.25 × secondary_mean`, then scale by `/ 20 × 100`. If a role has no secondary list, use primary mean alone (do not invent a zero secondary band).
- If **any** attribute required for that role’s primary or secondary list is JSON `null` in the player dump → that role’s score is **null** (signals bad/incomplete attribute data).
- Position suitability (`positions`) does **not** enter the role score.
- Scores are computed in Rust (thick backend). WebView does not reimplement the formula.
- Catalog is **static, offline** — checked into the repo. No live fetch from Sidekick or SortItOutSI at runtime.
- Combined score: `w_ip × ip + w_oop × oop` with `w_ip + w_oop = 1` (or equivalent pair). Default `0.5 / 0.5`. If either side is null → combined is null. Weight persistence/UI is out of scope.

## Non-goals

- Player search UI, profile role grids, squad planner, optimizer
- Persisting or editing IP/OOP weight preferences
- Blending position familiarity into scores
- Hidden attributes or personality in role scores (visible attributes only)
- Live scraping of external role sites
- Recomputing scores for an existing snapshot without a new Load Data
- Tuning weights of individual attributes beyond the primary/secondary band split

## Current-state map

- Relevant components: Rust `features/snapshot` ingest + sanity query; React sanity list; no scoring feature yet
- Data model: `players.attributes_json` (PascalCase keys, `number | null`); `positions_json` separate
- Persistence: migration v2 `saves` / `snapshots` / `players`; no score tables
- Existing behavioral assumptions: `null` attribute means unknown — never coerced to 0 on ingest
- Architectural seams: thin frontend / thick Rust; compose features in routes; dump body never crosses IPC
- Tests: `cargo test` for ingest; Vitest + Playwright stub for sanity list
- Primary risks: catalog completeness vs FM26 role set; attribute display-name → dump-key mapping; ingest latency with full role matrix

## Feature architecture (this feature)

```text
Rust features/scoring (new)
  → static role catalog (role id, phase IP|OOP, position labels, primary[], secondary[])
  → attribute key map (human labels → dump PascalCase)
  → score_role(attrs, role) → Option<u8> / Option<i32> 0–100
  → combine_roles(ip, oop, weights) → Option<…>

Rust features/snapshot (extend)
  → on ingest, after player rows: compute all role scores → persist
  → sanity query exposes a thin score proof field

React features/snapshot (extend)
  → sanity list shows the proof column

SQLite migration v3
  → player_role_scores (snapshot_id, uid, role_id, phase, score) or equivalent
```

`scoring` owns formula and catalog. `snapshot` owns when scores are written and how they are queried for the proof. No cross-feature imports on the React side.

## Uncertainty register

### Known

- Dump attribute keys are PascalCase without spaces (`OffTheBall`, `WorkRate`, `JumpingReach`, `Teamwork`, …) per `bridge/Layouts/Fm263Layout.cs`.
- Sidekick role API (`/api/roles/<slug>`) exposes `key_attributes` + IP/OOP `phase` but **not** secondary attributes.
- SortItOutSI FM26 guide lists **Key** and **Preferred** per role — use as primary/secondary source.
- Roadmap: scores on ingest; user accepts possible later move to on-demand if Load Data becomes too slow.

### Assumptions

- Primary = SortItOutSI Key; secondary = Preferred. Roles with only Key use primary-only scoring.
- Role inventory follows FM26 IP/OOP split (Sidekick index as checklist); attribute lists from SortItOutSI. Source mismatches are normal transcription work in commit 1 — not a planning blocker.
- Display labels like "Jumping" / "Off the Ball" map to dump keys `JumpingReach` / `OffTheBall`.
- Integer scores after rounding half-up (or Rust `round`) are fine for Score Badges.

### Decisions

- Primary/secondary band split **75% / 25%** after within-band means (handles unequal attribute counts).
- Any null required attribute → null role score.
- Position suitability separate from score.
- Compute and persist on ingest; document ponytail for lazy/on-demand if ingest scoring dominates Load Data time.
- Combined weights are function parameters only in this feature.
- Thin proof only; real score UI is player search (order 4).
- Single PR for the whole feature (five atomic commits) — small review surface; no user-visible value until ingest lands.

### Unknowns

- Practical ingest cost for full role × player matrix on a large dump — measure in the ingest commit; replan if unacceptable.
- Catalog source mismatches (role present in only one guide, renamed, or Key-only vs Key+Preferred) — resolve during commit 1; ask the developer only when a role cannot be reconciled without a product call.

### Risks

- Catalog transcription errors produce silently wrong rankings — mitigate with fixture tests on a few known roles and golden attribute maps.
- Ingest slowdown — ponytail; optional later `/spike` or plan change to lazy scoring.

## Walking skeleton

Static catalog for a handful of roles → pure `score_role` GREEN in `cargo test` → persist all scores on ingest → sanity list shows one sample score after Load Data (stubbed in smoke).

## Delivery plan

### PR 1 — Role scoring on ingest

**Status:** Active

**Provisional PR title:** `feat(scoring): add role scores on ingest`

**Purpose:** Deliver the offline catalog, pure score/combine engine, persist-on-ingest path, and thin sanity proof in one short-lived PR with five atomic commits.

**Depends on:** Snapshot ingest (complete).

**Merge to trunk when:** All five commits are done, gate green, sanity proof visible with stubbed IPC.

#### Commit 1 — Role catalog with primary and secondary attributes

**Status:** Completed — `0b08dd1`

**Work:** Add Rust `features/scoring` with a static FM26 role catalog: stable `role_id`, display name, phase (`in_possession` | `out_of_possession`), position tags, primary attribute keys, secondary attribute keys (dump PascalCase). Include a small label→key map for transcription. Source primary/secondary from SortItOutSI Key/Preferred; use Sidekick IP/OOP inventory as the completeness checklist. Reconcile source mismatches in this commit; record leftovers in Discoveries; ask only when a role cannot be included without a product call.

**Out of scope for this commit:**
- Scoring formula implementation beyond catalog types
- SQLite, ingest, UI, IPC

**Validation:** `cargo test` for catalog invariants (unique role ids, non-empty primary lists, keys ⊆ known dump attribute set, every phase value valid). `./scripts/dev check` when Rust is touched.

**Provisional commit:** `feat(scoring): add FM26 role attribute catalog`

#### Commit 2 — Per-role 0–100 score function

**Status:** Active

**Work:** Implement `score_role`: within-band means → 75/25 blend (or primary-only if no secondary) → `/20×100` → rounded integer. Any null in the used attribute set → `None`. Unit tests: equal bands, unequal primary/secondary counts, empty secondary, null attribute → None, known fixture maps.

**Out of scope for this commit:**
- Combined IP+OOP helper
- Persistence / ingest

**Validation:** Focused `cargo test` on scoring module; gate.

**Provisional commit:** `feat(scoring): score roles from primary and secondary attributes`

#### Commit 3 — Combined IP and OOP score helper

**Status:** Pending

**Work:** Pure `combine_role_scores(ip, oop, ip_weight)` (oop weight = `1 - ip_weight`), default weight `0.5`. Null if either input score is null or weight out of `[0, 1]`. Tests for 50/50, custom weights, null propagation.

**Out of scope for this commit:**
- Persisting weights, planner UI, IPC for combine

**Validation:** `cargo test`; gate.

**Provisional commit:** `feat(scoring): combine IP and OOP role scores`

#### Commit 4 — Score table migration and ingest write path

**Status:** Pending

**Work:** Migration v3: `player_role_scores` keyed by `(snapshot_id, uid, role_id)` with `phase` and nullable `score`. On snapshot ingest, after players insert, compute all catalog roles per player and insert scores in the same transaction (or clearly bounded follow-on inside the ingest transaction). Cascade delete with snapshot. Ponytail comment if batching is simplified: upgrade to lazy/on-demand scoring if ingest scoring time becomes a Load Data bottleneck (measure in tests or manual note).

**Out of scope for this commit:**
- React UI, combine IPC, weight settings

**Validation:** `cargo test` ingest fixtures assert expected scores for golden player attrs; rollback still leaves prior snapshot+scores untouched; `./scripts/dev check`.

**Provisional commit:** `feat(scoring): store role scores during snapshot ingest`

#### Commit 5 — Sanity-list score proof

**Status:** Pending

**Work:** Extend sanity player DTO/query with one proof field (fixed sample role score, e.g. a stable catalog id, or “scores present” count). Update React sanity table and Playwright/Vitest stubs. No score browser.

**Out of scope for this commit:**
- Search filters, profile role grid, weight UI

**Validation:** `./scripts/dev test`, smoke stub, `./scripts/dev check`.

**Provisional commit:** `feat(scoring): show sample role score on sanity list`

## Active work

**PR:** 1 — Role scoring on ingest

**Commit:** Per-role 0–100 score function

### RED test (active commit)

Assert `score_role` returns the expected 0–100 integer for a fixture attribute map (equal primary/secondary means → 75/25 blend), and returns `None` when any required attribute is null — fails because the scoring function does not exist.

### Expected outcome

Pure `score_role` in `features/scoring` with unit tests for equal bands, unequal band sizes, empty secondary (primary-only), and null → None; no combine helper, ingest, or UI.

### Explicit exclusions

Combined IP+OOP helper, persistence / ingest, React, IPC.

## Discoveries and replanning

- Sidekick `/api/roles/*` is useful for IP/OOP inventory and key-only lists but lacks Preferred/secondary — SortItOutSI is the primary/secondary source for the catalog.
- Dump keys differ from guide labels (`Jumping` → `JumpingReach`, spaced names → PascalCase). Catalog must store dump keys only.
- Delivery plan collapsed from two PRs to one (five commits) — small feature, no mergeable mid-feature user value.
- Catalog commit 1 reconciliation: Sidekick “generic” OOP hubs (`goalkeeper_oop`, `centre_back_oop`, …) map to SortItOutSI’s named OOP variants (Line-Holding/Sweeper Keeper, Covering/Stopping CB, …) — catalog uses the named variants. SortItOutSI-only roles kept: `wide_centre_back_ip`, `overlapping_centre_back_ip`, `covering_wide_centre_back_oop`, `stopping_wide_centre_back_oop`, `pressing_defensive_midfielder_oop`. Sidekick `no_nonsense_center_back_ip` spelling normalized to `no_nonsense_centre_back_ip`. Deep-Lying Playmaker primary follows SortItOutSI Key (includes `OffTheBall`); Sidekick key list omits it. Where SortItOutSI listed the same attribute in Key and Preferred, catalog keeps it in primary only (bands must be disjoint for 75/25 scoring).

## Completed work

| PR | Commit | Hash | Notes |
| --- | --- | --- | --- |
| 1 | Role catalog with primary and secondary attributes | `0b08dd1` | 68 roles; SortItOutSI Key/Preferred; disjoint bands |

## Final validation

At feature end: full `./scripts/dev test`, `./scripts/dev check`, smoke; manual Load Data on Windows confirms sanity proof scores; spot-check a few roles against expected attribute means.

## Documentation impact

- Update `.wiki/ARCHITECTURE.md` data flow for scoring on ingest + `player_role_scores`.
- Completed feature record at `/finish-feature`.
- TODO: move to Completed; Plan next → player search.
