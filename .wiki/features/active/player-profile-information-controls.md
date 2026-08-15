# Player Profile Information Controls and Layout

## Status

Validation

## Intent

Deliver Linear issues JAY-5, JAY-8, and JAY-9 as one cohesive player-profile PR: let the user conceal in-game hidden information with one save-scoped profile control, reorganize attributes into the familiar FM category layout, and distinguish the strongest in-possession and out-of-possession roles for both current and potential ability.

## User-visible behavior

- Every player profile has one control for revealing or concealing hidden information. The preference belongs to the active app save, so changing it on one player immediately affects every player in that save and survives app restarts.
- Existing and new saves default to revealing hidden information.
- When hidden information is revealed, the profile shows PA, projected potential attributes, potential role scores, potential IP/OOP header summaries, Hidden values, Personality values, and the existing Boost CA and Wonderkid Mentality actions.
- When hidden information is concealed, the profile still shows CA, current visible attributes, current role scores, and current IP/OOP header summaries. PA and projected/potential values are not rendered, Hidden and Personality tabs show a clear concealed state, and both development actions are absent because their previews, availability, and results disclose PA or personality data.
- The concealment preference is a presentation choice, not an authorization boundary: Rust may still load the complete player record needed by existing domain operations, but concealed values must not enter the rendered DOM, accessible names, tooltips, status copy, or disabled-state explanations.
- Attributes have four tabs. Outfield players see Outfield, Goalkeeping, Hidden, and Personality and default to Outfield. Players with GK familiarity of 15 or higher see Goalkeeping, Outfield, Hidden, and Personality and default to Goalkeeping. For those goalkeeper profiles, First Touch, Passing, and Technique join the alphabetized Goalkeeping list and are omitted from the Outfield Technical column.
- Legacy `tab=technical`, `tab=mental`, and `tab=physical` profile URLs normalize to Outfield. An explicit canonical tab wins; missing or invalid tab values use the player-sensitive default.
- The header contains four fixed role summaries: Current IP, Current OOP, Potential IP, and Potential OOP. Each selects the highest non-null score from roles attached to a position with familiarity 15 or higher; equal scores keep catalog order.
- When hidden information is concealed, the Potential IP and Potential OOP summary slots remain visible as concealed placeholders so the header does not reflow or imply missing data.
- The profile remains usable by keyboard and at the supported 1280×800 and 1600×900 desktop sizes without clipped labels, overlapping controls, or nested page scrolling.

## Invariants

- The active save's current snapshot remains the sole source of player data.
- One preference row is not created per player or snapshot. The boolean lives on `saves` and follows active-save switching.
- Concealment affects only the player profile. Search, Squad, Planner, Academy, exports, and stored snapshot data keep their current behavior.
- Current values never derive from projected values. Potential attributes and role scores keep the existing Rust projection and scoring rules.
- IP means `in_possession`; OOP means `out_of_possession`. Role phase comes from the Rust catalog metadata already returned with each role score.
- Header role candidates still require at least one attached position with familiarity `>= 15`; phase splitting must not weaken that eligibility rule.
- Missing values stay missing and render as `—` when information is revealed. Concealment must not be represented as zero or as ordinary missing data.
- The WebView does not open SQLite. All persistence and active-save selection stay behind typed Tauri commands.
- The preference mutation is explicit and idempotent: the frontend sends the desired revealed state instead of a blind database toggle.
- Changing the preference must not mutate player, snapshot, scoring, Planner, or bridge data.

## Non-goals

- A global application setting or Settings-screen control.
- Per-player, per-snapshot, or session-only visibility preferences.
- Concealing hidden information outside the player-profile route.
- Treating the preference as an access-control, encryption, or anti-tampering boundary.
- Changing score formulas, projections, position familiarity, role catalog data, or IP/OOP terminology.
- Adding attribute comparison charts, radar charts, player comparison, facepacks, or position-suitability redesign.
- Redesigning the development-boost workflows beyond preventing concealed information from leaking through them.
- Updating Linear issue status or publishing a branch/PR during planning.

## Current-state map

- Relevant components: `src/app/routes/players.$uid.tsx` composes the profile; `player-overview-panel.tsx`, `player-attributes-panel.tsx`, `player-roles-panel.tsx`, and `player-development-boosts-panel.tsx` own the visible sections; `profile-tab.ts`, `attribute-groups.ts`, and `position-families.ts` own bounded presentation logic.
- Data model: `get_player(uid)` returns identity, current and projected attributes, hidden/personality maps, current and potential role scores, CA, and PA for one player in the active save's current snapshot.
- Persistence and migrations: `saves` owns save-wide preferences and context; the migration registry is currently v22 in `src-tauri/src/db/migrations.rs`. `.wiki/ARCHITECTURE.md` still describes the registry as v21 and must be reconciled with the new migration after implementation.
- Existing behavioral assumptions: profile attribute URL state now has four canonical tabs (legacy visible-group values normalize to Outfield); the overview now exposes phase-specific Current/Potential IP/OOP summaries; the role panel still renders both score bases; boost previews expose PA-derived caps and personality values.
- Architectural seams: Rust `features/player` owns player query/mutation IPC; React `features/player-profile` owns profile state and presentation; TanStack Query `playerKeys.all` already provides the save/snapshot invalidation boundary.
- Project validation commands: `./scripts/dev test [target...]`, `./scripts/dev check`, `./scripts/dev smoke`, and `./scripts/dev format [paths...]`.
- Primary risks: leaking concealed values through secondary UI, corrupting save preference defaults during migration, phase summaries selecting an unfamiliar position, and compressing the desktop layout below readable widths.

## Feature architecture

Migration v23 adds a constrained `saves.reveal_hidden_player_information` integer boolean with default `1`. `get_player` reads that value through the same active-save/current-snapshot lookup and exposes it as `hiddenInformationRevealed` on `PlayerDetailDto`. A player-owned `set_player_hidden_information_revealed(revealed)` command updates only the active save and returns the persisted state. The command delegates SQLite work to the Rust player service, and it is registered beside the existing player commands.

The profile route owns the mutation because it already composes all profile panels. Success invalidates the complete `playerKeys.all` tree so the open player refetches and inactive cached profiles become stale; failure keeps the last server-backed state and renders an inline alert. Active-save switching continues to invalidate the same query tree through existing app-shell behavior.

React treats concealment as a render boundary. The overview keeps four role-summary slots but supplies concealed placeholders for the potential pair; the attribute and role panels receive the preference and omit potential or raw hidden values; the route omits `PlayerDevelopmentActions` while concealed. The complete DTO remains internal because this is a user-controlled presentation mode, not a security boundary.

The attribute layout remains data-driven. `attribute-groups.ts` separates general Technical and Set Pieces keys while retaining the existing Mental, Physical, Goalkeeping, Hidden, and Personality key lists. The Outfield panel renders three semantic sections in one responsive row; narrower layouts stack before labels or values become unreadable. A goalkeeper-specific presentation moves First Touch, Passing, and Technique into the alphabetized Goalkeeping group without duplicating them in Outfield. The outer profile grid gives the attribute panel enough width for the three-column view while preserving the pitch and role table.

The four header summaries reuse the existing playable-position filter and score selectors after partitioning roles by catalog phase. No new score computation or Rust response shape is needed for JAY-9.

## Uncertainty register

### Known

- The developer confirmed one player-profile control, persisted per save and shared across all players in that save.
- The developer confirmed revealed information as the default.
- The developer confirmed Outfield, Goalkeeping, Hidden, and Personality tabs and four header role summaries.
- JAY-8's reference uses simultaneous Technical, Mental, and Physical columns, with Set Pieces separated within Technical.
- The profile query already returns all data needed for concealment and all four summaries.
- Repowise reports the profile route and overview as change hotspots and the migration registry as a high-risk file; its index was fresh at `e82b24985af7d297abea95446e055e36c2616f17` during planning.

### Assumptions

- “Hidden information” covers PA, every PA-derived projection, raw Hidden and Personality values, and controls whose state or feedback discloses those values.
- Current CA, visible current attributes, current IP/OOP role scores, positions, and identity/contract facts remain visible when concealment is active.
- The four summary requirement means four stable header slots. Concealed potential slots show an explicit state rather than disappearing.
- The toggle label can use “Hide hidden info” / “Reveal hidden info”; final concise copy may be adjusted during implementation without changing the contract.

### Decisions

- Persist one constrained boolean on `saves`; do not use local storage, a global setting, a new preference table, or player rows.
- Default the migration column to revealed (`1`) for both existing and future saves.
- Keep complete player data in the IPC DTO and enforce concealment at React render boundaries. This avoids duplicating the player query and preserves existing boost-domain inputs while matching the non-security product goal.
- Omit both development actions when information is concealed. Merely redacting their numeric copy is insufficient because eligibility, cap, and success states reveal PA or personality facts.
- Keep all four attribute tabs available. Hidden and Personality use an explicit concealed panel so the control remains discoverable and navigation geometry is stable.
- Normalize legacy visible-group tab values to Outfield instead of treating existing URLs as invalid.
- Use the existing playable-position threshold to identify goalkeeper profiles. Reorder only their tabs and attribute presentation; explicit canonical URL tabs remain authoritative.
- Deliver all three issues in one PR because they change the same profile route, header, responsive grid, DTO fixtures, and browser coverage; splitting them would create temporary layout contracts and duplicated regression work.

### Unknowns

- No product or structural unknown blocks implementation.
- Native Tauri/WebView layout evidence is not available during planning and remains a required final manual check.

### Risks

- A concealed value could remain in visually hidden text, a tooltip, a sort control, a status message, or a boost action even when it is not visibly painted.
- An explicit-state mutation could complete after an active-save switch. The command updates the save active at invocation and the existing save-switch invalidation must leave the newly active profile authoritative; do not add speculative cross-save state machinery unless a focused test exposes a race.
- The Outfield three-column panel and four-summary header may become cramped at supported desktop widths.
- Migration tests use the registry's exact order. v23 must be additive and fresh/existing database coverage must prove default `1` and persistence isolation.

## Walking skeleton

Commit 1 is the walking skeleton: add migration v23, return the preference from `get_player`, expose one explicit setter command, render the profile toggle, and prove that one save-scoped change survives a player change while concealed PA/potential/hidden/personality/development-action content is absent. It crosses storage, Rust IPC, TanStack Query, and the rendered route before either layout redesign begins.

## Delivery plan

### PR 1 — Player profile information controls and layout

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/player-profile-information-controls`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** Squash

**Required check:** `check` with strict branch protection

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(profile): refine information controls and role summaries`

**Purpose:** Resolve JAY-5, JAY-8, and JAY-9 in the one profile-owned review boundary where their state, header, responsive layout, fixtures, and browser behavior interact.

**Depends on:** Existing player profiles, potential role scores, and player development boosts. No earlier feature PR is required.

#### Commit 1 — Persist profile information visibility

**Status:** Completed

**Provisional commit:** `feat(profile): persist hidden information visibility`

**Work:** Add the save-scoped revealed/concealed preference end to end and prevent direct or indirect profile disclosure while it is concealed.

**Out of scope:**

- Attribute tab/layout restructuring.
- Four-way IP/OOP summary selection.
- Any non-profile consumer or global Settings control.

**Implementation packet:**

- Add a complete vertical slice from additive save migration through Rust query/mutation IPC to one accessible profile control and all affected render boundaries.
- Treat concealment as a presentation contract, not data authorization. The query may return complete values, but concealed facts must not be rendered anywhere, including off-screen accessibility content and action state.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — add v23 `saves.reveal_hidden_player_information`, registry assertions, default/backfill proof, constraint proof, and per-save persistence coverage.
- `src-tauri/src/features/player/query.rs` — read the active save preference with the current snapshot and carry it on `PlayerDetail`; extend active-save isolation tests.
- `src-tauri/src/features/player/service.rs` — update the active save to the explicit requested state and return a clear error if no active save exists; test save isolation and idempotence.
- `src-tauri/src/features/player/commands.rs` — expose `hiddenInformationRevealed` on `PlayerDetailDto` and wrap the setter command.
- `src-tauri/src/lib.rs` — register the new command.
- `src/features/player-profile/types/player-detail.ts` and `src/features/player-profile/api/` — type the preference and add the setter invocation.
- `src/app/routes/players.$uid.tsx` — own mutation, pending/error state, profile-tree invalidation, and preference propagation to all panels.
- `src/features/player-profile/components/player-overview-panel.tsx` — conceal PA and potential summary values while preserving the summary geometry.
- `src/features/player-profile/components/player-attributes-panel.tsx` — render visible current-only values and explicit Hidden/Personality concealment without concealed values in the DOM.
- `src/features/player-profile/components/player-roles-panel.tsx` — remove the potential column and potential sorting path while concealed; restore both from server state when revealed.
- `src/features/player-profile/components/player-development-boosts-panel.tsx` or its route composition — ensure neither boost action, preview, result, nor error disclosure renders while concealed.
- `src/testing/player-ipc-mock.ts`, `src/testing/setup.ts`, and `e2e/tauri-ipc-stub.ts` — model the new DTO field and stateful setter behavior.
- `src/app/routes/players.$uid.test.tsx` and `e2e/smoke.spec.ts` — prove persistence scope, mutation failure, concealment completeness, restoration, and keyboard interaction.

**Behavior and data flow:**

- `get_player(uid)` resolves the active save/current snapshot, returns the player plus that save's revealed flag, and preserves existing no-snapshot/not-found behavior.
- The user activates the profile control; React sends the inverse of the currently rendered server state to `set_player_hidden_information_revealed` and disables the control while pending.
- Rust writes only the active `saves` row. Success invalidates `playerKeys.all`, the open profile refetches, and any other player in that save subsequently reads the same state.
- A command failure leaves the current view unchanged, reenables the control, and produces an inline accessible error. No optimistic value is allowed to masquerade as persisted state.
- A save switch continues through existing app-shell invalidation. Each save returns its own preference.

**Ordered implementation steps:**

1. Add RED Rust migration/query/service tests for default revealed behavior, explicit updates, and isolation between two saves.
2. Add RED route tests that expect the control, persistence invocation, complete concealment, restored values, and an accessible failure state.
3. Add v23 and the smallest Rust query/service/command changes that turn the persistence proofs GREEN.
4. Add the typed frontend invocation, fixture/stub support, route mutation, and conditional rendering until route tests are GREEN.
5. Add RED/GREEN Playwright coverage for keyboard toggle use and navigation to another player with the same save setting.
6. Refactor only while the focused proofs remain green, format touched paths, then run commit validation.

**Tests and proof:**

- Expected RED: migration registry/default assertions fail because v23 and the column do not exist; route assertions fail because the DTO/control and concealment behavior do not exist.
- Rust GREEN: a fresh DB and a v22-shaped DB both migrate to v23 with revealed state; two saves retain independent explicit states; unknown active-save state errors without mutation; `get_player` returns the selected save's state.
- React GREEN: revealed is the fixture default; concealment removes PA, projected values, potential scores/sorts, hidden/personality numbers, and development actions from both visible and accessible queries; a failed setter reports an alert and retains state.
- Browser GREEN: keyboard activation conceals values, another profile shares the setting, and restoring it reveals values again within the same save.

**Patterns to verify:**

- Copy additive constrained-boolean migration conventions from `PLAYER_BOOST_RECOVERY_SQL` and exact registry assertions in `migrations.rs`.
- Copy active-save selection and service error conventions from `features/snapshot/service.rs`; keep the mutation player-owned because the control and read model are profile-owned.
- Copy TanStack mutation error and invalidation patterns from the existing profile boost flow, but invalidate only `playerKeys.all` because this preference affects no other consumer.
- Reuse `Button` and existing token roles. Do not add a general-purpose toggle primitive unless the focused implementation proves the native button cannot meet the interaction contract.

**Constraints and non-goals:**

- Store `0|1` with a SQLite `CHECK`; deserialize to a Rust/TypeScript boolean.
- Do not erase, null, or recompute stored player data when concealment changes.
- Do not leak through `aria-label`, `title`, `sr-only`, tooltip, disabled reason, modal, status, or mutation-result text.
- Keep the profile usable when no current snapshot or player is found; no control is needed without a player.
- Do not update Search/Squad table fields or their persisted layouts.

**Dependencies and sequencing:**

- Starts from migration registry v22 and the existing `PlayerDetail` IPC shape.
- Must complete before the layout and summary commits so both later commits consistently respect the preference.

**Validation:**

- `./scripts/dev format src e2e`
- `./scripts/dev test 'src/app/routes/players.$uid.test.tsx'`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Expected evidence: focused Vitest and Rust tests pass, the full commit gate passes, and the profile smoke scenario proves save-scoped concealment without visible or accessible leaks.

**Stop conditions:**

- The setting cannot be read and written atomically against the active save without changing a shared save-management contract.
- Existing boost command behavior requires concealed controls to remain rendered.
- A save switch during the setter produces a reproducible cross-save write or stale-view defect that the recorded invalidation boundary cannot resolve.
- Migration v23 cannot preserve every existing save as revealed by default.

**Review mandate:**

- Inspect every profile render path for direct and indirect concealed-value leakage.
- Verify the migration is additive, constrained, defaults existing rows to `1`, and does not alter context tokens or active-save selection.
- Verify the command updates only the active save and accepts an explicit state rather than blindly toggling.
- Verify mutation failure and active-save changes cannot display an unpersisted preference.
- Verify all player caches become stale after success without invalidating unrelated feature trees.
- Verify keyboard name/state/error semantics and focus remain usable.

#### Commit 2 — Group attributes by FM category

**Status:** Completed

**Provisional commit:** `feat(profile): group attributes by FM category`

**Work:** Replace six attribute tabs with four and show the three outfield categories together in an FM-like, responsive layout.

**Out of scope:**

- Changing attribute sources, values, projection math, or tier colors.
- Changing role-panel behavior beyond width/layout adjustments needed for the shared profile workspace.
- Adding charts or player comparison.

**Implementation packet:**

- Make Outfield the canonical visible-attribute tab and render Technical, Mental, and Physical as parallel semantic sections. Split Corners, Free Kicks, Long Throws, and Penalty Taking into a Set Pieces subsection inside Technical, matching the Linear reference without changing stored keys.
- Preserve the concealment contract from commit 1: Outfield and Goalkeeping show Current → Potential only when revealed and current-only otherwise; Hidden and Personality retain explicit concealed states.

**Files and responsibilities:**

- `src/features/player-profile/utils/profile-tab.ts` and its test — define four canonical tabs, use the player-sensitive Outfield or Goalkeeping default, and normalize legacy visible-group search values.
- `src/features/player-profile/utils/attribute-groups.ts` and its test — expose outfield category/subsection rows without duplicating or dropping keys; preserve null and potential mapping.
- `src/features/player-profile/components/player-profile-tabs.tsx` — render Outfield, Goalkeeping, Hidden, and Personality with existing accessible tab semantics.
- `src/features/player-profile/components/player-attributes-panel.tsx` — render three outfield columns, Technical Set Pieces subsection, single-category Goalkeeping, and concealment states.
- `src/app/routes/players.$uid.tsx` — rebalance or stack the Attributes/Role fit grid so both panels remain readable and update the matching loading skeleton.
- `src/app/routes/players.$uid.test.tsx` and `e2e/smoke.spec.ts` — cover tab normalization, category presence/order, missing values, concealment, keyboard tabs, and supported desktop layouts.

**Behavior and data flow:**

- A validated canonical `tab` search value selects one of four stable panels. Old Technical/Mental/Physical values resolve to Outfield; missing or invalid values resolve after player load to Outfield for outfield players and Goalkeeping for goalkeeper profiles.
- Attribute maps remain unchanged. Utility functions select ordered keys and the panel renders the same current/potential values in a new hierarchy.
- Revealed state controls whether projected values are passed to value rows; concealed state never creates hidden potential nodes.
- Responsive layout keeps one scroll owner per panel and stacks category columns before their contents clip.

**Ordered implementation steps:**

1. Add RED utility and route tests for four tabs, legacy normalization, three Outfield categories, Set Pieces, and complete key coverage.
2. Change the bounded tab and attribute-group models until utility tests are GREEN.
3. Recompose the panel and route grid with the smallest responsive changes that make route tests GREEN.
4. Add/adjust Playwright assertions at 1280×800 and 1600×900 for tab keyboard behavior, headings, and overflow.
5. Refactor only while focused tests stay green, format touched paths, then run commit validation.

**Tests and proof:**

- Expected RED: `parseProfileTab` still returns six group tabs and the panel renders only one visible category.
- GREEN assertions: exactly four tab labels; outfield profiles start with Outfield; goalkeeper profiles start with Goalkeeping and move First Touch, Passing, and Technique there without duplication; Outfield contains Technical, Set Pieces, Mental, and Physical in order; nulls stay `—`; reveal/conceal behavior from commit 1 is unchanged.
- Browser proof: supported desktop viewports have no clipped tab labels, overlapping values, unintended horizontal page scroll, or inaccessible tab panels.

**Patterns to verify:**

- Retain the existing static `attribute-groups.ts` key ownership and `PlayerProfileTabs` ARIA implementation.
- Follow the Linear JAY-8 reference hierarchy while using existing ValueScout panels, type scale, spacing, score tiers, and tokens rather than reproducing FM chrome.
- Preserve the route's single full-height workspace and current panel scroll ownership.

**Constraints and non-goals:**

- Do not change PascalCase dump keys or introduce frontend-derived attribute values.
- Do not duplicate Set Pieces values in both Technical and the subsection.
- Do not use color as the sole value indicator or lower existing contrast/focus treatment.
- Do not add a new breakpoint or reusable layout abstraction without an observed need.

**Dependencies and sequencing:**

- Depends on commit 1's preference prop/render boundary.
- No database or Rust changes are expected.

**Validation:**

- `./scripts/dev format src e2e`
- `./scripts/dev test 'src/features/player-profile/utils/attribute-groups.test.ts' 'src/features/player-profile/utils/profile-tab.test.ts' 'src/app/routes/players.$uid.test.tsx'`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Expected evidence: focused and full gates pass; Playwright confirms four-tab navigation and readable desktop layout.

**Stop conditions:**

- The three-column Outfield layout cannot remain readable at 1280×800 without changing the overall profile workspace hierarchy.
- The Linear reference requires an attribute key not present in the current dump contract.
- Tab normalization would break a documented public URL contract rather than only canonicalizing internal profile state.

**Review mandate:**

- Verify every visible attribute key appears exactly once and Set Pieces is only a presentation subdivision.
- Verify URL parsing, defaulting, and keyboard tab semantics for all four canonical tabs and legacy values.
- Verify concealed potential/hidden/personality values do not reappear through the new multi-section renderer.
- Verify null handling, tier labels, DOM order, and accessible headings remain truthful.
- Verify the profile and panel scroll owners at both supported desktop sizes.

#### Commit 3 — Split best roles by phase

**Status:** Completed

**Provisional commit:** `feat(profile): split best roles by phase`

**Work:** Replace the two mixed-phase header summaries with current and potential IP/OOP summaries while preserving eligibility and concealment rules.

**Out of scope:**

- Role score computation, catalog membership, combined scores, Planner weights, or pitch filtering.
- Additional role summaries beyond the confirmed four.
- Showing potential summaries while information is concealed.

**Implementation packet:**

- Partition the already playable role list by `in_possession` and `out_of_possession`, then apply the existing current and potential best-score selectors to each partition. Render four fixed slots with concise IP/OOP labels and catalog-order tie behavior.
- Preserve header balance with long role names, missing phase scores, pending visibility mutations, and concealed potential summaries.

**Files and responsibilities:**

- `src/features/player-profile/utils/position-families.ts` and its test — add or prove phase partitioning without changing playable-position familiarity or stable best-score selection.
- `src/features/player-profile/components/player-overview-panel.tsx` — calculate and render Current IP, Current OOP, Potential IP, and Potential OOP; use concealed placeholders for the potential pair when required.
- `src/app/routes/players.$uid.test.tsx` — prove phase, score basis, familiarity threshold, null, tie, and concealment behavior at the rendered seam.
- `e2e/tauri-ipc-stub.ts` and `e2e/smoke.spec.ts` — provide distinct phase winners and verify all four labels/scores and responsive header layout.

**Behavior and data flow:**

- `rolesForPlayablePositions` first excludes roles whose position tags never reach familiarity 15.
- The remaining catalog-ordered roles are partitioned by exact phase string.
- Current selectors compare `score`; potential selectors compare `potentialScore`; nulls are ignored and equal scores preserve input/catalog order.
- Missing winners render `—`. Concealed potential slots render “Concealed” and no role name or score from the DTO.

**Ordered implementation steps:**

1. Add RED helper/route tests with different IP/OOP and current/potential winners, catalog-order ties, unfamiliar high scorers, null phases, and concealed potential output.
2. Add the smallest phase-selection helper or composition that turns utility tests GREEN.
3. Recompose the overview summary into four stable slots and make route tests GREEN.
4. Add/adjust Playwright assertions for all four summaries and header layout at supported desktop sizes.
5. Refactor only while focused tests stay green, format touched paths, then run commit and feature validation.

**Tests and proof:**

- Expected RED: the overview still exposes only one mixed current winner and one mixed potential winner.
- GREEN assertions: four distinct slots choose the correct phase/basis winner; a score from familiarity 14 is excluded; catalog ties remain stable; null-only phases show `—`; concealed potential slots contain no score or role name.
- Browser proof: four summaries remain readable beside identity/facts/actions at 1280×800 and 1600×900 with no overlap or page-level horizontal scrolling.

**Patterns to verify:**

- Reuse `rolesForPlayablePositions`, `bestRoleScore`, and `bestPotentialRoleScore` instead of creating another scoring abstraction.
- Reuse `rolePhaseLabel` terminology and `BestRoleSummary`/`ScoreBadge` accessibility patterns.
- Keep phase partitioning in profile utilities; Rust already supplies authoritative phase metadata.

**Constraints and non-goals:**

- Do not combine IP and OOP scores or choose roles from unfamiliar positions.
- Do not change catalog order to alphabetic order for tie-breaking.
- Do not place concealed role names/scores in title text, accessible labels, or hidden layout placeholders.
- Keep four structural slots even when values are null or concealed.

**Dependencies and sequencing:**

- Depends on commit 1's concealment state and commit 2's final profile width/layout.
- Completes the sole PR's product scope and triggers full feature validation before finish-feature review.

**Validation:**

- `./scripts/dev format src e2e`
- `./scripts/dev test 'src/features/player-profile/utils/position-families.test.ts' 'src/app/routes/players.$uid.test.tsx'`
- `./scripts/dev test`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Expected evidence: focused and full suites pass, the full commit gate passes, and browser coverage proves four correct summaries in revealed and concealed states.

**Stop conditions:**

- Catalog phase metadata is incomplete or inconsistent for any profile role needed by the four summaries.
- The confirmed four-summary header cannot fit supported viewports without changing agreed information hierarchy.
- A change to eligibility, scoring, or projection becomes necessary to select phase winners.

**Review mandate:**

- Verify phase and score-basis selection independently for all four summaries.
- Verify familiarity filtering happens before phase ranking and ties retain catalog order.
- Verify null and concealed states are distinct, truthful, and leak-free.
- Verify accessible names identify IP/OOP and Current/Potential without duplicating or hiding scores.
- Verify the completed profile layout at both supported desktop sizes and regression behavior in the role panel.

## Active work

Implementation is complete for PR 1. Feature close-out has not run.

**Next action:** Run `$workflow-finish-feature` for feature-level validation, documentation reconciliation, and archival before publication.

## Discoveries and replanning

- Planning combined JAY-5, JAY-8, and JAY-9 into one PR because all three converge on the same profile route, header, responsive grid, fixtures, and browser proof. They remain separate commits for focused rollback and review.
- The developer narrowed JAY-5 from its original global/per-session possibilities to one profile control backed by a per-save preference, with revealed as the default.
- Inspection found that both existing development actions indirectly disclose concealed data: Boost CA reveals PA caps/eligibility and Wonderkid Mentality reveals personality values/eligibility. The plan therefore omits both actions while concealed.
- The implementation registry is already v22 (`drop_demo_value_table`) although current architecture prose says v21. The feature uses v23 and the documentation reconciliation must correct the complete migration description instead of preserving the stale number.
- Manual validation refined goalkeeper presentation: GK familiarity of 15 or higher now makes Goalkeeping the first and default tab, moves First Touch, Passing, and Technique into that alphabetized group, and keeps the remaining attributes under Outfield.
- Combined review found that Global Search, Search results, and Squad still injected the former Outfield default into ordinary profile navigation. Those entry points now leave the tab unset so the loaded player selects the default; user-selected canonical tabs remain authoritative.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Persist profile information visibility | `98ebc71` | Save-scoped revealed/concealed preference, active-save IPC, profile render boundaries, failure handling, and keyboard/browser coverage | Clean after 1 reviewer pass; two Medium accessibility/error-copy findings corrected and rechecked | None |
| PR 1 | Commit 2 — Group attributes by FM category | `dc76f97` | Four canonical tabs, Outfield Technical/Mental/Physical sections, Technical Set Pieces subsection, legacy URL normalization, and responsive profile workspace | Clean after 3 reviewer passes; expanded-rail stacking and label/role overflow checks added and rechecked | None |
| PR 1 | Commit 3 — Split best roles by phase | `Pending record` | Exact IP/OOP phase partitioning after familiarity filtering, four Current/Potential summary slots, catalog-order ties, null placeholders, and concealed potential summaries | Clean; focused tests, full Vitest, repository check, and 36-test smoke suite pass | None |

## Final validation

- `./scripts/dev format src e2e`
- `./scripts/dev test` (420 passed)
- `./scripts/dev check` (401 Rust tests passed; 2 ignored)
- `./scripts/dev smoke` (36 passed)
- `git diff --check <feature-base>...HEAD`
- Manual native Tauri/WebView check at 1280×800 and 1600×900: reveal/conceal with keyboard, navigate between players and saves, inspect both outfield and goalkeeper tab order/defaults, confirm the goalkeeper attribute split and four summary slots, exercise mutation failure, and verify no clipping, overlap, nested page scrolling, or concealed-value disclosure.
- Fresh-context feature-complete review after all three commits, followed by documentation reconciliation through `$workflow-finish-feature`.
- `./scripts/dev bridge-test` is not required unless implementation changes bridge code. `./scripts/dev mutate` remains unsupported and must not be reported as passed.

## Documentation impact

Complete during reconciliation. Update `.wiki/ARCHITECTURE.md` for migration v23, the save-owned preference and player IPC flow, four profile tabs, concealment behavior, and phase-specific summaries. Update `.wiki/DESIGN.md` for the four-tab attribute hierarchy, three-column Outfield layout, four summary slots, and concealment states. Reconcile this ledger into `.wiki/features/completed/` and update `.wiki/TODO.md` after final validation; no ADR is currently justified because the persistence choice follows the existing save-owned preference boundary.
