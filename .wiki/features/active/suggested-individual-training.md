# Suggested Individual Training

## Status

Validation

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** f6798487ce9de155379b037cd6dee21f901debfb380566fd31e90efb8c82cbea

Ledger state alone creates no branch and switches no branch. The provisional branch `feature/suggested-individual-training` (from `main`) is a recorded intent only until an explicit `/skill:workflow-deliver-feature` invocation with a valid Delivery fingerprint authorizes it.

## Intent

Suggest one FM26 individual-training focus name per Club → Squad player that maximizes improvement of the player's current Planner-assigned tactic lane, using the same role catalog and score model the app already computes. The cell shows only the focus name, or `—` when there is no suggestion. The suggestion is advisory and read-only: it never writes to FM, snapshots, Planner assignments, tactics, or persistence (Linear JAY-46). There is no Details action, no Modal, and no supporting evidence UI.

## User-visible behavior

- Club → Squad player table shows a visible default column **Suggested Training** (new layouts; existing persisted Squad layouts that still match the old default gain it; customized layouts are unchanged).
- For a player assigned to a Planner lane, the cell shows only the focus name chosen by the approved ranking rule for that assigned lane.
- A player with no Planner assignment renders `—` with an accessible name stating there is no suggestion because the player is not assigned to a Planner lane.
- A player whose suggestion is unavailable renders `—` with an accessible name stating there is no suggestion.
- If `CA >= PA`, the cell is always `—`: ranking must not produce a displayed focus for a fully developed player.
- No Details action, no Modal, no gain, no role labels, and no mapped or contributing attribute evidence exist anywhere in the UI.
- The column supports the standard header menu: Move left/right, Add column (re-add path), Remove column, resize. The column is explicitly non-sortable: header click never sorts it, it carries no sort affordance, and it never becomes the active sort. Search tables never expose this Squad-only field.
- Rows keep their fixed heights; the cell shows only the focus name or `—`.
- Determinism: the same player, lane, and attributes always produce the same suggestion.

## Invariants

- Target is always the player's current Planner-assigned tactic lane (unique per player per save). No best-role fallback, no user-selected role.
- Whole-inventory precondition: if any current attribute required by any focus in the applicable inventory (outfield or goalkeeper) is missing/null, there is no suggestion (`None`, rendered as `—`). The worker never skips that focus and recommends another. Baseline role-required values remain required (a missing lane-role primary/secondary attribute also yields no suggestion). The all-zero first-inventory behavior below applies only when the full inventory is evaluable.
- Ranking rule: for each applicable FM26 focus, simulate +1 to every mapped attribute currently below 20, recompute the assigned lane's configured IP/OOP score, and choose the focus with the largest **unrounded** combined-score gain. Respect the lane `ip_weight`. Linear list order is the deterministic final tie-break (strict `>` comparison in inventory order). All-zero gains (e.g. fully maxed attributes) still return the first inventory-order focus, but only when the whole inventory is evaluable. Gain is comparison-local only: it is never displayed or exported.
- No separate current/potential suggestions; no projected-potential gaps; no potential path at all.
- Goalkeeper focus inventory applies only to the GK Planner lane (`lane_id == "goalkeeper"`); the outfield inventory applies otherwise.
- Read-only advisory: no FM write, no snapshot/Planner/tactic/persistence mutation, no new IPC mutation. Tactic reads for this feature must not seed a default tactic as a side effect. A save with zero tactic rows yields all-unassigned cells without calling `load_tactic`. Partial or corrupt tactic rows still call `load_tactic` and surface its error honestly.
- Current visible attributes (`players.attributes_json`) are not concealed by the `reveal_hidden_information` preference; concealed state only affects PA/potential/hidden/personality presentation elsewhere.
- Cells render `—`, never `null`, `N/A`, `0`, or blank. Colour is never the sole indicator; the focus name or `—` is the fact, exposed through an appropriate accessible name for the visible value or dash. The UI carries no explanation beyond that name.
- Thin frontend / thick backend: Rust owns focus ranking; the WebView renders DTO values. No WebView SQL. Rust owns no sort ordering for this field because the column is non-sortable. Shared modules (`src/utils/**`, `src/stores/**`, shared table header) never import `@/features/**` or `@/app/**`; no cross-feature imports (compose in `src/app/routes/my-club.tsx`).
- Bounded 50-row Squad pages are preserved with SQL `LIMIT`/`OFFSET` paging unchanged; per-page suggestion derivation adds no new IPC round-trips.
- Fully developed players get no suggestion: when `CA >= PA`, the Squad path returns `None` before ranking, so the cell is always `—` and no focus is produced or displayed. The gate lives in the Squad path because the pure ranker never sees CA/PA.
- One null/string outward contract: unassigned, unavailable, and fully developed states all collapse to `None` (`null` over IPC, rendered as `—`). The UI only needs value-or-dash, so no lane, role, attribute, or gain field crosses the outward boundary.

## Non-goals

- No FM write or bridge action from a suggestion; the UI never implies one.
- No sorting by this column in any surface (header, URL state, backend, mock, or test); the column is configurable, removable, resizable, and re-addable, but never sortable.
- No current-vs-potential wording split (single suggestion; JAY-46 answers NO to differing paths).
- No best-role or user-selected-role targeting; no per-role picker.
- No evidence UI: no Details action, no Modal, no gain display, no role labels, no mapped or contributing attribute display, and no replacement tooltip, chart, inspector, or profile surface. The developer explicitly rejected that evidence UI.
- No Search/Moneyball/Staff columns, filters, or sorts for this field.
- No age-based trainability gating, no physical-decline rules.
- No migration beyond the additive Squad-layout version bump; no new table.
- No release metadata (ordinary feature PR).

## Current-state map

- Relevant components:
  - `src-tauri/src/features/scoring/score.rs` — `score_role` owns the 75/25 primary/secondary blend, `/20*100` scaling, nearest-integer rounding, `None` on missing/null required attributes. Commit 2 added the shared unrounded entry point `score_role_unrounded` beside it.
  - `src-tauri/src/features/scoring/catalog.rs` — 79 static FM26 roles; `DUMP_ATTRIBUTE_KEYS` (PascalCase); every Linear focus label maps to an existing dump key (verified: `FreeKicks`, `Corners`, `PenaltyTaking`, `LongThrows`, `JumpingReach`, `WorkRate`, `OffTheBall`, `LongShots`, `FirstTouch`, `CommandOfArea`, `OneOnOnes`, `RushingOut` all present).
  - `src-tauri/src/features/scoring/combine.rs` — `combine_role_scores(ip, oop, ip_weight)` (rounded `Option<u8>`, `None` on missing score or out-of-range weight).
  - `src-tauri/src/features/planner/suggested_training.rs` (new in Commit 2) — owns both focus inventories, GK-vs-outfield selection by `lane_id == "goalkeeper"`, the whole-inventory precondition, and `suggest_for_lane`. Commit 4 removes `SuggestedFocus` entirely so `suggest_for_lane` returns the winning focus name only (`Option<&'static str>`); the best gain stays a local variable inside the ranking loop and never crosses a module boundary.
  - `src-tauri/src/features/planner/tactic.rs` — 11 ordered save-scoped lanes (`DEFAULT_LANE_IDS`, index 0 is `goalkeeper`); `TacticLane` links IP/OOP position+role and owns `ip_weight`; `load_tactic` is `pub(super)` read-only (no seeding) and rejects zero rows because validation requires exactly 11 lanes; `get_tactic` seeds a default. The Squad path (Commit 3) counts rows before calling it, so absence never reaches `load_tactic`.
  - `src-tauri/src/features/planner/depth.rs` — unique player assignment per save (`planner_assignments`: `save_id, string_id, lane_id, player_uid`); `resolve_assignment` reads persisted lane-role scores and blends with `combine_role_scores`.
  - `src-tauri/src/features/planner/squad.rs` — bounded Club Squad pages (`list_squad_players`, 50-row default, `MAX_SQUAD_PAGE_LIMIT = 200`, SQL `LIMIT`/`OFFSET` paging); joins `player_role_metrics`/Club DNA dynamic fields. Commit 3 added a read-only tactic presence check, read-only tactic load, a one-query assignment map, `attributes_json` reads, and per-row `suggested_training` as `Option<SquadSuggestedTraining>` (nested lane/role/evidence struct; Commit 4 simplifies it to `Option<String>`). `SquadSortField` has no suggested-training variant and gains none.
  - `src-tauri/src/features/planner/commands.rs` — `SquadPlayerDto` (camelCase) + `DynamicValueDto`; `list_squad_players` IPC (active-save scoped, no mutation). Commit 3 added the nested `suggestedTraining` DTO object (Commit 4 simplifies it to `string | null`).
  - `src-tauri/src/features/player_metrics/resolver.rs` — closed `MetricField` catalog; tactic/Club DNA precedents for synthetic fields. This feature adds no `MetricField` and no sort expression.
  - `src/features/squad/components/squad-overview-panel.tsx` — Squad table adapter; `tableColumnForMetric` falls back to `undefined` for unknown IDs; uses `PlayerTableHeader` default metrics (`PLAYER_METRICS`); cells render `ScoreBadge`/`—`; sort replacement via committed/requested observers. `requestedFields` currently forwards every non-basic column ID and must explicitly exclude the new field. No suggestion work exists in this panel yet (the interrupted Commit 4 never landed).
  - `src/components/ui/modal/modal.tsx` — existing shared Modal with `open`, `onClose`, `title`, focus trap, Escape/backdrop close, and focus restoration (`returnFocusTo`/`fallbackFocusTo`). This feature does not use it.
  - `src/features/squad/types/squad-player.ts` — frontend DTO mirror (`dynamicValues` optional record).
  - `src/features/squad/types/squad-sort.ts` — `isSquadSortField` accepts only `getPlayerMetric` IDs; `SQUAD_SORT_FIELDS` fixed basics. The new ID stays outside the shared catalog, so this file correctly rejects it as a sort field with no change.
  - `src/components/player-table/player-table-header.tsx` — `ConfigurableTableHeader` takes a table-level `sortable` flag only (click/title/`aria-sort` all key off it); the Add-column menu lists only `metric.sortable` entries. No per-column sortability exists yet; the minimum extension is planned in Commit 5.
  - `src/utils/player-metrics.ts` — shared catalog where `playerMetric` defaults `sortable` to `true`. The new field is not added here; the worker verifies no currently supplied catalog contains `sortable: false` so the header extension changes no existing Add-menu content.
  - `src/stores/use-player-table-store.ts` — persisted per-table layouts, `PLAYER_TABLE_LAYOUT_VERSION = 6`; `search` and `squad` share `DEFAULT_PLAYER_TABLE_COLUMN_IDS` via `defaultLayout`; `isAllowedColumnId` gates synthetic IDs per table via `getPlayerMetric(id)?.sortable === true` for `search`/`squad`. No shared suggested-training module exists yet.
  - `src/app/routes/my-club.tsx` — `validateSearch`/`squadSortForSearch` via `isSquadSortField`; `squadKeys` (`["planner","squad"]`) invalidation. No sort-validation change is needed because the new ID is already rejected as a sort field.
  - Tests/mocks: `src-tauri/src/features/planner/squad_tests.rs` (Commit 3 attachment tests; evidence and gain assertions removed in Commit 4), `planner/suggested_training.rs` inline engine tests (evidence and gain assertions removed in Commit 4), `src/app/routes/my-club-squad.test.tsx` (route suite; no suggestion coverage yet), `src/stores/use-player-table-store.test.ts` (layout migration seam; still version 6), `src/testing/squad-ipc-mock.ts` (local sort + paging mock; carries no suggested-training sort and gains none), `e2e/smoke.spec.ts` Squad overview block (~line 958; asserts headers).
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
  - `planner/suggested_training.rs` (Commit 2) — owns the two focus inventories with dump-key mappings, GK-vs-outfield selection by `lane_id == "goalkeeper"`, the whole-inventory evaluability precondition, and the pure ranking function `suggest_for_lane`. Imports the role catalog and the shared unrounded scorer; no SQL. Commit 4 deletes `SuggestedFocus` entirely so the function returns the winning focus name only (`Option<&'static str>`); the best gain stays a local variable inside the ranking loop.
  - `scoring/score.rs` — owns `score_role_unrounded` (same band means and 75/25 blend, no rounding; added in Commit 2) used by both baseline and simulation.
  - `planner/squad.rs` — `list_squad_players` performs a read-only `EXISTS`/count check on tactic rows (zero rows means every cell unassigned without calling `load_tactic` and without seeding), otherwise loads the tactic read-only (`load_tactic`, never seeding) so partial/corrupt rows error honestly, maps `player_uid → lane_id` for the save in one query, reads `attributes_json` plus CA/PA for page rows, and attaches a per-player suggestion. Commit 4 simplifies the cell to `Option<String>`: `suggestion_for_player` returns `None` for unassigned, tactic-less, incomputable, and `ca >= pa` rows (gate before ranking) and the winning focus name otherwise. SQL `LIMIT`/`OFFSET` paging is unchanged. No suggested-training sort path exists anywhere in the backend.
  - `planner/commands.rs` — carries `suggestedTraining: string | null` on `SquadPlayerDto` (camelCase, no new command).
- Frontend ownership:
  - `src/utils/suggested-training.ts` (new, neutral shared, required) — owns `SUGGESTED_TRAINING_COLUMN_ID = "suggested_training"` and a simple predicate (e.g. `isSuggestedTrainingColumnId`), imported by both the global store and Squad. It owns no label, metric definition, or presentation list. No shared→feature import exists in either direction.
  - `src/features/squad/utils/squad-columns.ts` (new, Squad-owned) — owns the Squad-only metric definition (label `Suggested Training`, left align, width 176, `sortable: false`), the Squad header metrics list (`PLAYER_METRICS` plus the Squad-only entry), and any Squad presentation helpers. `PLAYER_METRICS`, Moneyball catalogs, and Search paths are untouched.
  - `src/components/player-table/player-table-header.tsx` — minimum extension only: per-column sortability controls the header click/title/`aria-sort` for that column, and the Add menu may list a valid non-sortable metric so the Squad-only entry remains re-addable. Existing sortable columns keep identical behavior, proved by focused header tests plus verification that no currently supplied catalog contains `sortable: false`.
  - `squad-overview-panel.tsx` — renders the synthetic column (focus string or `—` with an appropriate accessible name for the visible value or dash), owns no Modal and no details state, passes the Squad metrics list to the header, and explicitly excludes the field from `requestedFields` (the DTO always carries it).
  - `src/features/squad/types/squad-player.ts` — mirrors the new DTO field as `suggestedTraining: string | null`.
  - `src/features/squad/types/squad-sort.ts` — unchanged; the Squad-only ID is not a sort field.
  - `src/stores/use-player-table-store.ts` — version 7: Squad-specific default gains the column at the far right; `isAllowedColumnId` accepts it for `squad` only via the shared predicate from `src/utils/suggested-training.ts`; v6→v7 migration appends it to persisted Squad layouts that still equal the v6 default, leaving customized layouts unchanged.
- Focus inventories (exact Linear JAY-46 order; order is the tie-break):
  - Outfield (16): Free Kick Taking (`Technique`, `FreeKicks`); Corner Taking (`Technique`, `Corners`); Penalty Taking (`Technique`, `PenaltyTaking`); Long Throws (`LongThrows`); Quickness (`Acceleration`, `Pace`); Agility and Balance (`Agility`, `Balance`); Strength (`JumpingReach`, `Strength`); Endurance (`WorkRate`, `Stamina`); Defensive Positioning (`Marking`, `Decisions`, `Positioning`); Attacking Movement (`Anticipation`, `Decisions`, `OffTheBall`); Shooting (`Finishing`, `LongShots`, `Technique`); Passing (`Passing`, `Technique`, `Vision`); Final Third (`Composure`, `Decisions`); Crossing (`Crossing`, `Technique`); Ball Control (`Dribbling`, `FirstTouch`, `Technique`); Aerial (`Heading`, `Bravery`).
  - Goalkeepers (14): Free Kick Taking; Corner Taking; Penalty Taking; Long Throws; Quickness; Agility and Balance; Strength; Endurance (same eight mappings as outfield); GK Reactions (`Reflexes`, `Anticipation`, `Concentration`); GK Tactical (`Communication`, `Decisions`, `Positioning`); GK Technique (`Handling`, `Composure`, `Technique`); GK Sweeping (`CommandOfArea`, `OneOnOnes`, `RushingOut`); GK Distribution (Long) (`Kicking`, `Throwing`); GK Distribution (Short) (`FirstTouch`, `Passing`, `Vision`).

## Uncertainty register

### Known

- HEAD is `076a77f1` on `feature/suggested-individual-training` (base `main`); Commits 1–3 are complete with the refs in Completed work. The original Commit 4 partial work was interrupted before validation/review and discarded from the worktree; a recovery stash exists but is not delivery truth. The worktree also carries the pre-existing unrelated modification to `.wiki/features/completed/snapshot-date-edit.md` (untouched by this plan and permitted to remain unstaged).
- No planned spec or active ledger exists for this feature; nothing to remove. `.wiki/TODO.md` already carries this feature as the Active entry and Next remains gender-data work (preserved).
- All 30 focus-mapped dump keys exist in `DUMP_ATTRIBUTE_KEYS` (verified against `catalog.rs`).
- `lane_id == "goalkeeper"` identifies the GK lane (`DEFAULT_LANE_IDS[0]`; default GK lane uses GK-only roles).
- `load_tactic` rejects zero rows because validation requires exactly 11 lanes, so the Squad path needs the `EXISTS`/count pre-check to distinguish absent tactics (all unassigned, no seeding) from partial/corrupt rows (honest error).
- The shared header has only a table-level `sortable` flag and the Add menu filters on `metric.sortable`; the minimum per-column extension is scoped in Commit 5.
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
- No evidence UI by developer approval: the cell shows only the focus name or `—` with an accessible name for the visible value or dash. There is no Details action, Modal, gain, role labels, mapped or contributing attribute display, and no replacement tooltip, chart, inspector, or profile surface. Rationale: the developer explicitly rejected that evidence UI and the solo-dev scope favors the smallest contract that answers the question.
- One null/string outward contract is intentional: unassigned, unavailable, and fully developed (`CA >= PA`) all collapse to `None`/`null`/`—` because the UI only needs value-or-dash. No lane, role, attribute, or gain field crosses the Squad DTO boundary.
- The max-CA gate lives in the Squad path (`suggestion_for_player`, which already reads CA/PA per row), not in the pure ranker: `ca >= pa` returns `None` before ranking so no focus is produced or displayed. Rationale: the ranker never sees CA/PA and its approved rule is unchanged.
- `SUGGESTED_TRAINING_COLUMN_ID` and its predicate live in neutral `src/utils/suggested-training.ts`, imported by both the global store and Squad. Label/metric/presentation list stay Squad-owned in `src/features/squad/utils/squad-columns.ts`. Rationale: one literal owner without a shared→feature import.
- Default position far right after Value (Club DNA append precedent); width 176 left-aligned (shared text-metric width).
- No ADR: single approved ranking rule with no retained meaningful alternative; rationale lives here.

### Unknowns

- None requiring a developer decision. All product behavior, ranking, placement, rollout, and read-only boundaries are explicitly approved. Remaining items are worker-verifiable implementation details with stop conditions in their packets.

### Risks

- Unrounded math drifting from the rounded ingest scores (user-visible lane scores stay rounded; gains are advisory and never displayed) — mitigate by keeping the shared helper + parity tests from Commit 2 unchanged.
- Whole-inventory gating reading as overly strict in review (one missing exotic attribute blanks the cell) — this is the approved correction, so a challenge escalates to a developer decision rather than a silent revert to focus-skipping.
- Shared-header extension affecting other tables — mitigate by verifying no existing catalog supplies `sortable: false` and by focused header tests proving sortable columns keep click/title/`aria-sort` behavior.
- Simplicity challenged in review as hiding useful signal — this is the approved developer contract, so a challenge escalates to a developer decision rather than re-adding evidence UI unprompted.
- Persisted-layout migration misclassifying customized layouts — mitigate with exact-equality check against the v6 default + dedicated store tests.

## Walking skeleton

The thinnest end-to-end path that proves the approach:

1. Trim the outward contract to a focus string or null with the `ca >= pa` gate, proving focus-or-null plus the gate in Rust tests while ranking stays unchanged.
2. Wire one Squad row end-to-end (tactic presence check + assignment join + string DTO + cell text or `—`) before menu and migration work.

## Delivery plan

### PR 1 — Suggested individual training in Club Squad

**Status:** Ready for publication

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

**Purpose:** Deliver the complete advisory Suggested Training column for the Club Squad table in one reviewable, revertible unit: ranking engine, Squad read-model wiring with a value-or-dash string contract, and Squad-only non-sortable presentation with layout rollout.

**Depends on:** None (single PR; Commits 1–3 complete on this branch).

#### Commit 1 — Record the approved feature plan

**Status:** Completed

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

**Status:** Completed

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

**Status:** Completed

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

#### Commit 4 — Simplify the outward suggestion to value-or-dash

**Status:** Completed

**Provisional commit:** `feat(squad): simplify suggested training to value-or-dash`

**Work:** Backend simplification: trim the outward suggestion contract to a focus string or null, add the max-CA gate, and remove obsolete evidence fields and tests. No frontend change.

**Size assessment:** ~60 changed non-test implementation lines; `Within the soft target`.

**Out of scope:**

- Frontend/store/panel work (Commit 5); ranking-rule changes (Commit 2 engine, consumed unchanged); SQLite migrations (none); Search/query changes; any sort path for this field.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/planner/suggested_training.rs` — delete `SuggestedFocus` entirely and change `suggest_for_lane` to return the winning focus name only (`Option<&'static str>`); keep the best gain as a local variable inside the ranking loop for comparison. Delete `focus_attributes`, `contributing_attributes`, every exported gain, and every helper that exists only to build them. Ranking math, inventory order, whole-inventory gating, and tie-break stay behaviorally identical. Update inline engine tests to assert focus names only with no gain assertions (see Tests and proof).
- `src-tauri/src/features/planner/squad.rs` — delete `SquadSuggestedTraining`; change `SquadPlayer.suggested_training` to `Option<String>` (focus name). `suggestion_for_player` takes the row CA/PA and returns `None` without ranking when `ca >= pa`; otherwise it ranks as before and maps the winner to its focus name, keeping `None` for unassigned, tactic-less, and incomputable rows. No role, attribute, or gain value leaves this module.
- `src-tauri/src/features/planner/commands.rs` — delete `SquadSuggestedTrainingDto` and its `From` impl; change `SquadPlayerDto.suggested_training` to `Option<String>` (`suggestedTraining: string | null`, no new command).
- `src-tauri/src/features/planner/squad_tests.rs` — remove evidence assertions/helpers; keep ranking and missing-input coverage; add max-CA gate tests.

**Behavior and data flow:**

- Page path is unchanged until the per-row suggestion: `ca >= pa` short-circuits to `None` before the engine runs, so a fully developed player never produces a displayed focus. Otherwise the approved ranking runs unchanged and only the winning focus name crosses into the DTO. Unassigned, unavailable, and fully developed states all collapse to one null rendered as `—` downstream. Errors stay honest: corrupt `attributes_json` surfaces the existing invalid-data error; partial/corrupt tactic rows surface the `load_tactic` error.

**Ordered implementation steps:**

1. RED: add failing page-level tests — a player with `ca == pa` and a player with `ca > pa` both carry `suggested_training: None`; a control player with `pa > ca` on the same lane still carries the expected focus.
2. GREEN: trim the structs and DTO, thread row CA/PA into `suggestion_for_player`, add the gate, and delete evidence-only code and tests. Keep every surviving ranking and missing-input test green without changing its expectation.
3. Refactor only while green; run `./scripts/dev check-rust` then `./scripts/dev check`.

**Tests and proof:**

- New gate tests (page level in `squad_tests.rs`): `ca == pa` → `None`; `ca > pa` → `None`; control `pa > ca` → expected focus string.
- Preserved tests: exact ranking outcomes (focus names only, no gain assertions), inventory order, GK-vs-outfield selection, whole-inventory unavailable (`None`), all-maxed first-focus behavior — expectations unchanged except evidence fields and gain assertions are gone.
- Removed tests/helpers: every assertion on `focus_attributes`, `contributing_attributes`, any `gain` value or `SuggestedFocus` shape, lane/role display identity, and any fixture helper that exists only to build those. No absence test is needed for deleted struct fields: compilation plus the surviving suite proves the removal.
- No sort tests exist or are added; existing sort/page tests keep proving bounded paging and unchanged order. The full `./scripts/dev check` gate proves no frontend, store, or header contract broke.

**Patterns to verify:**

- `squad_tests.rs` fixture helpers (`open_with_snapshot`, `add_picker_candidates`); `depth.rs::resolve_assignment` for weight use (unchanged).

**Constraints and non-goals:**

- Do not change the ranking rule, inventories, tie-break, whole-inventory gating, or rounding behavior. Do not touch the frontend, store, IPC surface beyond the field shape, Search resolvers, `SquadSortField`, or migrations. Do not round or export gains. Keep the 50-row page bound.

**Dependencies and sequencing:**

- Requires Commits 2–3 (engine ranking and page wiring, consumed unchanged apart from the trimmed contract).

**Validation:** `./scripts/dev check-rust` then `./scripts/dev check` (full repository commit gate).

**Stop conditions:** Stop if any surviving ranking expectation changes (the rule must not move), if CA/PA cannot reach `suggestion_for_player` from the already-read row without a new query, or if removing the nested struct forces an IPC surface change beyond the field shape (escalate before widening scope).

**Review mandate:** (1) ranking behavior identical for `pa > ca`; (2) `ca >= pa` never yields a focus; (3) no `SuggestedFocus` type and no `focus_attributes`, `contributing_attributes`, `gain`, lane, or role value crosses any module boundary (best gain stays local to the ranker); (4) DTO is `string | null`; (5) obsolete evidence tests/helpers and gain assertions are gone while ranking and missing-input tests survive; (6) no frontend, store, migration, or sort-path change in this commit; (7) both `./scripts/dev check-rust` and the full `./scripts/dev check` gate pass.

#### Commit 5 — Show Suggested Training in the Squad table

**Status:** Completed

**Provisional commit:** `feat(squad): show suggested training column by default`

**Work:** Squad-only non-sortable presentation of the Commit 4 string contract: column definition, minimum shared-header per-column extension, simple value-or-dash cell, Squad-local metrics list, store default v7 + migration, DTO mirror, IPC mock fixtures, focused tests, smoke verification. No Modal, no details state, no evidence UI.

**Size assessment:** ~120 changed non-test implementation lines; `Within the soft target` (single coherent Squad presentation outcome; tests excluded from the count).

**Out of scope:**

- Backend ranking/read-model changes (Commits 2–4, consumed unchanged); Search/Moneyball/Staff surfaces; new UI primitives, Details actions, Modals, tooltips, or any evidence UI; release work; any sort path for this field.

**Implementation packet:**

**Files and responsibilities:**

- `src/utils/suggested-training.ts` (new, neutral shared, required) — `SUGGESTED_TRAINING_COLUMN_ID = "suggested_training"` plus a simple predicate (e.g. `isSuggestedTrainingColumnId`), imported by both the global store and Squad. No label, metric, or presentation content. Imports nothing from `@/features/**` or `@/app/**`.
- `src/features/squad/utils/squad-columns.ts` (new, Squad-owned) — Squad-only metric (label `Suggested Training`, left align, width 176, `sortable: false`) and the Squad header metrics list (shared `PLAYER_METRICS` plus the Squad-only entry). No shared-catalog change.
- `src/components/player-table/player-table-header.tsx` — minimum extension only: per-column sortability gates the header click/title/`aria-sort` for that column, and the Add menu may list a valid non-sortable metric so the Squad-only entry stays re-addable. Before extending, verify no currently supplied catalog contains `sortable: false` so existing Add menus gain no new entries. Focused header tests prove sortable columns keep identical behavior.
- `src/features/squad/components/squad-overview-panel.tsx` — `tableColumnForMetric` fallback for the Squad-only ID (via the Squad metrics list, not the shared catalog); cell renders the focus string or `—` with an appropriate accessible name for the visible value or dash; no Modal, no open state, no Details action; fixed row heights preserved; pass the Squad metrics list to `PlayerTableHeader`; explicitly exclude the field from `requestedFields` (the DTO always carries it).
- `src/features/squad/types/squad-player.ts` — mirror `suggestedTraining: string | null`.
- `src/features/squad/types/squad-sort.ts` — no change; the ID is already rejected as a sort field.
- `src/stores/use-player-table-store.ts` — version `6 → 7`: Squad-specific default (`DEFAULT_SQUAD_TABLE_COLUMN_IDS`: v6 squad default + `suggested_training` far right; `search` keeps sharing `DEFAULT_PLAYER_TABLE_COLUMN_IDS` unchanged); `isAllowedColumnId` accepts the ID for `squad` only via the shared predicate from `src/utils/suggested-training.ts`; v6→v7 migration appends it to persisted Squad layouts exactly equal to the v6 default, otherwise sanitizes (strips it elsewhere).
- `src/testing/squad-ipc-mock.ts` — support `suggestedTraining: string | null` fixtures; no mock sort path is added (the UI can never emit it).
- `src/app/routes/my-club-squad.test.tsx` — default header present; focus text renders; `—` renders for null with the accessible name; non-sortable header (clicking it never changes sort state); remove/re-add via header menu.
- `src/components/player-table/player-table-header.test.tsx` (new or colocated) — non-sortable column renders no sort affordance, click does not call `onSortChange`, `title`/`aria-sort` carry the non-sortable shape; Add menu lists a valid non-sortable metric; existing sortable columns keep click/title/`aria-sort`/Add behavior.
- `src/app/routes/search.test.tsx` — Search-side absence proof: Search tables/menus never offer the ID and layout sanitization drops it; Search production paths stay behaviorally untouched.
- `src/stores/use-player-table-store.test.ts` — v7 migration: old-default gains the column; customized keeps order/content; other tables reject it; version bump sanitization.
- `e2e/smoke.spec.ts` — verify Squad overview block against the new default header set; update only assertions the new default legitimately changes.

**Behavior and data flow:**

- Layout contains `suggested_training` → column renders from `player.suggestedTraining` (excluded from `requestedFields`; DTO always carries it). A focus string renders as-is; `null` renders `—` with the accessible name. Header click on this column never sorts. Remove via menu; re-add via Add column (Squad menu only, backed by the header extension). New/default-matching layouts show it far right; customized layouts untouched.

**Ordered implementation steps:**

1. RED: failing header tests (per-column non-sortable behavior + Add listing), failing route tests (default header; focus and `—` states; non-sortable click), failing store migration tests. Keep the suite proportionate; do not rebuild the discarded oversized suite.
2. GREEN: ID/constant → Squad-only metric module → header extension → panel cell → store v7 + migration → DTO mirror → mock fixtures, in the smallest slices that turn each proof green.
3. Refactor only while green; keep Search/Moneyball/Staff production paths behaviorally untouched (test files may change for regression proof; verify with diff).
4. Run targeted suites, then full frontend gate + smoke.

**Tests and proof:**

- Route suite: default column order/position; focus string renders; null renders `—` with the accessible name; header click leaves sort state unchanged; menu remove → column gone + query unchanged; menu re-add → column back.
- Header suite: per-column sortability cases above, proving existing sortable columns are unaffected.
- Store suite: migration cases above + cross-table rejection test.
- Search-side absence proof in `search.test.tsx`; Search/Moneyball/Staff production paths stay behaviorally untouched (test files may change for regression proof).
- Accessibility proof: the cell exposes an appropriate accessible name for the visible value or dash; no information is hover-only.
- Smoke: Squad overview passes with the new default; no new IPC stub paths beyond the narrowed Squad fixture shape.

**Patterns to verify:**

- `squad-overview-panel.tsx` Club DNA `ScoreBadge`/`—` cell pattern; `nationality-cell` accessible-name pattern; `player-table-header.tsx` menu + `metrics` prop override; Club DNA store-append precedent (`addColumns("squad", ["club_dna"])`).

**Constraints and non-goals:**

- Do not add the ID to `PLAYER_METRICS`/Moneyball catalogs, Search components, filter ASTs, `getPlayerMetric`, or any sort validation. Do not change Search defaults. Do not add a Details action, Modal, tooltip, or any evidence UI. Keep rows at fixed heights. Respect `shared → features → app` imports.

**Dependencies and sequencing:**

- Requires Commit 4 (string/null DTO shape consumed unchanged).

**Validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx` then `./scripts/dev test src/app/routes/search.test.tsx` then `./scripts/dev test src/stores/use-player-table-store.test.ts` then `./scripts/dev test` then `./scripts/dev check` then `./scripts/dev smoke`.

**Stop conditions:** Stop if the v6→v7 migration cannot distinguish default from customized layouts exactly, if any existing catalog already supplies `sortable: false` (escalate before changing shared-header Add semantics), or if Search exclusion cannot be held structurally (escalate before touching shared catalogs).

**Review mandate:** (1) the ID and predicate have one neutral owner in `src/utils/suggested-training.ts`, imported by both the store and Squad, with no shared→feature import and no duplicated literal; (2) header diff is the minimum per-column extension with focused tests green and existing sortable behavior unchanged; (3) Search/Moneyball/Staff production paths behaviorally untouched; (4) store migration exact-equality + customized-layout preservation; (5) cell renders only the focus string or `—` with the accessible name and no Modal, state, or details affordance; (6) no hover-only information; (7) fixed row heights kept; (8) `requestedFields` excludes the field while the DTO always carries it; (9) no sort path for the field in URL, backend, mock, or tests; (10) smoke proves the user-visible default column.

## Active work

**PR:** Suggested individual training in Club Squad

**Active work:** None — implementation complete

**Commit:** None — feature validation and review

### RED or removal proof

All five planned packets completed their recorded RED → GREEN or planning proof.

### Expected outcome

The complete focus-or-dash Suggested Training feature is ready for full validation, feature review, and documentation reconciliation.

### Explicit exclusions

New implementation scope, release work, unrelated refactors, and the unrelated `snapshot-date-edit.md` modification.

## Discoveries and replanning

- 2026-09-05 developer-approved simplification (value-or-dash): the cell shows only the most beneficial Training Focus name or `—`, with an accessible name for the visible value or dash and no Details action, Modal, gain, role labels, or attribute evidence. `CA >= PA` always renders `—` with no displayed focus. This is a material contract change: the Delivery fingerprint is reset to `Pending plan review`, remaining packets are replanned as Commit 4 (backend simplification + max-CA gate, Active) and Commit 5 (simple Squad-only default column, Pending), and completed Commits 1–3 are preserved with immutable refs. The original Commit 4 partial work was interrupted before validation/review and discarded from the worktree; the recovery stash is not delivery truth.
- 2026-09-05 feature review correction: the initial whole-feature review found that exact focus-to-attribute mappings and endpoint `ip_weight` behavior lacked direct regression proof, plus one duplicate store test. The bounded test-only correction strengthens the inventory test, proves OOP-only and IP-only winners through the real ranker, and removes the duplicate. Full validation and correction review are clear. Correction ref: `Pending record`.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Suggested individual training in Club Squad | Commit 1 — Record the approved feature plan | 13b32b264f84d866dce03198374ba9f967118785 | Added the accepted schema 2 ledger and TODO Active pointer. | `ledger_state.py`: runnable; `delivery_state.py`: runnable; `git diff --cached --check`: passed. | Not applicable | Clear | 0 | None. |
| PR 1 — Suggested individual training in Club Squad | Commit 2 — Rank training focuses for an assigned tactic lane | 797ad112653f9913c340208290a597f78ab8b62e | Added the shared unrounded role scorer and pure exact-inventory lane-based training-focus ranker with whole-inventory gating and evidence attributes. | `./scripts/dev check-rust`: 787 passed, 2 ignored; LSP and `git diff --cached --check`: passed. | Pass | Clear | 0 | None. |
| PR 1 — Suggested individual training in Club Squad | Commit 3 — Attach suggestions to the Squad read model | 076a77f1530774c54c9988da92a0a72e0c4d870c | Added read-only tactic and assignment resolution plus per-row derived suggestions and the typed Squad DTO while preserving bounded paging and existing sorts. | `./scripts/dev check-rust`: 793 passed, 2 ignored; LSP and `git diff --cached --check`: passed. | Pass | Clear | 0 | None. |
| PR 1 — Suggested individual training in Club Squad | Commit 4 — Simplify the outward suggestion to value-or-dash | ab73d3a69ea7ad411a1c4ce69a4f7d96582cfaf6 | Replaced the evidence-rich result with a focus string or null, added the `ca >= pa` gate, and removed obsolete evidence fields and helpers without changing ranking behavior. | `cargo test --lib planner::`: 126 passed; `./scripts/dev check-rust`: 794 passed, 2 ignored; `./scripts/dev check`: passed; LSP and `git diff --check`: passed. | Pass | Clear | 0 | None. |
| PR 1 — Suggested individual training in Club Squad | Commit 5 — Show Suggested Training in the Squad table | 02d716e4b83b053ad883f0c2b350570f722c2c69 | Added the Squad-only default non-sortable column, focus-or-dash cell, exact-default layout migration, neutral ID ownership, and structural Search exclusion without evidence UI or a sort path. | Focused Squad: 133 passed; Search: 73 passed; store: 33 passed; header: 3 passed; `./scripts/dev test`: 846 passed; `./scripts/dev check`: passed; `TMPDIR=/home/jonas/.cache/pi-tmp ./scripts/dev smoke`: 55 passed; LSP and `git diff --check`: passed. | Pass | Clear | 1 | Initial review found custom-width layouts were migrated as defaults and redundant Search store assertions; correction preserved custom widths, shrank the test, and passed correction review. |

## Final validation

- `./scripts/dev test` (full) green.
- `./scripts/dev check` (commit gate) green.
- `./scripts/dev smoke` green (user-visible table workflow with the simple value-or-dash column).
- Feature review clear with no blocking findings.
- Documentation reconciliation: TODO active entry, completed-record archive on close-out, no ADR, BACKLOG unchanged.

## Documentation impact

Complete during reconciliation (TODO Active entry in Commit 1; close-out archive move before final merge).
