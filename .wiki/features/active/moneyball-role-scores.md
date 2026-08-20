# Moneyball Role Scores

## Status

Active

## Intent

Define a transparent, versioned Moneyball role-scoring model from the current
snapshot's imported performance percentiles, then expose those scores in
Moneyball Search and Player Profile without combining them with attribute-based
role scores.

The built-in catalog is grounded in the legacy FM26 archetype notes at pinned
commit [`366aa20`](https://github.com/JG1995/fm-valuescout-react/tree/366aa20b5282d3a63c94854ddb8da6992462b0c5/docs/notes/default-archetypes).
It preserves each source position-family definition as its own stable entity so
later editing does not silently couple roles that happen to have the same
metrics and weights today.

Linear source: [JAY-20](https://linear.app/jaycount/issue/JAY-20/define-moneyball-role-scoring).

## User-visible behavior

- Moneyball Search offers every built-in Moneyball role score as an optional
  column, numeric filter, and sort field.
- A Moneyball role score is rendered with the score badge ramp; an unavailable
  score is rendered as an em dash, never as zero.
- Search role scores use the table's selected comparison pool:
  - `Full CSV` composes the persisted full-import metric percentiles.
  - `Filtered` recomputes the contributing metric percentiles over the complete
    filtered comparison cohort, never the current page.
- A role-score filter is applied after its comparison cohort has been scored;
  it cannot recursively redefine the scores it filters.
- The Moneyball Player Profile uses the full imported cohort and replaces the
  General summary's best-role badges with best Moneyball IP and OOP scores.
- The ready Moneyball profile keeps the existing raw-statistics panel and adds
  a position-selected Moneyball role-fit panel.
- Each Moneyball profile role exposes its five contributing metric names,
  weights, direction, and percentile contribution, together with the catalog
  version and full-import comparison basis.
- The General profile role table uses the complete presentation inventory. A
  role without an attribute formula shows an em dash for Current and Potential.
- Unsupported attribute placeholders remain absent from General Search,
  Planner, tactics, ingest scoring, and potential-score materialization.

## Invariants

- Moneyball role scores, attribute role scores, and individual Moneyball metric
  percentiles are distinct values with distinct labels and IPC fields.
- Built-in catalog version 1 contains all 88 role/phase/position-family
  definitions from the linked Markdown sources. Identical definitions are not
  merged across position families.
- Left and right positions remain paired where the source has one shared
  definition. For example, the DL/DR Wing-Back definition is distinct from the
  WBL/WBR Wing-Back definition, while DL and DR share their source definition.
- Every catalog identity includes its position family, role, and phase. Duplicate
  display names therefore remain unambiguous in field IDs and are disambiguated
  by position tags in column and filter labels.
- The score is the rounded weighted mean of already-normalized 0-100 JAY-19
  metric percentiles. The scorer does not calculate rates from raw values.
- Direction is applied once by the JAY-19 percentile engine. The role scorer
  must not invert an already-normalized percentile again.
- Catalog inversion metadata must agree with the existing lower-is-better
  Moneyball metric catalog. The four inverted metrics used by the source roles
  are `minutes_per_goal`, `fouls_made_per_90`,
  `goals_conceded_per_90`, and `possession_lost_per_90`.
- Zero is a valid raw value and a valid percentile. It must not be treated as
  absent, false, or ineligible. An all-equal zero cohort retains JAY-19's neutral
  percentile of 50.
- A definition must have at least one metric, finite non-negative weights, and a
  strictly positive total weight. A zero total is rejected before scoring, so
  division by zero is impossible.
- If any contributing percentile is null, the role score is null. The scorer
  does not silently substitute zero or renormalize a partial definition.
- There is no minutes or appearances threshold and no sample-size shrinkage.
  Users who want a sample floor own it in their exported CSV or Search filters.
- Profile scores always use persisted full-import percentiles from the active
  save's effective current snapshot.
- Filtered Search scores use the complete pre-score comparison cohort, not a
  result page. Pagination occurs after any role-score filter and role-score sort.
- Under `AND`, non-role filters define the filtered comparison cohort and role
  rules are post-score predicates. Under `OR` mixed with role rules, the full
  imported cohort is the comparison and candidate set so a player that matches
  only a role rule cannot be discarded before scoring.
- Replacing the current snapshot's Moneyball import immediately changes all
  derived role scores because no independent role-score cache is persisted.
- No new dependency, database table, or migration is introduced for derived
  built-in scores.
- The current Moneyball CSV bounds remain the performance boundary. The query
  path computes only roles and metric keys requested by visible columns, filters,
  sorts, or the one profile being read.

## Non-goals

- User-created roles or editing built-in metrics and weights; JAY-21 owns that
  product capability and must reuse the scorer introduced here.
- Adding the missing attribute lists or formulas. That work is tracked in
  [JAY-31](https://linear.app/jaycount/issue/JAY-31/complete-missing-fm26-attribute-role-definitions).
- Combining attribute and Moneyball scores into one score.
- Adding minimum minutes, appearances thresholds, sample shrinkage, competition
  strength, team-possession, or other contextual adjustments.
- Recomputing or redefining JAY-19 raw values, null handling, percentile ranking,
  comparison-pool controls, or import replacement behavior.
- Splitting source definitions by left and right side when their linked metrics
  and weights are shared.
- Exposing unsupported attribute placeholders to General Search, Planner,
  tactics, or snapshot scoring.
- Persisting derived Moneyball role scores or backfilling existing imports.
- Publishing, committing, pushing, or opening a PR during planning.

## Current-state map

- Relevant components:
  - `src-tauri/src/features/moneyball/catalog.rs` owns 138 canonical statistic
    keys and 19 lower-is-better keys.
  - `src-tauri/src/features/moneyball/percentile.rs` converts raw cohort values to
    nullable 0-100 percentiles, preserves nulls, returns 50 for a constant
    population, and applies lower-is-better inversion.
  - `src-tauri/src/features/csv_import/service.rs` prepares the complete matched
    cohort outside the transaction and atomically replaces
    `player_moneyball_stats` for the current snapshot.
  - `src-tauri/src/features/moneyball/query.rs` reads one current player's raw
    statistics and persisted full-import percentiles.
  - `src-tauri/src/features/search/query.rs` selects either persisted full-import
    percentiles or recomputed filtered-cohort percentiles for requested raw
    Moneyball metrics.
  - `src-tauri/src/features/player_metrics/resolver.rs` validates dynamic Search
    fields against closed Rust catalogs before producing trusted SQL.
  - `src-tauri/src/features/search/filter.rs` compiles one flat AND/OR filter AST
    with at most 32 rules.
  - `src/utils/moneyball-search-metrics.ts` owns the synchronous Moneyball column
    and filter registry used by URL parsing and the metric picker.
  - `src/app/routes/players.$uid.tsx` owns General/Moneyball profile routing and
    currently renders only the raw Moneyball panel in Moneyball mode.
  - `src/features/player-profile/components/player-overview-panel.tsx` derives
    General best Current/Potential IP/OOP roles from playable positions.
  - `src/features/player-profile/components/player-roles-panel.tsx` renders the
    position pitch and Current/Potential attribute role table.
- Data model:
  - `player_moneyball_stats` is snapshot-owned, keyed by
    `(snapshot_id, player_uid)`, and stores canonical raw and full-import
    percentile JSON.
  - `player_role_scores` and `player_potential_role_scores` store only the 68
    supported attribute definitions.
  - Moneyball Search imports are limited to 1,000 CSV rows, so full candidate
    scoring is bounded independently of visible pagination.
- Persistence and migrations:
  - Migration 30 added nullable `percentiles_json`.
  - No migration is required. A legacy row without percentiles remains in the
    existing `needsReimport` state.
- Existing behavioral assumptions:
  - Moneyball Search defaults to the filtered comparison pool but retains an
    explicit Full CSV/Filtered control.
  - Raw Moneyball filters and sorts operate on raw values; percentile badges are
    display analysis.
  - Profile role summaries consider only positions with familiarity at least 15
    and keep catalog order for score ties.
  - Null scores already render as an em dash in the General role table.
- Architectural seams:
  - A pure role scorer can consume the same nullable percentile map used by both
    current Moneyball consumers.
  - The General profile query is the only consumer that needs the expanded
    presentation catalog. Attribute scoring and Planner can continue using
    `scoring::catalog::all_roles()` unchanged.
  - Search must introduce a bounded post-score path only when a Moneyball role
    appears in requested fields, filters, or sorting; ordinary General and raw
    Moneyball queries retain their current path.
- Project validation commands:
  - `./scripts/dev test [target...]`
  - `./scripts/dev format [paths...]`
  - `./scripts/dev check`
  - `./scripts/dev smoke`
  - `./scripts/dev release-metadata v0.9.0 none`
- Primary risks:
  - Transcribing 88 definitions and 440 metric weights incorrectly.
  - Accidentally applying inversion twice.
  - Treating numeric zero as missing during JSON or map handling.
  - Making a role-score filter recursively alter its own comparison population.
  - Computing filtered scores from the visible page rather than the full cohort.
  - Letting the expanded presentation inventory leak into Planner or persisted
    attribute scoring.
  - Duplicating role identity metadata inconsistently between Rust and React.

## Feature architecture

### Versioned built-in catalog

- Add a production catalog asset at
  `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` and a typed
  loader in `src-tauri/src/features/moneyball/role_catalog.rs`.
- The catalog top level owns `version: 1`; each definition owns a stable ID,
  display name, phase, position family, position tags, optional mapped attribute
  role ID, and five metric entries with canonical key, weight, and source
  inversion flag.
- Use the 88 Markdown definitions, not the legacy runtime JSON's later 85-row
  consolidation. In particular, keep ML/MR and AML/AMR Winger, Playmaking
  Winger, and Inside Winger entities separate even where version 1 is identical.
- Map supported presentation entries to the current attribute role IDs. Leave
  the 11 genuinely missing OOP definitions unmapped so General profile scores
  are explicitly null. Obvious label-order aliases map to the current score IDs
  rather than creating duplicate persisted attribute roles.
- The Rust loader is authoritative for weights and score validation. React gets
  a metadata-only mirror for synchronous Search field registration; tests pin
  version, count, IDs, family counts, phases, and position tags on both sides.

### Pure scoring contract

- Add `src-tauri/src/features/moneyball/role_score.rs` with a reusable scorer that
  accepts a validated definition and one player's nullable metric percentile
  map.
- Compute `round(sum(percentile * weight) / sum(weight))`, clamped to 0-100.
- Return `None` when the definition is invalid or any contributing percentile is
  null. Preserve `Some(0)` throughout.
- Produce explanation rows from the same calculation: metric key, weight,
  lower/higher direction, input percentile, and weighted contribution. Do not
  maintain a second explanation formula.
- The scorer accepts no minutes or appearances input. Rate and ratio calculation
  remains upstream in the imported canonical metrics.
- Keep the scorer independent of built-in storage so JAY-21 can pass a validated
  user definition through the same API.

### Profile composition and presentation inventory

- Extend the ready Moneyball profile query/DTO with catalog version and all 88
  role results composed from persisted full-import percentiles.
- Keep `noData` and `needsReimport` states unchanged and do not manufacture role
  results without a valid percentile map.
- Change the General player-profile query to iterate the presentation catalog.
  For a mapped entry, read the existing persisted Current score and calculate
  Potential with the mapped attribute definition. For an unmapped entry, return
  null for both. Do not add placeholder rows to score tables.
- In Moneyball mode, derive two best-role summaries from calculable IP/OOP roles
  attached to playable positions. There is no Potential Moneyball score.
- Render the existing Moneyball raw-statistics panel beside a Moneyball role-fit
  panel using the established profile workspace containment.
- Reuse the pitch-position, score badge, null formatting, sorting, and playable
  position helpers where their contracts fit. Add an accessible disclosure for
  each Moneyball definition instead of overloading the General Current/Potential
  table headers.

### Search cohort and post-score query path

- Register `moneyball_role.<stable-id>` as a closed integer metric source valid
  only in Moneyball Search.
- Collect the union of role IDs referenced by requested columns, role filters,
  and sorting. Collect only the canonical metric keys those definitions require.
- With `Full CSV`, compose roles from persisted percentile JSON.
- With `Filtered`, fetch contributing raw values for the complete base cohort,
  run the existing percentile engine once for the required metric union, and
  then run the pure role scorer.
- A flat `AND` filter with role rules uses its non-role rules as SQL prefilters;
  role rules are evaluated against the resulting scored cohort.
- A flat `OR` filter containing role rules scores the full import. The existing
  SQL compiler supplies membership for the non-role OR branch, while role-rule
  membership is evaluated after scoring; final membership is their union.
- When sorting by a Moneyball role, sort the scored bounded cohort in Rust,
  place nulls last in both directions, use player UID as the stable tie-breaker,
  then paginate. When sorting by an existing SQL field, preserve existing order
  while applying post-score membership before pagination.
- Set `total` after role-score predicates and before pagination.
- Leave the existing SQL-only path untouched when no Moneyball role is requested,
  filtered, or sorted.

### Frontend Search registry and rendering

- Add a metadata-only `src/utils/moneyball-role-catalog.ts` mirroring the backend
  version 1 identities.
- Append Moneyball role metrics to `MONEYBALL_SEARCH_METRICS`, grouped by source
  position family. Keep default Moneyball columns unchanged.
- Disambiguate repeated labels with phase and position tags, for example
  `Wing-Back (IP · DL/DR)` and `Wing-Back (IP · WBL/WBR)`.
- Render role dynamic values with `ScoreBadge`; keep raw metric values and their
  separate percentile badge behavior unchanged.
- Add concise filter-editor help when a Moneyball role rule is present: role
  rules apply after the comparison cohort is calculated.

## Uncertainty register

### Known

- JAY-19 is merged on `main` at planning time and supplies every required raw
  metric, inversion direction, full-import percentile, and filtered-cohort
  percentile seam.
- The source Markdown contains 88 role definitions across ten position families;
  every referenced role metric maps to one of 33 existing canonical metric keys.
- The source roles use four lower-is-better metrics, all already inverted by the
  current percentile engine.
- The current attribute catalog has 68 definitions. After mapping obvious label
  aliases, 11 generic OOP definitions have no attribute formula.
- JAY-31 now owns collecting and integrating those missing attribute lists.
- Repowise was unavailable during planning (`repowise: command not found`).
  Direct repository inspection and the current Codebase Memory index supplied
  the impact map; source files, tests, configuration, and Git remain authority.

### Assumptions

- The developer's DR-versus-WBR example means source position families remain
  distinct while symmetric left/right positions stay paired as documented.
- A missing contributing percentile makes the complete role unavailable; the
  developer's zero guidance does not authorize partial-weight renormalization.
- Profile explanation through metric/weight disclosures is sufficient for JAY-20;
  no separate analysis chart is required.
- The current 1,000-row import bound makes an in-memory post-score Search path
  proportionate when a role field requires it.

### Decisions

- Profile scores compare against the full imported cohort.
- Search scores follow the existing comparison-pool selection and filtered scores
  use the full filtered cohort, never the page.
- There is no minutes or appearances threshold and no sample adjustment.
- Zero is data, null is unavailable, and invalid zero-total definitions are
  rejected before division.
- Position-family definitions remain separate even when their version 1 metrics
  and weights are identical.
- The General profile shows the full presentation inventory with null placeholders;
  unsupported roles do not enter other attribute-score consumers.
- Scores are derived at read time from persisted percentiles; no migration or
  independent role-score cache is added.
- Mixed OR role filters use the full import as their comparison population to
  avoid recursive or premature exclusion.

### Unknowns

- No product decision blocks the first commit.
- Exact responsive disclosure density may need adjustment against the current
  profile height at implementation time; the behavior and accessibility contract
  are fixed, but spacing is not.
- Native desktop visual proof is not available during planning. Rust integration,
  React tests, browser smoke, keyboard use, and an actual 200% zoom pass remain
  the planned evidence.

### Risks

- The legacy runtime JSON consolidated three wide-role pairs and still contains
  other duplicate display identities. Copying it directly would violate the
  approved no-merge policy; the Markdown headings are the inventory authority.
- A metadata mismatch between the Rust and React catalogs could make a URL field
  appear selectable but fail backend validation.
- Post-score filtering can subtly produce incorrect totals or page boundaries if
  pagination remains in the pre-score SQL path.
- A General profile refactor could accidentally change Planner-facing IDs if it
  modifies `scoring::catalog::all_roles()` instead of adding a presentation map.

## Walking skeleton

Use one source definition to prove the whole contract before generalizing:

1. Parse and validate catalog version 1 and score one role from a complete
   persisted percentile map.
2. Return that derived score and its five contributions from the Moneyball
   profile query using the full import.
3. Render it in the Moneyball best-role summary and position role table while a
   mapped General row continues using its attribute score.
4. Request the same role as a Moneyball Search field, recompute it over a
   filtered cohort, and prove the value is independent of page size.
5. Generalize the same paths to all 88 validated definitions without introducing
   a second formula or persisted cache.

## Delivery plan

### PR 1 — Add Moneyball role scoring

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provider:** GitHub

**Base branch:** `main`

**Working branch:** `feature/moneyball-role-scores`

**Merge strategy:** Squash

**Release intent:** `none`

**Required check:** `check` (strict)

**CI repair rounds used:** 0 of 2

**Provisional PR title:** `feat(moneyball): add role scoring`

**Purpose:** Deliver one reviewable feature boundary containing the versioned
score contract and both requested consumers. Splitting publication would expose
an unused catalog or an incomplete Moneyball workspace without reducing a
deployment, migration, or compatibility risk.

**Depends on:** JAY-19 / commit `c7dfd98` on `main`; no unpublished branch.

#### Commit 1 — Define the built-in catalog and scorer

**Status:** Completed

**Provisional commit:** `feat(moneyball): define role score catalog`

**Work:** Add the complete version 1 position-family catalog, validation, pure
weighted scorer, and explanation model without changing IPC or UI behavior.

**Out of scope:**

- Player Profile and Search integration.
- Attribute catalog changes or JAY-31 formulas.
- Persistence, migrations, user editing, or sample thresholds.

**Implementation packet:**

- Transcribe all 88 source Markdown definitions from pinned commit `366aa20`.
  Use the Markdown inventory rather than the 85-row consolidated runtime JSON.
- Assign stable IDs containing position family, normalized role slug, and phase.
  Keep every source family separate; pair only the left/right position tags the
  source itself pairs.
- Record optional mappings to current attribute role IDs for presentation use.
  Leave the 11 missing generic OOP entries unmapped and map naming aliases to
  existing score IDs.
- Validate version, unique IDs, family counts, allowed phase/tags, exact five
  metrics per built-in, canonical metric keys, finite non-negative weights,
  positive totals, and inversion agreement with JAY-19.
- Implement one scoring function and derive explanation contributions from the
  same arithmetic. Treat `Some(0)` as calculable; return `None` for a missing
  input or invalid definition.
- Do not add minutes to the scorer and do not invert input percentiles.

**Files and responsibilities:**

- `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` — pinned
  version 1 definitions, identities, mappings, metric weights, and inversion
  metadata.
- `src-tauri/src/features/moneyball/role_catalog.rs` — typed loading, catalog
  lookup, definition validation, and catalog invariant tests.
- `src-tauri/src/features/moneyball/role_score.rs` — weighted score and
  explanation calculation with boundary tests.
- `src-tauri/src/features/moneyball/catalog.rs` — expose a narrow
  lower-is-better lookup for catalog cross-validation; do not change the set.
- `src-tauri/src/features/moneyball/mod.rs` — register the two new modules.

**Behavior and data flow:**

- The checked-in asset is compiled into Rust; there is no runtime file lookup or
  user-writable default at this stage.
- Catalog access validates the static data before consumers can request a role.
- Scoring receives a definition and one percentile map, verifies a positive
  denominator, stops on the first null input, and otherwise returns a rounded
  0-100 score plus contribution rows.
- Empty, unknown, non-finite, negative, zero-total, and inversion-mismatched
  definitions fail validation rather than producing a score.

**Ordered implementation steps:**

1. Add RED catalog tests for version 1, 88 identities, ten family counts,
   distinct DL/DR versus WBL/WBR identities, distinct ML/MR versus AML/AMR
   identities, canonical keys, weight totals, and inversion agreement.
2. Add RED scorer tests for a known weighted result, rounding boundaries,
   `Some(0)`, null input, and zero-total rejection.
3. Add the smallest catalog types, asset, validation, and scorer that turn the
   focused proofs GREEN.
4. Add an integration fixture proving a lower raw value becomes a higher role
   contribution exactly once through the existing percentile engine.
5. Refactor only while all catalog and scorer proofs stay green, then run format
   and the repository gate.

**Tests and proof:**

- Expected RED: the catalog/scorer modules and 88 stable identities do not exist.
- GREEN: exact catalog invariants pass; `[0, 0, 0, 0, 0]` percentiles score 0;
  a null metric returns unavailable; invalid zero total is rejected; constant
  raw zeros remain percentile 50 in the existing percentile proof.
- Include representative assertions from every family, with special coverage for
  the differing Full-Back/Wing-Back formulas and the four inverted source metrics.

**Patterns to verify:**

- `src-tauri/src/features/scoring/catalog.rs` — static catalog identity,
  position tags, and invariant-test style.
- `src-tauri/src/features/moneyball/catalog.rs` — canonical metric ownership.
- `src-tauri/src/features/moneyball/percentile.rs` — null, constant-population,
  zero, and inversion semantics to reuse rather than duplicate.
- `src-tauri/src/features/scoring/combine.rs` — small pure optional-score
  contract and boundary tests.

**Constraints and non-goals:**

- No second percentile algorithm, raw-rate calculation, dependency, migration,
  IPC command, or frontend mirror.
- Do not alter JAY-19's lower-is-better key set to make a transcription pass.
- Do not collapse duplicate definitions for convenience.

**Dependencies and sequencing:**

- JAY-19 must remain present on `main`.
- This commit is the only active work. Later commits consume its stable IDs and
  scorer; changing them after integration requires replanning affected packets.

**Validation:**

1. `./scripts/dev format`
2. `./scripts/dev check`

**Stop conditions:**

- A source heading cannot be mapped to one canonical metric key.
- A source definition has ambiguous positions, weights, direction, or phase.
- The developer's position-family decision would require splitting left and
  right beyond the source documents.
- Correct scoring would require changing JAY-19 percentile behavior or deriving
  a raw rate in this feature.
- The implementation needs a new dependency or persistence contract.

**Review mandate:**

- Verify all 88 source entities are present once and no cross-family merge
  occurred.
- Check that weights, phases, position tags, and inversion flags match the pinned
  source, especially the Full-Back/Wing-Back variants.
- Trace zero and null separately through deserialization, validation, scoring,
  rounding, and explanation output.
- Confirm inversion is validation-only at the composite layer and cannot run
  twice.
- Confirm invalid or zero-total definitions cannot divide by zero or panic.
- Confirm the scorer is reusable by JAY-21 without importing profile or Search
  concerns.

#### Commit 2 — Show profile role scores and General placeholders

**Status:** Active

**Provisional commit:** `feat(profile): show Moneyball role fit`

**Work:** Compose full-import Moneyball role scores for one profile, render best
IP/OOP summaries and a role-fit panel, and expand only the General profile's
presentation inventory with explicit attribute-score placeholders.

**Out of scope:**

- Search columns, filters, sorting, or filtered-cohort scores.
- New attribute formulas, score-table rows, Planner roles, or General Search
  fields.
- Potential Moneyball scores or combined scores.

**Implementation packet:**

- Extend only the ready Moneyball profile payload with catalog version and all
  role results calculated from persisted full-import percentiles.
- Include enough calculation detail for one accessible disclosure per role:
  metric key/label, weight, higher/lower direction, percentile, and weighted
  contribution. A null metric makes the score unavailable and is identified in
  the disclosure.
- Change the General profile query to emit the 88 presentation entries. Look up
  Current scores and calculate Potential through `attribute_role_id`; emit null
  for both when the mapping is absent. Keep persisted catalogs untouched.
- Reuse playable-position and tie behavior for two Moneyball best summaries.
- Keep the existing raw Moneyball panel. In ready state, restore the established
  two-panel profile workspace with raw metrics and Moneyball role fit. Preserve
  current no-data and re-import empty states.
- Use a Moneyball-specific score column and accessible details; do not relabel it
  Current or imply a Potential performance projection.

**Files and responsibilities:**

- `src-tauri/src/features/moneyball/query.rs` — full-import profile composition,
  ready-state role results, and integration tests.
- `src-tauri/src/features/moneyball/commands.rs` — versioned role and contribution
  DTOs with distinct field names.
- `src-tauri/src/features/player/query.rs` — presentation inventory mapping for
  General profile only, with null unsupported Current/Potential results.
- `src/features/moneyball/types/moneyball-profile.ts` — ready-state Moneyball
  role result types.
- `src/testing/moneyball-ipc-mock.ts` — representative full, null, and unavailable
  fixtures.
- `src/features/player-profile/types/player-detail.ts` — preserve the General
  presentation row contract while allowing presentation IDs.
- `src/features/player-profile/utils/position-families.ts` and test — reuse or
  minimally generalize best/playable/position sorting for one-score Moneyball
  rows without changing familiarity rules.
- `src/features/player-profile/components/player-overview-panel.tsx` — render
  General four-badge or Moneyball two-badge summaries from explicit role input.
- `src/features/player-profile/components/player-roles-panel.tsx` — retain
  General Current/Potential behavior and render mapped null placeholders.
- `src/features/moneyball/components/moneyball-role-fit-panel.tsx` and test —
  position picker, one score column, null state, sorting, and explanation
  disclosures.
- `src/features/moneyball/components/moneyball-profile-panel.tsx` and test — keep
  raw metrics/context focused; coordinate the ready profile content without
  duplicating score arithmetic.
- `src/app/routes/players.$uid.tsx` and test — ready two-panel layout, Moneyball
  summary inputs, and no-data/re-import behavior.

**Behavior and data flow:**

- `get_player_moneyball(uid)` resolves the active current snapshot and existing
  full-import percentile JSON, then calculates all definitions in catalog order.
- `get_player(uid)` still reads persisted attribute score rows, but presents them
  through the expanded inventory and calculates Potential only for mapped roles.
- React receives independent General and Moneyball role arrays. Moneyball mode
  never reads attribute role scores for its summary or role table.
- Best summaries skip null scores and roles outside playable positions. Ties keep
  catalog order. No-data and needs-reimport profiles expose unavailable summaries
  and the existing actionable empty state.

**Ordered implementation steps:**

1. Add RED Rust query tests for 88 full-import Moneyball results, a known score,
   a null contributing percentile, catalog version, and unchanged legacy states.
2. Add RED General player-query tests for mapped duplicate presentation rows and
   one unmapped role with null Current/Potential, while `all_roles()` remains 68.
3. Add RED route/component tests for Moneyball IP/OOP summaries, position-specific
   scores, metric/weight details, raw panel retention, General em dashes, keyboard
   disclosure, no data, and needs re-import.
4. Implement backend composition and DTOs, then make the smallest profile/UI
   changes that turn the tests GREEN.
5. Verify narrow and wide profile containment, refactor shared position helpers
   only where both modes use the exact same contract, and run the gate.

**Tests and proof:**

- Expected RED: Moneyball profile has no role results or role panel; General
  profile returns only 68 supported definitions.
- GREEN: a ready profile shows two Moneyball best scores and 88 selectable role
  entries, its raw metrics remain present, and an unsupported General OOP role
  reads `—` for Current and Potential.
- Negative coverage: null required percentile, no imported player, legacy import,
  no playable position, hidden attribute information, and duplicated display
  names in different position families.

**Patterns to verify:**

- `src-tauri/src/features/moneyball/query.rs` — current snapshot/no-data/re-import
  state machine and exact-catalog JSON parsing.
- `src-tauri/src/features/player/query.rs::load_role_scores` — persisted Current
  lookup plus query-time Potential calculation.
- `src/features/player-profile/components/player-roles-panel.tsx` — pitch,
  internal scroll owner, score badge, null, and sort conventions.
- `src/features/staff/components/staff-role-fit-panel.tsx` — one-score role-table
  presentation where applicable.
- `src/app/routes/players.$uid.tsx::profileWorkspaceClassName` — responsive
  two-panel containment.

**Constraints and non-goals:**

- `scoring::catalog::all_roles()` remains the supported attribute catalog and
  must not gain placeholders.
- No profile calculation uses filtered Search state or the current page.
- No hidden Potential attribute data is revealed through Moneyball scores.
- Explanation text must describe a weighted percentile score, not causal player
  quality or a combined role recommendation.

**Dependencies and sequencing:**

- Commit 1 IDs, mappings, validation, scorer, and explanation output are stable.
- Search work remains pending until this full-import consumer proves the scorer.

**Validation:**

1. `./scripts/dev test src/features/moneyball/components/moneyball-profile-panel.test.tsx src/features/moneyball/components/moneyball-role-fit-panel.test.tsx 'src/app/routes/players.$uid.test.tsx' src/features/player-profile/utils/position-families.test.ts`
2. `./scripts/dev format`
3. `./scripts/dev check`

**Stop conditions:**

- The General presentation map cannot remain isolated from Planner or persisted
  attribute scoring.
- Profile explanation requires duplicating score arithmetic in TypeScript.
- The ready two-panel workspace cannot contain all interaction at supported
  viewport/zoom without a product-level layout decision.
- A source identity cannot map unambiguously to an existing attribute role ID.

**Review mandate:**

- Trace one ready profile from persisted full-import percentiles through Rust
  scoring, IPC, best summary, table score, and explanation details.
- Verify Moneyball mode contains no attribute or Potential score substitution.
- Verify the 88-row General presentation map cannot expand ingest, Search,
  Planner, tactic, or materialization catalogs.
- Check null, zero, no-data, and needs-reimport states remain distinct and
  accessible.
- Check duplicate labels are selected by position identity, not array accident.
- Verify one internal scroll owner and keyboard/focus behavior for the added
  disclosures.

#### Commit 3 — Query, filter, and sort Search role scores

**Status:** Pending

**Provisional commit:** `feat(search): query Moneyball role scores`

**Work:** Add the trusted backend field, comparison-pool computation, post-score
filtering, sorting, totals, and pagination needed by Moneyball role fields.

**Out of scope:**

- Frontend picker, columns, filter help, or browser smoke.
- Persisted role-score caches, custom definitions, or changes to raw Moneyball
  filter semantics.

**Implementation packet:**

- Parse `moneyball_role.<id>` only in Moneyball view and only for a validated
  version 1 catalog ID. Expose it as a nullable integer dynamic value.
- Collect referenced role IDs from requested fields, sort, and filter rules; load
  only their union of metric keys.
- Preserve the existing query path byte-for-byte where no role field is involved.
- Add a role-aware bounded path following the Feature architecture's Full CSV,
  Filtered, AND, and mixed-OR population rules.
- Apply role predicates after scoring with the existing numeric operator meaning.
  Null never matches, including `neq`, matching persisted attribute role filters.
- When role sorting is selected, place unavailable scores last in ascending and
  descending order, keep UID as the deterministic tie-breaker, and paginate only
  after sorting and post-score filtering.
- Return `total` for the post-score result set and attach requested role scores to
  `dynamic_values`; keep individual metric percentiles in their existing map.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/resolver.rs` — closed Moneyball role
  source, ID validation, kind, and requested-field inspection.
- `src-tauri/src/features/search/filter.rs` — recognize role rules, partition
  post-score predicates, compile non-role membership, and retain the 32-rule
  trust boundary.
- `src-tauri/src/features/search/query.rs` — comparison-cohort loading, metric
  percentile reuse/recalculation, score application, post-score membership,
  role ordering, totals, pagination, and integration tests.
- `src-tauri/src/features/search/commands.rs` — preserve the existing request and
  response shape while serializing role scores through dynamic values.
- `src-tauri/src/features/moneyball/role_score.rs` — add only batch helpers that
  remain pure and shared if query integration proves they remove duplication.

**Behavior and data flow:**

- Ordinary queries continue through current SQL filtering, sorting, and paging.
- A role-aware query establishes a bounded candidate/comparison set before
  pagination, obtains persisted or recalculated metric percentiles, scores only
  referenced definitions, evaluates role rules, orders, counts, and slices.
- AND non-role filters narrow the filtered population. Role rules do not feed
  back into percentile calculation.
- Mixed OR queries score the full imported cohort, union SQL non-role membership
  with post-score role membership, and then sort/page.
- Replacement import behavior needs no invalidation call because every result is
  derived from the current `player_moneyball_stats` rows.

**Ordered implementation steps:**

1. Add RED resolver/filter tests for valid and invalid IDs, Moneyball-only
   availability, role-rule partitioning, numeric operators, and mixed AND/OR.
2. Add RED integration tests proving Full CSV composition, Filtered cohort
   recomputation across pages, AND post-score population, mixed OR full-cohort
   fallback, zero/null handling, role sort, null-last behavior, stable ties,
   post-filter total, and pagination.
3. Implement the smallest catalog-aware field and bounded role query path that
   turns the tests GREEN while leaving ordinary query tests unchanged.
4. Perturb a weight, comparison pool, page limit, and one post-score operator to
   prove the tests fail for the intended contract.
5. Refactor query helpers only while the full Search test module and gate remain
   green.

**Tests and proof:**

- Expected RED: role fields are rejected as unknown and Search cannot filter or
  sort by a derived role score.
- GREEN: the same player can have a different Full CSV and Filtered role score;
  Filtered results are invariant across page sizes; a score filter changes the
  result set but not its own comparison scores; null is last and never matches.
- Boundary coverage: empty cohort, one-player/constant cohort, raw zero, missing
  percentile, all-role-only filter, mixed OR, offset beyond total, unknown role,
  General-view rejection, and existing raw-only query parity.

**Patterns to verify:**

- `src-tauri/src/features/search/query.rs::filtered_moneyball_percentiles` — full
  filtered cohort calculation and current raw metric response mapping.
- `src-tauri/src/features/search/query.rs` potential-role materialization path —
  dynamic role collection before query execution, without copying its cache.
- `src-tauri/src/features/search/filter.rs` persisted role-score operators and
  null membership semantics.
- `src-tauri/src/features/player_metrics/resolver.rs::MetricSource` — closed
  source validation and trusted expression boundary.

**Constraints and non-goals:**

- Never interpolate a WebView-supplied role ID or metric key into SQL.
- Never calculate a filtered score from only selected rows or the visible page.
- Do not change raw Moneyball filter or individual-percentile behavior.
- Do not add a second role formula, percentage direction set, cache, migration,
  or unbounded query.

**Dependencies and sequencing:**

- Commits 1 and 2 fix catalog IDs, scorer behavior, and profile/full-import
  meaning before filtered Search adds population complexity.
- Frontend fields remain unavailable until Commit 4, so this backend addition is
  reachable only through tests/direct IPC during the intermediate commit.

**Validation:**

1. `./scripts/dev format`
2. `./scripts/dev check`

**Stop conditions:**

- Correct role filtering requires a recursive/fixed-point comparison population.
- Mixed OR semantics cannot be implemented without duplicating all raw filter
  evaluation or weakening the current SQL trust boundary.
- The current 1,000-row bound is bypassed or a query can score an unbounded
  player set.
- Existing raw-only Moneyball query results, totals, ordering, or pagination
  change.
- A schema/cache appears necessary for acceptable measured performance.

**Review mandate:**

- Trace Full CSV, Filtered AND, and mixed Filtered OR populations separately and
  verify none use page-limited rows.
- Verify score filters are post-score and cannot recursively improve or shrink
  their own population.
- Check total, offset, null ordering, UID ties, and ordinary-sort ordering after
  post-score membership.
- Verify every dynamic ID and metric key crosses a closed catalog before SQL or
  JSON-path construction.
- Compare raw-only query plans/results before and after to confirm the new path is
  opt-in.
- Check zero, null, constant, empty, and denominator boundaries.

#### Commit 4 — Expose Search role columns and filters

**Status:** Pending

**Provisional commit:** `feat(search): expose Moneyball role columns`

**Work:** Register and render all Moneyball role fields in Search, add role-filter
guidance, and prove the complete Search/Profile workflow in browser smoke.

**Out of scope:**

- Making role columns default.
- Built-in editing, custom roles, combined scores, or attribute role fields for
  unsupported placeholders.
- New visualizations or a redesign of the metric picker/table.

**Implementation packet:**

- Add a versioned metadata-only React catalog with the exact 88 backend IDs,
  labels, phases, families, and position tags. Pin parity through invariant tests.
- Append one integer Search metric per role, grouped by source family, with a
  distinct `moneyball_role.` prefix and disambiguated label.
- Reuse the current column picker, filter editor, URL persistence, dynamic column
  collection, virtualization, and numeric operators.
- Render role cells as `ScoreBadge` using a Moneyball-role-specific accessible
  name. Render null as an em dash. Do not format the value as a raw Moneyball
  metric or label it as an individual percentile.
- Add filter-editor guidance when role rules are present and keep default columns
  unchanged.
- Extend the 101+ row smoke fixture to prove a selected role column, score filter,
  role sort, filtered comparison state, navigation to a Moneyball profile, raw
  panel retention, best IP/OOP badges, and role-fit disclosure.

**Files and responsibilities:**

- `src/utils/moneyball-role-catalog.ts` and test — frontend identity metadata and
  parity invariants.
- `src/utils/moneyball-search-metrics.ts` and test — role metric registration,
  grouping, labels, numeric operators, widths, and unchanged defaults.
- `src/features/search/utils/filter-registry.ts` and tests where needed — consume
  role entries without a second registry.
- `src/features/search/components/search-results-panel.tsx` — score-badge/null
  rendering and accessible distinction from raw metric percentiles.
- `src/features/search/components/search-filter-editor-modal.tsx` and test —
  post-score cohort guidance.
- `src/features/search/utils/dynamic-columns.ts` and test — ensure role filter and
  sort fields request their dynamic values.
- `src/features/search/types/player-summary.ts` — document derived role values in
  the existing dynamic map; no parallel response map.
- `src/app/routes/search.test.tsx` — picker, request payload, score rendering,
  filter/sort URL state, comparison-pool differences, and default-column proof.
- `src/testing/search-ipc-mock.ts` — representative role values and requests.
- `e2e/tauri-ipc-stub.ts` and `e2e/smoke.spec.ts` — 101+ row integrated browser
  path and profile transition.

**Behavior and data flow:**

- The synchronous frontend registry admits only catalog IDs and passes them in
  requested fields/filter/sort using the backend-owned prefix.
- The route/query key already includes view, comparison pool, filters, sort,
  fields, and pagination; changing any relevant state requests a fresh score.
- The result table reads role scores from `dynamicValues`; individual raw metric
  percentiles remain in `moneyballPercentiles`.
- Clicking a player preserves explicit Moneyball profile navigation and the
  profile independently recomputes against the full import.

**Ordered implementation steps:**

1. Add RED metadata/registry tests for 88 exact IDs, position-family groups,
   duplicate-label disambiguation, numeric operators, and unchanged defaults.
2. Add RED route/component tests for role picker selection, requested field,
   ScoreBadge/null rendering, filter/sort IPC and URL state, help text, and a
   comparison-pool score change.
3. Implement the metadata mirror, registry entries, rendering, and help text with
   no new table abstraction.
4. Extend the existing Moneyball smoke fixture and scenario; verify virtualization
   still uses one internal scroll owner with 101+ rows.
5. Run focused tests, format, the full gate, and browser smoke.

**Tests and proof:**

- Expected RED: role fields are absent from the picker/filter registry and their
  dynamic values render as neutral raw cells.
- GREEN: both Wing-Back position-family fields are independently selectable;
  role score 0 renders as a valid weakest score, null renders `—`, filters/sorts
  reach IPC, and raw percentile presentation remains unchanged.
- Browser proof covers keyboard selection, filtered comparison, role sorting,
  virtualization, profile navigation, profile summary, role table, details, and
  Back restoration.

**Patterns to verify:**

- `src/utils/player-metrics.ts` and `src/utils/role-catalog.ts` — current role
  family grouping and synchronous field identity mirror.
- `src/utils/moneyball-search-metrics.ts` — raw Moneyball metric registration and
  unchanged defaults.
- `src/features/search/components/search-results-panel.tsx` — current role and raw
  metric cell rendering branches.
- JAY-19 scenarios in `src/app/routes/search.test.tsx` and `e2e/smoke.spec.ts` —
  comparison-pool, virtualization, URL state, and profile navigation.

**Constraints and non-goals:**

- Do not add all 88 roles to default columns or General Search.
- Do not calculate, invert, or explain scores in TypeScript.
- Preserve accessible picker grouping, keyboard behavior, virtualized table
  containment, and URL limits.
- Keep one role identity catalog mirror; do not copy the list into individual
  components.

**Dependencies and sequencing:**

- Commit 3 backend IDs and query behavior are final.
- This is the final implementation commit. After its checkpoint clears, move the
  feature to Validation and run `$workflow-finish-feature` before publication.

**Validation:**

1. `./scripts/dev test src/utils/moneyball-role-catalog.test.ts src/utils/moneyball-search-metrics.test.ts src/features/search/components/search-filter-editor-modal.test.tsx src/features/search/utils/dynamic-columns.test.ts src/app/routes/search.test.tsx`
2. `./scripts/dev format`
3. `./scripts/dev check`
4. `./scripts/dev smoke`

**Stop conditions:**

- Frontend and backend IDs cannot be kept in deterministic parity without a new
  source-generation or catalog-fetching architecture.
- Adding role fields breaches the 256 requested-field bound or existing URL
  parsing limits.
- Role rendering introduces a second table scroll owner or breaks 101+ row
  virtualization.
- Browser proof disagrees with the backend population contract or profile uses
  filtered Search scores.

**Review mandate:**

- Compare all frontend IDs, phases, families, and position tags with the backend
  catalog and verify duplicate labels are distinguishable.
- Trace picker/filter/sort/URL/query-key/request/response/render for one role.
- Verify role score 0, null, raw metric value, metric percentile, and attribute
  role score remain visually and semantically distinct.
- Verify default columns and ordinary raw Moneyball interaction remain unchanged.
- Verify 101+ row virtualization, one scroll owner, keyboard picker/filter use,
  profile navigation, and Back restoration.
- Check that role-filter guidance accurately describes AND and mixed OR behavior.

## Active work

**PR:** PR 1 — Add Moneyball role scoring

**Commit:** Commit 2 — Show profile role scores and General placeholders

### RED proof

Add Rust query tests for the 88 full-import Moneyball role results and add
General profile tests for mapped and unmapped presentation rows. Add route and
component tests for summaries, disclosures, raw-panel retention, and the ready,
no-data, and re-import states. They must fail because the profile has no role
results or role-fit panel yet.

### Expected outcome

The ready Moneyball profile composes full-import role scores and explanations,
while the General profile shows the complete presentation inventory with null
Current/Potential values for unsupported attribute roles. Search and persisted
attribute consumers remain unchanged.

### Explicit exclusions

- Search columns, filters, sorting, and filtered-cohort scores.
- Attribute definition work from JAY-31.
- Database, migration, custom-role, sample-size, combined-score, and Potential
  Moneyball work.

## Discoveries and replanning

- 2026-08-20 — The current JAY-19 percentile engine already applies all source
  inversion directions, including red cards and goals conceded generally and
  the four inverted metrics actually referenced by the linked roles. JAY-20 will
  validate direction metadata but not invert twice.
- 2026-08-20 — The linked Markdown has 88 definitions. The legacy runtime JSON
  has 85 because it consolidated three identical ML/MR versus AML/AMR definitions;
  the developer explicitly rejected cross-position-family merging, so Markdown
  identities govern version 1.
- 2026-08-20 — Profile uses the full import. Search follows Full CSV/Filtered and
  filtered scores use the complete filtered cohort.
- 2026-08-20 — No minutes threshold or shrinkage. Zero remains data; null remains
  unavailable; zero-total definitions are invalid.
- 2026-08-20 — The General profile will show the complete inventory with null
  unsupported scores. JAY-31 owns the 11 missing attribute formulas, while
  Planner and other attribute consumers remain on the supported catalog.
- 2026-08-20 — Read-time derivation removes migration, cache invalidation, and
  legacy-row backfill work while preserving deterministic replacement behavior.
- 2026-08-20 — The pinned Markdown label `Shots on Target Ratio` is preserved
  as source metadata and explicitly maps to the existing `shots_on_target_per_90`
  canonical key; the JAY-19 catalog remains unchanged.

## Completed work

- Commit 1 — Define the built-in catalog and scorer — Pending record.

## Final validation

1. Every commit has passed its focused tests, `./scripts/dev format`,
   `./scripts/dev check`, and a fresh commit review with no unresolved Critical,
   High, or Medium findings.
2. Run `./scripts/dev check` against the complete feature state.
3. Run `./scripts/dev smoke` with the 101+ Moneyball fixture.
4. Run `./scripts/dev release-metadata v0.9.0 none`; if a newer published tag
   exists at publication time, update this command before running it rather than
   validating stale release metadata.
5. Manually verify at actual 200% browser zoom and a narrow viewport:
   - Moneyball raw and role panels remain contained with one internal scroll
     owner each.
   - Position selection, score sorting, and every explanation disclosure are
     keyboard reachable with visible focus.
   - General unsupported rows and Moneyball null scores read as em dashes.
6. Verify one representative real imported fixture through Rust tests: profile
   uses full-import percentiles; filtered Search is page-invariant; replacement
   import changes derived results without a role-score cache.
7. Confirm native Tauri packaging is not claimed by browser smoke. No packaging
   proof is required because this feature changes no permissions, installer,
   updater, bridge, or platform API.
8. Run feature-complete review and documentation reconciliation before preparing
   the template-complete draft PR with release intent `none`.

## Documentation impact

Complete during reconciliation after implementation is true:

- `.wiki/ARCHITECTURE.md` — record the versioned Moneyball role catalog,
  read-time weighted composition, full/filtered population boundary, and absence
  of a derived score cache.
- `.wiki/DESIGN.md` — record Moneyball role columns/filters, score distinction,
  profile best-role replacement, raw-plus-role panel layout, and explanation
  interaction.
- `.wiki/CONCEPT.md` — update only if its Moneyball product boundary needs to
  name composite role analysis; do not duplicate formulas.
- `.wiki/TODO.md` — move JAY-20 from Active to Completed and retain JAY-31 in
  Linear rather than duplicating its attribute-list backlog here.
- Reconcile and archive this ledger under `.wiki/features/completed/` during
  `$workflow-finish-feature`.
