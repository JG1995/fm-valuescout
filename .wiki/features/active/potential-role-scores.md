# Potential Role Scores

## Status

Active

## Intent

Project how well a player can perform each existing tactical role after developing from current ability (CA) to potential ability (PA), while preserving the current role score as the measure of present fit. The projection adapts FMSuperScout's position-sensitive attribute-growth model, deliberately applies its otherwise-unused mental growth factor, and feeds the projected visible attributes through FM ValueScout's existing role scorer. It does not introduce a separate meta score or a probability that the player reaches PA.

## User-visible behavior

- The player profile Roles tab shows a current score and a potential score for every role in the existing catalog.
- The Squad Planner shows a current combined score and a potential combined score for each resolved assignment, using only that lane's selected in-possession and out-of-possession roles and its saved phase weight.
- Current and potential scores are visibly and accessibly distinguished; an unavailable score renders as an em dash rather than zero.
- Loading FM data does not gain another all-player scoring pass. Potential scores are calculated when the profile or Planner read model is requested and then benefit from the existing TanStack Query cache.
- A player whose PA is not greater than CA has a potential role score equal to the current role score.

## Invariants

- The existing role catalog, primary/secondary bands, 75/25 role weighting, 0-100 scale, and Planner phase-weight combination remain authoritative.
- Potential role scores use projected visible attributes only. Hidden attributes, personality, and staff attributes do not participate.
- Attribute growth is position-sensitive and positive-only. Projected values are rounded to integers, capped at 20, and never below current values.
- Physical growth is damped by age using the adopted SuperScout thresholds. Mental growth is multiplied by `mentalGrowthFactor`: `1.0` below age 28, `1.15` from 28 through 31, and `1.25` from age 32.
- Unknown age uses neutral physical and mental multipliers (`1.0`); it must not erase position-profile growth or invent age damping.
- Multiple recognized natural position groups contribute the mean projected delta. Empty or unrecognized positions use the model's `ALL` profile.
- Missing or null visible attributes remain null. A role that requires a missing or null attribute has no potential score.
- The Planner applies the same lane roles and `ip_weight` to both current and potential combined scores.
- Current scores remain the persisted ingest-time values in `player_role_scores`; no current-score behavior changes.
- Potential projection and scoring run in Rust. React displays returned values and does not reproduce the formula.
- Potential scores are derived values. They are not persisted and require no database migration.

## Non-goals

- A meta score, future current-ability estimate, development probability, expected-value score, or odds that a player reaches PA.
- Projected-attribute tables or charts in the player profile.
- Potential-score search filters, sorting, global search results, exports, or snapshot history.
- Potential scores in the Planner slot picker, candidate list, optimizer objective, or automatic assignment decisions.
- Changes to the profile Overview best-role hero, which continues to describe current ability.
- Changes to Load Data, the bridge dump schema, memory scanning, snapshot ingest, or persisted score tables.
- Configurable projection coefficients, growth curves, position profiles, or mental-growth factors.

## Current-state map

- **Relevant components:** `src-tauri/src/features/scoring/` owns the 68-role catalog, current role scorer, and Planner combination helper; `src-tauri/src/features/player/query.rs` assembles the profile read model; `src-tauri/src/features/planner/depth.rs` resolves assignments and their selected-role combined score; `src/features/player-profile/` and `src/features/planner/` render those read models.
- **Data model:** current-snapshot players already expose required `ca` and `pa`, nullable `age`, natural positions, and nullable visible attributes. Profile role rows currently return one `score`; Planner assignments currently return one `combined_score`.
- **Persistence and migrations:** snapshot ingest synchronously computes all current role scores into `player_role_scores`. Potential values have no storage and need no migration.
- **Existing behavioral assumptions:** profile Roles lists the full catalog grouped by position family. A Planner lane selects one IP role, one OOP role, and an IP weight. One player UID can resolve to at most one assignment in a save.
- **Architectural seams:** Rust owns business logic and SQLite access behind existing `get_player` and `get_planner_depth` IPC commands. TanStack Query owns frontend server-state caching and invalidation after Load Data or save changes.
- **Project validation commands:** `./scripts/dev test [target...]`, `./scripts/dev format [paths...]`, `./scripts/dev check`, and `./scripts/dev smoke` are the applicable stable surfaces. Bridge validation is not implicated.
- **Primary risks:** incorrectly transcribing the empirical position profiles; accidentally leaving the mental multiplier inert like the upstream implementation; applying projection to every player during ingest; inconsistent profile and Planner formulas; and making the dense Planner matrix unreadable.
- **Planning evidence:** direct repository inspection is authoritative because the Repowise index was stale at planning time (`4ad07c4` versus worktree `79c08a7`).

## Feature architecture

Add one pure Rust projection module under `src-tauri/src/features/scoring/`. It owns the CA anchors, empirical position/attribute profiles, FM-position-to-profile-group mapping, interpolation or extrapolation, physical and mental age factors, multiple-position averaging, null preservation, rounding, and the 20-point cap. The empirical data and behavior are adapted from FMSuperScout at commit [`0f270d39`](https://github.com/mavarobli/FMSuperScout/blob/0f270d39a9cdc850ddfe653710d4904f13709cb5/app/app.js#L2738-L2808), with provenance retained in code alongside the existing project permission record in `.wiki/notes/superscout-permission.md`.

Projection is deliberately evaluated on bounded read paths:

1. `get_player` projects one player's visible attributes once and scores all 68 catalog roles. It returns `score` and `potential_score` on each existing role row.
2. `get_planner_depth` projects each resolved assigned player once, scores only the assignment lane's selected IP/OOP role pair, combines them with the existing `ip_weight`, and returns `combined_score` plus `potential_combined_score`.

This avoids extending the already-expensive Load Data path, avoids storing derived values that would become stale when the formula changes, and keeps navigation work bounded. A profile request performs one projection plus 68 cheap score calculations. A Planner request performs at most one projection plus two score calculations for each uniquely resolved assigned player. Existing query caching prevents repeated IPC work during unchanged navigation.

The existing IPC commands, query keys, and invalidation behavior remain in place. The frontend extends existing DTOs and renders the extra values with the existing score badge primitive. No new command, database table, background worker, or frontend calculation layer is introduced.

## Uncertainty register

### Known

- The upstream model uses CA anchors `[80, 110, 140, 170]` and position-specific per-attribute profiles, interpolating between anchors and extrapolating outside them.
- Its per-attribute delta is the mean across applicable position groups of `max(0, profile(PA) - profile(CA))`.
- Its position grouping is GK; central defender; fullback/wingback; defensive midfield; central midfield; wide midfield/wing; attacking midfield; and striker, with `ALL` as fallback.
- Its physical factors are `1.0` through age 23 or when age is unknown, `0.55` at 24-26, `0.30` at 27-29, `0.12` at 30-32, and `0.05` above 32.
- FMSuperScout defines `mentalGrowthFactor` as `1.15` from age 28 and `1.25` from age 32 but does not call it in its current projection implementation.
- The current schema already contains every input required by the projection and the current read models already cross the necessary Rust-to-React seams.

### Assumptions

- The natural-position keys retained by the bridge are the correct position inputs for the model; position proficiency values do not weight one recognized group more heavily than another.
- Read-time projection remains imperceptible at the bounded profile and assignment counts. This assumption must be checked with focused timing evidence before persistence or batching is considered.
- In the Planner, “the selected role” means the lane's configured IP/OOP role pair expressed through the same combined-score semantics already shown in the assignment cell.

### Decisions

- Apply `mentalGrowthFactor` to mental-attribute deltas. This is an intentional, tested divergence from upstream's currently inert helper.
- Treat `PA <= CA` as an identity projection so potential score equals current score, rather than returning no projection as upstream does.
- Recalculate potential values on existing Rust read paths and rely on existing TanStack Query caching. Do not add ingest work or persistence without measurement showing the bounded reads are a problem.
- Keep current and potential role scoring on the same catalog and scoring functions so the only changed input is the projected attribute map.
- Show compact Current/Potential treatments in existing role rows and assignment cells rather than adding a profile tab or Planner column family.
- Keep upstream commit provenance next to the empirical coefficients and projection behavior.

### Unknowns

- The exact native desktop query duration for a profile and a fully populated Planner is not yet measured. Commit validation must capture enough evidence to confirm that read-time projection does not create visible navigation lag.
- The smallest legible two-score treatment in the Planner at the supported 1280x800 viewport must be confirmed in browser and native-desktop evidence rather than assumed from markup alone.

### Risks

- A coefficient transcription error can produce plausible but wrong potential scores. Golden fixtures and boundary tests must fail on such drift.
- Profile and Planner call sites could convert or default nulls differently. Backend and UI tests must preserve unavailable scores as unavailable.
- Re-projecting the same assigned player in multiple query branches could add avoidable work. The Planner read model must reuse one projection per resolved assignment/player within a request.
- Adding score labels can widen Planner cells or obscure player names. Responsive smoke evidence must cover the minimum supported viewport.

## Walking skeleton

A Rust fixture player with CA below PA is projected through one recognized position profile, its projected visible attributes are scored by the existing role scorer, and the resulting potential role score appears beside the current score on one profile role row. The same projection service then supplies one resolved Planner assignment's selected IP/OOP role pair and displays its potential combined score beside the existing current combined score.

## Delivery plan

### PR 1 — Potential role scores

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(scoring): add potential role scores`

**Branch:** `feature/potential-role-scores`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** Squash

**Required checks:** Strict — required `check` status must pass

**Feature close-out:** Not run

**CI repair rounds:** 0

**Build-feature-loop profile:** Terra xhigh

**Purpose:** Deliver one reviewable vertical feature: a tested projection engine, full profile role-score presentation, and selected-role Planner presentation. No migration or independently deployable foundation justifies splitting the shared contract across multiple PRs.

**Depends on:** The schema-v6 SuperScout parity and scan-hardening foundation merged to `main` through `b9ff83b`. The `feature/potential-role-scores` branch starts from that synchronized commit.

#### Commit 1 — Project visible attributes to player potential

**Status:** Completed

**Provisional commit:** `feat(scoring): project attributes to player potential`

**Work:** Add the pure Rust projection model, pinned empirical profile data with provenance, public scoring-module entrypoint, and golden plus boundary tests. Mental growth must be actively applied.

**Out of scope:**

- Player-profile and Planner DTO or UI changes.
- Database, ingest, query, or IPC changes.
- Probability, expected-value, and projected-attribute UI concepts.

**Implementation packet:**

- **Owners and files:** add focused projection source under `src-tauri/src/features/scoring/` and expose it from `src-tauri/src/features/scoring/mod.rs`; keep algorithm tests beside the module. Split static empirical profiles from the algorithm only if their size otherwise obscures review.
- **Existing patterns to verify:** nullable visible-attribute representation in `score.rs` and player query parsing; role catalog attribute keys; existing FMSuperScout provenance conventions in ADR-0016 and `.wiki/notes/superscout-permission.md`.
- **Constraints and invariants:** deterministic pure function; no SQLite or Tauri dependency; positive-only deltas; neutral unknown-age factors; mental thresholds at 28 and 32; multi-position mean; `ALL` fallback; identity for `PA <= CA`; null preservation; integer rounding and cap 20.
- **Dependencies and ordering:** this commit establishes the sole projection contract consumed by both later read-model commits. No UI work may duplicate or anticipate its formula.

**Implementation profile:** Terra xhigh — the algorithm is bounded, but the large empirical coefficient surface and deliberate upstream divergence require careful transcription and exhaustive boundary reasoning.

**Review profile:** Sol High — a plausible coefficient or threshold defect would silently mis-rank every downstream potential score, and review must compare implementation, attribution, and golden evidence.

**Validation:** Run the focused Rust scoring tests during RED/GREEN, then `./scripts/dev format` and `./scripts/dev check`. Evidence must include a young-player golden fixture matching the pinned upstream behavior and separate fixtures proving the active mental multiplier divergence at ages 28 and 32.

**Stop conditions:** Stop and replan if required profile data is absent from the pinned source, attribute names do not map unambiguously to the current catalog, implementation requires bridge/schema changes, or implementation requires commits that are not part of the synchronized `main` base.

**Review mandate:**

- Verify every CA anchor, position group, attribute key, and coefficient against the pinned source.
- Verify interpolation/extrapolation and positive-only delta behavior at and around every anchor.
- Verify physical age boundaries and that mental factors are actually applied at ages 28 and 32.
- Verify multiple positions average deltas and empty/unrecognized positions use `ALL`.
- Verify `PA <= CA`, null attributes, rounding, and cap behavior preserve the stated invariants.
- Verify the module remains pure and introduces no ingest, persistence, or frontend scoring path.

#### Commit 2 — Show potential scores for every profile role

**Status:** Active

**Provisional commit:** `feat(profile): show potential scores for every role`

**Work:** Extend the existing player read model and frontend type with `potential_score`, project the requested player once, score all catalog roles, and render Current/Potential values on every Roles-tab row with responsive and accessible labels.

**Out of scope:**

- Changes to the Overview best-role hero or other profile tabs.
- Potential search filters, sorting, new navigation, or projected attributes.
- Planner behavior.

**Implementation packet:**

- **Owners and files:** `src-tauri/src/features/player/query.rs` and its tests; `src/features/player-profile/types/player-detail.ts`; `src/features/player-profile/components/player-roles-panel.tsx`; `src/app/routes/players.$uid.test.tsx`; `src/testing/player-ipc-mock.ts`; extend `e2e/smoke.spec.ts` only where populated profile evidence is needed.
- **Existing patterns to verify:** `load_role_scores` catalog merge and score ordering; `ScoreBadge` variants and accessible names; current role-family grouping; player query option caching and Load Data invalidation.
- **Constraints and invariants:** one projection per profile request; all 68 role rows retain current scores; potential uses the same role definition; null remains an em dash; Current/Potential names are not conveyed by color alone; overview remains current-only.
- **Dependencies and ordering:** depends on Commit 1's projection API. Extend the existing IPC response rather than adding a command or query key.

**Implementation profile:** Luna Max — this is a bounded extension of established player-query, DTO, mock, and Roles-panel patterns with no new architecture.

**Review profile:** Sol Medium — consequence is localized to one read model and panel, but review must catch missing catalog rows, accidental current-score replacement, null coercion, and accessibility regressions.

**Validation:** RED with a Rust player-query fixture and `./scripts/dev test 'src/app/routes/players.$uid.test.tsx'`; GREEN with focused Rust and route tests; then `./scripts/dev format`, `./scripts/dev check`, and `./scripts/dev smoke`. Evidence must show a populated profile with all role rows exposing distinct accessible Current and Potential values and an unavailable potential rendered as an em dash.

**Stop conditions:** Stop and replan if projection requires a second player query, the IPC shape cannot remain backward-compatible within the feature branch, all-role scoring creates measurable visible profile latency, or a legible two-value role row requires redesigning profile navigation.

**Review mandate:**

- Verify one player projection is reused across every catalog role.
- Verify current persisted scores are unchanged and every catalog role still appears in the same order/group.
- Verify `potential_score` follows the projection engine and does not fall back to current except for identity projection.
- Verify null values remain semantically unavailable in Rust, TypeScript, mocks, and rendered output.
- Verify Current/Potential labels and score badges have distinct accessible names.
- Verify existing Overview and Attributes behavior is untouched.

#### Commit 3 — Show potential score for assigned Planner roles

**Status:** Pending

**Provisional commit:** `feat(planner): show potential score for assigned roles`

**Work:** Extend resolved Planner assignments with `potential_combined_score`, compute it from only the lane's selected IP/OOP roles and saved phase weight, and display it compactly beside the current combined score in the depth matrix.

**Out of scope:**

- Potential scores in slot candidates, the slot picker, or search.
- Optimizer ranking, allocation, or preferred-foot behavior.
- Tactic editing, assignment persistence, or database changes.

**Implementation packet:**

- **Owners and files:** `src-tauri/src/features/planner/depth.rs`, `src-tauri/src/features/planner/depth_tests.rs`, and test support as required; `src/features/planner/types/depth.ts`; `src/features/planner/components/planner-depth-table.tsx`; `src/app/routes/planner.test.tsx`; `src/testing/planner-ipc-mock.ts`; `e2e/smoke.spec.ts` for populated responsive evidence.
- **Existing patterns to verify:** `resolve_assignment` state branches; `combine_role_scores`; lane lookup and `ip_weight`; resolved, outside-pool, and unresolved assignment semantics; depth-table density and score-badge accessibility.
- **Constraints and invariants:** project once per resolved assigned player within a request; score only the selected lane role pair; combine current and potential with the same weight; unresolved assignments expose neither score; resolved outside-pool players may expose both; do not add potential fields to slot candidates or optimizer inputs.
- **Dependencies and ordering:** depends on Commit 1. It may follow the profile commit without depending on profile UI; both consume the same projection service.

**Implementation profile:** Terra xhigh — the Planner query has several assignment-state branches and a dense responsive matrix, so the small surface has cross-layer correctness and layout risk.

**Review profile:** Sol High — incorrect role/weight selection could present a convincing but tactically wrong score, and the UI must preserve the supported minimum viewport.

**Validation:** RED with focused `depth_tests` proving selected-lane potential combination and state behavior plus `./scripts/dev test 'src/app/routes/planner.test.tsx'`; GREEN with focused backend and route tests; then `./scripts/dev format`, `./scripts/dev check`, and `./scripts/dev smoke`. Browser evidence must cover a populated Planner at 1280x800 and 1600x900 with player names and both scores readable.

**Stop conditions:** Stop and replan if computing the score requires changing assignment persistence, slot-candidate/optimizer contracts, or an additional per-assignment database query; if a player can resolve to multiple conflicting current-snapshot rows; or if the minimum viewport cannot remain usable without a broader Planner redesign.

**Review mandate:**

- Verify the exact lane IP role, OOP role, and saved `ip_weight` feed both combined scores.
- Verify each resolved player projection is reused and no N+1 database query is introduced.
- Verify resolved, outside-pool, unresolved, missing-role-score, and null-attribute branches return honest values.
- Verify slot candidates and optimizer behavior remain current-score-only.
- Verify Current/Potential text and accessible names are unambiguous.
- Verify player names and assignment controls remain usable at the minimum supported viewport.

## Active work

**PR:** PR 1 — Potential role scores

**Commit:** Commit 2 — Show potential scores for every profile role

### RED proof

Extend the player-query fixture and profile route test before production code. The tests must fail because role rows have no `potential_score` and the Roles tab has no distinct accessible Current/Potential values. Cover a populated profile plus an unavailable potential score rendered as an em dash.

### Expected outcome

`get_player` projects one requested player once, scores all catalog roles with the existing scorer, and returns current plus potential scores. The Roles tab presents both values accessibly without changing Overview or Attributes behavior.

### Explicit exclusions

Do not change the profile Overview or Attributes tabs, search, navigation, snapshot ingest, Planner behavior, projected-attribute presentation, or database schema.

## Discoveries and replanning

- Planning confirmed that ingest already computes all 68 current role scores for every loaded player and contains a documented lazy/on-demand upgrade trigger. Potential calculation therefore moved to bounded profile and Planner reads rather than adding to Load Data.
- Planning found FMSuperScout's `mentalGrowthFactor` is defined but unused. This feature deliberately applies it and requires direct threshold tests rather than treating upstream output as authoritative for mental growth.
- Branch handoff completed on 2026-08-09: local `main` was fast-forwarded to `b9ff83b`, and `feature/potential-role-scores` was created from that commit. The feature therefore includes the merged scan-hardening foundation without carrying branch-only history.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 — Potential role scores | Commit 1 — Project visible attributes to player potential | Pending record | Pure Rust projection API with pinned profiles and regression tests | Sol High — Accept after 1 fix round | None |

## Final validation

**Feature review profile:** Sol High — the formula and two consumers can agree while still being systematically wrong, so feature review must trace representative projected attributes through profile and Planner scoring and assess responsive UI evidence.

Before feature review:

- Run `./scripts/dev format` and confirm no formatting changes remain.
- Run `./scripts/dev test` and retain the projection, player-profile, and Planner focused evidence.
- Run `./scripts/dev check` and require the full commit gate to pass.
- Run `./scripts/dev smoke` with populated profile and Planner coverage at 1280x800 and 1600x900.
- Capture a representative fixture from inputs through projected attributes, every profile role score, and the selected Planner lane's potential combined score; verify that both consumers use the same projection and role definitions.
- Measure or instrument the bounded profile and populated-Planner reads sufficiently to show no visible navigation delay and no new Load Data work. If read latency is material, replan from evidence rather than adding speculative persistence.
- Perform native desktop inspection of the profile Roles tab and populated Planner when the FM/Tauri runtime is available. If it is unavailable at close-out, record the evidence gap and obtain a developer decision rather than claiming native validation.
- Dispatch the ledger-selected fresh-context feature reviewer only after all planned commits and required evidence are complete.

## Documentation impact

During reconciliation, update `.wiki/ARCHITECTURE.md` with the implemented projection ownership, read-time calculation boundary, and extended profile/Planner read models. Update `.wiki/DESIGN.md` with the final Current/Potential presentation and responsive behavior. Keep the upstream pin and permission/provenance trail current if implementation differs from this plan. No ADR is planned because the feature follows existing Rust business-logic, IPC, query-cache, and non-persistent-derived-data boundaries; create one only if implementation requires a consequential boundary change.
