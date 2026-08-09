# Potential Role Scores

## Status

Active

## Intent

Project how well a player can perform each existing tactical role after developing from current ability (CA) to potential ability (PA), while preserving current attributes and role scores as the measures of present ability and fit. The projection adapts FMSuperScout's position-sensitive attribute-growth model, deliberately applies its otherwise-unused mental growth factor, and feeds the projected visible attributes through FM ValueScout's existing role scorer. The profile exposes the projected attributes and best potential role, and the Planner can allocate squads against either current or potential role fit. The feature does not introduce a separate meta score or a probability that the player reaches PA.

## User-visible behavior

- The player profile Roles tab shows a current score and a potential score for every role in the existing catalog.
- The player profile Overview shows **Best Potential Role** beside **Best Role**. Each slot selects its own highest non-null score and keeps catalog order on ties.
- The player profile Attributes tab shows every visible attribute as Current → Potential. Hidden and Personality values remain current-only because the projection does not model them.
- The Squad Planner shows a current combined score and a potential combined score for each resolved assignment, using only that lane's selected in-possession and out-of-possession roles and its saved phase weight.
- The Squad Planner keeps **Optimize squads** for current scores and adds **Optimize by potential**. Potential optimization uses projected role scores but otherwise follows the same eligibility, ordering, matching, foot-preference, manual-assignment, and transactional replacement rules.
- Current and potential scores are visibly and accessibly distinguished; an unavailable score renders as an em dash rather than zero.
- Loading FM data does not gain another all-player scoring pass. Potential values are calculated on bounded profile and Planner reads or when the user explicitly invokes potential optimization; unchanged reads benefit from the existing TanStack Query cache.
- A player whose PA is not greater than CA has projected visible attributes and potential role scores equal to the current values.

## Invariants

- The existing role catalog, primary/secondary bands, 75/25 role weighting, 0-100 scale, and Planner phase-weight combination remain authoritative.
- Potential role scores use projected visible attributes only. Hidden attributes, personality, and staff attributes do not participate.
- Attribute growth is position-sensitive and positive-only. Projected values are rounded to integers, capped at 20, and never below current values.
- Physical growth is damped by age using the adopted SuperScout thresholds. Mental growth is multiplied by `mentalGrowthFactor`: `1.0` below age 28, `1.15` from 28 through 31, and `1.25` from age 32.
- Unknown age uses neutral physical and mental multipliers (`1.0`); it must not erase position-profile growth or invent age damping.
- Multiple recognized natural position groups contribute the mean projected delta. Empty or unrecognized positions use the model's `ALL` profile.
- Missing or null visible attributes remain null. A role that requires a missing or null attribute has no potential score.
- Best Role continues to use current persisted scores. Best Potential Role uses potential scores from the same ordered role catalog; the two slots can name different roles.
- The Attributes tab returns and displays the exact projected visible-attribute map used for potential role scoring. It does not project Hidden or Personality values, and it does not run a second projection for presentation.
- The Planner applies the same lane roles and `ip_weight` to both current and potential combined scores.
- Current optimization remains based on persisted `player_role_scores`. Potential optimization projects each eligible source-player record once while building that team's candidate set, scores only the tactic's selected role pairs, and applies the existing allocation and foot-preference rules to those potential combined scores.
- Both optimizer modes retain manual assignments, replace prior `optimizer` assignments, and commit or roll back as one transaction. The chosen score basis is an invocation input, not persisted assignment metadata.
- Current scores remain the persisted ingest-time values in `player_role_scores`; no current-score behavior changes.
- Potential projection and scoring run in Rust. React displays returned values and does not reproduce the formula.
- Potential scores are derived values. They are not persisted and require no database migration.

## Non-goals

- A meta score, future current-ability estimate, development probability, expected-value score, or odds that a player reaches PA.
- Projection of Hidden or Personality attributes, attribute growth outside the existing Attributes tab, or projected-attribute charts.
- Potential-score search filters, sorting, global search results, exports, or snapshot history.
- Potential scores in the Planner slot picker or candidate list. Potential scores enter only the explicit potential-optimizer objective and the assigned-cell display.
- Replacing or relabeling the existing current-score optimizer as potential-first.
- Changes to Load Data, the bridge dump schema, memory scanning, snapshot ingest, or persisted score tables.
- Configurable projection coefficients, growth curves, position profiles, or mental-growth factors.

## Current-state map

- **Relevant components:** `src-tauri/src/features/scoring/` owns the 68-role catalog, current role scorer, Planner combination helper, and implemented pure projection service; `src-tauri/src/features/player/query.rs` assembles the profile read model and its current/potential role rows; `src-tauri/src/features/planner/depth.rs` resolves assigned current/potential lane scores; `src-tauri/src/features/planner/optimizer.rs` builds current-score candidates and owns allocation; `src/features/player-profile/` and `src/features/planner/` render and invoke those read and mutation paths.
- **Data model:** current-snapshot players expose required `ca` and `pa`, nullable `age`, natural positions, and nullable visible attributes. Profile role rows now return `score` and `potential_score`; the profile DTO still returns only the current visible-attribute map. Planner assignments now return `combined_score` and `potential_combined_score`; `optimize_planner_depth` still accepts no score-basis input and uses persisted current role scores.
- **Persistence and migrations:** snapshot ingest synchronously computes all current role scores into `player_role_scores`. Potential values have no storage and need no migration.
- **Existing behavioral assumptions:** profile Roles lists the full catalog grouped by position family; Overview derives Best Role in React from catalog-ordered current scores; Attributes groups visible, Hidden, and Personality values in React. A Planner lane selects one IP role, one OOP role, and an IP weight. The optimizer precomputes one lane-score vector per candidate, then shares it across ranked allocation and exact matching. One player UID can resolve to at most one assignment in a save.
- **Architectural seams:** Rust owns business logic and SQLite access behind existing `get_player`, `get_planner_depth`, and `optimize_planner_depth` IPC commands. TanStack Query owns frontend server-state caching and invalidation after Load Data or save changes.
- **Project validation commands:** `./scripts/dev test [target...]`, `./scripts/dev check-rust`, `./scripts/dev format [paths...]`, `./scripts/dev check`, and `./scripts/dev smoke` are the applicable stable surfaces. Bridge validation is not implicated.
- **Primary risks:** selecting the wrong potential best role, projecting twice during a profile read, implying that Hidden or Personality attributes are projected, feeding current values into the potential optimizer or potential values into the current optimizer, broadening the source-scoped candidate query, changing allocation rules while adding the score basis, and overcrowding the profile or Planner action toolbar.
- **Planning evidence:** direct repository inspection is authoritative because the Repowise index remains stale (`4ad07c4` versus worktree `273b111`).

## Feature architecture

The implemented pure Rust projection module under `src-tauri/src/features/scoring/` owns the CA anchors, empirical position/attribute profiles, FM-position-to-profile-group mapping, interpolation or extrapolation, physical and mental age factors, multiple-position averaging, null preservation, rounding, and the 20-point cap. The empirical data and behavior are adapted from FMSuperScout at commit [`0f270d39`](https://github.com/mavarobli/FMSuperScout/blob/0f270d39a9cdc850ddfe653710d4904f13709cb5/app/app.js#L2738-L2808), with provenance retained in code alongside the existing project permission record in `.wiki/notes/superscout-permission.md`.

Projection is deliberately evaluated on bounded read paths:

1. `get_player` projects one player's visible attributes once, scores all 68 catalog roles, and reuses the same projected map for the Attributes tab. It returns `score` and `potential_score` on each existing role row plus a separate projected visible-attribute map.
2. `get_planner_depth` projects each resolved assigned player once, scores only the assignment lane's selected IP/OOP role pair, combines them with the existing `ip_weight`, and returns `combined_score` plus `potential_combined_score`.
3. `optimize_planner_depth` accepts a validated Current or Potential score basis. Current keeps the existing persisted-role-score candidate path. Potential projects only eligible players from the configured team sources when the user invokes the action, scores the tactic's selected role pairs, and supplies those values to the same ranked and exact allocation pipeline.

This avoids extending the already-expensive Load Data path, avoids storing derived values that would become stale when the formula changes, and keeps navigation work bounded. A profile request performs one projection plus 68 cheap score calculations. A Planner read performs at most one projection plus two score calculations for each uniquely resolved assigned player. A potential optimization performs its broader candidate projection only after an explicit user action and retains the existing pending feedback and transaction boundary. Existing query caching prevents repeated IPC work during unchanged navigation.

The existing IPC commands, query keys, and invalidation behavior remain in place. Overview derives both best-role summaries from the bounded role-score DTO, while the Attributes tab only formats Rust-returned current and projected values. The Planner sends the selected score basis through the existing optimizer command and reconciles the same depth cache and slot-candidate queries. No new command, database table, background worker, or frontend scoring layer is introduced.

## Uncertainty register

### Known

- The upstream model uses CA anchors `[80, 110, 140, 170]` and position-specific per-attribute profiles, interpolating between anchors and extrapolating outside them.
- Its per-attribute delta is the mean across applicable position groups of `max(0, profile(PA) - profile(CA))`.
- Its position grouping is GK; central defender; fullback/wingback; defensive midfield; central midfield; wide midfield/wing; attacking midfield; and striker, with `ALL` as fallback.
- Its physical factors are `1.0` through age 23 or when age is unknown, `0.55` at 24-26, `0.30` at 27-29, `0.12` at 30-32, and `0.05` above 32.
- FMSuperScout defines `mentalGrowthFactor` as `1.15` from age 28 and `1.25` from age 32 but does not call it in its current projection implementation.
- The current schema already contains every input required by the projection and the current read models already cross the necessary Rust-to-React seams.
- Commits 1 through 3 implemented the projection service, all-role profile potential scores, and assigned-lane Planner potential scores without adding persistence or Load Data work.
- The existing optimizer already isolates allocation from score construction through each candidate's lane-score vector. It scopes player and persisted-score reads to configured team sources before ranked and exact matching.

### Assumptions

- The natural-position keys retained by the bridge are the correct position inputs for the model; position proficiency values do not weight one recognized group more heavily than another.
- Read-time projection remains imperceptible at the bounded profile and assignment counts. This assumption must be checked with focused timing evidence before persistence or batching is considered.
- In the Planner, “the selected role” means the lane's configured IP/OOP role pair expressed through the same combined-score semantics already shown in the assignment cell.
- “Best Potential Role” means the highest potential score across the same full 68-role catalog used by Best Role, without filtering by positional familiarity.
- “Current attribute → Potential attribute” applies to visible attributes only. Hidden and Personality attributes stay current-only because the adopted model has no projection contract for them.
- “Optimize by potential” changes only the candidate lane-score basis. All eligibility, ordering, manual-assignment, preferred-foot, replacement, rollback, and tie-break rules remain shared with current optimization.

### Decisions

- Apply `mentalGrowthFactor` to mental-attribute deltas. This is an intentional, tested divergence from upstream's currently inert helper.
- Treat `PA <= CA` as an identity projection so potential score equals current score, rather than returning no projection as upstream does.
- Recalculate potential values on existing Rust read paths and rely on existing TanStack Query caching. Do not add ingest work or persistence without measurement showing the bounded reads are a problem.
- Keep current and potential role scoring on the same catalog and scoring functions so the only changed input is the projected attribute map.
- Show compact Current/Potential treatments in existing role rows and assignment cells rather than adding a profile tab or Planner column family.
- Derive Best Role and Best Potential Role in React from the existing ordered role rows. Do not add summary fields or a second scoring path in Rust.
- Return the projected visible-attribute map from the existing profile read and reuse the projection already needed by role scoring. Keep Hidden and Personality presentation unchanged.
- Extend the existing optimizer command with one validated score-basis input and keep one allocation implementation. Do not add a parallel command, matcher, assignment provenance, or persisted basis field.
- Keep **Optimize squads** as the primary current-score action and add **Optimize by potential** as a secondary action. Both actions share pending/error exclusion and cache reconciliation, while success text identifies the selected basis.
- Keep upstream commit provenance next to the empirical coefficients and projection behavior.

### Unknowns

- The exact native desktop query duration for a profile and a fully populated Planner is not yet measured. Commit validation must capture enough evidence to confirm that read-time projection does not create visible navigation lag.
- The smallest legible two-score treatment in the Planner at the supported 1280x800 viewport must be confirmed in browser and native-desktop evidence rather than assumed from markup alone.
- The responsive layout for two equivalent Overview summary slots and paired visible attributes must be confirmed at the supported 1280x800 and 1600x900 desktop viewports.
- Potential-optimizer duration for a representative full configured club family is not yet measured. Validation must distinguish acceptable user-invoked computation with pending feedback from a UI-blocking or unbounded candidate path.

### Risks

- A coefficient transcription error can produce plausible but wrong potential scores. Golden fixtures and boundary tests must fail on such drift.
- Profile and Planner call sites could convert or default nulls differently. Backend and UI tests must preserve unavailable scores as unavailable.
- Re-projecting the same assigned player in multiple query branches could add avoidable work. The Planner read model must reuse one projection per resolved assignment/player within a request.
- Adding score labels can widen Planner cells or obscure player names. Responsive smoke evidence must cover the minimum supported viewport.
- Profile presentation could compute a second projection or silently drift from the potential role-score inputs. The DTO and backend fixture must prove one shared projected map.
- An unvalidated optimizer basis or crossed frontend argument could silently allocate the wrong squad while presenting plausible scores.
- Potential candidate construction could scan players outside configured sources, repeat projection per tactic lane, or hold the database transaction long enough to make the action feel stuck.
- Two optimizer buttons could permit concurrent mutations, show the wrong pending label, or report a success message for the wrong score basis.

## Walking skeleton

A Rust fixture player with CA below PA is projected through one recognized position profile. The same projected map supplies all profile role scores and the visible Current → Potential attribute values, while Overview independently selects the current and potential best roles. In the Planner, two eligible candidates trade places between current and potential combined score; the current action selects the current-fit player and the potential action selects the future-fit player through the same allocation pipeline.

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

**Purpose:** Deliver one reviewable vertical feature: a tested projection engine, current/potential profile evidence, selected-role Planner presentation, and an explicit potential-score optimizer. The added profile and optimizer work shares the same projection contract and unpublished branch, so no migration or independent merge boundary justifies another PR.

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

**Status:** Completed

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

**Status:** Completed

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

#### Commit 4 — Show the best potential role on Overview

**Status:** Completed

**Provisional commit:** `feat(profile): show the best potential role`

**Work:** Add a Best Potential Role summary beside the existing Best Role summary. Derive both from the existing catalog-ordered role rows so current and potential can select different roles without another backend field or scoring path.

**Out of scope:**

- Projected attribute presentation or profile DTO changes.
- Changes to role-row scoring, ordering, or grouping.
- Planner behavior.

**Implementation packet:**

- **Owners and files:** `src/features/player-profile/utils/position-families.ts` and its tests; `src/features/player-profile/components/player-overview-panel.tsx`; `src/app/routes/players.$uid.test.tsx`; `src/testing/player-ipc-mock.ts`; `e2e/tauri-ipc-stub.ts` and `e2e/smoke.spec.ts` for a populated responsive profile proof.
- **Existing patterns to verify:** `bestRoleScore` non-null selection and catalog-order tie behavior; the Roles tab's distinct Current/Potential accessible names; the Overview hero `ScoreBadge`; existing profile Panel spacing and 1280x800 layout.
- **Constraints and invariants:** Best Role still reads `score`; Best Potential Role reads `potentialScore`; ties keep the first catalog row independently; unavailable values render as `—` without a fake badge; visible labels and badge names distinguish Current from Potential; the two equivalent slots wrap without changing tabs or profile data ownership.
- **Dependencies and ordering:** depends on Commit 2's `potentialScore` role rows. It must remain a frontend derivation over the bounded profile DTO.

**Implementation profile:** Luna Max — the data already exists and the change is a bounded extension of the current best-role utility, Overview panel, and profile test patterns.

**Review profile:** Sol Medium — review must catch a selector that accidentally compares current scores, changes current tie behavior, hides unavailable data, or produces two visually similar but inaccessible summaries.

**Validation:** RED with `./scripts/dev test 'src/app/routes/players.$uid.test.tsx'` proving a fixture whose current and potential best roles differ; GREEN with the focused route and utility tests; then `./scripts/dev format`, `./scripts/dev check`, and `./scripts/dev smoke`. Browser evidence must show both labelled summaries inside the Overview panel at 1280x800 and 1600x900.

**Stop conditions:** Stop and replan if the potential best role cannot be derived from the existing role rows without duplicating backend scoring, if the Overview requires a new IPC command or persisted summary, or if two equivalent summaries cannot fit the supported viewport without broader profile navigation or layout changes.

**Review mandate:**

- Verify current and potential selectors read only their own score field and can choose different roles.
- Verify null exclusion and catalog-order ties independently for both selectors.
- Verify Best Role preserves its current semantics while both unavailable states remain honest.
- Verify visible and accessible labels distinguish Best Role from Best Potential Role without relying on colour.
- Verify both summaries remain readable and bounded at 1280x800 and 1600x900.

#### Commit 5 — Show projected visible attributes

**Status:** Completed

**Provisional commit:** `feat(profile): show projected visible attributes`

**Work:** Extend the existing player read model with the projected visible-attribute map already used for potential role scoring, then render Current → Potential values for Technical, Mental, Physical, and Goalkeeping attributes. Keep Hidden and Personality sections current-only.

**Out of scope:**

- Projection of Hidden or Personality attributes.
- Attribute charts, growth deltas, colour-coded improvement, editing, search, or sorting.
- Another profile query, projection pass, tab, or cache key.
- Planner behavior.

**Implementation packet:**

- **Owners and files:** `src-tauri/src/features/player/query.rs` and `commands.rs` plus focused tests; `src/features/player-profile/types/player-detail.ts`; `src/features/player-profile/components/player-attributes-panel.tsx`; existing attribute-group helpers as needed; `src/app/routes/players.$uid.test.tsx`; `src/testing/player-ipc-mock.ts`; `e2e/tauri-ipc-stub.ts` and `e2e/smoke.spec.ts`.
- **Existing patterns to verify:** the projection currently created inside the profile role-score path; nullable `BTreeMap` DTO serialization; static visible, Hidden, and Personality group membership; role-row Current → Potential treatment; shared formatters and em-dash behavior.
- **Constraints and invariants:** project once per `get_player` request; reuse that exact map for all role scores and attribute presentation; return visible keys only; preserve nulls; keep raw FM 1–20 values; render a text-accessible Current → Potential pair for every visible row; keep Hidden and Personality rows single-valued and unchanged.
- **Dependencies and ordering:** depends on Commit 1's projection and Commit 2's profile read path. The implementation may refactor where the existing projection is produced, but must not change the formula or role-score results.

**Implementation profile:** Luna Max — the projection and profile seam already exist, and the work is a bounded DTO and presentation extension with strong Rust and route-test analogues.

**Review profile:** Sol Medium — review must trace one projected map through Rust, serialization, and the Attributes panel and catch duplicate projection, null coercion, hidden-attribute implication, or current-role-score drift.

**Validation:** RED with `./scripts/dev check-rust` for a player-query fixture that expects the exact projected map and `./scripts/dev test 'src/app/routes/players.$uid.test.tsx'` for visible Current → Potential rows; GREEN with both commands; then `./scripts/dev format`, `./scripts/dev check`, and `./scripts/dev smoke`. Evidence must prove the returned projected attributes match direct `project_attributes` output, role potentials are unchanged, null stays unavailable, Hidden and Personality remain single-valued, and the Attributes panel remains readable at 1280x800 and 1600x900.

**Stop conditions:** Stop and replan if presentation requires a second projection or database query, if the projected map cannot be shared with all-role scoring, if the model would need to invent Hidden or Personality growth, if the IPC payload becomes unbounded, or if paired values require a new tab or broad layout redesign.

**Review mandate:**

- Verify `get_player` computes one projected visible-attribute map and reuses it for both DTO output and all potential role scores.
- Verify every current and projected visible value preserves its exact FM integer or null state.
- Verify Hidden and Personality data and presentation remain current-only.
- Verify role potential scores and catalog order do not change during the projection-sharing refactor.
- Verify Current → Potential meaning is visible, accessible, and not conveyed by colour alone.
- Verify the paired rows remain scannable without horizontal overflow at both supported desktop viewports.

#### Commit 6 — Optimize squads by potential

**Status:** Active

**Provisional commit:** `feat(planner): optimize squads by potential`

**Work:** Add a validated Current/Potential score basis to the existing optimizer command and candidate-score construction, then add a secondary **Optimize by potential** action beside **Optimize squads**. Both modes use the same ranked and exact allocation pipeline and return the existing reconciled depth model.

**Out of scope:**

- Potential scores in the slot picker or candidate-list DTO.
- New allocation, suitability, age, rank, foot-preference, tie-break, team-order, or string-order rules.
- A second optimizer command, matcher, query cache, provenance value, or persisted score-basis field.
- Automatic potential optimization on load, navigation, tactic save, or snapshot refresh.

**Implementation packet:**

- **Owners and files:** `src-tauri/src/features/planner/optimizer.rs`, `optimizer_tests.rs`, and `commands.rs`; existing scoring catalog, projection, score, and combination helpers only as dependencies; `src/features/planner/api/optimize-planner-depth.ts`; Planner types if the basis needs a shared frontend union; `src/features/planner/components/planner-optimizer-controls.tsx` and `planner-depth-matrix.tsx`; `src/app/routes/planner.test.tsx`; `src/testing/planner-ipc-mock.ts`; `e2e/tauri-ipc-stub.ts`; `e2e/smoke.spec.ts`.
- **Existing patterns to verify:** source-scoped candidate queries; `OptimizerCandidate.lane_scores`; ranked allocation before `match_lanes`; `allocation_score` foot behavior; manual UID reservation; delete-and-replace transaction with rollback; one optimizer mutation's depth-cache replacement, slot-candidate invalidation, pending/error status, and Clear-all exclusion.
- **Constraints and invariants:** validate exactly `current` or `potential` at the Rust command boundary; current mode continues to use persisted role scores; potential mode loads only configured source players and required projection inputs, projects each candidate record once while building its team candidate set, scores only the tactic's selected IP/OOP role pairs, combines with the saved lane weight, then applies the existing foot rule; both modes share all downstream allocation and persistence code; only one optimizer or clear mutation may run at a time; success and pending labels identify the chosen basis.
- **Dependencies and ordering:** depends on Commit 1's projection API and Commit 3's current/potential assignment display. It may reuse profile projection behavior but must not depend on profile UI or change the slot-candidate contract.

**Implementation profile:** Terra xhigh — the requested outcome is settled, but candidate loading, transactional replacement, score-basis validation, allocation invariants, and two coordinated mutation controls require material cross-layer judgment.

**Review profile:** Sol High — a basis mix-up or changed candidate scope can silently allocate a plausible but wrong squad, and review must trace both objectives through the same eligibility, ranking, matching, foot, transaction, and UI-state paths.

**Validation:** RED with `./scripts/dev check-rust` for a source-scoped fixture where the current-best and potential-best candidates differ and `./scripts/dev test 'src/app/routes/planner.test.tsx'` for the missing potential action and basis argument; GREEN with both commands; then `./scripts/dev format`, `./scripts/dev check`, and `./scripts/dev smoke`. Rust evidence must show Current selects the current-fit player, Potential selects the future-fit player, missing projected requirements remain ineligible, manual rows survive, prior optimizer rows are replaced, invalid bases fail safely, and rollback remains atomic. Browser evidence must exercise both actions, basis-specific pending/success/error behavior, mutual exclusion with Clear all, cache reconciliation, and toolbar fit at 1280x800 and 1600x900.

**Stop conditions:** Stop and replan if potential optimization requires persistence or a migration, a second command or duplicated matcher, per-candidate or per-lane database queries, reads outside configured team sources, projection during Load Data or navigation, changed assignment provenance, weakened rollback/manual protection, or a confirmed unresponsive action that cannot be fixed within the existing bounded mutation path.

**Review mandate:**

- Verify the command rejects unknown score bases and the frontend sends the intended basis for each button.
- Verify current mode still consumes persisted `player_role_scores` and produces unchanged allocations for existing fixtures.
- Verify potential mode uses the candidate's CA, PA, age, natural positions, visible attributes, exact lane role pair, and saved `ip_weight` through the shared scoring helpers.
- Verify source, team-age, position-suitability, rank, preferred-foot, manual-reservation, team/string order, filled-lane, and UID tie-break rules are identical between modes.
- Verify potential projection is once per candidate record rather than once per lane and introduces no N+1 or whole-snapshot query.
- Verify both modes replace only prior optimizer rows and preserve transaction rollback and manual assignments.
- Verify the two actions cannot run concurrently, identify their pending/outcome state, reconcile depth and candidate caches, and leave Clear all safe.
- Verify the extra toolbar action remains keyboard-operable and readable at 1280x800 and 1600x900.

## Active work

**PR:** PR 1 — Potential role scores

**Commit:** Commit 6 — Optimize squads by potential

### RED proof

Add a source-scoped optimizer fixture where the current-best and potential-best players differ, and Planner route assertions for the missing potential action and score-basis argument. The focused Rust and route tests must fail because the optimizer accepts no basis and the toolbar exposes only current optimization.

### Expected outcome

Current optimization keeps selecting from persisted role scores; potential optimization projects each configured-source candidate once and selects the future-fit player through the same allocation, foot, transaction, cache, and manual-reservation paths. The Planner exposes clear, mutually exclusive current and potential actions with basis-specific pending and outcome state.

### Explicit exclusions

Do not add potential scores to the slot picker or candidate DTO, change optimizer eligibility/ranking/matching/foot/tie-break rules, add a second command or matcher, persist the basis, optimize automatically, or move potential work into Load Data or navigation.

## Discoveries and replanning

- Planning confirmed that ingest already computes all 68 current role scores for every loaded player and contains a documented lazy/on-demand upgrade trigger. Potential calculation therefore moved to bounded profile and Planner reads rather than adding to Load Data.
- Planning found FMSuperScout's `mentalGrowthFactor` is defined but unused. This feature deliberately applies it and requires direct threshold tests rather than treating upstream output as authoritative for mental growth.
- Branch handoff completed on 2026-08-09: local `main` was fast-forwarded to `b9ff83b`, and `feature/potential-role-scores` was created from that commit. The feature therefore includes the merged scan-hardening foundation without carrying branch-only history.
- Commit 3 extends the existing assignment query with the projection inputs, rather than issuing a second player query. Each matching assignment projects its visible attributes once, then scores only the lane's configured IP/OOP roles. Slot candidates and optimizer inputs retain their current-score-only shape.
- Browser smoke coverage confirms the assigned player name and both score badges remain within the Planner cell at 1280x800 and 1600x900. The visible arrow denotes current-to-potential; score badges retain distinct accessible Current and Potential names.
- Before PR 1 publication, the developer expanded the feature with Best Potential Role, projected visible-attribute presentation, and an explicit potential optimizer. The unpublished branch and existing projection contract remain one coherent PR; feature close-out remains not run.
- Profile inspection confirmed that Overview already receives all ordered current/potential role rows, so both best-role summaries stay frontend-derived. The Attributes addition must move the already-computed projected map into the profile DTO rather than run `project_attributes` twice.
- Optimizer inspection confirmed that candidate score construction is separate from ranked and exact allocation through `OptimizerCandidate.lane_scores`. The new score basis therefore belongs before allocation; potential mode must not duplicate the matcher or change assignment provenance.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 — Potential role scores | Commit 1 — Project visible attributes to player potential | `ce7c87a` | Pure Rust projection API with pinned profiles and regression tests | Sol High — Accept after 1 fix round | None |
| PR 1 — Potential role scores | Commit 2 — Show potential scores for every profile role | `126bf76` | One profile-read projection shared by all catalog roles, plus accessible Current/Potential rows | Sol Medium — Accept | None |
| PR 1 — Potential role scores | Commit 3 — Show potential score for assigned Planner roles | `273b111` | One assignment-read projection shared by the selected IP/OOP role pair, plus compact accessible current-to-potential score pairs | Sol High — Accept | None |
| PR 1 — Potential role scores | Commit 4 — Show the best potential role on Overview | `156037c` | Independent current/potential best-role selectors and responsive, accessible Overview summaries | Sol Medium — Accept | None |
| PR 1 — Potential role scores | Commit 5 — Show projected visible attributes | Pending record | One shared visible-attribute projection for role scoring and profile presentation, with accessible Current → Potential rows | Sol Medium — Accept | None |

## Final validation

**Feature review profile:** Sol High — the projection, profile summaries and attributes, assigned-lane display, and two optimizer objectives can agree locally while still selecting or presenting the wrong future fit, so feature review must trace representative inputs end to end and assess responsive and transactional evidence.

Before feature review:

- Run `./scripts/dev format` and confirm no formatting changes remain.
- Run `./scripts/dev test` and retain the projection, player-profile, and Planner focused evidence.
- Run `./scripts/dev check` and require the full commit gate to pass.
- Run `./scripts/dev smoke` with populated Overview, Attributes, Roles, and Planner coverage at 1280x800 and 1600x900.
- Capture a representative fixture from inputs through the single projected visible-attribute map, every profile role score, both best-role summaries, and the selected Planner lane's potential combined score; verify that every consumer uses the same projection and role definitions.
- Run current and potential optimization against a fixture where their best candidate differs. Verify only the score basis changes while source eligibility, age and position rules, lane ranks, foot preferences, manual reservations, team/string order, tie-breaks, provenance, replacement, and rollback remain shared.
- Capture representative current and potential optimizer durations for a populated configured club family. Confirm the potential action remains bounded to configured sources, shows immediate pending feedback, and does not introduce Load Data or navigation work. Replan from evidence if the action is unresponsive rather than adding speculative persistence.
- Measure or instrument the bounded profile and populated-Planner reads sufficiently to show no visible navigation delay and no new Load Data work. If read latency is material, replan from evidence rather than adding speculative persistence.
- Perform native desktop inspection of the populated profile Overview, Attributes, and Roles tabs plus both Planner optimizer actions when the FM/Tauri runtime is available. If it is unavailable at close-out, record the evidence gap and obtain a developer decision rather than claiming native validation.
- Dispatch the ledger-selected fresh-context feature reviewer only after all planned commits and required evidence are complete.

## Documentation impact

During reconciliation, update `.wiki/ARCHITECTURE.md` with the implemented projection ownership, single-projection profile read, extended profile and Planner read models, and Current/Potential optimizer command path. Update `.wiki/DESIGN.md` with the two best-role summaries, visible-attribute Current → Potential treatment, assigned-score pairs, two optimizer actions, and final responsive behavior. Update `.wiki/TODO.md` only when the feature-level status changes. Keep the upstream pin and permission/provenance trail current if implementation differs from this plan. No ADR is planned because the feature stays inside existing Rust business-logic, IPC, query-cache, transaction, and non-persistent-derived-data boundaries; create one only if implementation requires a consequential boundary change.
