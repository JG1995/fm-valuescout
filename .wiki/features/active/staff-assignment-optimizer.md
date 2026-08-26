# Staff Assignment Optimizer

## Status

Validation

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 1a28b2c619ce09a764219faebf26fde23587752922d402e54ee74739e9a8fadb

## Intent

Help the user assemble a club staff structure from the active save's current Staff Shortlist. The feature persists required slot counts, recommends one shortlisted person for each slot when an eligible score exists, and leaves explicit vacancies without changing Football Manager data.

## User-visible behavior

- The existing My Club **Staff Shortlist** workspace adds staff-assignment controls. It does not add a route or a top-level workspace.
- The user configures required slot counts from 0 through 50 before optimization. Counts are save-scoped and survive restart and snapshot replacement.
- Team-scoped targets use the save's enabled Planner teams and current display names. Result rows show those current configured names, while allocation uses Senior, Reserves, and Youth in that canonical order. Club result rows show **Club**. Manager targets exist only for enabled Reserves and Youth teams.
- A separate **Club** bucket owns Head of Youth Development, Director of Football, Technical Director, Loan Manager, Chief Scout, and Scout targets.
- Team buckets own Assistant Manager, Coaches, Set Piece Coach, Head Performance Analyst, Performance Analyst, Head Physio, Physio, Head of Sports Science, and Sports Scientist. Reserves and Youth also own Manager.
- **Optimize assignments** recomputes from the effective current snapshot, the active save's current shortlist join, the configured managed club, enabled Planner teams, saved targets, and persisted staff role scores.
- While a save, snapshot, managed-club, Planner-team, or shortlist context change is pending, **Optimize assignments** is disabled and any visible recommendation is hidden immediately.
- Each filled row shows the person, current-staff or recruitment classification, score, Preferred Job, and the constraints that made the assignment eligible. A Coach row also shows the coaching discipline that supplied its highest score.
- Each unfilled row remains visible as a vacancy. Its evidence reports how many matching shortlisted candidates lacked the required score; missing scores are unavailable and never become zero.
- Shortlisted people with an unrecognized Preferred Job remain visible in the existing Shortlist table but cannot fill an optimizer target.
- Results never modify Football Manager data and are not persisted.

## Invariants

- The candidate pool contains only staff in `staff_shortlist_entries` for the active save who join by UID to staff in that save's effective current snapshot.
- My Staff rows absent from the shortlist never enter the pool. The Shortlist's Preferred Job and Only unemployed presentation filters never narrow optimization.
- A shortlist person whose authoritative current-snapshot `staff.club` exactly equals the configured managed-club name is current staff. Every other joined shortlist person, including a null or different club, is a recruitment candidate.
- Employment, Club Job, qualifications, wage, contract, CA, PA, age, and other contract metadata never affect suitability or allocation.
- One staff UID fills at most one slot. Version 1 has no dual responsibilities.
- Preferred Job matching is whole-value, case-insensitive, and limited to the closed mapping in **Feature architecture**. It does not infer cross-job substitutions.
- A required persisted score must be present. A null or missing score excludes the candidate and contributes unavailable evidence; it is not an eligible low score.
- The optimizer ranks eligible candidates within each canonical Preferred Job group by score descending, then numeric UID ascending. It allocates that ranked group through its allowed scopes in Senior, Reserves, Youth, then Club order, and through slot numbers ascending.
- Rust owns target validation, current-context resolution, shortlist eligibility, job mapping, score selection, classification, ordering, allocation, vacancies, and result evidence. React renders bounded DTOs and does not recreate these rules.
- Recommendations are read-only advice. No command writes FM memory, staff rows, shortlist rows, scores, contracts, or recommendation results.
- Target replacement is transactional. The primary key is `(save_id, scope, job_id)`. A save request contains exactly one entry, including zero, for every pair currently allowed by Club and the enabled Planner teams. Missing, extra, duplicate, disabled-team, invalid-pair, and counts outside 0 through 50 reject the complete write before mutation. Only after full validation does Rust delete the save's prior rows and insert positive counts.
- Removing a Planner team with nonzero staffing targets requires confirmation in the existing **Manage teams** flow. The impact names the team, assignment count, and each affected canonical target job with its slot count; when assignments and targets both exist, one confirmation names all of them. A confirmed save deletes that team's targets, strings, assignments, and team row atomically. Re-adding the team starts its staff targets at zero. Club targets and other enabled-team targets remain unchanged. No target deletion is silent, and no disabled-team target remains hidden.
- Save and snapshot freshness authority is each row's immutable context token. Numeric IDs remain data and join keys only. While save, snapshot, managed-club, Planner-team, or shortlist context work is pending, Optimize is disabled and any visible recommendation is suppressed immediately. Each optimize request is also bound to the context key and request generation at its start. Every target or context reset invalidates that generation, so a late completion cannot restore an obsolete result even when save and snapshot tokens remain unchanged. After a successful context change, immutable-token and context-key replacement provides the final context identity, including after SQLite reuses a deleted snapshot ID.

## Non-goals

- Add staff who are not in the save-owned current-snapshot Staff Shortlist join.
- Respect the Shortlist's Preferred Job or Only unemployed presentation filters during optimization.
- Use employment or contract metadata in suitability scoring.
- Infer that one Preferred Job can cover another job, except for the explicit score aliases below.
- Support dual roles, part-time allocation, shared duties, cross-job fallback, manual assignment overrides, locked recommendations, or a general-purpose constraint solver.
- Persist recommendation results or recommendation history.
- Modify Football Manager staff, contracts, jobs, or shortlist data.
- Add a route, top-level navigation item, global Zustand store, frontend SQL, raw Tauri invoke, or frontend business-rule computation.
- Change existing Staff Search, Staff Profile, My Staff, Shortlist import, or Planner assignment behavior beyond required query invalidation and the approved Manage teams removal-impact, confirmation, and atomic cleanup behavior.

## Current-state map

- Relevant components: `src/app/routes/my-club.tsx` composes the mounted Staff Shortlist workspace and already owns save, snapshot, managed-club, Planner depth, shortlist import, and cross-feature invalidation context. `StaffSearchResultsPanel` renders the virtualized Shortlist table. `StaffShortlistImportModal` owns replacement imports. `PlannerDepthMatrix` and `PlannerTeamManagement` expose enabled Planner teams and display names.
- Data model: migration v27 owns save-scoped `staff_shortlist_entries`; current `staff` rows are snapshot-scoped; `staff_role_scores` permits nullable 0–100 scores by snapshot, UID, and role ID, while current ingest writes only calculable role rows and omits uncalculable roles; `managed_club_settings` stores one exact club name per save; `planner_teams` stores enabled `senior`, `reserves`, and `youth` categories and display names.
- Staff scoring: `src-tauri/src/features/staff/scoring.rs` defines 21 persisted role scores. This feature reuses `manager`, `assistant_manager`, the six outfield coaching scores, `set_piece_coach`, `head_performance_analyst`, `performance_analyst`, `head_of_youth_development`, `director_of_football`, `technical_director`, `loan_manager`, `scout`, `physio`, and `sports_scientist` without recalculation.
- Shortlist query: `src-tauri/src/features/staff/query.rs::list_staff_with_shortlist` selects the active save's current snapshot, joins shortlist rows by save and UID, binds filters, sorts null last, and distinguishes no-current-snapshot from no-shortlist. The optimizer needs the same ownership but must ignore presentation filters and paging.
- Persistence and migrations: `src-tauri/src/db/migrations.rs` applies monotonic `PRAGMA user_version` migrations. The current target is v34. Migration tests cover fresh databases, populated upgrades, constraints, cascades, and registry order.
- Planner-team configuration: `src-tauri/src/features/planner/teams.rs::save_team_settings` transactionally replaces enabled teams and removes a disabled team's strings and assignments after confirmation when assignments exist. `src/features/planner/components/planner-team-management.tsx::PlannerTeamManagement` derives the current assignment impact for its confirmation. Commit 2 extends this exact service, command DTO, API, component, IPC mock/stub, and route-test seam so nonzero staffing targets also require the same confirmation and join the same atomic cleanup.
- IPC and command registration: Staff commands live in `src-tauri/src/features/staff/commands.rs`, use the shared `Db` mutex, resolve the active save in Rust, and are registered in `src-tauri/src/lib.rs`. Frontend Staff fetchers use `src/lib/tauri-client.ts` through `src/features/staff/api/` modules. The current-snapshot contract now carries the immutable token from completed Commits 4 and 5. The completed Rust optimizer response still exposes raw scope IDs on allocator slots and does not carry the current Planner display name that result presentation requires.
- Route and Query patterns: `my-club.tsx` loads current save, snapshot, managed club, tactic, and Planner depth with TanStack Query. Completed Commit 5 added both immutable tokens to the Staff assignment context key; active Commit 6 adds the remaining managed-club, Planner-team, and shortlist-revision inputs plus pending-context suppression. TanStack mutation reset clears observable mutation state but does not cancel an in-flight callback, so response acceptance also needs an explicit request generation. The Shortlist workspace remains mounted inside an accessible hidden tab panel.
- UI patterns: use the shared `Button`, `Modal`, `Panel`, `EmptyState`, and `ScoreBadge`; use semantic labelled tables, visible pending/error/success text, focus restoration, and the existing dense desktop layout.
- Tests: Rust staff query tests use migrated temporary SQLite databases; Planner team tests cover transactional removal; `src/testing/staff-ipc-mock.ts` is the shared Vitest IPC seam; `src/app/routes/my-club-squad.test.tsx` covers mounted workspaces and context behavior; `e2e/smoke.spec.ts` plus `e2e/tauri-ipc-stub.ts` cover browser-visible Staff Shortlist and Planner management flows.
- Project validation commands: `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev check-rust` is the focused Rust gate. Mutation testing is unsupported.
- Delivery rules: `main` is the default base. GitHub permits squash only. Branch protection requires the strict GitHub Actions status `check`; force pushes and branch deletion through protection are disabled. Human PRs use `.github/pull_request_template.md`.
- Primary risks: assigning a person twice; treating null as zero; selecting My Staff outside the shortlist; accepting a near-match Preferred Job; trusting a reusable snapshot ID instead of its immutable context token; leaving targets for a disabled Planner team; deleting nonzero targets without a confirmation that names their impact; moving allocation into React; or growing a direct partitioning problem into a general optimizer.

## Feature architecture

Migration v35 adds `staff_assignment_targets(save_id, scope, job_id, slot_count)` with primary key `(save_id, scope, job_id)`. `scope` is one of `senior`, `reserves`, `youth`, or `club`; `job_id` is one closed canonical target; stored counts are positive integers from 1 through 50. A complete target-save request contains exactly one explicit 0-through-50 count for every pair currently allowed by Club and the enabled Planner teams. Rust computes that allowed set, rejects a missing, extra, duplicate, disabled-team, invalid, or out-of-range entry before any delete, then deletes the active save's old rows and inserts only positive rows in one transaction. Reads expand absent allowed rows to zero and reject rather than hide any persisted disabled or invalid pair. The primary key proves stored uniqueness, and save deletion cascades all targets.

`src-tauri/src/features/staff/assignment_targets.rs` owns the closed target catalog, allowed scope/job pairs, exact-completeness validation, count and duplicate validation, canonical target expansion, active-save target reads, positive-row compaction, and transactional replacement. It reads Planner team rows for availability and current display names. `planner::teams::planner_team_removal_impacts` computes each proposed removed team's assignment count and ordered nonzero staffing targets with canonical job label and slot count. A typed `get_planner_team_removal_impacts` command lets the existing **Manage teams** Modal name that impact before mutation, while `planner::teams::save_team_settings` recomputes the same impact inside its transaction and rejects an unconfirmed removal when assignments or any staffing target exists. One confirmation names the assignment count and every affected staffing target together when both exist. A confirmed save deletes the removed scopes' targets, assignments, strings, and team rows before commit. Re-enabling a team creates its empty Planner string and leaves all its target counts at zero.

The closed Preferred Job mapping is:

| Stored Preferred Job | Canonical target | Persisted score |
| --- | --- | --- |
| `Manager` | Manager | `manager` |
| `Assistant Manager` | Assistant Manager | `assistant_manager` |
| `Coach` | Coaches | highest available of the six outfield coaching scores |
| `Set Piece Coach` | Set Piece Coach | `set_piece_coach` |
| `Head Performance Analyst` | Head Performance Analyst | `head_performance_analyst` |
| `Performance Analyst` | Performance Analyst | `performance_analyst` |
| `Head of Youth Development` | Head of Youth Development | `head_of_youth_development` |
| `Director of Football` | Director of Football | `director_of_football` |
| `Technical Director` | Technical Director | `technical_director` |
| `Loan Manager` | Loan Manager | `loan_manager` |
| `Chief Scout` | Chief Scout | `scout` |
| `Scout` | Scout | `scout` |
| `Head Physio` | Head Physio | `physio` |
| `Physio` | Physio | `physio` |
| `Head of Sports Science` | Head of Sports Science | `sports_scientist` |
| `Sports Scientist` | Sports Scientist | `sports_scientist` |

Matching uses Rust's ASCII case-insensitive whole-string comparison against the trimmed import value. There are no other aliases. For `Coach`, choose the maximum available score in this tie order: Attacking Technical, Attacking Tactical, Defending Technical, Defending Tactical, Possession Technical, Possession Tactical. Return the selected discipline with the score. If all six values are unavailable, the candidate is unavailable.

`src-tauri/src/features/staff/assignment_optimizer.rs` owns pure candidate eligibility and direct allocation. It partitions candidates by the canonical Preferred Job mapping. Because each candidate has one Preferred Job and each mapping resolves to one target group, it sorts each group once and fills that group's requested scopes and slots directly. It does not introduce a matching library, graph model, or general solver. A global assigned-UID set still enforces the version-1 single-duty invariant.

`src-tauri/src/features/staff/assignment_optimizer_query.rs` owns the bounded current-state join and IPC result construction. It resolves and validates the active save and its expected immutable context token, then resolves the effective current snapshot and validates its expected immutable context token before reading the managed club, enabled Planner teams with their current persisted display names, shortlist existence, saved targets, relevant current staff rows, and left-joined persisted scores from the closed required-role catalog under the database lock. A missing or null score row remains unavailable. After pure allocation, the query maps each slot to a bounded Rust result row with `scopeDisplayName`: `Club` for Club scope and the matching enabled Planner team's current persisted display name for a team scope. React receives this label and does not map raw scope IDs. The numeric snapshot ID remains returned data and a join key, but it is not freshness authority. The command returns at most 1,750 slot rows: 29 possible team job/scope pairs plus 6 Club pairs, each capped at 50. It also returns one bounded evidence summary per canonical job group, including joined candidate count, eligible-score count, and unavailable-score count. It does not return the 10,000-row pool.

The optimizer response has explicit `stale_context`, `no_current_snapshot`, `no_managed_club`, `no_shortlist`, and `ready` states. The request and response carry the expected/resolved save and snapshot immutable context tokens; the snapshot ID can remain present as data. `ready` can contain zero current joined shortlist people or zero configured slots; the DTO reports both counts so the UI can distinguish those conditions. Rust derives all authoritative data from SQLite. Any token mismatch returns `stale_context`; deleting a snapshot and reusing its numeric ID cannot make an old request current.

React adds typed Staff API modules and Query keys. `StaffAssignmentOptimizer` lives inside the Staff feature, uses Query for saved targets and mutations for target replacement and on-demand optimization, and keeps only Modal and latest-result presentation state locally. The existing `get_current_snapshot` Rust query/DTO, frontend `SnapshotSummary`, and independent Load Data snapshot-summary contract gain the snapshot context token; `LoadDataOutcome` compares those tokens rather than numeric snapshot IDs when it decides whether the stored snapshot became effective. `my-club.tsx` passes a context key built from the active save ID and token, snapshot ID and immutable token, managed-club value, enabled Planner team identities/display names, and a route-owned shortlist import revision. It also derives `contextUnavailable` from the existing save, snapshot, managed-club, and Planner refresh state, `playerResultContextMutationKey`, and the Shortlist import Modal's local pending state through one narrow callback. Commit 2 assigns the shared `playerResultContextMutationKey` to `PlannerTeamManagement`'s existing `save_planner_teams` mutation while preserving its local `onPendingChange` signal for Planner controls. The route's existing `useIsMutating({ mutationKey: playerResultContextMutationKey })` therefore observes a Planner-team save as soon as it starts, so Commit 6's `contextUnavailable` immediately disables Optimize and suppresses any visible recommendation until the command resolves. `StaffAssignmentOptimizer` captures the starting context key and request generation for each optimize mutation. Target-save reset, any route context-key replacement, and the start of any `contextUnavailable` reset increment the generation and clear mutation/result state; completion presents a result only when its captured key and generation still match. This explicit guard is required because `optimize.reset()` alone does not cancel mutation callbacks. Existing Load Data, save switch, managed-club save, Planner-team save, and shortlist import invalidations refresh the underlying Query data.

The Shortlist toolbar adds **Configure slots** and **Optimize assignments** beside **Upload CSV**. The form Modal groups numeric controls by enabled team display name and Club, omits Senior Manager, validates 0-through-50 locally for prompt feedback, and relies on Rust for authoritative exact-completeness and range validation. The result `Panel` uses a semantic table ordered Senior, Reserves, Youth, Club by the Rust row order; each row renders the Rust-provided `scopeDisplayName`, canonical job label, and slot number. Filled rows show a `ScoreBadge`, candidate type, Preferred Job, and Coach discipline when applicable. Vacancy rows show an em dash for person and score plus the Rust-provided eligible/unavailable evidence. A compact note reports unsupported Preferred Jobs as excluded without removing those people from the Shortlist table.

## Uncertainty register

### Known

- Linear JAY-16 supplies the approved product behavior and the four developer decisions recorded in this ledger.
- There is no planned feature spec and no prior active ledger to promote.
- Ingest persists only calculable staff role scores and omits uncalculable role rows; the schema also permits null scores. Staff Search, Staff Profile, and Shortlist presentation already preserve missing values as unavailable.
- Staff Shortlist rows are save-owned but only current-snapshot UID joins are product-visible.
- Planner team identities are fixed and save-scoped; display names and availability are persisted in `planner_teams`.
- GitHub allows only squash merges and protects `main` with the strict required `check` status.

### Assumptions

- The slot maximum is decided rather than assumed: the developer approved 50 slots per canonical job in one scope. With 35 possible scope/job pairs, the worst-case result is 1,750 rows; a request above 50 is invalid rather than silently clamped.
- The imported Preferred Job strings are English FM labels. ASCII case-insensitive exact matching is therefore sufficient and agrees with current SQLite `COLLATE NOCASE` shortlist behavior.
- Candidate classification needs only exact equality between current `staff.club` and `managed_club_settings.club_name`; no club-family expansion applies to staff employment.

### Decisions

- Use `(save_id, scope, job_id)` as the target primary key. Require exactly one input entry, including zero, for every currently allowed Club or enabled-team pair. Validate exact completeness, uniqueness, team availability, pair validity, and 0-through-50 bounds before mutation; only then replace the complete active-save set transactionally and store positive rows.
- Confirm and delete disabled-team targets in the existing **Manage teams** flow. A nonzero assignment count or any nonzero staffing target requires confirmation; the impact names each affected canonical job and slot count. One confirmation names assignments and all targets together when both exist. The confirmed Planner transaction deletes targets, assignments, strings, and team rows atomically. Re-enabling starts targets at zero, and Rust never silently hides a disabled-team target.
- Keep Manager unavailable for Senior even if a caller submits it. Reject the complete target write rather than discard it.
- Keep Chief Scout, Scout, Head Physio, Physio, Head of Sports Science, and Sports Scientist as separate target groups. The named head roles share the approved underlying score but do not share candidates with their non-head roles.
- Use direct preferred-job partitioning, score-descending and UID-ascending sorting, then ordered slot filling. Do not use the Squad optimizer's exact matcher because this feature has no cross-job competition that requires matching.
- Return aggregated unavailable-score evidence per canonical job group and repeat the relevant counts on vacancy rows. This reports missing data truthfully without sending the complete 10,000-row shortlist over IPC.
- Keep recommendation results ephemeral in a context-keyed, generation-guarded mutation outcome. Every target or context reset invalidates the generation because TanStack mutation reset does not cancel an in-flight completion. Persist target counts only.
- Return `scopeDisplayName` from the Rust optimizer query on every result slot. Use the current persisted Planner display name for enabled team scopes and `Club` for Club scope; React renders the supplied label without owning a scope-name map.
- No ADR is warranted. The persistence, Rust authority, context guards, direct allocation, and route composition follow existing repository boundaries and do not establish a new cross-project architecture.

### Unknowns

- Representative real saves may use different capitalization for the approved English Preferred Job labels. The exact case-insensitive contract handles capitalization but intentionally does not handle different words.
- Native Tauri/WebView focus, numeric-input behavior, and dense result-table fit at supported window sizes cannot be proved by jsdom or the browser IPC stub.
- The current repository has no supported mutation-testing command, so automated fault-injection evidence is unavailable.

### Risks

- A SQL join or mapping regression could admit non-shortlisted staff or exclude a valid exact label.
- A missing or null score could be coerced to zero and incorrectly become eligible.
- Equal scores could produce nondeterministic assignments without the UID tie-break.
- Coach maximum selection could choose the wrong discipline or erase null when all six scores are absent.
- A stale result could remain visible during pending save, snapshot, managed-club, Planner-team, shortlist, or target work if the route relies only on eventual context-key replacement. A late optimize callback could also restore that result after `optimize.reset()` when the reset leaves save and snapshot tokens unchanged unless every reset invalidates an explicit request generation. Reusable numeric snapshot IDs remain unsafe freshness authority.
- Team removal could silently delete nonzero targets, omit targets from the impact when assignments also exist, orphan disabled-team rows, partially delete Planner data, or remove Club/other-team targets.
- An incomplete or duplicate replacement payload could erase valid counts unless Rust validates the exact allowed pair set before starting delete/insert.
- The UI could duplicate Rust allocation or scope-name rules, optimize only the currently filtered Shortlist page, or expose raw scope IDs instead of current configured Planner names.
- The new result panel could break the Shortlist's bounded-height table, keyboard path, or focus restoration.

## Walking skeleton

Persist one Assistant Manager slot for the enabled Senior team, run one Rust command against the active save's current shortlist join, assign the highest-scoring exact `Assistant Manager` candidate with a stable UID tie-break, and render the filled row or vacancy inside the existing Staff Shortlist workspace. Then extend the same closed catalog and direct path to every approved target, team, alias, evidence state, and context-change rule.

## Delivery plan

### PR 1 — Optimize staff assignments from the Shortlist

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/staff-assignment-optimizer

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** strict required GitHub Actions `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(staff): optimize shortlist assignments`

**Purpose:** Deliver one end-to-end, trunk-safe Staff Shortlist assignment-planning capability. Persistence, allocation, IPC, UI, and browser evidence share one user workflow and do not form useful independently publishable seams.

**Depends on:** Current `main`; no planned feature, open ledger, external service, or prior PR.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(staff): plan assignment optimization`

**Work:** Commit the independently reviewed staff-assignment ledger and TODO activation before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, migrations, executable configuration, generated files, BACKLOG changes, current-state documents, ADRs, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed ledger and TODO change after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/staff-assignment-optimizer.md` — approved intent, architecture, delivery authority, implementation packets, validation, and risk register.
- `.wiki/TODO.md` — move JAY-16 from no active entry into Active and link this ledger; retain the existing gender-data item in Next.
- `.wiki/BACKLOG.md` — deliberately unchanged because no deferred scope is promoted or reclassified.
- `.wiki/features/planned/` — deliberately unchanged because no planned spec exists.
- `.wiki/decisions/` — deliberately unchanged because no decision meets the ADR threshold.

**Behavior and data flow:**

- Record the reviewed planning truth in one schema-2 ledger and make TODO point to it before the feature branch receives implementation.

**Ordered implementation steps:**

1. Verify `feature/staff-assignment-optimizer` is based on `main` without changing reviewed content.
2. Confirm the worktree still preserves the unrelated developer-owned completed-feature modification.
3. Before review, keep the Delivery fingerprint placeholder and run the ledger classifier plus pre-review delivery classification only to inspect provisional packet fingerprints.
4. After a fresh complete plan review clears and the developer accepts the plan, require the ledger to contain the exact reviewed, recorded, developer-accepted Delivery fingerprint. Verify it with both classifiers before any authorized checkpoint.
5. Stage only the ledger and TODO, inspect the exact staged diff, and obtain the normal independent checkpoint review.

**Tests and proof:**

- Not applicable — independently reviewed planning documents only. The schema-2 ledger classifier proves lifecycle structure, complete packets, one Active commit, branch metadata, and active-work pointers.

**Patterns to verify:**

- `.wiki/features/active/README.md`, `.wiki/TODO.md`, the completed Staff Shortlist and Optional Planner Teams records, and current GitHub PR authority.

**Constraints and non-goals:**

- Do not alter feature intent, packet order, authority fields, implementation, tests, BACKLOG, current-state docs, ADRs, the unrelated completed record, or Git state outside the separately authorized delivery workflow.

**Dependencies and sequencing:**

- Requires a clear independent plan review, developer acceptance, classifier success, recorded Delivery fingerprint, and exact feature-branch activation before commit authority exists.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/staff-assignment-optimizer.md`

**Stop conditions:** Stop on an uncleared plan review, classifier error, substantive post-review change, fingerprint mismatch, branch mismatch, or any changed path other than the ledger and TODO.

**Review mandate:** Verify the ledger preserves every approved JAY-16 decision, uses current repository seams, gives every later commit an executable packet, records one PR with exact GitHub authority, and changes only approved planning paths without absorbing the unrelated completed-feature modification. The placeholder is valid only during planning before the fresh complete review; the authorized checkpoint must contain and revalidate the exact reviewed, recorded, developer-accepted Delivery fingerprint.

#### Commit 2 — Persist staff assignment targets

**Status:** Completed

**Provisional commit:** `feat(staff): persist assignment targets`

**Work:** Add migration v35 and Rust-owned complete target replacement, then extend Manage teams with typed removal impact, confirmation, and atomic disabled-team cleanup.

**Size assessment:** About 330 changed non-test implementation lines. Exceeds the soft target for a genuine atomic data-loss boundary: the table, exact replacement validator, transactional Planner cleanup, independent removal-impact command, and existing Manage teams confirmation must land together so no reachable build can silently delete nonzero targets or create target rows that team removal can orphan.

**Out of scope:**

- Candidate joins, Preferred Job mapping, recommendation allocation, Staff Shortlist configuration controls, result presentation, and browser optimizer fixtures.

**Implementation packet:**

- Add the smallest save-scoped target table and closed Rust catalog needed to store positive slot counts. Expose typed get/save commands, require one explicit entry for every currently allowed pair before mutation, and extend the existing Planner-team removal flow so one typed impact and confirmation cover assignments and staffing targets before the same transaction deletes all removed-team state.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — add `STAFF_ASSIGNMENT_TARGETS_SQL` with primary key `(save_id, scope, job_id)`, 1-through-50 stored-count checks, save cascade, migration v35 registration, and fresh/upgrade/constraint/uniqueness/cascade coverage without changing prior migration behavior.
- `src-tauri/src/features/staff/assignment_targets.rs` — own canonical scopes/jobs, the exact currently allowed pair set, 0-through-50 input validation, missing/extra/duplicate/disabled/invalid rejection, zero expansion, positive-row compaction, persisted-invalid-row detection, complete prevalidation, and delete/insert replacement in one transaction. Expose only the crate-visible target-count and removed-scope cleanup seams that Planner needs inside its transaction.
- `src-tauri/src/features/staff/mod.rs` — register the target module.
- `src-tauri/src/features/staff/commands.rs` — add camelCase target/input DTOs and thin `get_staff_assignment_targets` and `save_staff_assignment_targets` commands that resolve the active save and validate its immutable save context token in Rust.
- `src-tauri/src/features/planner/teams.rs::planner_team_removal_impacts` and `save_team_settings` — compute assignment count and an ordered `{ job_id, job_label, slot_count }` entry for every nonzero staffing target on each proposed removed team; recompute under the save transaction; require `confirm_populated_removal` when assignments or targets exist; and delete targets, assignments, strings, and team rows atomically while retaining Club and enabled-team targets.
- `src-tauri/src/features/planner/commands.rs::PlannerStaffingTargetRemovalImpactDto`, `PlannerTeamRemovalImpactDto`, `get_planner_team_removal_impacts`, and `save_planner_teams` — expose the typed pre-mutation job/slot impact for proposed settings and keep save-time confirmation authoritative.
- `src-tauri/src/lib.rs::run` — register the two Staff target commands and the Planner removal-impact command.
- `src-tauri/src/features/planner/teams_tests.rs` — prove impacts for assignments-only, targets-only, and both; no confirmation for zero impact; unconfirmed no-write behavior; confirmed all-or-nothing deletion; re-enable-at-zero behavior; and retention of Club, other-team, and other-save rows.
- `src/features/planner/types/team-removal-impact.ts` — define the typed team/display-name, assignment-count, and ordered staffing-target job-label/slot-count DTOs.
- `src/features/planner/api/fetch-planner-team-removal-impacts.ts` — call `get_planner_team_removal_impacts` through `invokeCommand` with the proposed complete Planner settings.
- `src/features/planner/api/save-planner-teams.ts::savePlannerTeams` — retain the single authoritative `confirmPopulatedRemoval` flag for the combined assignment/target removal.
- `src/features/planner/components/planner-team-management.tsx::RemovedTeam`, `requestSave`, confirmation content, and existing save mutation — import the shared `playerResultContextMutationKey`; assign it as the `mutationKey` of the existing `save_planner_teams` mutation; fetch the current Rust impact before save; open one confirmation when assignments or any target exists; and name the assignment count plus every target job/slot count together when both exist. Preserve cancel, focus restoration, pending, error, and the current local `onPendingChange` behavior for `PlannerDepthMatrix` controls.
- `src/testing/planner-ipc-mock.ts`, `src/testing/setup.ts`, and `src/app/routes/my-club-squad.test.tsx` — add typed impact fixtures/handler/request inspection and route-visible confirmation proofs for targets-only and combined impacts.
- `e2e/tauri-ipc-stub.ts` and the existing Planner Manage teams scenario in `e2e/smoke.spec.ts` — support the new impact command and preserve the existing removal workflow; Commit 7 adds optimizer-specific browser coverage rather than repairing this Planner contract later.
- Existing staff, shortlist, Planner, migration fixtures and tests — retain; do not add a second database helper unless existing migrated temporary-DB setup cannot express the proof.

**Behavior and data flow:**

- A target save resolves the active save and immutable save token, computes the currently allowed Club plus enabled-team pairs, and compares that set with the submitted entries. Missing, extra, duplicate, disabled, invalid, or out-of-range entries fail before the transaction deletes anything. After full validation, one transaction deletes all prior target rows for that save and inserts only positive entries; a forced insert failure rolls back to the exact prior set. Reads return one zero-or-positive entry per allowed pair and reject rather than hide persisted disabled or invalid rows.
- When the user removes a Planner team, `PlannerTeamManagement` asks Rust for `PlannerTeamRemovalImpactDto` rows for the proposed settings. Zero-impact teams need no confirmation. A nonzero assignment count or any staffing target opens the existing destructive confirmation; the team item names the assignment count and each affected job label with its slot count when both exist. The component's existing save mutation uses the shared `playerResultContextMutationKey`, so the route-level mutation observer sees `save_planner_teams` immediately while the existing local `onPendingChange` signal continues to disable Planner controls. `save_team_settings` recomputes the impact in its transaction so a stale preview cannot bypass confirmation, then a confirmed save deletes the removed scopes' target rows, assignments, strings, and team rows atomically. Re-enabling creates the existing empty Planner string and target reads expand that team's counts to zero.

**Ordered implementation steps:**

1. Add RED migration tests for the exact v35 columns, composite primary key, 1 and 50 acceptance, 0 and 51 rejection, populated-v34 upgrade, save cascade, and registry order.
2. Add RED target-service tests for an exact complete replacement, zero-row compaction, missing entry, extra/unknown pair, duplicate input, disabled-team pair, Manager-Senior pair, boundary count, stale save token, stored uniqueness, pre-mutation rejection, forced insert rollback, and rejection of a persisted disabled/invalid row instead of hiding it.
3. Add the migration and minimum target module; keep validation separate from mutation and make the persistence proofs GREEN.
4. Add RED Planner service tests for assignment-only, targets-only, combined, and zero impacts. Extend `planner_team_removal_impacts` and `save_team_settings` so an unconfirmed nonzero impact writes nothing and a confirmed save atomically deletes exactly the removed team's targets/assignments/strings/team while retaining Club, enabled-team, and other-save rows.
5. Add RED Planner API/component/route tests for the impact command, targets-only confirmation, one combined confirmation, cancel/focus/error behavior, and re-enable-at-zero outcome. Extend one already-planned route-visible confirmation proof to assert that the existing save mutation appears under `playerResultContextMutationKey` while its command is deferred; do not add a test case. Implement the DTO, command, typed frontend API, existing Modal changes, shared mutation key, IPC mock, and setup handler without changing the local `onPendingChange` behavior.
6. Update the existing browser stub and Manage teams smoke contract for the impact command. Register all commands and refactor only while focused Rust, route, and existing smoke proofs remain green.

**Tests and proof:**

- RED: fresh migration lacks `staff_assignment_targets`; v34 cannot read it; no current contract rejects incomplete replacement; and Manage teams cannot discover or confirm nonzero staffing-target loss.
- GREEN: migration tests assert the composite primary key, checks, uniqueness, populated-v34 no-backfill, and cascade. Service tests prove exact complete replacement, all trust-boundary failures before mutation, forced-failure rollback, and no hidden disabled targets. Planner Rust and route tests fail if target impact is omitted, assignments and targets split into separate confirmations, save-time impact is not recomputed, an unconfirmed call writes, cleanup reaches Club/another enabled team/another save, or a deferred `save_planner_teams` command is absent from the route-visible shared mutation count.
- Add/modify: migration unit tests, target-module tests, Planner-team service tests, one Planner removal-impact type/API, Planner IPC fixtures, route tests, and the existing Manage teams browser fixture/scenario.
- Delete: none.
- Retain: existing v1–v34 migration, Planner string/assignment confirmation and focus tests, Staff shortlist and role-score tests, and unrelated smoke workflows because each still protects a supported adjacent contract.

**Patterns to verify:**

- `PLANNER_TEAM_SETTINGS_SQL`, `STAFF_SHORTLIST_SCHEMA_SQL`, migration registry tests, `planner::teams::save_team_settings`, `PlannerTeamManagement::requestSave`, `playerResultContextMutationKey` use in managed-club and snapshot mutations, route-level `useIsMutating({ mutationKey: playerResultContextMutationKey })`, `save_planner_teams`, `resolveSavePlannerTeamsIpcMock`, the Manage teams smoke scenario, and immutable save-token validation in snapshot and Club DNA mutations.

**Constraints and non-goals:**

- Store no recommendation, candidate, score, employment, or snapshot data. Do not silently clamp counts, infer missing zero entries, discard invalid rows, hide disabled-team rows, or trust the impact preview as save authority. Staff target get/save must not require a current snapshot because targets are save-owned. Preserve the existing `save_team_settings` snapshot preflight for Manage teams; this packet extends its confirmed transaction and does not weaken or remove that current Planner contract. Do not recreate removed Planner teams. Keep SQL and authoritative impact/validation in Rust; React only presents the typed impact and confirmation.

**Dependencies and sequencing:**

- Depends only on Commit 1. Commit 3 consumes the canonical target catalog; later frontend commits consume the typed target command contract. The existing Planner workflow remains complete and green in this commit.

**Validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx` then `./scripts/dev check-rust` then `./scripts/dev smoke` then `./scripts/dev check`

**Stop conditions:** Stop if the exact allowed pair set cannot be computed before mutation, save-time removal impact cannot be recomputed in the same transaction, the existing Modal cannot present one combined confirmation, target cleanup cannot share the Planner transaction, `PlannerTeamManagement` cannot expose its existing save mutation through the shared key while retaining local `onPendingChange`, a persisted disabled target would be silently hidden, or the safe data-loss boundary requires a different persistence or public command contract.

**Review mandate:** Verify the composite key, constraints, migration, and rollback; exact complete payload and 0-through-50 validation before delete; stale-token and forced-insert rollback proofs; typed assignment/target impact with one combined confirmation; save-time impact recomputation and unconfirmed no-write behavior; atomic target/string/assignment/team cleanup; the existing Planner save mutation uses the shared `playerResultContextMutationKey` and retains local `onPendingChange`; zero-on-re-enable, no hidden disabled rows, and retention of Club/enabled-team/other-save rows; and valuable Rust, route, mock, and smoke tests.

#### Commit 3 — Allocate canonical staff job groups

**Status:** Completed

**Provisional commit:** `feat(staff): allocate assignment candidates`

**Work:** Implement and unit-test the pure Preferred Job mapping, score selection, deterministic direct allocation, vacancies, and bounded evidence model.

**Size assessment:** About 190 changed non-test implementation lines. Within the soft target; the direct partition/sort/fill algorithm avoids a solver dependency and remains isolated from SQL and IPC.

**Out of scope:**

- Database candidate loading, Tauri commands, target persistence changes, React, and browser behavior.

**Implementation packet:**

- Add a Staff-private pure allocator whose inputs are canonical targets and bounded joined candidates. Model missing scores explicitly, map only approved exact labels, choose Coach's best available discipline, sort deterministically, enforce global UID uniqueness, and emit assignments/vacancies plus per-job evidence.

**Files and responsibilities:**

- `src-tauri/src/features/staff/assignment_optimizer.rs` — define candidate, score-set, recommendation, vacancy, classification, Coach discipline, and evidence types; own exact mapping, score resolution, canonical ordering, and direct allocation.
- `src-tauri/src/features/staff/mod.rs` — register the private optimizer module.
- `src-tauri/src/features/staff/assignment_optimizer_tests.rs` or colocated `#[cfg(test)]` tests — cover the supported mapping and allocation contract with small explicit fixtures.
- `src-tauri/src/features/staff/scoring.rs` — deliberately retain the persisted score catalog and formulas unchanged; import role IDs rather than recalculate scores if visibility permits, or use exact existing IDs without changing the catalog.

**Behavior and data flow:**

- The allocator receives target rows in canonical scope/job/slot order and one candidate record per current Shortlist join. It maps one Preferred Job to at most one canonical job group, resolves the required existing score, partitions unavailable candidates into evidence, sorts eligible candidates by score descending and UID ascending, and walks that group's ordered target slots. It adds each selected UID to one global set and emits an explicit vacancy for every unfilled slot.

**Ordered implementation steps:**

1. Add RED table-driven tests for every exact Preferred Job mapping and every rejected near-match/unrecognized label.
2. Add RED Coach tests for highest available score, canonical discipline tie order, partial nulls, and all-null unavailability.
3. Add RED allocation tests for score order, UID tie-break, Senior→Reserves→Youth→Club scope order, Manager's Reserves/Youth restriction, 50-slot boundary ordering, one-UID uniqueness, explicit vacancies, current/recruitment preservation, and unavailable evidence.
4. Implement the minimum catalog-driven partition/sort/fill logic to make the proofs GREEN.
5. Refactor duplicate catalog lookups only when the focused tests stay green; do not introduce a generic optimizer abstraction.

**Tests and proof:**

- RED: no current module maps the approved labels or can produce assignment rows and vacancies.
- GREEN: table-driven pure tests fail if an alias crosses jobs, null becomes zero, Coach chooses the wrong tied discipline, order is unstable, a UID repeats, or a vacancy/evidence count disappears.
- Add: pure allocator tests and small in-memory candidate fixtures.
- Modify: module registration only.
- Delete: none.
- Retain: existing `staff::scoring` formula tests and Shortlist presentation tests; they protect persisted input and existing table behavior rather than allocation.

**Patterns to verify:**

- `staff::scoring::all_staff_roles`, `score_staff_role` null semantics, `planner::optimizer` stable UID conventions, and `staff-shortlist-presentation.ts` approved score IDs. Deliberately diverge from Planner's exact matcher because Preferred Job partitioning removes cross-slot competition.

**Constraints and non-goals:**

- No SQL, Tauri DTO, persistence, external crate, employment scoring, filter input, substring match, locale inference, or fallback substitution. Treat all missing values as unavailable. Keep evidence bounded to aggregate counts per canonical job group.

**Dependencies and sequencing:**

- Depends on Commit 2's canonical target types and ordering. Commit 4 supplies authoritative current-state candidates and exposes results over IPC.

**Validation:** `./scripts/dev check-rust` then `./scripts/dev check`

**Stop conditions:** Stop if one Preferred Job can validly map to multiple target groups, requirements introduce cross-job competition, the six coaching scores are not available as persisted values, deterministic direct allocation cannot satisfy one-UID uniqueness, or a general matcher becomes necessary.

**Review mandate:** Verify all 16 exact labels and exactly three shared-score mappings with no extra substitution: Chief Scout→`scout`, Head Physio→`physio`, and Head of Sports Science→`sports_scientist`. Verify generic Coach separately as the maximum across the six outfield coaching scores with the recorded discipline tie order; Manager scope; Coach null behavior; score-descending and UID-ascending ranking; canonical scope and slot order; global uniqueness and explicit vacancies; classification passthrough with bounded unavailable evidence; and tests that detect realistic wrong mappings, null handling, and ordering.

#### Commit 4 — Expose current-context recommendations

**Status:** Completed

**Provisional commit:** `feat(staff): expose assignment recommendations`

**Work:** Expose the immutable token on the current-snapshot contract, join the current Staff Shortlist to authoritative current staff and persisted scores, run the allocator, and expose a bounded token-guarded IPC command.

**Size assessment:** About 240 changed non-test implementation lines. Exceeds the soft target because one coherent Rust freshness boundary must extend the shared current-snapshot summary with its immutable token and consume that same token in the optimizer query/DTO; splitting would leave either an unusable token or an optimizer that still authorizes reusable numeric IDs.

**Out of scope:**

- Frontend controls, frontend snapshot types, result layout, route changes, browser stubs, new scores, or recommendation persistence.

**Implementation packet:**

- Add a Rust query service that resolves all current context, distinguishes bounded setup states, loads only joined shortlist candidates and relevant persisted scores, classifies exact managed-club membership, invokes Commit 3's allocator, and maps the bounded result to one typed command DTO.

**Files and responsibilities:**

- `src-tauri/src/features/snapshot/ingest.rs::SnapshotSummary` — add the immutable `context_token` field to the existing current/effective snapshot summary model without changing numeric ID ownership.
- `src-tauri/src/features/snapshot/query.rs::get_current_snapshot` and `map_snapshot_row` — select and map `snapshots.context_token` for the effective current row.
- `src-tauri/src/features/snapshot/commands.rs::SnapshotSummaryDto` and its conversion/serialization tests — expose `contextToken` through `get_current_snapshot` and Load Data stored/effective summaries.
- `src-tauri/src/features/staff/assignment_optimizer_query.rs` — resolve active save/context, current snapshot ID/token, managed club, enabled teams, shortlist existence, targets, joined candidates, the closed required-role catalog, left-joined score rows, counts, and state; invoke the pure allocator. Match both expected immutable tokens; use numeric IDs only for joins and returned data; preserve an omitted or null score row as unavailable.
- `src-tauri/src/features/staff/mod.rs` — register the query module.
- `src-tauri/src/features/staff/commands.rs` — add `optimize_staff_assignments(expectedSaveContextToken, expectedSnapshotContextToken)`, explicit state/result DTOs that return resolved save/snapshot IDs and tokens, and camelCase serialization.
- `src-tauri/src/lib.rs::run` — register the optimizer command.
- `src-tauri/src/features/staff/assignment_optimizer_query.rs` tests or `assignment_optimizer_query_tests.rs` — use migrated temporary SQLite databases for authoritative join, state, classification, target, score, immutable-token, and forced numeric-ID-reuse proofs.
- Existing `src-tauri/src/features/staff/query.rs` — retain its paged presentation query unchanged unless one small shared current-context helper can be reused without coupling filter/paging behavior.

**Behavior and data flow:**

- `get_current_snapshot` and Load Data return each snapshot's immutable context token with its numeric ID. `optimize_staff_assignments(expectedSaveContextToken, expectedSnapshotContextToken)` locks the database, resolves the active save and token, selects its effective current snapshot and token, and returns `stale_context` on either mismatch. It then requires managed-club configuration and shortlist existence, reads enabled Planner teams and saved targets, joins shortlist rows to that snapshot ID's staff by UID, and left-joins the closed required score IDs. A missing or null score row stays unavailable and contributes unavailable evidence; ingest is not required to materialize uncalculable role rows. The query classifies exact current club equality and runs direct allocation. The response returns resolved IDs and immutable tokens, source counts, ordered buckets/slots, and bounded job evidence. It performs no write, and a deleted snapshot whose numeric ID is later reused cannot satisfy the old token.

**Ordered implementation steps:**

1. Add RED snapshot query/DTO tests that require `contextToken` on `get_current_snapshot` and Load Data stored/effective summaries while retaining the numeric ID.
2. Add RED optimizer integration tests for stale save token, stale snapshot token, forced deletion and numeric snapshot-ID reuse with a different token, `no_current_snapshot`, `no_managed_club`, `no_shortlist`, ready-with-zero-joined-candidates, and ready-with-zero-targets.
3. Add RED join tests proving My Staff outside the shortlist is excluded and current Preferred Job/unemployment UI filters are irrelevant.
4. Add RED score/classification tests for exact managed-club equality, persisted Manager and the three shared-score mappings, separate Coach discipline selection, missing-row and explicit-null exclusion with unavailable evidence, unrecognized-job counts, deterministic allocation, and the 1,750-row output bound.
5. Implement the minimum current-snapshot token plumbing, parameterized query, and orchestration to make those tests GREEN.
6. Add response-token serialization and command-registration proof; run all Rust tests to catch migration, snapshot, Load Data, and Staff regressions.

**Tests and proof:**

- RED: no command can return current assignment recommendations or truthful setup states.
- GREEN: real SQLite tests fail if a non-shortlisted staff row enters, a stale save or snapshot token is trusted, forced numeric ID reuse retargets an old request, a missing or null score becomes eligible or loses unavailable evidence, a different club is marked current staff, a saved shortlist without a current join is called absent, or output order/bounds change.
- Add: current-context query tests, forced-ID-reuse fixture/proof, and optimizer command DTO serialization tests.
- Modify: snapshot current-query and summary DTO tests plus Staff command registration tests if present.
- Delete: none.
- Retain: paged Shortlist query, import transaction, Staff Search/Profile, score-ingest, and Planner tests because the optimizer is a read-only consumer.

**Patterns to verify:**

- `snapshot::query::get_current_snapshot`, `SnapshotMetadataDto` token serialization, snapshot-history forced-ID-reuse tests, `staff::query::list_staff_with_shortlist`, `staff::service::capture_boost_context` save/snapshot token checks, `managed_club` exact comparison, persisted role-score loading, and bounded DTO conventions in Staff/Planner commands.

**Constraints and non-goals:**

- Rust remains authoritative. Bind all values; do not build SQL from user strings. Do not accept Preferred Job filters or unemployment flags. Do not hold a lock across async work, persist results, recompute scores, or return raw rows/internal errors. Ready with no current joins must differ from no saved shortlist.

**Dependencies and sequencing:**

- Depends on Commit 2 targets and Commit 3 allocation. Frontend commits depend on the final DTO and command names from this packet.

**Validation:** `./scripts/dev check-rust` then `./scripts/dev check`

**Stop conditions:** Stop if `get_current_snapshot` cannot expose the existing immutable token without changing snapshot identity, current selection differs from the shared effective-current contract, forced numeric ID reuse can satisfy an old token, required role IDs are absent from the closed persisted catalog, missing/null rows cannot be distinguished and preserved as unavailable, the 1,750-row bound is not enforceable, or implementation requires changing shortlist import semantics.

**Review mandate:** Verify current-snapshot token plumbing; save and snapshot immutable-token authority under forced numeric ID reuse; exact save-owned shortlist/current-snapshot join with no My Staff or presentation-filter expansion; exact managed-club classification; closed-catalog, persisted-score-only left joins that preserve both missing and null rows as unavailable; state distinctions and parameter binding; the 1,750-row bound/order with no writes; and command/DTO registration plus realistic temporary-SQLite tests.

#### Commit 5 — Configure assignment slots in My Club

**Status:** Completed

**Provisional commit:** `feat(staff): configure assignment slots`

**Work:** Add typed frontend target APIs and an accessible configuration Modal inside the existing Staff Shortlist toolbar.

**Size assessment:** About 320 changed non-test implementation lines. Exceeds the soft target because one accessible, trunk-safe configuration outcome needs its typed API/query contract, complete 35-pair-capable draft form, mutation lifecycle, snapshot-token context ownership, and route composition together; splitting would leave either an unreachable API or a control that cannot safely persist the complete replacement contract.

**Out of scope:**

- Optimize action, recommendation result Panel, client-side allocation, route or global store additions, and optimizer-specific browser command handling or smoke changes.

**Implementation packet:**

- Add Staff-owned frontend types/fetchers/Query options for target reads and writes. Add a configuration Modal that renders the Rust-provided enabled teams and complete catalog, edits exactly one local entry for every returned allowed pair, validates 0-through-50 for prompt feedback, saves the complete draft, and reconciles Query/local state. Compose it in the existing Shortlist toolbar with a route-owned context key that includes both save and snapshot immutable tokens.

**Files and responsibilities:**

- `src/features/snapshot/types/snapshot.ts::SnapshotSummary` — add the `contextToken` field supplied by Commit 4's current-snapshot DTO.
- `src/features/memory-read/types/load-data.ts::LoadDataSnapshotSummary` — add the same `contextToken` field to the independent frontend Load Data result contract.
- `src/features/memory-read/components/load-data-outcome.tsx::resolveBanner` — use `storedSnapshot.contextToken === effectiveSnapshot.contextToken`, not numeric ID equality, to decide whether the stored snapshot became the effective snapshot.
- `src/features/memory-read/components/load-data-outcome.test.tsx` — update stored/effective fixtures with tokens and prove that equal numeric IDs with different tokens do not report the stored snapshot as latest.
- `src/features/staff/types/staff-assignment.ts` — typed target, scope/team, catalog, response, complete-save request, and save/snapshot immutable-context shapes matching Rust camelCase DTOs.
- `src/features/staff/api/fetch-staff-assignment-targets.ts` — invoke `get_staff_assignment_targets` through `invokeCommand`.
- `src/features/staff/api/save-staff-assignment-targets.ts` — invoke the typed save command with active save context token and complete draft.
- `src/features/staff/api/staff-assignment-targets-query-options.ts` — Query option keyed by the route context generation.
- `src/features/staff/api/staff-keys.ts` — add a narrow assignment-target key under the Staff root.
- `src/features/staff/components/staff-assignment-target-modal.tsx` — local draft, grouped numeric fields, validation, pending/error/success behavior, Modal focus/close rules, and save mutation.
- `src/features/staff/components/staff-assignment-target-modal.test.tsx` — accessible form, omitted Senior Manager, enabled-team names/order, 0/50 boundaries, exact complete request inspection, pending guard, Rust rejection display, save reset, and immutable-context replacement.
- `src/app/routes/my-club.tsx::MyClubStaffShortlistWorkspace` — add **Configure slots**; pass active save ID/token and current snapshot ID/token plus a context key built from both immutable tokens, managed club, enabled teams, and shortlist revision; keep the existing route and mounted workspace.
- `src/testing/snapshot-ipc-mock.ts`, `src/testing/staff-ipc-mock.ts`, and `src/testing/setup.ts` — add `contextToken` to current-snapshot fixtures and typed target read/save doubles with exact request inspection.
- `e2e/tauri-ipc-stub.ts` — add the immutable token to existing current-snapshot and Load Data summary fixtures so browser consumers share the production DTO; optimizer-specific command handling remains Commit 7.
- `src/app/routes/my-club-squad.test.tsx` — prove the control lives only in Staff Shortlist, uses configured team display names, persists an exact complete payload through the mock boundary, and resets when either immutable context token or another route context value changes, including the same numeric snapshot ID with a replacement token.

**Behavior and data flow:**

- The route supplies authoritative current presentation context, including the save ID/token and snapshot ID/token returned by current-snapshot IPC. The Staff target Query fetches Rust-expanded counts and team names. Opening the Modal creates one draft entry per returned allowed pair and does not infer omitted entries. Saving sends that exact complete target list plus the active save token; success invalidates the target key, closes with focus restoration, and clears any later recommendation outcome through the shared component boundary. Replacing a snapshot with the same numeric ID but a different immutable token creates a different Query/component context. The independent Load Data result type carries the same stored/effective snapshot tokens, and `LoadDataOutcome` uses token equality for its latest-snapshot message so numeric ID reuse cannot mislabel the outcome.

**Ordered implementation steps:**

1. Add RED snapshot and Load Data type/fixture tests for `contextToken`. In `load-data-outcome.test.tsx`, require different tokens on equal numeric IDs to produce the historical-snapshot message. Add Staff API/mock tests for exact command names, camelCase arguments, immutable-token key separation, exact complete request shape, and returned team/catalog shapes.
2. Add `contextToken` to `SnapshotSummary` and `LoadDataSnapshotSummary`, update `resolveBanner` to compare stored/effective tokens, and update the current-snapshot and Load Data fixtures plus the e2e stub before adding Staff UI plumbing.
3. Add RED accessible Modal tests for canonical grouping, no Senior Manager, 0 and 50 acceptance, invalid/empty/out-of-range rejection, one entry per allowed pair, pending lock, server error, and focus restoration.
4. Implement Staff types, fetchers, Query keys/options, and the minimum form component to turn those tests GREEN without inventing catalog pairs.
5. Add RED route tests for Shortlist-only placement, custom Planner display names, save-token replacement, snapshot-token replacement with forced same numeric ID, team/managed-club/shortlist context replacement, successful target refresh, and exact complete payload submission.
6. Compose the control in `MyClubStaffShortlistWorkspace` and refactor only while the focused Load Data, route, and component proofs stay green.

**Tests and proof:**

- RED: the Shortlist has no slot configuration control and the IPC mock rejects unknown commands.
- GREEN: Load Data outcome tests fail if equal numeric IDs override different immutable tokens. Component and route tests assert user-visible labels, semantic fields, 0/50 bounds, one submitted entry per currently allowed pair, pending/error/success phases, focus restoration, and immutable-token-safe replacement even when the numeric snapshot ID is reused.
- Add: dedicated Modal tests and assignment IPC mock fixtures.
- Modify: `LoadDataSnapshotSummary`, `LoadDataOutcome` fixtures/consumer tests, Snapshot summary types, current-snapshot and Load Data mocks/stubs, Staff keys/mock, and My Club route tests.
- Delete: none.
- Retain: Shortlist import, table virtualization, Preferred Job filters, Planner team management, and other My Club workspace tests because those contracts remain visible beside the new control.

**Patterns to verify:**

- `PlannerTeamManagement` form/confirmation/focus patterns, `StaffShortlistImportModal` context handling, `SnapshotMetadata.contextToken` precedent, `load-data-outcome.tsx::resolveBanner`, Staff Query options and keys, shared `Modal`, `TextField`, `Button`, and `my-club.tsx` route composition.

**Constraints and non-goals:**

- No raw invoke, cross-feature import from Staff to Planner, global store, URL state for the draft, frontend catalog invention, result allocation, or top-level route. Rust errors remain authoritative. Keep the Shortlist table's bounded-height layout intact.

**Dependencies and sequencing:**

- Depends on Commit 2's target commands and Commit 4's settled context contract. Commit 6 extends the same component boundary with recommendation actions/results.

**Validation:** `./scripts/dev test src/features/memory-read/components/load-data-outcome.test.tsx src/features/staff/components/staff-assignment-target-modal.test.tsx src/app/routes/my-club-squad.test.tsx` then `./scripts/dev check`

**Stop conditions:** Stop if the Rust DTO cannot supply the complete enabled-team/catalog display model, current-snapshot IPC lacks the immutable token, route composition cannot key by both immutable tokens without a global store, the Modal would need to invent or omit allowed pairs, or the Shortlist table loses its bounded scroll region.

**Review mandate:** Verify the independent `LoadDataSnapshotSummary` and `SnapshotSummary` contracts both carry `contextToken`; `LoadDataOutcome` and its fixtures/tests use token equality and reject equal-ID/different-token identity; the e2e Load Data/current-snapshot stub is updated; typed Staff invoke and Query ownership; save/snapshot immutable-token keys under numeric ID reuse; Shortlist-only integration and enabled display order; omitted Senior Manager plus 0-through-50 UX; exactly one submitted entry per allowed pair with Rust authority; pending/error/success and Modal keyboard/focus behavior; no result/business logic or cross-feature import; and focused user-observable tests plus updated current-snapshot and Load Data mocks/stubs.

#### Commit 6 — Present assignment recommendations

**Status:** Completed

**Provisional commit:** `feat(staff): present assignment recommendations`

**Work:** Add the on-demand optimize action, generation-safe result ownership, Rust-provided scope display labels, and accessible recommendation/vacancy evidence Panel.

**Size assessment:** About 400 changed non-test implementation lines. Exceeds the soft target because one coherent, trunk-safe recommendation outcome needs the Rust query/DTO label correction, typed token-guarded request/response, request-generation owner, setup/stale/error presentation, bounded semantic result table, and route reset wiring together. Splitting would leave either an incomplete optimizer DTO or a mutation that can display stale or raw-ID recommendations.

**Out of scope:**

- Recommendation persistence, manual overrides, dual duties, frontend ranking, new routes, general solver controls, and Playwright stub changes.

**Implementation packet:**

- Extend the settled Rust optimizer query/command result so every slot carries `scopeDisplayName`, using `Club` for Club and the current persisted display name for an enabled Planner team. Extend Staff assignment types/API and the Shortlist component with `optimize_staff_assignments(expectedSaveContextToken, expectedSnapshotContextToken)`. Bind each request to its starting context key and request generation; invalidate the generation on target save, route context-key replacement, and every immediate pending-context reset before clearing mutation/result state. Every completion must match that captured key and generation. Accept `ready`, `no_managed_club`, and `no_shortlist` only when both returned immutable tokens also match the current component context. Accept `no_current_snapshot` guidance only when the returned save token remains current and the response has no resolved snapshot token; accept `stale_context` guidance despite a changed returned save token and/or a changed or absent resolved snapshot token. Neither exceptional guidance state may present rows. Present Rust-provided scope labels, ordered ready rows, truthful setup/stale guidance, ScoreBadge values, Coach discipline, current/recruitment labels, vacancy evidence, and unsupported-job summary. `my-club.tsx` derives and passes `contextUnavailable` from its existing context query/mutation seams.

**Files and responsibilities:**

- `src-tauri/src/features/staff/assignment_optimizer_query.rs` — read the current enabled Planner team display names with the optimizer context and map each allocated slot to a bounded Rust result row whose `scope_display_name` is the matching persisted team name or `Club`; retain the pure allocator and its ordering unchanged.
- `src-tauri/src/features/staff/assignment_optimizer_query_tests.rs` — extend one focused current-state query fixture to prove a custom persisted team name and the fixed Club label reach their corresponding result slots.
- `src-tauri/src/features/staff/commands.rs::StaffAssignmentSlotDto` and the existing optimizer serialization test — carry `scopeDisplayName` on recommendation and vacancy rows and prove camelCase DTO serialization without moving the lookup into the command.
- `src/features/staff/types/staff-assignment.ts` — add explicit optimizer request/response context with save/snapshot IDs and immutable tokens plus state, recommendation, vacancy, `scopeDisplayName`, evidence, classification, and Coach discipline DTO types.
- `src/features/staff/api/optimize-staff-assignments.ts` — typed `invokeCommand` wrapper with expected save and snapshot immutable context tokens; numeric snapshot ID is not an authorization argument.
- `src/features/staff/components/staff-assignment-optimizer.tsx::StaffAssignmentOptimizer` — accept the route-owned `contextUnavailable` prop; own configure/optimize controls, request generation, starting-context capture, target/context invalidation, state-specific completion acceptance, setup/stale messages, and result composition; disable Optimize and render no prior recommendation whenever context is unavailable. Require matching returned save/snapshot tokens for `ready`, `no_managed_club`, and `no_shortlist`; require the current save token and absent resolved snapshot token for `no_current_snapshot`; allow `stale_context` guidance after the request key/generation check even when its returned save token changed and/or its resolved snapshot token changed or is absent. Render rows only for accepted `ready`.
- `src/features/staff/components/staff-assignment-results.tsx` — render each Rust-provided `scopeDisplayName` in the semantic ordered result table/Panel with accessible score names, explicit vacancies, classification, constraint explanation, unavailable counts, and unsupported Preferred Job note.
- `src/features/staff/components/staff-assignment-optimizer.test.tsx` — focused request/result, Rust custom-name presentation, explicit-state acceptance, pending, error, reset, evidence, score, Coach, accessibility, and immediate `contextUnavailable` suppression tests; use the already-planned state tests to prove `ready`, `no_managed_club`, and `no_shortlist` reject returned-token mismatch, `no_current_snapshot` accepts guidance only with the current save token and no snapshot token, and `stale_context` can show guidance but no rows after a current request key/generation despite a changed returned save token and/or a changed or absent resolved snapshot token. Strengthen the existing target-save reset case with the only added same-token delayed-completion proof; add no state matrix.
- `src/features/staff/components/staff-shortlist-import-modal.tsx::StaffShortlistImportModal` — extend the existing local `pending` seam with a narrow `onPendingChange` callback so the composing route can suppress recommendations as soon as shortlist replacement starts and clear the signal on every completion/context-reset path.
- `src/features/staff/components/staff-shortlist-import-modal.test.tsx` — prove `onPendingChange(true)` occurs before the deferred import resolves and returns to false on success, failure, cancellation, and context replacement without changing import ownership.
- `src/app/routes/my-club.tsx::MyClubPageContent` and `MyClubStaffShortlistWorkspace` — replace the target-only control with the complete optimizer component; increment the shortlist revision after import; include save token, snapshot ID and immutable token, managed club, Planner team signature, and revision in its key; derive `staffAssignmentContextUnavailable` from `isSnapshotRefreshing`, `isSavesRefreshing`, `isManagedClubRefreshing`, `isPlannerRefreshing`, `useIsMutating({ mutationKey: playerResultContextMutationKey })`, and the callback-fed shortlist-import pending state; pass it as `contextUnavailable`.
- `src/testing/staff-ipc-mock.ts` and `src/testing/setup.ts` — add optimizer state/result fixtures with resolved IDs/tokens, exact request inspection, delayed-result controls, and forced same-ID/replacement-token contexts.
- `src/testing/planner-ipc-mock.ts` — add the minimum deferred `save_planner_teams` resolver. Capture the validated request and complete it later through the mock's normal successful Planner depth update, including the submitted display-name change; do not add a second save path or broad scheduler controls.
- `src/app/routes/my-club-squad.test.tsx` — prove route-level immutable-context changes and Shortlist import suppress old results while filters do not affect the optimize request. Retarget the existing planned deferred route proof to render a recommendation, change a Planner display name or equivalent context-key input in the actual `PlannerTeamManagement` flow, start its deferred `save_planner_teams` command while old tokens and cached data remain present, assert immediate suppression and disabled Optimize, resolve the command successfully through the Planner IPC mock, wait for refreshed Planner data and the replaced assignment context key, and prove the obsolete recommendation does not return.
- Existing Shortlist results panel and table — retain unchanged except layout composition needed to fit the new Panel without breaking its scroll owner.

**Behavior and data flow:**

- Clicking Optimize captures the current request generation and route context key, then sends only the expected save and snapshot immutable context tokens. Rust reads the enabled Planner display names in the same current-state query and returns a bounded state/result whose slots include the applicable current name or `Club`, plus the resolved IDs and tokens. React shows pending text without changing control width. Every completion first requires the captured generation and key to remain current. `ready`, `no_managed_club`, and `no_shortlist` additionally require both returned tokens to match the mounted context because each state resolved the current snapshot. `no_current_snapshot` may show guidance only when its returned save token is current and its resolved snapshot token is absent. `stale_context` may show guidance when the captured request remains current even though Rust reports a changed save token and/or a changed or absent resolved snapshot token. Only accepted `ready` can render rows; setup and stale responses never render result slots or evidence as recommendations. React never uses numeric snapshot ID as freshness authority and never maps scope names, sorts, maps jobs, selects scores, classifies staff, or fills vacancies. Before a context change settles, `my-club.tsx` observes its existing save/snapshot/managed-club/Planner refetch booleans and `playerResultContextMutationKey`; the Shortlist Modal reports its existing local import-pending transition through `onPendingChange`. Their combined `contextUnavailable` disables Optimize, suppresses the result, and invalidates the request generation even though old tokens and cached data can remain present. Target save and context-key replacement do the same. `optimize.reset()` clears mutation state only; the generation/key check prevents any late callback from restoring the obsolete result. A successful deferred Planner save applies its normal depth update, changes the Planner signature, refreshes the route key, and cannot revive the previous recommendation. A deleted snapshot remains stale even if SQLite reuses its ID.

**Ordered implementation steps:**

1. Extend one current Rust optimizer query fixture as RED proof that a team slot receives the current persisted custom display name and a Club slot receives `Club`. Extend the existing command serialization proof for camelCase `scopeDisplayName` on slot DTOs.
2. Map allocator slots to the minimum query-owned bounded result-row type, add `scope_display_name`, and carry it through the command DTO without changing allocator rules or React mapping.
3. Add RED API and component tests for exact expected save/snapshot token arguments, each explicit Rust state, pending/error phases, the Rust-provided custom scope name, filled rows, vacancy rows, unavailable evidence, unsupported-job count, score labels, Coach discipline, and current/recruitment classification. In those already-planned state tests, require matching returned save/snapshot tokens for `ready`, `no_managed_club`, and `no_shortlist`; require the current save token and no resolved snapshot token for `no_current_snapshot`; and allow `stale_context` guidance, but no rows, after a current key/generation even when the returned save token changed and/or the resolved snapshot token changed or is absent. Add no state matrix.
4. Add RED component coverage proving `contextUnavailable` disables Optimize and suppresses an already-visible recommendation immediately. Strengthen the existing target-save reset case, rather than adding a case, so one optimize response stays delayed across a same-token target save and its later completion cannot restore the result. Add RED import-modal coverage for the existing local `pending` transition through `onPendingChange` while an import promise is deferred.
5. Add the typed optimize wrapper, minimum result Panel, Rust-provided scope-label rendering, state-specific request-generation/context-key/token guard, `contextUnavailable` prop handling, and narrow Shortlist import pending callback to make focused tests GREEN. Increment the generation before every target or context reset; do not treat `optimize.reset()` as cancellation. Ensure non-ready states cannot pass slot rows to presentation.
6. Extend `src/testing/planner-ipc-mock.ts` with one deferred `save_planner_teams` resolver that stores the validated request and, on explicit resolution, runs the same successful depth replacement used by the immediate path. Do not resolve to an unchanged synthetic depth or add another scenario-control layer.
7. Complete the existing route proof while also proving shortlist presentation filters do not enter the optimize request: render a recommendation, change one Planner display name or equivalent key input in the real Manage teams form, start the shared-keyed deferred save with old tokens and cached data present, assert immediate suppression and disabled Optimize, resolve the normal successful depth update, wait until refreshed Planner data replaces the assignment context key, and assert the old result never returns. Do not add a route scenario.
8. Wire the route key, state-specific response checks, shortlist revision, named refresh/mutation seams, and import-pending callback without effects that compute business rules or a global store. Retain the already-planned same-ID/new-snapshot-token rejection proof; add no same-token context matrix.
9. Verify semantic table headings, accessible ScoreBadge names, keyboard reach, focus after Modal/action completion, bounded Shortlist/result scrolling, and the 1,750-row contract.

**Tests and proof:**

- RED: the current Rust slot DTO has no display label, the result table renders raw scope IDs, and `optimize.reset()` lets an in-flight same-token completion call `onSuccess` after a target or context reset.
- GREEN: the focused Rust query/DTO proof fails if a persisted custom Planner name or fixed Club label is missing or serialized under the wrong field. The UI custom-name proof fails if React shows the raw scope ID or owns a mapping. The existing explicit-state tests fail if `ready`, `no_managed_club`, or `no_shortlist` accepts mismatched returned tokens; if `no_current_snapshot` accepts a changed save token or a present resolved snapshot token; if a current-request `stale_context` response cannot show guidance after a changed returned save token and/or a changed or absent resolved snapshot token; or if either exceptional guidance state renders rows. The strengthened single same-token target-save case fails if a delayed completion restores a result after generation invalidation. The completed Planner route proof fails if the real save mutation leaves the result visible or Optimize enabled while pending, if its deferred resolver bypasses the normal successful depth/display-name update, or if refreshed key data lets the old result return. Existing focused proofs still fail if React trusts numeric snapshot ID, misses the import pending transition, invents order, omits vacancy/unavailable evidence, displays zero for missing score, hides classification/Coach discipline, or sends filters.
- Add: optimizer/result component source and its focused test file plus the typed optimize API already owned by this packet.
- Modify: one Rust optimizer query fixture, the existing optimizer command serialization test, Staff assignment types/fixtures/mock, `src/testing/planner-ipc-mock.ts`, route tests, `StaffShortlistImportModal` tests, and the existing target-save reset case.
- Delete: none.
- Retain: pure allocator tests because scope display labels are query/presentation context rather than allocation rules; immutable-token delayed replacement proof; Shortlist table presentation/filters/import; Staff Profile navigation; target Modal tests; and Planner team tests because each protects a distinct supported contract. Do not add a same-token reset matrix, optional hardening, or duplicate Rust rules in frontend tests.

**Patterns to verify:**

- `assignment_targets.rs` team display-name read/order, `assignment_optimizer_query.rs` state-specific resolved-token shapes and current-context join, the existing optimizer command serialization test, context-generation guards in `StaffShortlistImportModal`, Planner optimize mutation feedback, `PlannerTeamManagement`'s existing save mutation and local `onPendingChange`, `resolveSavePlannerTeamsIpcMock`'s normal depth replacement, `SquadOverviewPanel` and `isSquadResultBlocked` context suppression, the route's exact `isSnapshotRefreshing`, `isSavesRefreshing`, `isManagedClubRefreshing`, `isPlannerRefreshing`, and `useIsMutating({ mutationKey: playerResultContextMutationKey })` seams, `ScoreBadge`, `Panel`, `EmptyState`, semantic Data Table rules, and route-owned invalidation/reset patterns in `my-club.tsx`.

**Constraints and non-goals:**

- React renders DTOs only. Do not duplicate scope-name, Preferred Job, score maximum, sorting, allocation, classification, or vacancy logic. Do not change the pure allocator for presentation context, persist results, put them in Zustand, or add cancellation/hardening beyond the required state-specific key/generation/token acceptance guard. Setup and stale states may show guidance only and must ignore any rows. Keep filters presentation-only. Use `—`, never `0`, for missing score. Preserve the existing table's route/profile behavior. Add only the one strengthened same-token delayed target-save proof and complete the existing Planner success proof; update the already-planned explicit-state tests rather than adding a matrix, reset matrix, extra route scenario, extra commit, or polish.

**Dependencies and sequencing:**

- Depends on Commit 4's optimizer command and Commit 5's typed target/configuration UI. Commit 7 adds assembled browser evidence and test-stub support only; it cannot change product rules or source.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-optimizer.test.tsx src/features/staff/components/staff-assignment-target-modal.test.tsx src/features/staff/components/staff-shortlist-import-modal.test.tsx src/app/routes/my-club-squad.test.tsx` then `./scripts/dev check-rust` then `./scripts/dev check`

**Stop conditions:** Stop if current persisted Planner display names cannot be read and attached in the existing Rust optimizer query; if the DTO lacks `scopeDisplayName`, the state-specific resolved-token shapes, or enough evidence to explain a recommendation/vacancy without frontend inference; if one request generation cannot invalidate every existing target/context reset; if state-specific acceptance cannot both reject mismatched current-context results and retain row-free `stale_context`/`no_current_snapshot` guidance; if the route observer cannot see and then complete `PlannerTeamManagement`'s save mutation through the mock's normal successful depth update; if that success cannot change a context-key input and refresh the route; if the Shortlist Modal's local pending state cannot report through one narrow callback; if forced numeric ID reuse or same-token reset can display an old delayed result; if 1,750 rows break the bounded workspace; or if correctness requires a new shared primitive, route, global store, commit, scenario, or broader test matrix.

**Review mandate:** Verify: (1) Rust query/DTO supplies each slot's current persisted team name or `Club`, with no React scope mapping; (2) every optimize request captures its starting context key/generation and every target/context reset invalidates that generation before reset; (3) `ready`, `no_managed_club`, and `no_shortlist` require matching returned save/snapshot tokens, while current-request `no_current_snapshot` and `stale_context` can show only coherent guidance under their recorded token rules and never rows; (4) route-derived `contextUnavailable` uses only the named refresh, shared Planner mutation, and narrow Shortlist pending seams; (5) the deferred Planner save changes a context-key input, suppresses immediately, resolves through the mock's normal successful depth update, refreshes context, and cannot restore the obsolete result; (6) filters remain excluded and all setup/stale/error states remain truthful; (7) semantic ordered rows preserve score/Coach/classification/vacancy/null/unsupported-job evidence within the 1,750-row bound; and (8) the already-planned focused Rust/UI proofs detect both reviewed defects without a new matrix, route scenario, optional hardening, extra commit, or product logic in Commit 7.

#### Commit 7 — Prove the browser assignment workflow

**Status:** Completed

**Provisional commit:** `test(staff): cover assignment optimization workflow`

**Work:** Extend the browser IPC stub and Playwright smoke suite for slot configuration, optimization, vacancies, and context replacement at supported viewport sizes.

**Size assessment:** No non-test implementation code. Test infrastructure and browser specs are excluded from the soft implementation target.

**Out of scope:**

- Product behavior changes, Rust/database proof, native file picking, native WebView-only claims, extra fixtures unrelated to the workflow, and implementation refactors to satisfy selectors.

**Implementation packet:**

- Add deterministic Staff assignment command handling to the existing Tauri browser stub and one focused smoke workflow that exercises the real route/components through accessible controls. Keep Rust allocation proof in Commit 4 tests; the stub returns a contract-shaped result rather than duplicating the allocator.

**Files and responsibilities:**

- `e2e/tauri-ipc-stub.ts` — support target get/save and optimize commands with small deterministic state/result fixtures, immutable save/snapshot token guards, returned context tokens, and Rust-contract `scopeDisplayName` on each slot, plus request capture only as needed by smoke. Reuse the current-snapshot token shape added in Commit 5; do not derive display names or duplicate allocator or replacement validation.
- `e2e/smoke.spec.ts` — configure team and Club counts, save, optimize, inspect filled/current/recruitment/Coach rows and vacancy/unavailable evidence, then change context and confirm results clear.
- Existing `src/testing/staff-ipc-mock.ts` and Vitest tests — retain; do not duplicate all component cases in Playwright.
- All product source under `src/` and `src-tauri/` — deliberately unchanged. This packet may edit only Playwright specs and test/stub infrastructure. A product defect stops this packet and returns to the exact owning Commit 2, 4, 5, or 6 packet for replanning, a new packet fingerprint, implementation, validation, and review.

**Behavior and data flow:**

- Playwright loads `/my-club?view=staff-shortlist` with the existing IPC stub, uses the visible Modal and buttons, and observes the semantic result table. The stub validates command shape and supplies deterministic Rust-contract DTOs, including the display label that product source renders without mapping; it does not derive that label or claim to prove SQLite or allocation.

**Ordered implementation steps:**

1. Add a RED smoke scenario against the already implemented route that fails because the browser stub lacks assignment command handling and no browser workflow assertion exists.
2. Add the minimum contract-shaped stub responses, including `scopeDisplayName` on each result slot, and target-save state needed to exercise the existing real UI. The stub carries the value only and implements no display-name lookup.
3. Assert grouped configured-team display names, omitted Senior Manager, 0/50 boundary presentation, persisted draft after reopen, pending-safe controls, filled and vacancy rows, score/Coach/classification evidence, and reset when the snapshot token changes even if its numeric ID does not.
4. Run the complete smoke suite at its existing viewport coverage and the full project gate.
5. Remove duplicate browser assertions already proved more strongly by Rust or Vitest tests. If an assertion exposes a product defect, stop without editing product source and return it to the exact owning packet/review.

**Tests and proof:**

- RED: the already implemented route reaches assignment commands that the browser stub rejects, and no Playwright scenario yet proves the assembled workflow.
- GREEN: smoke fails if accessible controls disappear, the Modal does not save, results do not render, vacancy/unavailable evidence disappears, or context replacement leaves an old result.
- Add/modify: one focused Playwright workflow and small stub fixtures.
- Delete: none unless a new assertion directly subsumes an assignment-specific duplicate introduced in this packet.
- Retain: existing Staff Shortlist filter/profile, My Club viewport, Planner team management, and navigation smoke because each protects a distinct supported workflow.

**Patterns to verify:**

- Existing `Staff Shortlist filters staff and adapts score columns` smoke, Planner `Manage teams` smoke, `stubTauriIpc` option-gated command handlers, and supported 1280×800/1600×900 layout checks.

**Constraints and non-goals:**

- Do not reimplement ranking in the stub, claim Rust coverage from Playwright, use brittle CSS selectors, or expand the global smoke fixture beyond the assignment contract. Prefer roles, names, and visible text.

**Dependencies and sequencing:**

- Depends on all prior implementation commits. It is the final planned test/stub-only commit; completion moves the feature to Validation before feature-level review and reconciliation.

**Validation:** `./scripts/dev smoke` then `./scripts/dev test` then `./scripts/dev check`

**Stop conditions:** Stop if smoke can pass without exercising real route components, the stub must reproduce allocator or exact-replacement logic, any product source change is needed, supported viewports cannot contain the result without a product redesign, or native-only behavior is being represented as browser-proved. A product defect returns to the exact owning packet and requires replanning, a new fingerprint, and a new worker/reviewer run before this test-only packet resumes.

**Review mandate:** Verify only test/spec/stub paths changed; the stub carries `scopeDisplayName` as Rust-provided result data and derives no scope label; immutable-token guards remain contract-shaped with no business-rule duplication; smoke uses accessible selectors; the workflow covers configuration through results and same-ID/new-token reset; viewport/table behavior is meaningful; no Rust claim relies on the stub; test overlap is proportionate; and any product defect stopped and returned to its exact owning packet rather than entering Commit 7.

## Discoveries and replanning

- No planned feature spec or existing active ledger exists, so planning creates one ledger and removes no promoted file.
- Current code already persists and exposes Manager's job-fit score even though older Shortlist history said Manager had no added score; current source and tests are authoritative, and this plan uses the persisted `manager` score.
- Preferred Job partitioning makes direct deterministic allocation sufficient. The Squad optimizer's exact matcher would add complexity without changing this feature's result.
- Planner-team removal already owns transactional string and assignment cleanup, so the same transaction is the narrowest coherent owner for confirmed removed-team staff targets.
- Correction round 1 applies the developer-approved confirm-and-delete policy, 50-slot limit, immutable snapshot-token freshness, exact complete replacement contract, accepted-fingerprint checkpoint rule, and test-only Commit 7 boundary. The Planner removal-impact UI scope and snapshot contract changes require a fresh complete plan review; keep the Delivery fingerprint placeholder until that review clears.
- Correction round 2 fixes only the four retained packet defects: it names the three shared-score mappings and treats Coach separately, preserves omitted/null score rows as unavailable, assigns the independent frontend Load Data token contract to Commit 5, and gives Commit 6 immediate pending-context suppression through current route and Shortlist import seams. Approved scope, architecture, one-PR order, packet order, and PR authority remain unchanged.
- Correction round 3 fixes the retained Planner-team pending-visibility defect without adding a commit or changing scope, architecture, packet order, or test depth. Commit 2 assigns the shared `playerResultContextMutationKey` to the existing `save_planner_teams` mutation while preserving local Planner pending behavior, and Commit 6 retargets its already-planned deferred route proof to that actual mutation.
- Bounded functional replan after Commit 6's initial review: review found two HIGH defects in the active packet. First, `optimize.reset()` clears mutation state but does not cancel callbacks, so a same-token target or context reset can be followed by an obsolete completion that restores recommendations. Commit 6 now owns one explicit starting-key/request-generation guard, invalidation on every target/context reset, one strengthened same-token delayed target-save proof, and completion of the already-planned deferred Planner success proof. Second, the completed Rust optimizer DTO exposes raw scope IDs and lacks current Planner display names. Commit 6 now minimally extends the Rust query/result DTO and focused Rust proof to return `scopeDisplayName` per slot, with one UI custom-name proof and no React mapping. Commit 7 only carries that corrected contract in its test stub. Completed statuses, refs/evidence, one-PR authority, feature scope, packet order, and Commit 7's test-only boundary remain unchanged. Fresh plan review, classification, fingerprint recording, and developer acceptance are required before implementation resumes.
- Correction round 1 resolves the two retained MEDIUM packet defects without changing scope, architecture, delivery order, test depth, or authority. Commit 6 now defines state-specific completion acceptance: every completion needs its captured key/generation; `ready`, `no_managed_club`, and `no_shortlist` also need matching returned save/snapshot tokens; `no_current_snapshot` may show row-free guidance only with the current save token and no resolved snapshot token; and `stale_context` may show row-free guidance despite a changed returned save token and/or a changed or absent resolved snapshot token. The packet also adds `src/testing/planner-ipc-mock.ts` and requires one minimal deferred team-save resolver that completes through the normal successful depth update. The existing route proof changes a Planner display name, resolves that save, waits for refreshed context, and verifies the obsolete recommendation stays absent. No state matrix or route scenario is added. Commit 7 remains test/stub-only, and the Delivery fingerprint stays pending until focused correction review clears.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Optimize staff assignments from the Shortlist | Commit 7 — Prove the browser assignment workflow | Pending record | Added a contract-shaped browser IPC fixture and one assembled Staff Shortlist configuration, recommendation, vacancy, viewport, and context-reset workflow. | Local and CI-serial smoke passed (50); full Vitest passed (651); `check` passed (Rust 661 passed, 2 ignored); e2e LSP and staged whitespace checks passed. | Pass | Clear | 1 | None |
| PR 1 — Optimize staff assignments from the Shortlist | Commit 6 — Present assignment recommendations | e5519ead8051b8ddfef69d8ae885b239c8534de6 | Added generation-safe token-bound optimization, Rust-provided scope display labels, pending-context suppression, and accessible recommendation/vacancy presentation. | Focused Vitest passed (148); `check-rust` passed (661 passed, 2 ignored); `check`, changed-path LSP, and staged whitespace checks passed. | Accepted gap — functional stale-result guards are implemented; developer declined further asynchronous test synchronization. | Accepted findings — reviewer confirmed functional fixes and retained only test synchronization. | 1 | Replanned Commit 6 DTO and request-generation scope after functional review; accepted revised Delivery fingerprint. |
| PR 1 — Optimize staff assignments from the Shortlist | Commit 5 — Configure assignment slots in My Club | 15c508931ba97482e6e962e121eda0c97501b85f | Added token-aware frontend snapshot contracts, typed target Query APIs, and an accessible context-safe Configure slots Modal in Staff Shortlist. | Focused Vitest passed (137 tests); `check`, TypeScript LSP, and staged whitespace checks passed. | Pass | Clear | 1 | None |
| PR 1 — Optimize staff assignments from the Shortlist | Commit 4 — Expose current-context recommendations | c85dde0702320744812f3a78472c18baf47effb1 | Added snapshot context-token summaries and a bounded token-guarded read-only optimizer query/IPC command over current Shortlist staff and persisted scores. | `check-rust` passed (661 passed, 2 ignored); `check`, Rust LSP, and staged whitespace checks passed. | Pass | Clear | 0 | None |
| PR 1 — Optimize staff assignments from the Shortlist | Commit 3 — Allocate canonical staff job groups | a70ec914fdd61e224a2fea7e387570fa1d3a0108 | Added the pure exact-label allocator with Coach discipline selection, deterministic one-duty allocation, vacancies, and bounded evidence. | `check-rust` passed (655 passed, 2 ignored); `check`, Rust LSP, and staged whitespace checks passed. | Accepted gap — production behavior is covered at representative seams; developer declined broader exhaustive mapping, tie-order, and multi-job ordering tests. | Accepted findings — reviewer found no functional defect; developer directed review to skip test-depth expansion. | 0 | None |
| PR 1 — Optimize staff assignments from the Shortlist | Commit 2 — Persist staff assignment targets | fd3de72af2c5d49e88cff83b86f49383afe7a84a | Added migration v35, token-bound exact target replacement, bounded team display data, combined Planner removal impact, and confirmed atomic cleanup. | Focused route tests passed (128); `check-rust` passed (648 passed, 2 ignored); smoke passed (49); `check` and staged whitespace checks passed. | Pass | Clear | 1 | None |
| PR 1 — Optimize staff assignments from the Shortlist | Commit 1 — Record the approved feature plan | 1e526f810b7a63a8b76c85c1a0839d22e9fabea4 | Recorded the reviewed schema-2 ledger and TODO activation on the authorized branch. | `ledger_state.py`, `delivery_state.py`, staged whitespace checks, and the pre-commit gate passed. | Not applicable | Accepted findings — developer directed reviews to fix functional issues only; retained one non-functional stale active-work sentence to preserve the accepted fingerprint. | 0 | None |

## Final validation

Run these automated commands after all implementation commits and before feature review:

1. `./scripts/dev test`
2. `./scripts/dev check`
3. `./scripts/dev smoke`

`./scripts/dev test` must discover the new Staff assignment component, Shortlist import pending-signal, Load Data outcome, and route Vitest cases. The frontend suite must include the single strengthened same-token delayed target-save proof; the already-planned explicit-state proofs for key/generation gating, current-context token matching, row-free `no_current_snapshot`, and row-free `stale_context`; Rust-provided custom scope-name presentation; equal-ID/different-token Load Data identity; same-ID/new-snapshot-token rejection; and the actual deferred Planner-team display-name save proof through immediate suppression, normal successful mock depth update, refreshed context, and no obsolete-result return. `./scripts/dev check` must discover and pass the Rust target, migration, optimizer-query, and DTO tests in addition to Rust formatting, Clippy, Biome, TypeScript, and secretlint. Those Rust tests must include exact missing/extra/duplicate replacement rejection, composite-key uniqueness, forced rollback, stale save token, forced snapshot-ID-reuse/token-mismatch, missing/null score unavailability, and current persisted team/Club `scopeDisplayName` proofs. `./scripts/dev smoke` must discover the Playwright assignment workflow and carry the corrected Rust-provided display-label contract, but it proves only the browser route against the IPC stub; it does not prove Rust, SQLite, or display-name lookup.

Perform one native Tauri/WebView manual pass because automation cannot prove native focus and assembled IPC behavior:

- Launch the documented desktop development app with `pnpm tauri dev`; there is no `./scripts/dev` wrapper for this native run.
- Use a representative current snapshot, configured managed club, enabled Planner teams with at least one custom display name, and a real imported Staff Shortlist.
- Configure targets, close and reopen the Modal, restart the app, and confirm counts persist for the same save.
- Optimize with current staff, recruitment candidates, a Coach with partial scores, a missing required score, an unrecognized Preferred Job, and at least one vacancy. Confirm names, classification, score, Coach discipline, and unavailable evidence agree with Staff Search/Profile values.
- Change the Shortlist presentation filters and confirm optimization still uses the whole current join.
- Give one Planner team two nonzero staffing targets and no assignments, try to disable it, and confirm **Manage teams** names each canonical job and slot count before any deletion. Cancel and confirm nothing changes; then confirm removal and verify its targets, strings, and team row disappear together while Club and remaining-team targets persist.
- Repeat with both assignments and staffing targets on one team. Confirm one removal dialog names the assignment count and every target job/count, then verify one confirmed transaction removes both sets of data. Re-enable the team and confirm its assignment structure is empty and every staffing target starts at zero.
- With a recommendation visible, start each supported save switch, Load Data or snapshot promotion, managed-club change, Planner-team change, and shortlist replacement. Confirm Optimize disables and the old result disappears while the operation is still pending; after each successful change settles, confirm the old result does not return before a new optimization.
- At 1280×800 and 1600×900, complete the workflow by keyboard, verify Modal focus trap/restoration, reach every result row, and confirm the Shortlist and recommendation regions keep bounded scrolling.
- Confirm no FM data changes after configuration or optimization.

If the native environment or representative save is unavailable, report those checks as validation gaps rather than passes. Mutation testing remains unsupported and must not be reported as passed.

## Documentation impact

Complete during reconciliation. Expected owners are `.wiki/ARCHITECTURE.md`, `.wiki/DESIGN.md`, `.wiki/TODO.md`, and the completed feature record only after implementation makes the behavior current. No ADR or debug report is currently warranted.
