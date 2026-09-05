# Suggested Individual Training

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 7836059db6108ec9b379f0e728c2d56bcab9926de7c7cc2d868b1a99c2db96a7

Ledger state alone creates no branch and switches no branch. The provisional branch `feature/suggested-individual-training` (from `main`) is a recorded intent only until an explicit `/skill:workflow-deliver-feature` invocation with a valid Delivery fingerprint authorizes it.

## Intent

Suggest one FM26 individual-training focus per Club → Squad player that maximizes improvement of the player's current Planner-assigned tactic lane, using the same role catalog and attribute evidence the score model already computes. The suggestion is advisory and read-only: it never writes to FM, snapshots, Planner assignments, tactics, or persistence (Linear JAY-46).

## User-visible behavior

- Club → Squad player table shows a visible default column **Suggested Training** (new layouts; existing persisted Squad layouts that still match the old default gain it; customized layouts are unchanged).
- For a player assigned to a Planner lane, the cell shows the focus name (e.g. `Defensive Positioning` is an example shape; the exact name comes from the Linear inventory below) chosen by the approved ranking rule for that assigned lane.
- The focus cell contains an explicit keyboard-operable **Details** action that stops row activation and opens one Squad-owned Modal built on the existing shared Modal (`src/components/ui/modal/modal.tsx`, no new UI primitive). The Modal shows the focus, the assigned lane's IP/OOP role labels, the focus-mapped and contributing attributes, and the labeled simulated gain.
- A player with no Planner assignment renders `—` with a non-hover-only accessible explanation (`Not assigned to a Planner lane` in the accessible text).
- A player whose suggestion is unavailable renders `—` with a non-hover-only accessible explanation naming the assigned lane roles instead of a guess.
- The column supports the standard header menu: Move left/right, Add column (re-add path), Remove column, resize. The column is explicitly non-sortable: header click never sorts it, it carries no sort affordance, and it never becomes the active sort. Search tables never expose this Squad-only field.
- Rows keep their fixed heights; the cell shows only the focus name or `—`, and all evidence lives in the Modal and accessible text.
- Determinism: the same player, lane, and attributes always produce the same suggestion.

## Invariants

- Target is always the player's current Planner-assigned tactic lane (unique per player per save). No best-role fallback, no user-selected role.
- Whole-inventory precondition: if any current attribute required by any focus in the applicable inventory (outfield or goalkeeper) is missing/null, the whole suggestion is unavailable (`focus: None` with lane roles present). The worker never skips that focus and recommends another. Baseline role-required values remain required (missing lane-role primary/secondary attribute also yields unavailable). The all-zero first-inventory behavior below applies only when the full inventory is evaluable.
- Ranking rule: for each applicable FM26 focus, simulate +1 to every mapped attribute currently below 20, recompute the assigned lane's configured IP/OOP score, and choose the focus with the largest **unrounded** combined-score gain. Respect the lane `ip_weight`. Linear list order is the deterministic final tie-break (strict `>` comparison in inventory order). All-zero gains (e.g. fully maxed attributes) still return the first inventory-order focus, but only when the whole inventory is evaluable.
- No separate current/potential suggestions; no projected-potential gaps; no potential path at all.
- Goalkeeper focus inventory applies only to the GK Planner lane (`lane_id == "goalkeeper"`); the outfield inventory applies otherwise.
- Read-only advisory: no FM write, no snapshot/Planner/tactic/persistence mutation, no new IPC mutation. Tactic reads for this feature must not seed a default tactic as a side effect. A save with zero tactic rows yields all-unassigned cells without calling `load_tactic`. Partial or corrupt tactic rows still call `load_tactic` and surface its error honestly.
- Current visible attributes (`players.attributes_json`) are not concealed by the `reveal_hidden_information` preference; concealed state only affects PA/potential/hidden/personality presentation elsewhere.
- Missing values render `—`, never `null`, `N/A`, `0`, or blank. Colour is never the sole indicator; the focus name / `—` is the fact, and the Modal plus accessible text carry the explanation without hover.
- Thin frontend / thick backend: Rust owns focus ranking; the WebView renders DTO values. No WebView SQL. Rust owns no sort ordering for this field because the column is non-sortable. Shared modules (`src/utils/**`, `src/stores/**`, shared table header) never import `@/features/**` or `@/app/**`; no cross-feature imports (compose in `src/app/routes/my-club.tsx`).
- Bounded 50-row Squad pages are preserved with SQL `LIMIT`/`OFFSET` paging unchanged; per-page suggestion derivation adds no new IPC round-trips.

## Non-goals

- No FM write or bridge action from a suggestion; the UI never implies one.
- No sorting by this column in any surface (header, URL state, backend, mock, or test); the column is configurable, removable, resizable, and re-addable, but never sortable.
- No current-vs-potential wording split (single suggestion; JAY-46 answers NO to differing paths).
- No best-role or user-selected-role targeting; no per-role picker.
- No new UI primitive: evidence uses the existing shared Modal, not a new modal, chart, comparison inspector, tooltip component, or profile surface.
- No Search/Moneyball/Staff columns, filters, or sorts for this field.
- No age-based trainability gating, no physical-decline rules.
- No migration beyond the additive Squad-layout version bump; no new table.
- No release metadata (ordinary feature PR).

## Current-state map

- Relevant components:
  - `src-tauri/src/features/scoring/score.rs` — `score_role` owns the 75/25 primary/secondary blend, `/20*100` scaling, nearest-integer rounding, `None` on missing/null required attributes. No unrounded entry point exists yet.
  - `src-tauri/src/features/scoring/catalog.rs` — 79 static FM26 roles; `DUMP_ATTRIBUTE_KEYS` (PascalCase); every Linear focus label maps to an existing dump key (verified: `FreeKicks`, `Corners`, `PenaltyTaking`, `LongThrows`, `JumpingReach`, `WorkRate`, `OffTheBall`, `LongShots`, `FirstTouch`, `CommandOfArea`, `OneOnOnes`, `RushingOut` all present).
  - `src-tauri/src/features/scoring/combine.rs` — `combine_role_scores(ip, oop, ip_weight)` (rounded `Option<u8>`, `None` on missing score or out-of-range weight).
  - `src-tauri/src/features/planner/tactic.rs` — 11 ordered save-scoped lanes (`DEFAULT_LANE_IDS`, index 0 is `goalkeeper`); `TacticLane` links IP/OOP position+role and owns `ip_weight`; `load_tactic` is `pub(super)` read-only (no seeding) and rejects zero rows because validation requires exactly 11 lanes; `get_tactic` seeds a default. No `EXISTS`/count pre-check exists yet in the Squad path.
  - `src-tauri/src/features/planner/depth.rs` — unique player assignment per save (`planner_assignments`: `save_id, string_id, lane_id, player_uid`); `resolve_assignment` reads persisted lane-role scores and blends with `combine_role_scores`.
  - `src-tauri/src/features/planner/squad.rs` — bounded Club Squad pages (`list_squad_players`, 50-row default, `MAX_SQUAD_PAGE_LIMIT = 200`, SQL `LIMIT`/`OFFSET` paging); joins `player_role_metrics`/Club DNA dynamic fields; does **not** join assignments, tactic, or `attributes_json`. `SquadSortField` has no suggested-training variant and gains none.
  - `src-tauri/src/features/planner/commands.rs` — `SquadPlayerDto` (camelCase) + `DynamicValueDto`; `list_squad_players` IPC (active-save scoped, no mutation).
  - `src-tauri/src/features/player_metrics/resolver.rs` — closed `MetricField` catalog; tactic/Club DNA precedents for synthetic fields. This feature adds no `MetricField` and no sort expression.
  - `src/features/squad/components/squad-overview-panel.tsx` — Squad table adapter; `tableColumnForMetric` falls back to `undefined` for unknown IDs; uses `PlayerTableHeader` default metrics (`PLAYER_METRICS`); cells render `ScoreBadge`/`—`; sort replacement via committed/requested observers. `requestedFields` currently forwards every non-basic column ID and must explicitly exclude the new field. No Modal usage exists yet in this panel.
  - `src/components/ui/modal/modal.tsx` — existing shared Modal with `open`, `onClose`, `title`, focus trap, Escape/backdrop close, and focus restoration (`returnFocusTo`/`fallbackFocusTo`). This is the evidence surface; no new primitive is approved.
  - `src/features/squad/types/squad-player.ts` — frontend DTO mirror (`dynamicValues` optional record).
  - `src/features/squad/types/squad-sort.ts` — `isSquadSortField` accepts only `getPlayerMetric` IDs; `SQUAD_SORT_FIELDS` fixed basics. The new ID stays outside the shared catalog, so this file correctly rejects it as a sort field with no change.
  - `src/components/player-table/player-table-header.tsx` — `ConfigurableTableHeader` takes a table-level `sortable` flag only (click/title/`aria-sort` all key off it); the Add-column menu lists only `metric.sortable` entries. No per-column sortability exists yet; the minimum extension is planned in Commit 4.
  - `src/utils/player-metrics.ts` — shared catalog where `playerMetric` defaults `sortable` to `true`. The new field is not added here; the worker verifies no currently supplied catalog contains `sortable: false` so the header extension changes no existing Add-menu content.
  - `src/stores/use-player-table-store.ts` — persisted per-table layouts, `PLAYER_TABLE_LAYOUT_VERSION = 6`; `search` and `squad` share `DEFAULT_PLAYER_TABLE_COLUMN_IDS` via `defaultLayout`; `isAllowedColumnId` gates synthetic IDs per table via `getPlayerMetric(id)?.sortable === true` for `search`/`squad`. No shared suggested-training module exists yet.
  - `src/app/routes/my-club.tsx` — `validateSearch`/`squadSortForSearch` via `isSquadSortField`; `squadKeys` (`["planner","squad"]`) invalidation. No sort-validation change is needed because the new ID is already rejected as a sort field.
  - Tests/mocks: `src-tauri/src/features/planner/squad_tests.rs` (Rust seam), `src/app/routes/my-club-squad.test.tsx` (route suite), `src/stores/use-player-table-store.test.ts` (layout migration seam), `src/testing/squad-ipc-mock.ts` (local sort + paging mock; carries no suggested-training sort and gains none), `e2e/smoke.spec.ts` Squad overview block (~line 958; asserts headers).
- Data model:
  - `players(snapshot_id, uid, attributes_json, …)` — current visible attributes as nullable JSON map; sparse omissions possible.
  - `planner_tactic_lanes(save_id, lane_order, lane_id, ip_weight, …, ip_position, ip_role_id, oop_position, oop_role_id)`.
  - `planner_assignments(save_id, string_id, lane_id, player_uid, …)` — player UID unique per save.
  - No new table. Suggestion is derived per page row at read time.
- Persistence and migrations:
  - Player-table layouts in `localStorage` (`fm-valuescout-player-table-layouts`), version 6. No SQLite migration.
  - Store migration 6→7 carries the rollout rule (below).
- Existing behavioral assumptions:
  - Squad cohort = exact current-club match with `managed_club_settings`; unconfigured/empty states render guidance, not the table.
  - `get_player` concealment never hides current visible attributes (profile renders current values while concealed; only PA/potential/hidden/personality are withheld).
- Architectural seams:
  - `list_squad_players` (Rust) → `list_squad_players` IPC → `squadPlayersQueryOptions`/`squadKeys` → `SquadOverviewPanel`.
  - Store (`usePlayerTableStore`) ↔ panel via `layouts.squad`.
  - Route (`my-club.tsx`) owns sort validation; header owns menu/sort UI.
- Project validation commands:
  - `./scripts/dev test [target...]` (targeted Vitest), `./scripts/dev test` (full), `./scripts/dev check` (commit gate), `./scripts/dev check-rust` (Rust gate: format, Clippy, tests), `./scripts/dev smoke` (Playwright product suite where the user workflow warrants it).
  - Ledger classifier per the installed workflow skill (`ledger_state.py`); delivery fingerprint via `delivery_state.py` after plan review.
- Primary risks:
  - Shared `DEFAULT_PLAYER_TABLE_COLUMN_IDS` leaking a Squad-only field into Search (must introduce a Squad-specific default instead).
  - Shared-header per-column extension leaking into other tables (mitigated by verifying no existing catalog supplies `sortable: false` and by focused header tests proving existing columns stay sortable).
  - Unrounded-gain math drifting from `score_role`/`combine_role_scores` (mitigated by one shared unrounded helper + tests).
  - Smoke/route suites asserting exact Squad headers breaking on the new default column (expected updates, not regressions).

## Feature architecture

- Backend ownership (all under `src-tauri/src/features/`, planner-scoped):
  - `planner/suggested_training.rs` (new) — owns the two focus inventories with dump-key mappings, GK-vs-outfield selection by `lane_id == "goalkeeper"`, the whole-inventory evaluability precondition, and the pure ranking function `suggest_for_lane(attributes, lane) -> SuggestionOutcome`. Imports the role catalog and the shared unrounded scorer; no SQL.
  - `scoring/score.rs` — gains `score_role_unrounded` (same band means and 75/25 blend, no rounding) used by both baseline and simulation.
  - `planner/squad.rs` — `list_squad_players` performs a read-only `EXISTS`/count check on tactic rows (zero rows means every cell unassigned without calling `load_tactic` and without seeding), otherwise loads the tactic read-only (`load_tactic`, never seeding) so partial/corrupt rows error honestly, maps `player_uid → lane_id` for the save in one query, reads `attributes_json` for page rows, and attaches a per-player suggestion cell value. SQL `LIMIT`/`OFFSET` paging is unchanged. No suggested-training sort path exists anywhere in the backend.
  - `planner/commands.rs` — extends `SquadPlayerDto` with the typed suggestion cell (camelCase, no new command).
- Frontend ownership:
  - `src/utils/suggested-training.ts` (new, neutral shared) — owns `SUGGESTED_TRAINING_COLUMN_ID = "suggested_training"` and a simple predicate (e.g. `isSuggestedTrainingColumnId`) importable by the store and Squad. It owns no label, metric definition, or presentation list. No shared→feature import exists in either direction.
  - `src/features/squad/utils/squad-columns.ts` (new, Squad-owned) — owns the Squad-only metric definition (label `Suggested Training`, left align, width 176, `sortable: false`), the Squad header metrics list (`PLAYER_METRICS` plus the Squad-only entry), and any Squad presentation helpers. `PLAYER_METRICS`, Moneyball catalogs, and Search paths are untouched.
  - `src/components/player-table/player-table-header.tsx` — minimum extension only: per-column sortability controls the header click/title/`aria-sort` for that column, and the Add menu may list a valid non-sortable metric so the Squad-only entry remains re-addable. Existing sortable columns keep identical behavior, proved by focused header tests plus verification that no currently supplied catalog contains `sortable: false`.
  - `squad-overview-panel.tsx` — renders the synthetic column (focus text or `—`), owns one Modal instance with its open state (shared Modal owns focus trap and restoration), renders a keyboard-operable Details action per suggestion cell that stops row activation and opens the Modal with focus, assigned IP/OOP role labels, mapped/contributing attributes, and labeled simulated gain, shows `—` with accessible explanations for unassigned/unavailable states, passes the Squad metrics list to the header, and explicitly excludes the field from `requestedFields` (the DTO always carries it).
  - `src/features/squad/types/squad-player.ts` — mirrors the new DTO field.
  - `src/features/squad/types/squad-sort.ts` — unchanged; the Squad-only ID is not a sort field.
  - `src/stores/use-player-table-store.ts` — version 7: Squad-specific default gains the column at the far right; `isAllowedColumnId` accepts it for `squad` only via the shared predicate (not via the shared metric catalog); v6→v7 migration appends it to persisted Squad layouts that still equal the v6 default, leaving customized layouts unchanged.
- Focus inventories (exact Linear JAY-46 order; order is the tie-break):
  - Outfield (16): Free Kick Taking (`Technique`, `FreeKicks`); Corner Taking (`Technique`, `Corners`); Penalty Taking (`Technique`, `PenaltyTaking`); Long Throws (`LongThrows`); Quickness (`Acceleration`, `Pace`); Agility and Balance (`Agility`, `Balance`); Strength (`JumpingReach`, `Strength`); Endurance (`WorkRate`, `Stamina`); Defensive Positioning (`Marking`, `Decisions`, `Positioning`); Attacking Movement (`Anticipation`, `Decisions`, `OffTheBall`); Shooting (`Finishing`, `LongShots`, `Technique`); Passing (`Passing`, `Technique`, `Vision`); Final Third (`Composure`, `Decisions`); Crossing (`Crossing`, `Technique`); Ball Control (`Dribbling`, `FirstTouch`, `Technique`); Aerial (`Heading`, `Bravery`).
  - Goalkeepers (14): Free Kick Taking; Corner Taking; Penalty Taking; Long Throws; Quickness; Agility and Balance; Strength; Endurance (same eight mappings as outfield); GK Reactions (`Reflexes`, `Anticipation`, `Concentration`); GK Tactical (`Communication`, `Decisions`, `Positioning`); GK Technique (`Handling`, `Composure`, `Technique`); GK Sweeping (`CommandOfArea`, `OneOnOnes`, `RushingOut`); GK Distribution (Long) (`Kicking`, `Throwing`); GK Distribution (Short) (`FirstTouch`, `Passing`, `Vision`).

## Uncertainty register

### Known

- HEAD is `339ea2c9` on `main`; worktree has a pre-existing unrelated modification to `.wiki/features/completed/snapshot-date-edit.md` (untouched by this plan and permitted to remain unstaged).
- No planned spec or active ledger exists for this feature; nothing to remove. `.wiki/TODO.md` already carries this feature as the Active entry and Next remains gender-data work (preserved).
- All 30 focus-mapped dump keys exist in `DUMP_ATTRIBUTE_KEYS` (verified against `catalog.rs`).
- `lane_id == "goalkeeper"` identifies the GK lane (`DEFAULT_LANE_IDS[0]`; default GK lane uses GK-only roles).
- `load_tactic` rejects zero rows because validation requires exactly 11 lanes, so the Squad path needs the `EXISTS`/count pre-check to distinguish absent tactics (all unassigned, no seeding) from partial/corrupt rows (honest error).
- The shared header has only a table-level `sortable` flag and the Add menu filters on `metric.sortable`; the minimum per-column extension is scoped in Commit 4.
- The shared Modal (`src/components/ui/modal/modal.tsx`) already provides dialog semantics, focus trap, Escape/backdrop close, and focus restoration; the Squad panel owns Modal open state only.
- Squad cohort is single-club and small; per-page suggestion evaluation is bounded by `MAX_SQUAD_PAGE_LIMIT` paging and the managed-club membership itself.
- Publication: one short-lived feature branch from `main`, GitHub, `.github/pull_request_template.md`, squash merge, strict required status `check`; no release metadata for ordinary feature work.

### Assumptions

- `attributes_json` carries current visible attributes per Squad-row player with the same nullable-map shape `score_role` consumes (compact ingest validates the visible domain; sparse omissions surface as null).
- No currently supplied metric catalog contains `sortable: false` (worker verifies against current catalog source before extending the header, so the Add-menu change surfaces only the new Squad-only entry).

### Decisions

- One PR (single review/merge boundary). No migration, no new IPC service, no cross-feature contract: every commit is trunk-safe behind the Squad-only column ID and store allowlist. A split would add branch overhead only.
- The column is non-sortable by approved correction: configurability (move, add, remove, resize) is retained, but no URL, backend, mock, or test sort path exists for it. Rationale: suggestion text has no meaningful order and a Rust cohort-ordering path would add unbounded complexity for no user value.
- Suggestion computed at Squad read time from live `attributes_json` + assigned lane (no persisted suggestion, no lazy cache): cohort is small, tactic/assignments change independently, and read-time derivation cannot go stale.
- Tactic presence is checked with a read-only `EXISTS`/count query before `load_tactic` in the Squad path (never `get_tactic`): listing the Squad must not seed a default tactic as a side effect. Zero rows means all cells unassigned. Partial/corrupt rows still call `load_tactic` and error honestly. Rationale: `load_tactic` cannot distinguish absence from corruption because it rejects both.
- Whole-inventory evaluability gates the suggestion: a missing/null value for any attribute required by any focus in the applicable inventory makes the whole cell unavailable. The worker never skips that focus and recommends another. Rationale: skipping would silently substitute a second-best improvement for an incomputable best, which misrepresents the approved ranking rule. Baseline role-required values remain required.
- All-zero gains (e.g. fully maxed attributes) still return the first inventory-order focus, but only when the full inventory is evaluable: deterministic consequence of the approved rule, not a special case.
- `contributing_attributes` = focus-mapped attributes (below 20, actually simulated) that also appear in either lane role's primary/secondary bands, in focus-mapping order. This is the overlap that explains the gain.
- Evidence uses the existing shared Modal opened from an explicit Details action: cell `title`/hover-only text is replaced because hover-only evidence is not keyboard- or touch-reachable. No new UI primitive is introduced. Rationale: the dense table keeps fixed row heights while the Modal carries the full evidence without hover dependence.
- `SUGGESTED_TRAINING_COLUMN_ID` and its predicate live in neutral `src/utils/suggested-training.ts` importable by both the store and Squad; label/metric/presentation list stay Squad-owned. Rationale: the store needs the ID for its allowlist but shared modules must not import from `@/features/**`, and a duplicated literal would create two owners.
- Default position far right after Value (Club DNA append precedent); width 176 left-aligned (shared text-metric width).
- No ADR: single approved ranking rule with no retained meaningful alternative; rationale lives here.

### Unknowns

- None requiring a developer decision. All product behavior, ranking, placement, rollout, and read-only boundaries are explicitly approved in the dispatch. Remaining items are worker-verifiable implementation details with stop conditions in their packets.

### Risks

- Unrounded math drifting from the rounded ingest scores (user-visible lane scores stay rounded; gains are advisory deltas) — mitigate with shared helper + property tests tying unrounded to `score_role` rounding.
- Whole-inventory gating reading as overly strict in review (one missing exotic attribute blanks the cell) — this is the approved correction, so a challenge escalates to a developer decision rather than a silent revert to focus-skipping.
- Shared-header extension affecting other tables — mitigate by verifying no existing catalog supplies `sortable: false` and by focused header tests proving sortable columns keep click/title/`aria-sort` behavior.
- Modal judged excessive chrome for a dense table in review — stop condition in Commit 4 escalates the presentation choice rather than inventing a third evidence surface unprompted.
- Persisted-layout migration misclassifying customized layouts — mitigate with exact-equality check against the v6 default + dedicated store tests.

## Walking skeleton

The thinnest end-to-end path that proves the approach:

1. Add `score_role_unrounded` beside `score_role` with a rounding-parity test.
2. Add `planner/suggested_training.rs` with both inventories and `suggest_for_lane`, proving one outfield ranking + one GK ranking + one whole-inventory-unavailable case in Rust tests.
3. Wire one Squad row end-to-end (tactic presence check + assignment join + DTO + cell text with Details action) before menu, Modal, and migration work.

## Delivery plan

### PR 1 — Suggested individual training in Club Squad

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/suggested-individual-training`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict required GitHub Actions status `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(squad): suggest individual training per assigned lane`

**Purpose:** Deliver the complete advisory Suggested Training column for the Club Squad table in one reviewable, revertible unit: ranking engine, Squad read-model wiring, and Squad-only non-sortable presentation with layout rollout.

**Depends on:** None (no prior PR; HEAD `339ea2c9`).

#### Commit 1 — Record the approved feature plan

**Status:** Active

**Provisional commit:** `docs(squad): record approved feature plan`

**Work:** Commit the independently reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation (notably the pre-existing unrelated modification to `.wiki/features/completed/snapshot-date-edit.md`, which remains untouched and is explicitly permitted to stay unstaged).

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/suggested-individual-training.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — move this feature to Active (ledger link + Linear JAY-46 ref) while preserving the unrelated Next item.
- `.wiki/BACKLOG.md` — no change (no approved deferred scope).
- No ADR (threshold not met); no planned spec to remove.

**Behavior and data flow:**

- Move planning truth into one reviewed active ledger and record the exact delivery sequence before implementation.

**Ordered implementation steps:**

1. Verify the active branch is `feature/suggested-individual-training` with base `main` at/near `339ea2c9` without changing Git state.
2. Confirm the worktree contains only the reviewed planning paths as staged changes (the unrelated `snapshot-date-edit.md` modification stays untouched and unstaged by explicit permission).
3. Run the ledger classifier and any repository documentation check scoped to the selected/staged planning paths only.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The ledger classifier and documentation checks prove structural consistency.

**Patterns to verify:**

- The active-ledger schema 2 template, current TODO ownership rules, and Conventional Commits subject format.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, plan scope, packet order, or reviewed decisions. Do not create/switch branches beyond the fingerprinted activation; ledger state alone grants no branch authority.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** ledger classifier (`ledger_state.py` via the installed workflow skill path) plus the repository documentation check when one exists, both scoped to the selected/staged planning paths only.

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files (the permitted unstaged `snapshot-date-edit.md` modification is out of scope for this check).

#### Commit 2 — Rank training focuses for an assigned tactic lane

**Status:** Pending

**Provisional commit:** `feat(planner): rank individual training focuses per lane`

**Work:** Pure-Rust suggestion engine: unrounded scorer plus focus inventories and the approved ranking function with whole-inventory gating. No query, DTO, or UI change.

**Size assessment:** ~160 changed non-test implementation lines; `Within the soft target`.

**Out of scope:**

- Squad query/DTO wiring (Commit 3), all frontend/store work (Commit 4), persistence or migration changes, IPC changes.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/scoring/score.rs` — add `pub fn score_role_unrounded(attributes: &HashMap<String, Option<u8>>, role: &RoleDefinition) -> Option<f64>` sharing `band_mean` with `score_role` (same 75/25 blend and `/20*100` scale, no rounding).
- `src-tauri/src/features/planner/suggested_training.rs` (new, registered in `src-tauri/src/features/planner/mod.rs`) — own: `OUTFIELD_FOCUSES` (16) and `GOALKEEPER_FOCUSES` (14) as `(&str, &[&str])` in exact Linear order with dump-key mappings from the Feature architecture section; `is_goalkeeper_lane(lane)` (`lane_id == "goalkeeper"`); `pub struct SuggestedFocus { focus: &'static str, focus_attributes: Vec<&'static str>, gain: f64, contributing_attributes: Vec<&'static str> }` where `focus_attributes` is the ordered mapped-attribute list of the chosen inventory entry (engine-owned; `squad.rs` and the frontend consume it without duplicating or re-querying the inventory); `pub fn suggest_for_lane(attributes: &HashMap<String, Option<u8>>, lane: &TacticLane) -> Option<SuggestedFocus>` implementing whole-inventory precondition → baseline → per-focus +1 simulation (only values `< 20`) → max unrounded gain with strict `>` in inventory order → `None` when baseline is incomputable or the inventory is not fully evaluable. Consume `all_roles()` for lane `ip_role_id` / `oop_role_id` definitions and `lane.ip_weight` for the blend.
- Unit tests in both files (inline `#[cfg(test)]`).

**Behavior and data flow:**

- Pure function in, deterministic suggestion out. First check the union of mapped attributes across the applicable inventory: any missing/null value makes the whole result `None` without evaluating another focus. Then compute baseline unrounded IP/OOP phase scores from current attributes; per-focus simulation bumps mapped attributes `< 20` by exactly 1 and recomputes both phase scores; gain = blended simulated minus blended baseline using `lane.ip_weight`; max gain wins, inventory order breaks ties. No I/O, no side effects.

**Ordered implementation steps:**

1. RED: add failing tests — unrounded parity with `score_role` rounding (e.g. fixture role: unrounded `62.5` rounds to `Some(63)`); outfield ranking picks the max-gain focus on a crafted attribute map; GK lane uses the GK inventory; missing lane-role attribute → `None`; missing any focus-mapped attribute in the applicable inventory (e.g. null out `Corners` while ranking an unrelated lane) → `None` with no fallback focus; all-20 map → first inventory focus with `0.0` gain; tie → inventory order.
2. GREEN: implement `score_role_unrounded` + `suggested_training.rs` to the approved rule.
3. Refactor only while the focused proof stays green; keep the engine free of rusqlite/Tauri imports.
4. Run the Rust gate.

**Tests and proof:**

- `score.rs`: unrounded-parity test (asserts `score_role(...) == Some(unrounded.round() as u8)` over fixture + perfect-attribute cases); missing/null attribute → `None`.
- `suggested_training.rs`: hand-computed ranking test (worker crafts attributes where two focuses both gain and asserts the larger wins, including the exact ordered `focus_attributes` of the chosen entry); inventory-order test asserting the exact 16/14 focus-name sequences; GK-vs-outfield selection test on the `goalkeeper` lane vs another lane with identical attributes; whole-inventory unavailable tests (missing lane-role attr; missing one focus-mapped attr anywhere in the applicable inventory → `None`, asserting no fallback focus is returned); all-maxed test (first focus, zero gain, full inventory evaluable); contributing-attributes overlap test.
- Tie each addition to the ranking contract; no GUI or IPC tests here.

**Patterns to verify:**

- `score.rs` 75/25 blend and `combine.rs` weight semantics (blend shape only — engine stays unrounded until comparison); `depth.rs` `resolve_assignment` for how `ip_weight` is consumed; catalog `DUMP_ATTRIBUTE_KEYS` for key validity (add a test asserting every mapped key is a known dump key).

**Constraints and non-goals:**

- Do not touch queries, DTOs, IPC, frontend, store, or migrations. Do not round intermediate gains. Do not guess missing values and do not skip an unevaluable focus. Do not add trainability/age rules.

**Dependencies and sequencing:**

- None (first implementation commit; pure logic).

**Validation:** `./scripts/dev check-rust`.

**Stop conditions:** Stop if any mapped key is not a valid dump key, if the unrounded helper cannot share `band_mean` without changing `score_role` behavior, or if float determinism across runs is in doubt (escalate before inventing epsilon tie-breaks).

**Review mandate:** (1) inventory names/order/keys match Linear exactly; (2) GK selection is exactly `lane_id == "goalkeeper"`; (3) whole-inventory precondition precedes ranking and never falls back; (4) +1 only applies below 20; (5) gains use unrounded phase scores with `ip_weight`; (6) tie-break is strict-`>` inventory order; (7) `None` conditions match the Decisions section; (8) no I/O or persistence imports; (9) tests fail for a wrong-focus, rounded-gain, or focus-skipping implementation.

#### Commit 3 — Attach suggestions to the Squad read model

**Status:** Pending

**Provisional commit:** `feat(squad): attach suggested training to squad pages`

**Work:** Wire the Commit 2 engine into `list_squad_players`: tactic presence check, read-only tactic load, assignment join, per-row suggestion, typed DTO. No frontend change and no sort path.

**Size assessment:** ~90 changed non-test implementation lines; `Within the soft target`.

**Out of scope:**

- Frontend/store/panel work (Commit 4); engine changes (Commit 2, reuse as-is); SQLite migrations (none); Search/query changes; any sort path for this field (URL, backend, mock, or test).

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/planner/squad.rs` — in `list_squad_players`: run a read-only `EXISTS`/count check for tactic rows for the save (zero rows → every cell unassigned without calling `load_tactic` and without seeding); otherwise load tactic read-only via `tactic::load_tactic` (never `get_tactic`; partial/corrupt rows propagate its error honestly); one `SELECT player_uid, lane_id FROM planner_assignments WHERE save_id` map; add `p.attributes_json` to the page SELECT and parse per row to the nullable map shape; attach `suggested_training` per player via `suggested_training::suggest_for_lane` with the row's assigned lane. The DTO's `focus_attributes` is the engine-owned ordered mapped list copied across the boundary (`squad.rs` performs no inventory lookup of its own). Extend `SquadPlayer` with `pub suggested_training: Option<SquadSuggestedTraining>` where `None` = unassigned and `SquadSuggestedTraining { lane_id, ip_role_id, ip_role_display, oop_role_id, oop_role_display, focus: Option<String>, focus_attributes: Vec<String>, contributing_attributes: Vec<String>, combined_gain: Option<f64> }`. Unavailable-with-lane semantics (`focus == None`): the struct keeps lane/role identity while `focus` is `None`, `focus_attributes` and `contributing_attributes` are empty, and `combined_gain` is `None`, consistently (role display names from the role catalog). SQL `LIMIT`/`OFFSET` paging is unchanged. No `SquadSortField` variant is added.
- `src-tauri/src/features/planner/commands.rs` — mirror the new field in `SquadPlayerDto` (camelCase: `suggestedTraining: { laneId, ipRoleId, ipRoleDisplay, oopRoleId, oopRoleDisplay, focus, focusAttributes, contributingAttributes, combinedGain } | null`).
- `src-tauri/src/features/planner/squad_tests.rs` — extend coverage (see Tests and proof).

**Behavior and data flow:**

- Page path: cohort membership unchanged → tactic presence check → tactic (read-only) + assignment map + page rows with attributes → per-row pure suggestion → DTO. Zero tactic rows → all `None` with no tactic write. Unassigned UID → `None`. Assigned but incomputable (missing role-required or any inventory-mapped attribute) → struct keeping lane/role identity with `focus: None`, empty `focus_attributes`/`contributing_attributes`, and `combined_gain: None`. Errors: corrupt `attributes_json` for a row surfaces the existing invalid-data error shape (no guessing, no row skipping); partial/corrupt tactic rows surface the `load_tactic` error honestly.

**Ordered implementation steps:**

1. RED: failing tests — assigned player page carries the expected focus + lane roles + contributing attributes; unassigned player carries `None`; tactic-less save yields all-`None` without creating tactic rows (assert row count unchanged and `load_tactic` never reached); partial tactic rows (e.g. delete half the lanes) surface an honest error rather than silent unassigned cells; missing-attribute player carries `focus: None` with lane roles present (both the lane-role-missing and inventory-missing shapes).
2. GREEN: minimal `squad.rs` + `commands.rs` change to satisfy the tests.
3. Remove nothing (additive field); refactor only while green.
4. Run the Rust gate.

**Tests and proof:**

- Page-attachment test on the `open_with_snapshot` + `add_picker_candidates` fixture with one assigned lane (assert exact focus, ordered mapped `focus_attributes` crossing the boundary, role IDs/displays, contributing attributes, gain sign).
- Tactic-absence test (save with zero tactic rows → all `None`, and tactic-row count unchanged — assert no seeding side effect).
- Partial-tactic test (fewer than 11 rows → honest error, not silent `None` cells).
- Unavailable tests (null out a lane-role attribute → `focus: None` with lane roles present and empty `focus_attributes`/`contributing_attributes`/`combined_gain: None`; null out one inventory-mapped attribute unrelated to the lane roles → same unavailable shape with no fallback focus).
- No-write test: deny-triggers or row-count assertions proving no `planner_tactic_lanes`/`planner_assignments`/player writes.
- Reuse `deny_potential_writes`-style trigger proof where practical.
- No sort tests for this field exist or are added; existing sort/page tests prove the preserved contracts when the column is unused: SQL `LIMIT`/`OFFSET` paging stays bounded, page totals are unchanged, existing sort order and UID tie behavior are unchanged, and the DTO additively gains suggestion data.

**Patterns to verify:**

- `depth.rs::resolve_assignment` (assignment resolution + `combine` weight use); `squad_tests.rs` fixture helpers (`open_with_snapshot`, `add_picker_candidates`, `set_current_role_score`).

**Constraints and non-goals:**

- No seeding of tactic rows from the Squad path. No persistence of suggestions. No change to Search resolvers, filters, Moneyball paths, sort validation, or `SquadSortField`. No frontend-visible string formatting beyond raw DTO values (frontend humanizes keys). Keep the 50-row page bound.

**Dependencies and sequencing:**

- Requires Commit 2 engine (consumed unchanged).

**Validation:** `./scripts/dev check-rust`.

**Stop conditions:** Stop and replan (via developer decision, not silent scope cut) if the Squad path cannot read the tactic without seeding, if `attributes_json` shape diverges from the scorer's map shape, or if the presence check cannot distinguish absent from partial rows.

**Review mandate:** (1) `EXISTS`/count pre-check proven by test with zero rows never reaching `load_tactic`; (2) partial/corrupt rows error honestly; (3) single assignment query (no N+1); (4) `None` vs `focus: None` semantics exact, including whole-inventory gating; (5) no new migration/table and no sort path; (6) DTO camelCase matches frontend mirror; (7) corrupt-attributes error honesty; (8) existing sorts/pages keep their preserved observable contracts when the column is unused (bounded `LIMIT`/`OFFSET`, unchanged totals, unchanged sort order and UID tie behavior, additive DTO gain).

#### Commit 4 — Show Suggested Training in the Squad table

**Status:** Pending

**Provisional commit:** `feat(squad): show suggested training column by default`

**Work:** Squad-only non-sortable presentation: shared ID module, Squad-only column definition, minimum shared-header per-column extension, cell + Details + Squad-owned Modal rendering, Squad-local metrics list, store default v7 + migration, DTO mirror, IPC mock fixtures, route/store/header tests, smoke verification.

**Size assessment:** ~190 changed non-test implementation lines; `Within the soft target` (single coherent Squad presentation outcome; tests excluded from the count).

**Out of scope:**

- Backend ranking/read-model changes (Commits 2–3, consumed unchanged); Search/Moneyball/Staff surfaces; new UI primitives; release work; any sort path for this field.

**Implementation packet:**

**Files and responsibilities:**

- `src/utils/suggested-training.ts` (new, neutral shared) — `SUGGESTED_TRAINING_COLUMN_ID = "suggested_training"` plus a simple predicate (e.g. `isSuggestedTrainingColumnId`). No label, metric, or presentation content. Importable by the store and Squad; imports nothing from `@/features/**` or `@/app/**`.
- `src/features/squad/utils/squad-columns.ts` (new, Squad-owned) — Squad-only metric `{ id, label: "Suggested Training", align: "left", defaultWidth: 176, sortable: false }`, and the Squad header metrics list (shared `PLAYER_METRICS` plus the Squad-only entry). No shared-catalog change.
- `src/components/player-table/player-table-header.tsx` — minimum extension only: per-column sortability gates the header click/title/`aria-sort` for that column, and the Add menu may list a valid non-sortable metric so the Squad-only entry stays re-addable. Before extending, verify no currently supplied catalog contains `sortable: false` so existing Add menus gain no new entries. Focused header tests prove sortable columns keep identical behavior.
- `src/features/squad/components/squad-overview-panel.tsx` — `tableColumnForMetric` fallback for the Squad-only ID (via the Squad metrics list, not the shared catalog); cell renders focus text or `—` (`on-surface-variant`); per-cell keyboard-operable Details action (a real button that stops propagation so row activation never fires) opening one Squad-owned Modal instance with focus, `IP … / OOP …` role displays, mapped/contributing attributes humanized via `labelFromPascal` (the mapped list is the DTO `focusAttributes` consumed as-is; no inventory lookup or duplication in the frontend), and labeled simulated gain (`+X.X`); unassigned vs unavailable accessible strings for the `—` states; fixed row heights preserved; pass the Squad metrics list to `PlayerTableHeader`; explicitly exclude the field from `requestedFields` (DTO always carries it). Panel owns Modal open state; the shared Modal owns focus trap and restoration.
- `src/features/squad/types/squad-player.ts` — mirror `suggestedTraining` (nullable struct: `laneId, ipRoleId, ipRoleDisplay, oopRoleId, oopRoleDisplay, focus, focusAttributes, contributingAttributes, combinedGain`).
- `src/features/squad/types/squad-sort.ts` — no change; the ID is already rejected as a sort field.
- `src/stores/use-player-table-store.ts` — version `6 → 7`: Squad-specific default (`DEFAULT_SQUAD_TABLE_COLUMN_IDS`: v6 squad default + `suggested_training` far right; `search` keeps sharing `DEFAULT_PLAYER_TABLE_COLUMN_IDS` unchanged); `isAllowedColumnId` accepts the ID for `squad` only via the shared predicate; v6→v7 migration appends it to persisted Squad layouts exactly equal to the v6 default, otherwise sanitizes (strips it elsewhere).
- `src/testing/squad-ipc-mock.ts` — support `suggestedTraining` fixtures; no `suggested_training` mock sort path is added (unknown-sort fallback behavior for the ID needs no dedicated test because the UI can never emit it).
- `src/app/routes/my-club-squad.test.tsx` — default header present; suggestion text + Details action opening the Modal with expected focus, role labels, focus-mapped plus contributing attributes, and gain; Modal close returns focus to the invoking Details action; `—` for unassigned and unavailable with accessible explanations; non-sortable header (clicking it never changes sort state); remove/re-add via header menu.
- `src/components/player-table/player-table-header.test.tsx` (new or colocated) — focused header tests: non-sortable column renders no sort affordance, click does not call `onSortChange`, `title`/`aria-sort` carry the non-sortable shape; Add menu lists a valid non-sortable metric; existing sortable columns keep click/title/`aria-sort`/Add behavior.
- `src/app/routes/search.test.tsx` — Search-side absence proof: Search tables/menus never offer the ID and layout sanitization drops it; Search production paths stay behaviorally untouched.
- `src/stores/use-player-table-store.test.ts` — v7 migration: old-default gains the column; customized keeps order/content; other tables reject it; version bump sanitization.
- `e2e/smoke.spec.ts` — verify Squad overview block against the new default header set; update only assertions the new default legitimately changes.

**Behavior and data flow:**

- Layout contains `suggested_training` → column renders from `player.suggestedTraining` (excluded from `requestedFields`; DTO always carries it). `focus != null` → focus text + Details action opening the Modal evidence. Assigned but `focus == null` → `—` + unavailable explanation naming lane roles; Modal still opens from Details with the same explanation. `suggestedTraining == null` → `—` + not-assigned text. Header click on this column never sorts. Remove via menu; re-add via Add column (Squad menu only, backed by the header extension). New/default-matching layouts show it far right; customized layouts untouched.

**Ordered implementation steps:**

1. RED: failing shared-ID/predicate test (if unit-tested at the Squad seam), failing header tests (per-column non-sortable behavior + Add listing), failing route tests (default header; suggestion + Details + Modal content incl. focus restoration; both `—` states; non-sortable click), failing store migration tests.
2. GREEN: shared ID module → Squad-only metric module → header extension → panel render + Details + Modal → store v7 + migration → DTO mirror → mock fixtures, in the smallest slices that turn each proof green.
3. Refactor only while green; keep Search/Moneyball/Staff production paths behaviorally untouched (test files may change for regression proof; verify with diff).
4. Run targeted suites, then full frontend gate + smoke.

**Tests and proof:**

- Route suite: named meaningful cases (default column order/position; Details opens Modal with focus, both role labels, contributing attributes, labeled gain; Escape/close returns focus to Details; unassigned vs unavailable distinction; header click on this column leaves sort state unchanged; menu remove → column gone + query unchanged; menu re-add → column back).
- Header suite: per-column sortability cases above, proving existing sortable columns are unaffected.
- Store suite: migration cases above + cross-table rejection test.
- Search-side absence proof in `search.test.tsx` (picker/menu never offers the ID; layout sanitization drops it); Search/Moneyball/Staff production paths stay behaviorally untouched (test files may change for regression proof).
- Accessibility proof: Details is a keyboard-reachable button, the Modal uses the shared dialog semantics, and no evidence is hover-only (route + smoke coverage, not `title`-only text).
- Smoke: Squad overview passes with the new default; no new IPC stub paths beyond the extended Squad fixture shape.

**Patterns to verify:**

- `squad-overview-panel.tsx` Club DNA `ScoreBadge`/`—` cell pattern; `nationality-cell` accessible-name pattern; `player-table-header.tsx` menu + `metrics` prop override; Club DNA store-append precedent (`addColumns("squad", ["club_dna"])`); existing Modal consumers for `open`/`onClose`/focus-restoration ownership shape.

**Constraints and non-goals:**

- Do not add the ID to `PLAYER_METRICS`/Moneyball catalogs, Search components, filter ASTs, `getPlayerMetric`, or any sort validation. Do not change Search defaults. Do not add a UI primitive, tooltip component, or hover-only evidence. Keep rows at fixed heights; evidence lives in the Modal + accessible text, never in wrapped cell text. Respect `shared → features → app` imports (neutral shared module owns only the ID + predicate; Squad owns label/metric/presentation).

**Dependencies and sequencing:**

- Requires Commits 2–3 (engine + DTO shape consumed unchanged).

**Validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx` then `./scripts/dev test src/app/routes/search.test.tsx` then `./scripts/dev test src/stores/use-player-table-store.test.ts` then `./scripts/dev test` then `./scripts/dev check` then `./scripts/dev smoke`.

**Stop conditions:** Stop if the shared Modal cannot carry the evidence within fixed row heights (escalate the presentation choice, do not invent a new primitive), if the v6→v7 migration cannot distinguish default from customized layouts exactly, if any existing catalog already supplies `sortable: false` (escalate before changing shared-header Add semantics), or if Search exclusion cannot be held structurally (escalate before touching shared catalogs).

**Review mandate:** (1) single literal owner (shared module) with no shared→feature import and no duplicated literal; (2) header diff is the minimum per-column extension with focused tests green and existing sortable behavior unchanged; (3) Search/Moneyball/Staff production paths behaviorally untouched (`search.test.tsx` absence proof green; test files may change for regression proof); (4) store migration exact-equality + customized-layout preservation; (5) Modal shows focus, both role labels, mapped/contributing attributes, labeled gain; (6) no hover-only information (keyboard Details + dialog semantics + focus restoration, all covered by tests); (7) fixed row heights kept; (8) `requestedFields` excludes the field while the DTO always carries it; (9) no sort path for the field in URL, backend, mock, or tests; (10) smoke proves the user-visible default column.

## Active work

**PR:** Suggested individual training in Club Squad

**Commit:** Record the approved feature plan

### RED or removal proof

Not applicable — independently reviewed planning documents only. Proof is the ledger classifier (`ledger_state.py` via the installed workflow skill path) and the repository documentation check scoped to the selected/staged planning paths.

### Expected outcome

Reviewed ledger at `.wiki/features/active/suggested-individual-training.md` plus an Active entry in `.wiki/TODO.md` (Next preserved), committed on `feature/suggested-individual-training` with no implementation or unrelated files (the permitted unstaged `snapshot-date-edit.md` modification remains out of scope).

### Explicit exclusions

Implementation, tests, executable configuration, generated files, the unrelated `snapshot-date-edit.md` modification, BACKLOG changes, ADR creation, and any Linear/GitHub/Git mutation.

## Discoveries and replanning

- None yet.

## Completed work

No completed commits yet. Schema 2 requires exactly one row per `Completed` commit; rows are added as commits complete.

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |

## Final validation

- `./scripts/dev test` (full) green.
- `./scripts/dev check` (commit gate) green.
- `./scripts/dev smoke` green (user-visible table workflow with Modal evidence).
- Feature review clear with no blocking findings.
- Documentation reconciliation: TODO active entry, completed-record archive on close-out, no ADR, BACKLOG unchanged.

## Documentation impact

Complete during reconciliation (TODO Active entry in Commit 1; close-out archive move before final merge).
