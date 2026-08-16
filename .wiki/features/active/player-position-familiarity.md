# Complete Player Position Familiarity

## Status

Active

## Intent

Preserve the complete raw FM26 position-familiarity map for every scanned player instead of letting the bridge discard legitimate secondary values. The feature establishes one versioned data contract from memory extraction through snapshot persistence, then makes every consumer choose its own explicit relevance rule rather than treating a missing JSON key as a position threshold.

This work implements [Linear JAY-14](https://linear.app/jaycount/issue/JAY-14/store-complete-player-position-familiarity-data).

## User-visible behavior

- A new scan stores all 15 canonical familiarity slots in this order: `GK`, `SW`, `DL`, `DC`, `DR`, `DM`, `ML`, `MC`, `MR`, `AML`, `AMC`, `AMR`, `ST`, `WBL`, and `WBR`.
- Every successfully read FM value from 0 through 20 survives bridge extraction and snapshot ingestion exactly, including secondary positions below 15 and below the strongest-position band.
- An unread slot remains distinguishable from a successfully read zero in the dump and persisted JSON.
- Search and Squad position summaries list every positive recorded familiarity, strongest first. Position-presence filters match positive familiarity, not merely an existing JSON key.
- Dynamic `pos.<slot>` metrics continue to expose the exact raw value so a user can filter or sort by a chosen threshold.
- The player profile position chart includes all 15 canonical slots, including `SW`, and displays every positive familiarity. Zero, unread, and legacy-missing slots remain non-positive and do not become a best or playable position.
- The profile's initial position remains the strongest positive familiarity. Its current and potential position summaries continue to include only playable positions rated at least 15.
- Potential attribute and role-score projection continues to use the established natural-position rule rather than treating all 15 stored keys as natural positions.
- Planner and optimizer eligibility continues to require a familiarity of at least 15 for the relevant tactic lane. A newly preserved value below 15 cannot make a player eligible.
- Academy position labels show every positive recorded familiarity and exclude zero, unread, and legacy-missing slots.
- Existing sparse snapshots remain usable as sparse data. They are not backfilled; users must run Load Data with the schema-v7 bridge to obtain complete familiarity maps.

## Invariants

- The dump contains exactly the 15 canonical position keys for every newly scanned player.
- A successfully read byte in the inclusive range 0 through 20 is emitted as that exact integer.
- An unread byte is emitted as JSON `null`; a missing key is reserved for legacy sparse snapshots and is not produced by schema v7.
- An out-of-range byte is not accepted as a valid FM familiarity and follows the same non-value path as unread data.
- Snapshot ingestion preserves the validated map without recomputing, filtering, or normalizing its values.
- Consumer semantics are explicit and remain distinct:

| Meaning | Rule | Consumers |
| --- | --- | --- |
| Recorded position | Familiarity `> 0` | Search/Squad `Position`, exact position-presence filters, Academy labels, profile pitch labels, best-position selection |
| Playable position | Familiarity `>= 15` | Profile current/potential summaries, goalkeeper profile mode, Planner lane eligibility, optimizer candidate eligibility |
| Natural projection position | Familiarity `>= max(15, strongest - 2)`; if no positive value reaches 15, use the first tied strongest positive value in canonical layout order | Potential attribute projection and every current/potential role-score consumer |
| Raw familiarity | Exact integer `0..20`, or `null` when unread | `positions_json`, player/Academy DTOs, dynamic `pos.<slot>` metrics |

- A legacy missing key behaves like an unread value in consumers. It never qualifies merely because the snapshot predates schema v7.
- A full map cannot change results that previously received the equivalent sparse natural-position map. In particular, potential projection for `AMR=20`, `MR=17`, and `AMC=14` must match the legacy projection from `{AMR: 20}`.
- When all positive values are below 15, the strongest-position fallback breaks ties by canonical layout order, independent of JSON-map iteration order.
- The scoring projection model version does not change unless implementation evidence shows that identical legacy semantic inputs produce different projected attributes.
- Position familiarity remains snapshot-owned memory data. No SQLite migration or separate relational position table is introduced.
- The bridge remains the only owner of memory layout and read coverage; Rust remains the trust boundary for dump validation and persistence.
- No consumer derives meaning from key presence alone after this feature.

## Non-goals

- Inferring a player's tactical best role or best position from attributes, role scores, current ability, or potential ability.
- Changing the FM familiarity thresholds, role catalog, role-score formula, potential growth model, or Planner tactic model.
- Recomputing stored current role scores during ingest from position familiarity.
- Backfilling complete maps into existing snapshots or fabricating values for keys that older dumps omitted.
- Adding a schema migration, a position-familiarity history table, or user-configurable thresholds.
- Changing CSV-imported `best_position` or textual `positions` fields; those are separate enrichment inputs.
- Supporting non-canonical position keys or converting `SW` into another position.
- Adding profile roles for `SW` when the existing role catalog has none. The chart can select and display `SW` without inventing a role definition.
- Publishing, merging, releasing, or changing version/changelog metadata as part of planning.

## Current-state map

- Relevant components:
  - `bridge/Extraction/PlayerIdentityReader.cs` batch-reads the canonical position bytes but filters them through `max(15, strongest - 2)` before producing `PlayerIdentity.Positions`.
  - `bridge/Layouts/Fm263Layout.cs` owns the 15 canonical slot offsets and ordering.
  - `bridge/Models/DumpDocument.cs` and the dump writer serialize only integer-valued position entries.
  - `bridge/Protocol/BridgeProtocol.cs` and `src-tauri/src/features/memory_read/dump_validation.rs` currently agree on dump schema v6.
  - `src-tauri/src/features/snapshot/ingest.rs` already persists the received `positions` object as JSON without a position threshold.
  - `src-tauri/src/features/scoring/projection.rs` accepts position names that callers currently obtain from every JSON key.
  - Player detail, the potential cache, Planner depth, and optimizer call the projection seam.
  - `src-tauri/src/features/search/filter.rs` and `src-tauri/src/features/player_metrics/resolver.rs` currently use position key presence or every `json_each` entry.
  - `src/features/player-profile/utils/position-families.ts` owns best/playable position decisions and a 14-slot pitch layout that omits `SW`.
  - `src/features/player-profile/components/player-roles-panel.tsx` renders pitch familiarity and position-specific role fit.
  - `src-tauri/src/features/academy/service.rs`, `src-tauri/src/features/academy/commands.rs`, and the Academy TypeScript DTOs currently require integer-only maps.
  - `academy-class-workspace.tsx` and `academy-add-players-modal.tsx` currently render every map key.
- Data model:
  - Schema-v6 dumps use a sparse `positions` object whose values are integers.
  - SQLite stores that object in `players.positions_json` and does not impose a relational position schema.
  - Current player and Academy DTOs expose `Record/BTreeMap<string, integer>` and therefore cannot represent unread schema-v7 entries.
- Persistence and migrations:
  - Snapshot ingest writes the JSON object verbatim into the snapshot-owned player row.
  - No migration is required. Existing rows remain sparse; new schema-v7 rows carry complete nullable maps.
  - `snapshots.schema_version` records which dump contract produced the row.
- Existing behavioral assumptions:
  - Sparse keys currently happen to mean "natural enough for projection," so projection callers pass all keys.
  - Search/Squad `Position` happens to work because sparse keys are all meaningful; key presence becomes wrong when all keys exist.
  - Profile and Planner playable decisions already use an explicit 15 threshold and must retain it.
  - Academy and Search summary formatting assume that every map key should be displayed.
- Architectural seams:
  - C# owns byte coverage and raw extraction.
  - The JSON schema and Rust validator form a fail-closed protocol boundary.
  - Snapshot ingest owns atomic persistence but not position interpretation.
  - Shared scoring projection owns the natural-position selection used by all potential-score paths.
  - Search/player-metric resolution owns SQLite query semantics; React renders returned values.
  - Feature-local profile and Academy helpers own presentation semantics.
- Project validation commands:
  - `./scripts/dev format`
  - `./scripts/dev bridge-test`
  - `./scripts/dev test [target...]`
  - `./scripts/dev check`
  - `./scripts/dev smoke`
- Primary risks:
  - Enabling schema v7 before consumers are hardened would make every player match every position, display all slots as recorded, and project potential across unrelated position groups.
  - Collapsing unread bytes to zero would satisfy map completeness while losing the trust-boundary distinction required by JAY-14.
  - Fixing only the profile would leave Search, Academy, Planner potential scores, or optimizer behavior silently wrong.
- This plan touches the same profile surface as Player Profile Information Controls and Layout; implementation must start from synchronized `main`, which contains that feature's squash merge, rather than from its old feature branch.

## Feature architecture

The bridge emits facts, not relevance decisions. For each canonical offset it checks byte-level read coverage, accepts an in-range value exactly, and emits `null` when no trustworthy value is available. The versioned Rust validator rejects missing or extra canonical keys and rejects values outside `null | integer 0..20`. Snapshot ingestion then stores the validated object unchanged.

Consumers interpret that one raw map at their existing layer:

1. Projection receives position/value entries and selects natural positions inside the shared scoring seam. Callers can no longer accidentally turn completeness into scoring meaning by passing `.keys()`.
2. Search and Squad SQL use a positive-value predicate for human-readable positions and position presence. Dynamic per-position metrics retain raw numeric behavior.
3. Profile utilities distinguish strongest positive, playable, and non-positive/unread values. The pitch supplies the complete canonical layout, while role fit continues to use the existing catalog.
4. Academy converts its nullable DTO map to positive labels before display.
5. Planner lane and optimizer eligibility retain their existing explicit `>= 15` checks. Their potential scores use the corrected shared projection seam.

The PR remains one atomic review boundary because producer completeness and consumer meaning form one protocol compatibility contract. Its commits harden consumers before the schema switch so every intermediate commit remains testable and the final schema-v7 commit cannot expose key-presence behavior.

## Consumer compatibility matrix

| Surface | Current sparse-map dependency | Required complete-map behavior | Primary proof |
| --- | --- | --- | --- |
| Player detail potential scores | Passes every key to projection | Shared projection selects only natural values | Complete and equivalent sparse maps project identically |
| Potential score cache | Passes every key to projection | Uses the same shared natural selection | Cache computation matches direct player detail computation |
| Planner assigned-player potential | Passes every key to projection | Low and unread values do not alter projected attributes | Complete-map depth result matches sparse-map result |
| Optimizer potential mode | Passes every key to projection | Projection is natural-only; lane eligibility remains `>= 15` | `MR=17` can qualify, `AMC=14` cannot, unrelated zeros cannot |
| Search `Position` filter | Tests key existence/non-null | Tests numeric familiarity `> 0` | Exact position includes positive secondary and excludes zero/null |
| Search/Squad `Position` column | Lists every JSON key | Lists positive values strongest-first | `AMR 20, MR 17, AMC 14` renders `AMR, MR, AMC` only |
| Dynamic `pos.*` metrics | Reads a chosen raw JSON value | Keeps exact `0..20`; unread/missing remains null | Zero and null remain distinguishable in query results |
| Profile best position | Maximum over sparse map | Maximum positive value only | `AMR=20` remains initial selection despite complete zero/null slots |
| Profile pitch | Fixed 14-slot layout and sparse values | All 15 slots, every positive value, non-positive/unread unqualified | `SW` exists and 20/17/14 are visible without making 14 playable |
| Profile summaries/GK mode | Explicit `>= 15` | Unchanged explicit playable rule | 17 included, 14 excluded; GK 15 switches mode, GK 14 does not |
| Academy labels | Lists every key | Lists positive values strongest-first | Secondary 14 survives; zero/null omitted |
| Legacy snapshots | Missing implies absent | Missing treated like unread/non-positive | Existing v6 snapshot remains readable and does not gain positions |

## Uncertainty register

### Known

- Linear JAY-14 requires all 15 canonical FM familiarity values, exact persistence, explicit downstream thresholds, and a v6-to-v7 rescan boundary.
- The bridge already reads the full contiguous position block; the data loss occurs when `ReadNaturalPositions` filters the result.
- `BlockReadResult.CountReadableBytes` and the existing fake reader can prove the difference between unread coverage and a read zero.
- Schema v6 is a frozen bridge/Rust contract, so changing map shape and nullability requires schema v7.
- Snapshot ingest already preserves the positions JSON object and needs validation/integration proof, not a new persistence design.
- The four potential-projection callers currently pass position keys without ratings.
- Profile summaries, goalkeeper mode, Planner lane eligibility, and optimizer eligibility already encode the playable floor of 15.
- The role catalog has no dedicated `SW` role tags, although the projection groups `SW` with central defenders.
- The implementation branch `feature/player-position-familiarity` starts from fetched `origin/main` at `3e77e7864c0ed980630cbc0948f512c1dfd19f95`, which contains Player Profile Information Controls and Layout as PR #55.

### Assumptions

- FM26 familiarity bytes have the established valid range 0 through 20; values outside the range are not trustworthy domain values.
- Positive familiarity is the least surprising meaning of an unqualified user-facing `Position` field because it preserves legitimate secondary data without making zero-valued canonical slots appear as positions.
- The established bridge fallback to the strongest positive position when no value reaches 15 is intentional projection compatibility and should be retained.
- Existing sparse snapshot JSON remains accepted by read-side consumers after DTO nullability changes; only new dump ingestion requires the exact schema-v7 key set.
- Profile chart geometry can accommodate the added central `SW` row without redesign, subject to the 1280x800 smoke assertion.

### Decisions

- Represent schema-v7 positions as exactly 15 keys whose values are `integer 0..20 | null`.
- Use `null` for unread or invalid bytes and integer `0` for a successfully read zero. Never omit a schema-v7 key.
- Keep complete maps in the existing `positions_json` column; do not add a migration or derived position table.
- Move the natural-position rule into `project_attributes` (or a private helper owned by that module) and pass familiarity values, not already-selected keys, from all callers.
- Preserve the bridge's below-15 tie behavior: select only the first tied strongest value in canonical layout order. Keep the canonical comparison explicit in the projection module instead of relying on caller map order.
- Do not increment the projection model version when the same semantic sparse input produces the same output; schema-v7 snapshots create fresh cache rows.
- Define Search/Squad `Position`, exact position presence, Academy labels, pitch labels, and best-position selection as positive familiarity `> 0`.
- Keep `pos.<slot>` as the raw metric surface rather than applying the positive or playable threshold.
- Keep playable semantics at `>= 15` for profile summaries, goalkeeper mode, Planner, and optimizer.
- Display every positive profile familiarity and treat zero, null, and legacy missing as non-positive. The UI need not invent a role for `SW`.
- Deliver one PR with consumer-hardening commits before the protocol-enabling commit.
- Do not create an ADR: the feature versions an existing protocol and preserves existing layer ownership rather than introducing a durable architectural alternative.

### Unknowns

- Live FM proof depends on finding a representative player with a strong position plus at least one legitimate familiarity below the old bridge threshold. If no convenient player is known, a bridge fixture remains the deterministic acceptance proof and live inspection can use any multi-position player.
- The final profile-pitch spacing with the added `SW` slot requires browser verification; no product decision is open unless it fails the existing 1280x800 containment contract.

### Risks

- A validator that checks only object shape could allow sparse schema-v7 maps and preserve the original defect under a new version.
- Treating `null` as zero in DTO parsing would erase whether FM memory was actually readable.
- Repeating the natural threshold in each caller would invite future drift between player detail, cache, Planner, and optimizer.
- A broad shared frontend abstraction could couple unrelated displays; helpers should remain feature-local unless present duplication proves otherwise.
- Fixture replacement can create false confidence if every test sees only sparse v6 data. At least one complete map with positive, zero, and null entries must cross each affected seam.
- Profile-controls code can be overwritten or tests can regress if implementation starts from the currently diverged feature branch instead of its merged result.
- The Repowise index inspected during planning was behind the working-tree HEAD; all advisory risk findings were verified against current source, and implementation must continue to treat the index as stale until refreshed.

## Walking skeleton

The thinnest safe path is deliberately consumer-first:

1. Feed `AMR=20`, `MR=17`, `AMC=14`, zero-valued unrelated slots, and one unread slot into the shared projection seam. Prove that it selects the same natural group and produces the same projected attributes as the legacy sparse `{AMR: 20}` map.
2. Put the same complete map through Search/Squad, profile, Academy, Planner, and optimizer focused tests so every former key-presence assumption has an explicit predicate before completeness is enabled.
3. Have the bridge extract that shape, have the schema-v7 validator accept it, and prove snapshot ingest persists the exact positive, zero, and null values.

There is no safe schema-first skeleton: turning on all 15 keys before steps 1 and 2 would make existing consumers observably wrong even if extraction and ingestion passed.

## Delivery plan

### PR 1 — Preserve complete position familiarity

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/player-position-familiarity`

**Base branch:** `main`

**Base ref:** `3e77e7864c0ed980630cbc0948f512c1dfd19f95`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(memory-read): preserve complete position familiarity`

**Purpose:** Deliver one versioned bridge-to-SQLite completeness contract together with all read-side compatibility changes required to keep current best-position, profile, projection, Search, Academy, Planner, and optimizer behavior correct.

**Depends on:** Synchronized `main`, including completed Player Profile Information Controls and Layout; completed FM26 memory read, schema-v6 parity, snapshot ingest, potential role scores, player profiles, Search, Academy, Planner, and optimizer foundations.

#### Commit 1 — Derive natural projection positions from familiarity

**Status:** Completed

**Provisional commit:** `refactor(scoring): derive natural positions from familiarity`

**Work:** Make the shared potential-projection seam accept position familiarity values and own the natural-position rule, then migrate player detail, potential cache, Planner depth, and optimizer callers without changing legacy sparse-map results.

**Out of scope:**

- Bridge extraction, schema v7, fixture replacement, Search display/filter semantics, profile pitch layout, Academy labels, or Planner eligibility thresholds.
- Changing projection curves, attribute-group definitions, role scoring, or `PROJECTION_MODEL_VERSION`.

**Implementation packet:**

- Change the projection input from a list of preselected position keys to entries carrying a key and optional familiarity.
- Select positive values, find the strongest, and retain values at `max(15, strongest - 2)`. If no positive value reaches 15, retain only the first tied strongest value in canonical layout order to preserve the bridge's existing fallback. If there is no positive value, retain no position group and keep the existing all-attributes fallback.
- Keep the threshold selection private to the scoring projection module so all four callers share one rule.
- Update player-detail position parsing/DTOs as needed to preserve `null` rather than rejecting or coercing it. Existing integer-only sparse JSON must still parse.
- Update the potential cache, Planner depth, and optimizer adapters to pass values rather than `.keys()`. Do not duplicate threshold logic at those call sites.
- Retain existing explicit Planner/optimizer `>= 15` suitability checks; this commit changes potential projection input only.

**Files and responsibilities:**

- `src-tauri/src/features/scoring/projection.rs` — own natural-position selection, canonical fallback ordering, and compatibility tests.
- `src-tauri/src/features/player/query.rs` — parse nullable familiarity and pass entries to player-detail projection.
- `src-tauri/src/features/player/commands.rs` — serialize nullable player-detail positions without information loss.
- `src-tauri/src/features/player_metrics/potential_cache.rs` — pass raw familiarity entries into cached potential projection.
- `src-tauri/src/features/planner/depth.rs` — pass assigned-player familiarity into potential scoring.
- `src-tauri/src/features/planner/optimizer.rs` — pass candidate familiarity into potential scoring while preserving lane eligibility.
- `src-tauri/src/features/planner/depth_tests.rs` and `optimizer_tests.rs` — prove Planner/optimizer compatibility with complete maps.

**Behavior and data flow:**

- Player JSON is parsed into nullable familiarity entries.
- Each potential-score path calls the shared projection function with attributes, CA/PA/age inputs, and the unfiltered familiarity map.
- Projection selects natural position groups once, projects visible attributes as before, and the existing role-scoring call consumes the result.
- A complete map and its equivalent schema-v6 sparse natural map produce identical projected attributes and potential role scores.
- Zero, unread, missing, and below-threshold positive values cannot add a projection group.

**Ordered implementation steps:**

1. Add a RED unit test in `projection.rs` showing that a complete 20/17/14/0/null map currently adds unrelated position groups or cannot be expressed.
2. Add focused RED integration assertions for at least player detail and one Planner/optimizer path using the complete map.
3. Change the projection contract and natural selector minimally, then migrate every compiler-identified caller.
4. Keep the sparse compatibility tests and new complete-map tests GREEN; refactor only duplicate input adaptation.
5. Run formatting and the full commit gate in the recorded order.

**Tests and proof:**

- RED: projecting the complete map differs from `{AMR: 20}` because current callers pass all keys, or the current API cannot carry ratings/null.
- GREEN: complete and sparse equivalent maps produce identical projected attributes.
- Boundary: `MR=17` is excluded when the strongest is 20, `MR=18` is included, `AMC=14` is excluded, zero/null/missing are excluded, strongest-positive fallback works when all positives are below 15, and a reversed-input-order tie below 15 still selects the first canonical position.
- Integration: player detail, potential cache, Planner assigned-player potential, and optimizer potential mode all use the same result.
- Regression: lane suitability still accepts a required position at 15 and rejects it at 14.

**Patterns to verify:**

- Keep the existing position-group mapping and all-attributes fallback in `projection.rs`.
- Match fallback ties to `Fm263Layout.PositionEntries` canonical order rather than `BTreeMap`, insertion, or alphabetical order.
- Follow current JSON parsing error context in player query code; add nullability without a new deserialization layer.
- Keep `is_suitable_for_lane` as the Planner/optimizer playable-position owner.

**Constraints and non-goals:**

- No caller may prefilter by reimplementing the natural threshold.
- Do not treat canonical key presence as familiarity.
- Do not change outputs for currently valid sparse schema-v6 inputs.
- Do not bump a cache/model version without a demonstrated output change for the same semantic input.

**Dependencies and sequencing:**

- Start only from a synchronized base containing the completed profile-controls feature.
- This commit must land before the schema-v7 producer commit.
- Later commits may reuse the nullable map type but must not widen this commit into display work.

**Validation:**

1. `./scripts/dev format`
2. `./scripts/dev check`

Expected evidence: Rust unit/integration tests pass, all four projection call sites compile, existing sparse fixtures retain their results, and no frontend or bridge behavior changes.

**Stop conditions:**

- Stop and replan if identical sparse semantic inputs change projected attributes or scores.
- Stop and ask the developer if repository evidence contradicts the strongest-positive fallback below 15.
- Stop if a fifth production projection caller is found; add it to the impact map and tests before continuing.
- Stop if the synchronized base does not contain the completed four-summary profile behavior this feature must preserve.

**Review mandate:**

- Confirm every production `project_attributes` caller supplies familiarity values.
- Confirm natural threshold and fallback behavior exactly match the former bridge semantics.
- Confirm tied strongest values below 15 select one canonical-first position independent of map iteration order.
- Confirm null, zero, missing, and below-threshold values cannot add a position group.
- Confirm Planner/optimizer playable eligibility remains separate and unchanged.
- Confirm no unjustified projection model-version bump or cache invalidation was added.
- Confirm tests compare observable projected/scored output, not only the selected-key helper.

#### Commit 2 — Make Search and Squad use positive familiarity

**Status:** Active

**Provisional commit:** `refactor(search): filter recorded position familiarity`

**Work:** Replace key-presence assumptions in shared Search/Squad position fields with explicit positive-value SQL while preserving exact raw dynamic position metrics.

**Out of scope:**

- UI redesign, new filter operators, playable/natural threshold changes, bridge/schema changes, and profile/Academy behavior.

**Implementation packet:**

- Compile the unqualified `position is/is not` behavior against a numeric value greater than zero, not `json_extract(...) IS NOT NULL` or bare key existence.
- Build the shared `Position` summary from `json_each` entries whose value is an integer greater than zero, ordered by familiarity descending and key ascending for ties.
- Preserve `pos.<slot>` as a direct raw numeric metric. A read zero must remain queryable as zero, while unread/null/legacy missing remains null.
- Verify Squad receives the corrected position summary through the existing shared player-metric resolver rather than adding Squad-specific formatting.

**Files and responsibilities:**

- `src-tauri/src/features/search/filter.rs` — explicit recorded-position predicate and operator tests.
- `src-tauri/src/features/player_metrics/resolver.rs` — positive-only, strongest-first `Position` display SQL and raw `pos.*` preservation.
- `src-tauri/src/features/search/query.rs` — end-to-end Search filtering, display, sorting, and dynamic metric assertions.
- `src-tauri/src/features/planner/squad_tests.rs` — focused proof that Squad's shared `Position` column does not list canonical zero/null entries.

**Behavior and data flow:**

- Search filter input resolves a canonical slot and compiles a positive numeric predicate over `positions_json`.
- The resolver expands the JSON object, keeps only positive integers, orders them deterministically, and concatenates the labels for Search and Squad.
- Dynamic metrics bypass the display predicate and return the selected raw familiarity.
- Complete maps expose legitimate secondary values without causing every position filter to match every player.

**Ordered implementation steps:**

1. Add RED query tests using `AMR=20`, `MR=17`, `AMC=14`, one zero, and one null.
2. Prove the current `Position` field lists or matches zero/null canonical keys incorrectly.
3. Add the smallest explicit predicates in filter compilation and metric resolution.
4. Keep exact-position and negative-operator behavior GREEN and add the Squad shared-column proof.
5. Run focused frontend tests only if affected fixtures cross TypeScript, then the commit gate.

**Tests and proof:**

- `Position` renders `AMR, MR, AMC` for the representative complete map and excludes zero/null.
- `position is AMC` matches the 14-valued secondary position; a zero-valued slot and a null-valued slot do not match.
- `position is not` remains the logical inverse for positive familiarity, including legacy missing keys.
- Sorting remains strongest-first with a stable alphabetical tie break.
- `pos.AMC` returns 14, a chosen zero slot returns 0, and unread/missing returns null.
- Squad's position column uses the same result without local parsing.

**Patterns to verify:**

- Reuse current JSON1 SQL construction and bound-parameter conventions in `filter.rs`.
- Preserve the player-metric resolver as the shared Search/Squad field owner.
- Preserve canonical-key validation for dynamic `pos.*` metrics.

**Constraints and non-goals:**

- Do not use the playable floor for the general `Position` field; legitimate positive secondary familiarity is intentionally visible.
- Do not turn `pos.*` into a boolean or thresholded metric.
- Do not add a generalized SQL abstraction unless the two focused expressions cannot remain clear.

**Dependencies and sequencing:**

- Depends on Commit 1 only as PR ordering; it can use synthetic complete JSON before schema v7 exists.
- Must be complete before the schema-v7 fixture becomes the default.

**Validation:**

1. `./scripts/dev format`
2. `./scripts/dev check`

Expected evidence: Search and Squad Rust tests prove complete-map behavior, raw metrics retain zero/null distinctions, and the repository gate passes.

**Stop conditions:**

- Stop and replan if `Position` is documented or tested elsewhere as playable-only rather than recorded-positive.
- Stop if Squad bypasses the shared resolver in a second production path; add that path to the packet rather than patching it incidentally.
- Stop if SQLite JSON1 cannot distinguish integer zero from null with the current schema; demonstrate the failure before considering a data-model change.

**Review mandate:**

- Confirm no key-presence predicate remains for position relevance.
- Confirm positive secondary values below 15 stay visible and filterable.
- Confirm zero, null, and missing have intentional, tested behavior.
- Confirm dynamic raw metrics are not thresholded.
- Confirm Search and Squad share one resolver contract.
- Inspect SQL parameterization and operator negation for injection or three-valued-logic errors.

#### Commit 3 — Display complete familiarity without changing playability

**Status:** Pending

**Provisional commit:** `feat(profile): display complete position familiarity`

**Work:** Make player-profile and Academy DTOs nullable, add the missing `SW` pitch slot, display every positive familiarity, and preserve strongest/playable behavior in all profile summaries and Academy labels.

**Out of scope:**

- New `SW` role definitions, a pitch redesign, configurable thresholds, Search SQL, projection changes, and schema activation.

**Implementation packet:**

- Change player-detail and Academy TypeScript/Rust position maps to allow nullable values without coercion.
- Add `SW` to the profile's canonical pitch layout in the central defensive area.
- Make best-position selection ignore null, zero, and legacy missing entries and continue choosing the strongest positive value with the existing deterministic fallback.
- Keep goalkeeper mode and current/potential role summaries on the explicit playable floor of 15.
- Render all positive ratings, including secondary 17 and 14 values. Keep zero/null/missing visually non-positive (`—`) unless existing design-system evidence requires a more specific accessible label.
- Add one small Academy-local formatter/selector used by both candidate and class-member displays; return positive keys strongest-first and exclude zero/null/missing.
- Update IPC fixtures and mocks to exercise nullable complete maps instead of weakening their types with casts.

**Files and responsibilities:**

- `src/features/player-profile/types/player-detail.ts` — nullable familiarity DTO contract.
- `src/features/player-profile/utils/position-families.ts` — full 15-slot layout, strongest-positive selection, and explicit playable predicates.
- `src/features/player-profile/utils/position-families.test.ts` — unit boundaries for 20/17/14/0/null and `SW`.
- `src/features/player-profile/components/player-roles-panel.tsx` — complete pitch rendering and selected-position accessibility.
- `src/features/player-profile/components/player-overview-panel.tsx` and `player-attributes-panel.tsx` — verify existing summary/GK calls remain explicitly playable.
- `src/app/routes/players.$uid.test.tsx` — route-level best-position, pitch, four-summary, and tab-mode regression proof.
- `src-tauri/src/features/academy/service.rs` and `commands.rs` — preserve nullable map values in Academy reads and DTOs.
- `src/features/academy/types/academy.ts` — nullable candidate/member map contract.
- `src/features/academy/utils/academy-workspace.ts` or a focused sibling utility — positive, strongest-first Academy labels.
- `src/features/academy/components/academy-class-workspace.tsx` and `academy-add-players-modal.tsx` — consume the shared Academy formatting rule.
- `src/app/routes/academy.test.tsx`, `src/testing/player-ipc-mock.ts`, and `src/testing/academy-ipc-mock.ts` — complete-map fixtures and user-visible assertions.
- `e2e/smoke.spec.ts` — profile pitch containment and `SW`/secondary familiarity smoke coverage at the existing viewport.
- `.wiki/DESIGN.md` — record the implemented 15-slot pitch and recorded-versus-playable presentation contract when the behavior becomes true.

**Behavior and data flow:**

- Rust deserializes stored sparse or complete JSON to a nullable map and serializes it unchanged to the player and Academy commands.
- Profile utilities select the strongest positive rating for initial focus and use `>= 15` only where the UI says playable/current/potential.
- The pitch always presents the canonical slots; positive ratings receive their number/tier, while non-positive or unread entries cannot become selected as the best position.
- Academy derives a deterministic positive-label list from the same nullable DTO values.
- A complete map therefore adds legitimate visible secondary positions without changing the profile's best selection, goalkeeper mode, playable summaries, or role-score phase controls.

**Ordered implementation steps:**

1. Add RED utility and route tests with the representative complete map, including `SW`, zero, and null.
2. Add RED Academy tests showing current `Object.keys` rendering leaks every canonical slot.
3. Widen DTOs and parsers to preserve null, then implement the smallest profile and Academy selection changes.
4. Add `SW` to the pitch and verify selection, empty-role behavior, keyboard semantics, and layout containment.
5. Update the current design owner, run focused frontend tests, smoke, and the commit gate.

**Tests and proof:**

- Profile initially selects `AMR` from 20/17/14 even when every canonical key exists.
- Pitch exposes `AMR 20`, `MR 17`, and `AMC 14`, contains `SW`, and does not label zero/null as positive familiarity.
- Selecting a positive `SW` works and truthfully shows no matching roles when the catalog supplies none.
- Current and potential position summaries include AMR/MR but exclude AMC 14; all four profile score summaries from the profile-controls feature remain present.
- `GK=15` activates goalkeeper behavior and `GK=14` does not.
- Academy renders `AMR, MR, AMC`, not all 15 canonical keys.
- Keyboard/focus behavior and the 1280x800 profile shell remain usable after adding `SW`.

**Patterns to verify:**

- Preserve the current profile pitch button and role-region semantics in `player-roles-panel.tsx`.
- Reuse the existing deterministic position ordering/tie behavior in profile utilities where applicable.
- Keep Academy formatting in its feature utility instead of coupling it to profile layout.
- Follow the completed profile-controls design for four current/potential IP/OOP summaries and concealment behavior.

**Constraints and non-goals:**

- Accessibility is a safety carve-out: every added/changed pitch control must keep an accurate accessible name, focus state, and keyboard behavior.
- Do not infer roles for `SW` or alias its displayed value to `DC`.
- Do not show zero/null canonical keys as recorded positions in Academy.
- Do not collapse null to zero in Rust or TypeScript.
- Do not refactor the profile layout beyond what the fifteenth slot requires.

**Dependencies and sequencing:**

- Depends on Commit 1's nullable player-detail contract and the merged profile-controls base.
- Must remain green against sparse fixtures as well as synthetic complete maps.
- Must be complete before the schema-v7 golden fixture becomes the default.

**Validation:**

1. `./scripts/dev format`
2. `./scripts/dev test src/features/player-profile/utils/position-families.test.ts 'src/app/routes/players.$uid.test.tsx' src/app/routes/academy.test.tsx`
3. `./scripts/dev check`
4. `./scripts/dev smoke`

Expected evidence: focused Vitest tests pass, the full gate passes, and Playwright proves the complete pitch remains contained and operable.

**Stop conditions:**

- Stop and ask the developer if adding `SW` requires a product choice between pitch layouts rather than a contained row adjustment.
- Stop and replan if nullable map values propagate beyond player/profile/Academy DTOs into an undocumented public contract.
- Stop if profile-controls behavior is absent from the base or its tests conflict with this packet.
- Stop if the role catalog contains a latent `SW` contract that current source inspection missed; update the matrix before implementing presentation behavior.

**Review mandate:**

- Confirm all 15 canonical slots, especially `SW`, are present once and laid out intentionally.
- Confirm strongest, recorded, and playable semantics are not conflated.
- Confirm null is preserved across Rust IPC and TypeScript.
- Confirm the four current/potential IP/OOP profile summaries and concealment behavior remain intact.
- Confirm Academy no longer renders all canonical keys.
- Confirm accessibility and 1280x800 containment evidence is meaningful.
- Confirm no unsupported `SW` role or adjacent profile redesign was introduced.

#### Commit 4 — Activate and persist dump schema v7

**Status:** Pending

**Provisional commit:** `feat(memory-read): emit complete position familiarity`

**Work:** Make the bridge emit the exact complete nullable 15-slot map, validate it as schema v7, preserve it through snapshot ingest, replace the golden dump, and document the new protocol/rescan contract.

**Out of scope:**

- Backfilling old snapshots, database migration, configurable thresholds, consumer redesign, release metadata, or bridge memory-layout changes.

**Implementation packet:**

- Replace `ReadNaturalPositions` filtering with coverage-aware raw extraction over the existing 15 canonical offsets.
- Use block-read coverage per byte. Emit an in-range read byte exactly, emit null for unread coverage, and do not silently convert invalid bytes into a successful zero.
- Change C# position map models to nullable integer values and keep all canonical keys in layout order.
- Increment both bridge and Rust dump schema constants from 6 to 7.
- Validate that every player `positions` object has exactly the canonical key set, no extra keys, and only `null` or integer values from 0 through 20.
- Create `golden_dump_v7.json` with `AMR=20`, `MR=17`, `AMC=14`, at least one read zero, and at least one unread null. Use it for current-schema tests across ingest/readers.
- Retain the schema-v6 golden data as a stale-version fixture and prove it is rejected for new ingestion with the update-plugin/rescan recovery message.
- Add a snapshot-ingest assertion against the stored `positions_json`, including the secondary 17/14, zero, and null values.
- Update bridge schema/reference documentation and current architecture. State plainly that existing snapshots remain sparse and require a new scan for complete values.

**Files and responsibilities:**

- `bridge/Extraction/PlayerIdentityReader.cs` — complete coverage-aware extraction and removal of relevance filtering.
- `bridge/Layouts/Fm263Layout.cs` — canonical source reused for all 15 emitted slots; offsets do not change.
- `bridge/Models/DumpDocument.cs` — nullable position values in the serialized dump model.
- `bridge/Protocol/BridgeProtocol.cs` — schema-v7 constant.
- `bridge/Output/DumpWriter.cs` — keep the compact writer's schema-version comment aligned with v7.
- `bridge/Tests/IdentityExtractionTests.cs` — exact 20/17/14/0/null map proof and legacy filter regression.
- `bridge/Tests/ExtractionBatchingTests.cs` — complete-map batching/coverage proof without per-position reads.
- `bridge/Tests/Fakes/FakeMemoryReader.cs` — existing unread-range support; change only if a focused coverage assertion requires it.
- `src-tauri/src/features/memory_read/dump_validation.rs` — exact schema-v7 position validation and stale-v6 rejection.
- `src-tauri/src/features/memory_read/fixtures/golden_dump_v7.json` — authoritative current-protocol fixture.
- `src-tauri/src/features/memory_read/fixtures/golden_dump_v6.json` — retained or relocated stale-version fixture.
- `src-tauri/src/features/snapshot/ingest.rs` and affected fixture consumers — exact persistence and current-fixture updates.
- `bridge/DUMP_SCHEMA.md` — schema-v7 shape, canonical keys, value meanings, and v6 incompatibility.
- `bridge/README.md` — remove the natural-position filtering claim and document complete raw extraction.
- `.wiki/ARCHITECTURE.md` — current bridge/validator/persistence contract and downstream interpretation boundary.

**Behavior and data flow:**

- The bridge batch-reads the position region and consults coverage for each canonical byte.
- It serializes a fixed-key object containing exact integers or nulls into a schema-v7 dump.
- Rust rejects the entire dump before mutation if any player's key set or value contract is invalid.
- On success, transactional snapshot ingest stores the object unchanged in `players.positions_json` and normal current-snapshot selection proceeds as before.
- The consumer protections from Commits 1 through 3 interpret the complete map without key-presence regressions.
- Schema-v6 dump files cannot be newly ingested, while already persisted schema-v6 snapshots remain readable as legacy sparse maps.

**Ordered implementation steps:**

1. Add RED bridge tests for a 20/17/14 player where the current extractor drops 17/14, plus read-zero and unread-byte assertions.
2. Add RED Rust validator tests for missing key, extra key, null, zero, negative, over-20, non-integer, and stale-v6 cases.
3. Make the smallest C# model/extractor and synchronized schema-constant changes that turn extraction/validation GREEN.
4. Add the v7 golden fixture and exact snapshot-persistence proof; update every current-fixture reference deliberately.
5. Update intrinsic bridge docs and current architecture, then run bridge, frontend/Rust, full gate, and smoke validation.

**Tests and proof:**

- Bridge emits exactly 15 keys and preserves AMR 20, MR 17, AMC 14, a read zero, and an unread null.
- Bridge batching remains one bounded block path and does not add 15 independent reads per player.
- Validator accepts the complete representative map and rejects missing/extra keys, values below 0 or above 20, floats/strings, and stale v6.
- Failed validation remains pre-mutation and leaves existing snapshots untouched.
- Ingested `positions_json` equals the validated fixture map exactly.
- All readers using the v7 golden fixture retain their existing functional outputs under the explicit rules established earlier.
- Existing sparse database fixtures still deserialize without invented values.

**Patterns to verify:**

- Reuse `TryReadBlockWithCoverage` and `CountReadableBytes` rather than adding position-specific memory reads.
- Follow existing schema-version mismatch errors and golden-fixture rollover patterns in `dump_validation.rs`.
- Preserve the atomic snapshot-ingest boundary and required-value context in `ingest.rs`.
- Keep protocol facts in `bridge/DUMP_SCHEMA.md` and implemented system facts in `.wiki/ARCHITECTURE.md`.

**Constraints and non-goals:**

- Bridge and Rust schema versions must change together in one commit.
- No schema-v7 success path may omit a canonical key.
- No read zero may be serialized as null, and no unread byte may be serialized as zero.
- Do not rewrite existing SQLite rows or synthesize legacy values.
- Do not change the memory offsets or scanning architecture.
- Keep validation fail-closed before any persistent mutation.

**Dependencies and sequencing:**

- Depends on Commits 1 through 3. Do not switch the current golden fixture or schema constant until all consumers pass complete-map tests.
- Requires the .NET 6 SDK for the bridge suite and the normal repository toolchain for `check`.
- Live FM validation occurs only after deterministic suites pass and a schema-v7 plugin is installed through the established bridge-install workflow.

**Validation:**

1. `./scripts/dev format`
2. `./scripts/dev bridge-test`
3. `./scripts/dev test`
4. `./scripts/dev check`
5. `./scripts/dev smoke`

Expected evidence: bridge tests prove coverage/value fidelity, all schema-v7 fixture consumers pass, the full commit gate passes, and smoke confirms the principal user surfaces remain operable.

**Stop conditions:**

- Stop if byte-level coverage cannot reliably distinguish unread from read zero with the existing block API; this is a trust-boundary unknown that requires replanning, not coercion.
- Stop if any canonical offset/order differs between layout, schema docs, and validator; reconcile the source of truth before implementation.
- Stop if the v7 fixture exposes another production consumer that derives relevance from key presence; add a focused consumer-hardening commit or amend an uncommitted packet before enabling v7.
- Stop if validation cannot remain pre-mutation or if a migration appears necessary.
- Stop before live install if deterministic bridge and repository gates are not green.

**Review mandate:**

- Confirm exact 15-key completeness and synchronized schema versioning.
- Confirm read coverage, zero, null, invalid, and range semantics at the bridge/Rust boundary.
- Confirm snapshot ingest stores the exact object and remains transactional.
- Confirm stale-v6 dumps are rejected truthfully while existing persisted sparse snapshots remain readable.
- Confirm no threshold remains in bridge extraction.
- Confirm the golden fixture exercises positive secondary, zero, and null values across affected readers.
- Confirm bridge/schema/architecture documentation matches executable behavior.

## Active work

**PR:** PR 1 — Preserve complete position familiarity

**Commit:** Commit 2 — Make Search and Squad use positive familiarity

### RED proof

Add Search query fixtures with `AMR=20`, `MR=17`, `AMC=14`, one zero, and one null. The current resolver lists every canonical key and the current position-presence SQL treats key presence as relevance. The failing observable assertions are positive-only labels and exact filters that include `AMC=14` but exclude zero and null.

Add a dynamic metric assertion for `pos.AMC=14`, a read zero, and an unread null. Keep the proof at the shared Search/Squad resolver seam so Squad does not gain a second formatting path.

### Expected outcome

Search and Squad use positive recorded familiarity for position labels and exact presence filters, while dynamic `pos.*` metrics retain exact raw values. Complete maps no longer make zero or unread canonical slots appear as positions.

### Explicit exclusions

- No UI redesign, new filter operators, playable/natural threshold changes, bridge/schema, profile, Academy, projection, documentation, Git publication, version, or changelog changes.
- Do not threshold or booleanize dynamic `pos.*` metrics.

## Discoveries and replanning

- Planning inspection found that the bridge's discarded secondary values are only half of JAY-14: a complete map would also change key-driven projection, Search/Squad, Academy, and profile behavior. The delivery plan therefore hardens every identified consumer before enabling schema v7.
- Planning inspection found that the profile pitch omits canonical `SW`; Commit 3 adds the slot without inventing a role catalog entry.
- Planning inspection found no persistence migration requirement because snapshot ingest already stores raw JSON. The schema work is limited to versioned validation, fixtures, exact persistence proof, and documentation.
- Planning inspection found the Repowise index behind the current working-tree HEAD. Its hotspot signals were used only as advisory routing and were verified against current files.
- Before the planning checkpoint, `feature/player-position-familiarity` was created from fetched `origin/main` at `3e77e7864c0ed980630cbc0948f512c1dfd19f95`; the old profile-controls branch was not reused.
- No implementation deviations have been recorded.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Derive natural projection positions from familiarity | Pending record | Nullable familiarity feeds the shared natural-position projection rule; player detail, potential cache, Planner depth, and optimizer callers no longer pass key presence | Clean; no CRITICAL/HIGH/MEDIUM/NITPICK findings | None |

## Final validation

Run in this order after every planned commit is completed:

1. `./scripts/dev format`
2. `./scripts/dev bridge-test`
3. `./scripts/dev test`
4. `./scripts/dev check`
5. `./scripts/dev smoke`
6. `git diff --check 3e77e7864c0ed980630cbc0948f512c1dfd19f95...HEAD`

Required deterministic evidence:

- The bridge suite proves all 15 canonical keys, exact 20/17/14 preservation, read zero, unread null, invalid range handling, and unchanged batched extraction.
- Rust validation proves exact keys and `null | integer 0..20`, rejects stale v6 and malformed v7 before mutation, and preserves the complete object through ingest.
- Projection tests prove complete and equivalent sparse maps produce identical projected attributes/scores across player detail, cache, Planner, and optimizer.
- Search and Squad tests prove positive-only summaries/filtering and raw `pos.*` zero/null behavior.
- Profile tests prove strongest-positive selection, all 15 pitch slots including `SW`, every positive rating, playable/GK threshold boundaries, four profile summaries, and concealment compatibility.
- Academy tests prove positive-only, strongest-first labels.
- Existing sparse snapshot fixtures remain readable without backfill or fabricated values.
- Smoke covers the profile pitch and principal Search/Planner navigation at 1280x800 without overflow or keyboard/focus regression.

Required live/manual evidence before feature-complete review:

1. Build and install the schema-v7 bridge through the established `./scripts/dev bridge-install` path, restart FM as required, and run Load Data against FM26.3.2.
2. Inspect one representative multi-position player in the sanitized dump/database path and confirm all 15 keys plus exact positive secondary values. If available, compare a 20/17/14-shaped player; otherwise record the actual values used.
3. Confirm Search/Squad lists all positive recorded positions, exact filters include a below-15 positive and exclude zero/unread, and raw dynamic metrics retain exact values.
4. Confirm the profile chooses the strongest position, shows every positive and `SW`, keeps only `>= 15` positions in current/potential summaries, and retains all four current/potential IP/OOP values.
5. Confirm Planner permits a familiarity of 15 or greater, rejects 14 for lane eligibility, and potential optimization does not broaden from complete canonical keys.
6. Confirm Academy labels show positive positions only.
7. Open an existing schema-v6 snapshot and confirm it remains readable as sparse legacy data; state that a new scan is required for completeness.

The feature-complete review must inspect the complete PR diff with special attention to trust-boundary validation, all projection callers, key-presence searches, profile accessibility/layout, legacy snapshots, and documentation truthfulness.

## Documentation impact

Complete during implementation and reconciliation:

- `bridge/DUMP_SCHEMA.md` — bump to schema v7 and define the exact canonical keys, integer/null meanings, invalid values, and v6 rescan boundary.
- `bridge/README.md` — replace the natural-position filtering description with complete raw familiarity extraction.
- `.wiki/ARCHITECTURE.md` — record the implemented bridge-to-SQLite nullable map contract and explicit consumer interpretation boundary.
- `.wiki/DESIGN.md` — record the 15-slot profile chart including `SW`, positive familiarity display, and separate playable threshold.
- `.wiki/TODO.md` — keep this ledger under Active while the feature is in progress, then move the reconciled record to Completed.
- This ledger — record commit refs, validation/review evidence, deviations, PR publication metadata, and final release intent; condense and archive it under `.wiki/features/completed/` at feature completion.
- No ADR is planned because existing architecture and ownership remain intact.
