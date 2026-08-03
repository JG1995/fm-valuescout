# Planner Optimizer Preferences

## Status

Active

## Intent

Let the user tune optimization for each linked tactic lane instead of applying one squad-wide score blend. Each lane can own its IP/OOP weight, an optional importance rank, and a preferred-foot rule while manual assignments remain absolute overrides.

## User-visible behavior

- Each linked tactic lane has its own IP/OOP score weight. There is no squad-wide weight.
- Each lane can be unranked or assigned a unique importance rank. Within each existing team and string boundary, the optimizer fills ranked lanes first in ascending rank order, then sends the remaining unranked lanes through the existing exact matcher.
- Each lane accepts **Either**, **Left**, **Right**, or **Both** as its preferred-foot setting. **Either** means no restriction. **Both** requires a player whose snapshot value is `either`, which is how the bridge represents a genuinely two-footed player.
- A preferred-foot rule can be **Preferred** or **Strict**. A soft mismatch subtracts five points from the optimizer-only allocation score. A strict mismatch makes that player ineligible for that lane.
- Two-footed players satisfy Left and Right preferences. Empty or unknown player-foot data mismatches Left, Right, and Both, but remains eligible for Either.
- The five-point adjustment does not change persisted role scores or the Planner's displayed combined role score.
- Manual assignments remain in place and reserve their player UID before every optimizer rule. The optimizer does not reject, move, or clear a manual assignment because of weight, priority, foot, age, suitability, or missing-score rules.
- Saving tactic preferences keeps the existing draft-retention, pending, success, error, and Planner cache-invalidation behavior.

## Invariants

- Rust owns tactic validation, persistence, combined-score calculation, optimizer eligibility, ordered allocation, and deterministic tie-breaking.
- React owns only the editable draft, accessible controls, mutation state, and Query cache reconciliation.
- Lane IDs and lane order remain stable across the linked IP and OOP tactic and across every team and string.
- Each lane weight is finite and within `0..=1`. IP and OOP weights always sum to 1.
- Non-null lane priorities are unique within the tactic. Partial rankings and gaps are valid; `null` means unranked.
- Team order remains Senior, Reserves, Youth. String order remains ascending within each team.
- Ranked allocation is scoped to the current string. It does not pull a later team or string ahead of the existing order.
- With every lane unranked, priority handling preserves the existing exact per-string allocation path.
- After ranked lanes reserve their selected players, the exact matcher maximizes the remaining optimizer allocation score, then filled lanes, then the stable UID tie-break.
- Strict foot rules remove only optimizer candidate-to-lane edges. Soft foot rules change only the optimizer objective.
- Existing club-family, team-age, position-suitability, complete-score, provenance, transaction, and selected-team Clear Squad contracts remain unchanged.
- Snapshot `preferred_foot = 'either'` means the player is two-footed. The tactic's unrestricted value uses the distinct internal value `any` even though the UI label is **Either**.
- The WebView does not calculate authoritative combined scores, foot matches, or optimizer decisions.

## Non-goals

- Retaining or migrating unreleased tactic configuration. The schema migration may reset tactic rows to the default eleven lanes.
- A squad-wide or fallback IP/OOP weight.
- Applying strict foot rules to the manual slot picker or manual assignments.
- Making the five-point soft penalty user-configurable.
- Changing ingested role scores, FM position suitability, team age rules, club-family pools, or assignment provenance.
- Reordering teams, strings, tactic lane identities, or custom string names.
- Formation comparison, gap recommendations, transfer suggestions, or general optimizer presets.
- Reading separate left-foot and right-foot strength values from FM beyond the bridge's existing `left`, `right`, `either`, or empty snapshot value.

## Current-state map

- Relevant components: `src/features/planner/components/planner-tactic-editor.tsx` owns the tactic draft and selected-lane weight control; `planner-tactic-pitch.tsx` owns phase-specific lane selection and role controls; `planner-depth-matrix.tsx` owns the Optimize mutation; `src/app/routes/planner.test.tsx` and `e2e/` cover the user path.
- Data model: each stable `TacticLane` owns its IP/OOP positions, role IDs, and `ipWeight`; `PlannerTactic` is the eleven-lane collection.
- Persistence and migrations: migration v8 removes the obsolete `planner_tactics` parent table and resets tactic rows into save-scoped `planner_tactic_lanes` with `ip_weight`; v6 added strings and assignments; v7 added assignment provenance. Planner assignments identify lanes by stable text ID and do not reference the tactic tables.
- Existing behavioral assumptions: combined scores use the target lane's weight; the optimizer reserves all manual UIDs, processes Senior then Reserves then Youth and ordered strings, and runs an exact matcher for all remaining lanes in each string.
- Architectural seams: `tactic.rs` validates and persists the full tactic; `depth.rs` calculates displayed assignment and picker scores; Planner-private `optimizer.rs` loads candidates and owns matching; `commands.rs` maps the typed IPC contract; React calls IPC through `src/lib/tauri-client.ts`.
- Player-foot input: the bridge and snapshot schema expose `left`, `right`, `either`, or an empty string. The bridge emits `either` when both decoded foot attributes are at least 14.
- Project validation commands: `./scripts/dev test [target...]`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` remains unsupported.
- Primary risks: tactic-table replacement must not delete assignments; partial priority must not bypass the exact matcher for unranked lanes; `any` and player value `either` must not be conflated; soft penalties must not leak into displayed or persisted role scores; added controls must fit the dense Planner at the supported viewports.

## Feature architecture

The save-scoped tactic becomes a complete set of eleven lane rows. Removing the global weight leaves `planner_tactics` with no independent state, so the first migration replaces the old tactic tables with `planner_tactic_lanes` rows that reference `saves` directly and own their score weight. Because the product is unreleased and the developer waived tactic compatibility, this migration resets existing tactic rows; the next `get_planner_tactic` call seeds the validated default lanes. Saves, club-family rows, strings, and assignments remain intact because assignments use stable lane IDs without a tactic foreign key.

Later additive migrations add optional unique priority and preferred-foot settings to each lane. `tactic.rs` remains the authoritative full-tactic save and validation boundary. The IPC request and response carry the same lane-owned fields to React.

`depth.rs` combines the two role scores with the target lane's weight for assignment and picker evidence. `optimizer.rs` uses that same base combined score, then derives an allocation score at runtime. A matching or unrestricted foot keeps the base score; a soft mismatch uses `max(0, base - 5)`; a strict mismatch creates no lane edge. The derived allocation score is neither persisted nor presented as a role score.

For each existing team and ordered string, the optimizer skips manually occupied lanes, sorts the remaining ranked lanes by ascending importance, and assigns each ranked lane its highest eligible allocation score with the existing stable UID tie-break. It reserves each chosen UID immediately. The existing exact matcher then receives the remaining unranked lanes and available candidates. When no lane has a priority, the entire set follows the existing exact path.

React adds one shared optimizer-settings section for the selected linked lane. It appears once even in the **Both** tactic view, rather than duplicating controls inside both phase panels. Native sliders and selects expose the lane weight, optional priority, foot, and Preferred/Strict mode. The mode control is disabled when the foot setting is Either. Duplicate priority and incomplete tactic errors remain inline and disable Save while preserving the draft.

No new dependency, top-level feature, IPC command, capability, or ADR is required.

## Uncertainty register

### Known

- The optimizer and tactic are private parts of the existing Planner feature boundary.
- Manual assignments already reserve their player UID before automatic allocation and survive optimizer reruns.
- Player snapshot `either` is the repository's two-footed representation, not an unrestricted query value.
- Planner tactic saves already invalidate depth and slot-candidate queries.

### Assumptions

- Priority is optional per lane. Any subset of the eleven lanes can be ranked, and unused numeric ranks may remain as gaps.
- Ranked-lane selection is intentionally greedy and transparent: each ranked lane takes its best currently available candidate before the next rank.
- The current fixed five-point soft penalty is sufficient until real use shows that it needs tuning.
- Resetting tactic rows on migration is acceptable because the product has not been released. Planner assignments can remain because default lane IDs stay stable.

### Decisions

- Remove the global score weight rather than keeping a default-plus-override model.
- Preserve team and string order; apply lane priority only inside each string.
- Keep manual placement as an override for every optimizer rule.
- Use `any` for the unrestricted tactic value and reserve `either` for a two-footed player value.
- Keep displayed combined scores foot-neutral and apply the penalty only inside optimizer allocation.
- Keep the penalty fixed at five points and floor the result at zero.

### Unknowns

- No product or structural unknown blocks the first commit.
- The final density and control grouping need native populated-state inspection at 1280x800 and 1600x900; repository evidence cannot prove the visual result during planning.

### Risks

- A table-replacement migration could cascade farther than intended if the current foreign-key graph is misunderstood.
- A priority implementation could accidentally replace the existing no-priority exact behavior with greedy allocation.
- A soft penalty incorporated into the base combined score could make Planner evidence disagree with Search and Profiles.
- Duplicate or ambiguous priorities could make allocation order non-deterministic.
- The four lane settings could crowd or duplicate the existing phase-specific controls in Both view.

## Walking skeleton

PR 1, commit 1 replaces the global weight with a lane-owned weight through SQLite, Rust validation and scoring, the typed IPC contract, and one shared selected-lane React control. Saving and reloading a changed lane weight must change that lane's Planner combined score without changing another lane.

## Delivery plan

### PR 1 — Per-lane optimizer preferences

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(planner): add per-lane optimizer preferences`

**Purpose:** Deliver the complete per-lane score-weight, importance, and preferred-foot workflow inside the existing Planner boundary. One PR is appropriate because the three settings share the same tactic lane, save action, optimizer, and review surface; no foundation needs an independent trunk merge.

**Merge to trunk when:** All three commits are complete, the full validation and populated native-view evidence pass, and the feature-complete Sol High review has no retained CRITICAL, HIGH, or MEDIUM finding.

**Depends on:** Completed Squad Planner, Squad Optimizer, and Planner module refactor.

#### Commit 1 — Use per-lane scoring weights

**Status:** Completed

**Provisional commit:** `feat(planner): use per-lane scoring weights`

**Work:** Replace the global tactic weight with one authoritative weight per linked lane, reset unreleased tactic rows during migration, use the lane weight for Rust-owned depth, picker, and optimizer base scores, and expose one accessible selected-lane weight editor.

**Out of scope:**

- Priority persistence or allocation.
- Preferred-foot persistence or optimizer rules.
- Any change to role-score ingest or manual assignment behavior.

**Implementation packet:**

- Owners and files: `src-tauri/src/db/migrations.rs`; `src-tauri/src/features/planner/tactic.rs`, `depth.rs`, `optimizer.rs`, `commands.rs`, and their focused tests; `src/features/planner/types/tactic.ts`, `utils/tactic-editor.ts`, `components/planner-tactic-editor.tsx`, `components/planner-tactic-pitch.tsx`, and a Planner-local selected-lane optimizer-settings component if separation keeps the editor focused; `src/testing/planner-ipc-mock.ts`; `src/app/routes/planner.test.tsx`; `e2e/tauri-ipc-stub.ts`; current-state architecture and design text made true by this commit.
- Existing patterns to verify: migration v7 upgrade test; full-tactic replacement in `save_tactic`; default lane seeding; `combine_role_scores`; tactic-save draft retention and depth/candidate invalidation; selected lane shared across IP/OOP pitch views.
- Constraints and invariants: migration removes only tactic configuration, not saves, club sources, strings, or assignments; default lane IDs remain unchanged; each lane weight is finite in `0..=1`; React never calculates authoritative combined scores; no global or fallback weight remains in the domain or UI.
- Dependencies and ordering: first commit; establishes the lane-owned tactic shape consumed by later commits.

**Implementation profile:** Terra xhigh — the outcome is defined, but the commit combines a destructive unreleased-data migration with Rust scoring, IPC, React draft state, and cache-sensitive Planner behavior.

**Review profile:** Terra xhigh — review must trace the foreign-key impact, persisted contract, score ownership, and frontend/backend round trip rather than treating the change as a local control move.

**Validation:** Run `./scripts/dev test src/app/routes/planner.test.tsx` for the lane-edit/save/reload and draft-retention behavior, `./scripts/dev check` for the migration, Rust tactic/depth/optimizer tests, TypeScript, and static gates, then `./scripts/dev smoke` for the Planner tactic save path. Expected evidence includes a v7-to-v8 migration test showing assignments survive while tactics reset, lane-specific score fixtures, and unchanged results for default 50/50 lanes.

**Stop conditions:** Replan if removing `planner_tactics` would delete or invalidate Planner assignments, if another current consumer requires a root tactic weight, if stable lane IDs cannot preserve existing assignments after reset, or if lane-specific scoring requires a new cross-feature/public scoring contract.

**Review mandate:**

- Confirm the migration resets only tactic rows and advances fresh and v7 databases correctly.
- Confirm default seeding restores exactly eleven stable lanes at 50/50.
- Trace each lane weight through validation, SQLite, DTOs, depth assignments, picker candidates, and optimizer base scores.
- Confirm one lane's weight cannot change another lane's score.
- Confirm the WebView does not recompute combined scores.
- Check save errors retain the edited draft and successful saves refresh depth and candidate evidence.
- Check the selected-lane control is single, labelled, keyboard-operable, and not duplicated in Both view.

#### Commit 2 — Prioritize ranked tactic lanes

**Status:** Active

**Provisional commit:** `feat(planner): prioritize ranked tactic lanes`

**Work:** Persist one optional unique importance rank per lane, expose it in the selected-lane settings, allocate ranked empty lanes first inside each current team/string boundary, and retain the exact matcher for all remaining unranked lanes.

**Out of scope:**

- Preferred-foot settings or score adjustment.
- Team, string, or tactic-lane reordering.
- Changes to manual assignment protection.

**Implementation packet:**

- Owners and files: migration registry and tests in `src-tauri/src/db/migrations.rs`; tactic model, validation, persistence, and DTO mapping in Planner Rust; `optimizer.rs` and `optimizer_tests.rs`; Planner tactic types, draft validation, selected-lane settings, IPC mocks, route tests, and browser stub/smoke where the contract changes; current-state docs made true by the commit.
- Existing patterns to verify: `PLANNER_TEAMS`, `load_ordered_strings`, manual-lane filtering, reserved UID handling, `match_lanes`, stable UID ranking, and native `<select>` conventions.
- Constraints and invariants: priorities are nullable, unique, and bounded to 1 through 11; gaps are valid; manual lanes are skipped; ranked selection is ascending and deterministic; chosen UIDs are reserved immediately; unranked lanes keep exact total-score allocation; all-null priorities preserve the current matcher path.
- Dependencies and ordering: depends on commit 1's lane-owned tactic model and selected-lane settings surface.

**Implementation profile:** Terra xhigh — the change modifies a proven exact-allocation algorithm and must preserve several ordering, reservation, and no-priority invariants.

**Review profile:** Terra xhigh — a locally plausible priority loop can silently reduce or reorder unrelated allocations, so review must compare concrete allocation paths and regression fixtures.

**Validation:** Run `./scripts/dev test src/app/routes/planner.test.tsx`, `./scripts/dev check`, and `./scripts/dev smoke`. Expected Rust evidence covers no priorities, partial priorities with gaps, ranked conflicts for one flexible candidate, a ranked lane with no eligible player, manual occupancy, multiple strings, team order, and stable UID ties; frontend evidence covers unique-rank validation, draft retention, and save/reload.

**Stop conditions:** Replan if priority must cross a team or string boundary, if ranked behavior cannot remain deterministic with partial rankings, if the existing exact matcher must be replaced for unranked lanes, or if a priority requires changing manual assignment semantics.

**Review mandate:**

- Prove all-null priorities use the existing exact allocation result.
- Prove ranked lanes are handled only within the active team/string iteration.
- Check a higher-ranked lane reserves its selected UID before the next rank.
- Check unranked lanes still maximize their remaining exact objective.
- Check manual assignments override rank and remain globally reserved.
- Check duplicate ranks are rejected authoritatively and surfaced before save.
- Check rank gaps and a ranked blank lane do not consume unrelated candidates.

#### Commit 3 — Apply preferred-foot optimizer rules

**Status:** Pending

**Provisional commit:** `feat(planner): apply preferred-foot optimizer rules`

**Work:** Persist per-lane foot and Preferred/Strict mode, expose the controls once for the selected lane, filter strict mismatches, and apply the accepted five-point soft mismatch penalty to optimizer allocation only.

**Out of scope:**

- Filtering or rejecting manual picker candidates.
- Persisting an adjusted optimizer score or changing displayed combined role scores.
- Making the penalty configurable or reading richer foot-strength data.

**Implementation packet:**

- Owners and files: migration and tests; Planner tactic types, validation, persistence, and DTOs; `optimizer.rs` candidate loading and matching helpers plus `optimizer_tests.rs`; selected-lane settings UI, tactic draft utilities, IPC mock, route test, Playwright stub and smoke; current-state architecture/design text made true by the commit.
- Existing patterns to verify: the source-scoped optimizer score query, player snapshot `preferred_foot`, stable exact-matcher objective, manual reservation, native select fields, inline save validation, and the Optimize smoke path.
- Constraints and invariants: tactic values are `any`, `left`, `right`, and `both`; player values remain `left`, `right`, `either`, or empty; player `either` matches left, right, and both; `any` always matches; soft mismatch is `max(0, base - 5)`; strict mismatch produces no edge; base combined scores and role-score rows remain unchanged; mode is disabled or normalized when the tactic foot is `any`.
- Dependencies and ordering: depends on commit 1's lane-owned scoring and commit 2's ranked/unranked allocation pipeline so the same allocation score feeds both paths.

**Implementation profile:** Terra High — the rule is bounded and specified, but it crosses persisted enums, source-scoped candidate data, ranked and exact paths, and an accessibility-sensitive control surface.

**Review profile:** Terra High — ordinary cross-layer behavioral review is sufficient when focused tests prove every foot mapping, strict exclusion, soft penalty, and manual override.

**Validation:** Run `./scripts/dev test src/app/routes/planner.test.tsx`, `./scripts/dev check`, and `./scripts/dev smoke`. Expected Rust evidence covers any/left/right/both against left/right/either/empty player values, strict blank lanes, a soft preference overturning at most a five-point base-score gap, floor-at-zero, ranked and unranked paths, and manual mismatches remaining untouched. Frontend and smoke evidence cover saving all four choices, toggling Preferred/Strict, the disabled Either mode, draft retention, and an Optimize run after preferences save.

**Stop conditions:** Replan if live snapshot data uses a two-footed representation other than `either`, if the candidate query must be widened beyond configured sources, if the penalty cannot feed both ranked and exact objectives without changing displayed scores, or if strict rules would have to mutate manual rows.

**Review mandate:**

- Verify the `any` tactic value cannot be confused with player `either`.
- Verify two-footed players satisfy left and right as well as both.
- Verify strict mismatches remove only optimizer lane edges and never manual rows.
- Verify soft mismatches subtract exactly five with a zero floor in ranked and exact paths.
- Verify displayed and persisted combined role scores stay foot-neutral.
- Verify the optimizer score query remains scoped by snapshot, save, team, and configured club sources.
- Check the controls are labelled, keyboard-operable, single in Both view, and preserve failed-save drafts.

## Active work

**PR:** PR 1 — Per-lane optimizer preferences

**Commit:** Prioritize ranked tactic lanes

### RED proof

Add the smallest Rust optimizer test with one ranked lane and one flexible candidate that proves the ranked lane takes that candidate before the exact matcher allocates the remaining unranked lanes. Add the all-unranked control fixture proving the existing exact allocation result is unchanged. Before implementation, the rank-bearing DTO, validation, and allocation-order assertions must fail for the missing lane priority.

Add a focused Planner route test that sets a rank on one selected lane, saves, and reloads it. It must fail because the current selected-lane editor and IPC contract have no priority field.

### Expected outcome

An optional, unique rank is saved on each lane. Within a string, the optimizer reserves eligible players for ranked lanes in ascending rank order, then leaves unranked lanes to the existing exact matcher; all-null ranks retain the current allocation result.

### Explicit exclusions

- Do not add preferred-foot fields or rules.
- Do not refactor unrelated Planner components or scoring modules.
- Do not change manual assignment, age, suitability, team, string, or club-family behavior.

## Discoveries and replanning

- Planning confirmed that the product is unreleased, so tactic migration compatibility is not required. The first migration may reset tactic rows, but it must preserve saves, Planner strings, and assignments.
- The developer confirmed priority is lane order within each existing string, not a replacement for team or string order.
- The developer confirmed manual assignments override every optimizer rule.
- The developer accepted a fixed five-point optimizer-only penalty for soft foot mismatches.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Use per-lane scoring weights | Pending record | Terra xhigh | Terra xhigh, accepted | None |

## Final validation

- `./scripts/dev format` makes no unintended changes.
- `./scripts/dev test` passes the complete frontend suite.
- `./scripts/dev check` passes Biome, TypeScript, secretlint, Rust formatting, Clippy, migrations, and Rust tests.
- `./scripts/dev smoke` passes the complete browser product suite with the extended Planner IPC stub.
- A fresh Sol High feature-complete review traces the exact recorded implementation set and retains no CRITICAL, HIGH, or MEDIUM finding.
- Native populated-state evidence at 1280x800 and 1600x900 shows one shared selected-lane settings surface, every control and status without clipping, keyboard operation and visible focus, saved-state reload, and an Optimize result for ranked, preferred, and strict examples. An empty Planner shell is not sufficient evidence.
- Manual evidence confirms that a deliberately mismatched manual assignment survives Optimize unchanged.
- Mutation testing is recorded as unsupported unless `./scripts/dev mutate` gains a real implementation before feature completion.

## Documentation impact

Planning creates this ledger, marks the feature active in `TODO.md`, and records the planned interaction in `DESIGN.md`. Each implementation commit must update the current-state architecture or design statements that it makes true. Feature completion will reconcile and archive this ledger; no ADR is currently warranted.
