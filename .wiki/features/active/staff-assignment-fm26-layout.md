# Staff Assignment FM26 Layout

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 7f1bcb357515da22c2c2f9c037341441ff3ba3453b528e4b9e7e834c38cf7de7

## Intent

Redesign Staff Shortlist assignment planning to follow FM26's Coaching, Recruitment, and Medical staff-limit layout. Keep enabled Planner squads, saved target ownership, and Rust-authoritative optimization while adding the FM26 coach composition, real lead-role candidate pools, Recruitment Analyst support, and a collapsible result panel.

## User-visible behavior

- **Configure slots** groups controls by enabled Planner squad and then by **Coaching**, **Recruitment**, and **Medical**.
- Senior shows its squad roles plus every club-wide role when Senior is enabled. If Senior is disabled, a standalone **Club** group shows the club-wide roles. Club-wide roles appear once and never repeat under Reserves or Youth.
- Coaching contains club-wide Head of Youth Development and Head Performance Analyst plus each enabled squad's Manager where allowed, Assistant Manager, Coaches, Set Piece Coach, and Performance Analyst. Senior never exposes Manager.
- Recruitment is club-wide and contains Director of Football, Chief Scout, Technical Director, Scout, Recruitment Analyst, and Loan Manager.
- Medical contains club-wide Head Physio and Head of Sports Science plus each enabled squad's Physio and Sports Scientist.
- Assistant Manager remains separate from Coaches. Doctor and Chief Doctor do not appear.
- Club-wide lead roles accept 0 or 1. Scout and Recruitment Analyst, and all squad-specific count roles, accept 0 through 50.
- Saving the redesigned catalog resets all targets from the prior catalog once during database migration. The first target read after upgrade returns the new allowed catalog with zero counts instead of a persisted-disallowed-row error.
- The shared Coaches count accepts Preferred Job Coach, Fitness Coach, and Goalkeeping Coach. It applies the exact JAY-44 composition at 0, 1, 2, 3, 8, 9, 10, 16, 17, and higher values.
- Every complete six-General allocation contains Attacking Technical, Attacking Tactical, Defending Technical, Defending Tactical, Possession Technical, and Possession Tactical. A partial General allocation first fills the maximum possible required slots, then chooses distinct disciplines and candidates that maximize the applicable total score. Missing required General-specialization, Fitness, or Goalkeeping candidates leave typed vacancies.
- Chief Scout uses Preferred Job Scout and the Scout score. Head Physio uses Preferred Job Physio and the Physio score. Head of Sports Science uses Preferred Job Sports Scientist and the Sports Scientist score.
- Head Performance Analyst uses Preferred Job Head Performance Analyst or Performance Analyst and ranks both by the Head Performance Analyst score. Only Preferred Job Performance Analyst can fill later ordinary Performance Analyst slots, using the Performance Analyst score.
- Recruitment Analyst uses Preferred Job Recruitment Analyst and the existing persisted Recruitment Analyst score.
- Lead roles allocate before their corresponding ordinary slots. A selected lead cannot appear again in an ordinary slot or any other assignment.
- Assignment results have an accessible panel-header chevron. Collapse hides only the body; it keeps the current result in memory and does not invoke optimization again.

## Invariants

- `staff_assignment_targets.scope = 'club'` remains the semantic persistence owner for all club-wide roles. The Senior or standalone Club placement in **Configure slots** is presentation only.
- The candidate pool remains the active save's Staff Shortlist joined by UID to current-snapshot staff. Current Shortlist table filters do not narrow it.
- Rust owns the closed target catalog, section metadata, per-target maximum, target validation, score selection, coach composition, specialization matching, lead priority, allocation, vacancies, evidence, and result order.
- The WebView does not calculate staff eligibility, coach composition, specialization choice, scores, or assignments.
- One UID fills at most one slot across the complete result.
- Canonical allocation priority remains Senior, Reserves, Youth, Club where the lead-before-ordinary rule does not require a club lead first.
- Missing or null persisted scores are unavailable. They never become zero.
- A required General specialization, Fitness Coach, or Goalkeeping Coach without an eligible candidate produces a vacancy. No other coach type substitutes for it.
- A complete General cycle covers all six distinct specializations. A partial cycle cannot use the same specialization twice and cannot use one candidate twice.
- For each enabled squad's General slots, matching first maximizes filled required slots, then total applicable specialization score, then chooses the lexicographically smallest complete assignment sequence ordered by General cycle, canonical discipline, and numeric UID. It considers all six disciplines for a partial cycle rather than selecting a fixed prefix.
- Head Performance Analyst consumes from the combined Head Performance Analyst and Performance Analyst lead pool before ordinary Performance Analyst allocation. A Head Performance Analyst-preferred candidate is never eligible for an ordinary Performance Analyst slot.
- Chief Scout, Head Physio, and Head of Sports Science consume from their corresponding ordinary Preferred Job pools before ordinary slots.
- Target replacement remains complete and transactional for the new allowed pair set. Missing, extra, duplicate, disabled-team, invalid-pair, and out-of-range entries reject the whole write before mutation.
- Save and snapshot immutable context tokens, the route context key, request generations, pending-context suppression, and current-result clearing rules remain unchanged.
- Optimization remains read-only advice. It does not persist results or change Football Manager, staff, scores, shortlist entries, or contracts.
- Collapsing a current result changes component-local presentation state only. Context or target replacement still clears an obsolete result under the existing contracts.

## Non-goals

- Preserve target counts saved under the pre-JAY-44 catalog.
- Add Doctor, Chief Doctor, dual duties, manual locks, persisted recommendations, recommendation history, or Football Manager writes.
- Merge Assistant Manager into the Coaches count.
- Let one coach type cover a vacancy assigned to another required type.
- Change the Staff Shortlist candidate-pool, current-snapshot, managed-club classification, shortlist filter, or context-token contracts.
- Change Planner team availability, display-name persistence, removal confirmation, or canonical team priority beyond consuming the redesigned target catalog.
- Add a route, navigation item, dependency, general-purpose cross-feature solver, frontend SQL, raw Tauri invoke, or global Zustand state.
- Update current-state ARCHITECTURE or DESIGN during planning.
- Prepare or publish a release. Release work requires a separate explicit workflow after delivery.

## Current-state map

- Relevant components: `src/features/staff/components/staff-assignment-target-modal.tsx::StaffAssignmentTargetModal` renders one fieldset per enabled team plus Club from the Rust target response. `staff-assignment-results.tsx::StaffAssignmentResults` renders a fixed open `Panel`. `staff-assignment-optimizer.tsx::StaffAssignmentOptimizer` owns the latest accepted optimization result and clears it only for target or context changes.
- Data model: migration v35 creates `staff_assignment_targets(save_id, scope, job_id, slot_count)` with the four scopes, a nonblank job ID, a stored 1-through-50 count, and `(save_id, scope, job_id)` as the primary key. The schema does not encode the allowed pair catalog.
- Persistence and migrations: `src-tauri/src/features/staff/assignment_targets.rs` owns the current 10 team jobs and 6 Club jobs, expands absent rows to zero, and rejects persisted pairs outside the active catalog. Because club-wide HPA and medical leads are currently team-scoped, changing the catalog without a reset would make prior rows fail the first read.
- Existing target behavior: Senior excludes Manager. Any nonempty subset of Senior, Reserves, and Youth can be enabled. Planner removal impact and cleanup read only nonzero targets for the removed team scope and preserve Club targets.
- Staff scoring: `src-tauri/src/features/staff/scoring.rs` already persists all 21 role scores, including `coach_fitness`, `coach_goalkeeping`, and `recruitment_analyst`. The optimizer query currently selects only 18 score IDs and omits those three.
- Existing optimizer assumption: `assignment_optimizer.rs` maps one exact Preferred Job to one target group, ranks each group by one score, and fills ordered slots directly. Preferred Job Coach uses the highest available of six General scores. This cannot implement lead fallback pools, lead-before-ordinary consumption, three coach types, or specialization matching.
- Candidate query: `assignment_optimizer_query.rs::load_candidates` joins current staff to the save-owned shortlist, preserves missing scores, classifies current staff by exact managed-club name, and returns a bounded pool. `canonical_job_id` also drives unsupported Preferred Job counts.
- Result ordering and limits: current allocation sorts Senior, Reserves, Youth, Club and enforces one global UID set. `assignment_optimizer_query.rs::MAX_STAFF_ASSIGNMENT_SLOTS` currently caps 1,750 result slots from the former 35-pair catalog. The exact redesigned maximum is 1,108: 20 enabled-team count pairs × 50, 2 Club count pairs × 50, and 8 Club lead pairs × 1.
- IPC: `src-tauri/src/features/staff/commands.rs` maps target and result DTOs to camelCase. `src-tauri/src/lib.rs` already registers get, save, and optimize commands. Frontend API modules use the sole `invokeCommand` wrapper.
- UI state: TanStack Query owns saved targets. `StaffAssignmentOptimizer` keeps the accepted result locally and already protects it with immutable tokens, a context key, and a request generation. A local expanded flag can hide the result body without changing result ownership.
- UI patterns: shared `Panel` supports a header `actions` slot. Shared `Button` and Lucide chevrons provide the current icon-button, focus, and `aria-expanded` analogues. The Modal, fieldset, semantic table, and ScoreBadge patterns remain applicable.
- Tests: Rust target and optimizer modules use migrated temporary SQLite databases and pure candidate fixtures. Component tests use `src/testing/staff-ipc-mock.ts`. `src/app/routes/my-club-squad.test.tsx` protects context suppression. `e2e/tauri-ipc-stub.ts` and the Staff Shortlist smoke scenario prove the browser-visible configuration and result workflow.
- Project validation commands: `./scripts/dev test`, `./scripts/dev check`, `./scripts/dev check-rust`, and `./scripts/dev smoke`.
- Delivery rules: `main` is the base. Publication uses GitHub and `.github/pull_request_template.md`. The merge method is squash. The strict required GitHub Actions status is `check`.
- Primary risks: leaving pre-redesign rows that fail target reads; persisting club roles under a team for presentation convenience; duplicating Club controls; accepting more than one lead; selecting a fixed partial specialization prefix; greedy General assignment that lowers the total score; allowing coach-type substitution; consuming an ordinary candidate before its lead; dropping Recruitment Analyst at the SQL or DTO seam; duplicating a UID; or collapsing by discarding and rerunning the result.

## Feature architecture

Migration v36 keeps the v35 table and deletes every existing `staff_assignment_targets` row once. This is the minimum compatible reset for the sole-user pre-release app. A fresh database creates v35 and then applies the empty reset. A populated v35 database retains saves and Planner-team settings but starts the redesigned target catalog at zero. The migration and target-catalog change land together, and a populated-upgrade test opens the migrated database through the ordinary migration path before calling the target read.

`assignment_targets.rs` remains the closed catalog owner. It returns ordered target metadata with `section` (`coaching`, `recruitment`, or `medical`) and `max_slot_count` so React can group and constrain controls without maintaining a second role catalog. The allowed pairs are:

| Persisted scope | Section | Target | Maximum |
| --- | --- | --- | --- |
| `club` | Coaching | Head of Youth Development | 1 |
| `club` | Coaching | Head Performance Analyst | 1 |
| `club` | Recruitment | Director of Football | 1 |
| `club` | Recruitment | Chief Scout | 1 |
| `club` | Recruitment | Technical Director | 1 |
| `club` | Recruitment | Scout | 50 |
| `club` | Recruitment | Recruitment Analyst | 50 |
| `club` | Recruitment | Loan Manager | 1 |
| `club` | Medical | Head Physio | 1 |
| `club` | Medical | Head of Sports Science | 1 |
| `senior` | Coaching | Assistant Manager | 50 |
| `senior` | Coaching | Coaches | 50 |
| `senior` | Coaching | Set Piece Coach | 50 |
| `senior` | Coaching | Performance Analyst | 50 |
| `senior` | Medical | Physio | 50 |
| `senior` | Medical | Sports Scientist | 50 |
| `reserves`, `youth` | Coaching | Manager | 50 |
| `reserves`, `youth` | Coaching | Assistant Manager | 50 |
| `reserves`, `youth` | Coaching | Coaches | 50 |
| `reserves`, `youth` | Coaching | Set Piece Coach | 50 |
| `reserves`, `youth` | Coaching | Performance Analyst | 50 |
| `reserves`, `youth` | Medical | Physio | 50 |
| `reserves`, `youth` | Medical | Sports Scientist | 50 |

Rust validates each submitted count against its target's returned maximum. It still requires one explicit value, including zero, for every allowed pair. The database continues to store only positive counts. The exact result maximum is 1,108: 20 team count pairs × 50 + 2 Club count pairs × 50 + 8 Club lead pairs × 1. Commit 2 changes `assignment_optimizer_query.rs::MAX_STAFF_ASSIGNMENT_SLOTS` and its legal-boundary fixture together. It also adds `recruitment_analyst` to the allocator's canonical target IDs and group indexing so every legal configured slot emits a vacancy before later candidate packets make Recruitment Analyst eligible and fillable. Planner team removal continues to inspect and delete only that team's nonzero targets; every club-wide target remains under `club` and therefore survives removal of any Planner team.

The target Modal derives one ordered presentation model from Rust's teams and targets. For each enabled team it renders Coaching, Recruitment when applicable, and Medical. If Senior exists, it inserts all `club` targets into the Senior presentation group by their Rust section while retaining `scope: 'club'` in every draft and save input. If Senior does not exist, it appends one standalone Club presentation group with the same sections. Reserves and Youth render only targets whose persisted scope matches that team. A target key remains `scope:jobId`, so moving a Club control in presentation cannot duplicate or retarget it.

`assignment_optimizer_query.rs` adds `coach_fitness`, `coach_goalkeeping`, and `recruitment_analyst` to the closed selected score list and score set. It does not recalculate them. Candidate recognition covers only the approved Preferred Job labels. Recruitment Analyst maps to its same-named target. Fitness Coach and Goalkeeping Coach enter only the Coaches family. Chief Scout, Head Physio, and Head of Sports Science labels no longer form separate candidate groups; the ordinary labels Scout, Physio, and Sports Scientist supply both their lead and ordinary slots. Head Performance Analyst and Performance Analyst both enter the HPA lead pool, while only Performance Analyst remains eligible for ordinary analysis slots.

The allocator uses explicit phases rather than the former one-label partition:

1. Allocate club-wide Head Performance Analyst, Chief Scout, Head Physio, and Head of Sports Science before their corresponding ordinary slots. Rank by the required lead score descending and numeric UID ascending. Reserve the selected UID globally.
2. Allocate ordinary Performance Analyst, Scout, Physio, and Sports Scientist slots from remaining candidates in Senior, Reserves, Youth, Club order where those scopes exist. Use each ordinary score and numeric UID ties.
3. Allocate the remaining exact one-target roles and Recruitment Analyst in canonical scope and slot order.
4. Allocate each enabled squad's Coaches count in Senior, Reserves, then Youth order while sharing the same reserved UID set.

The exact Coaches composition for count `n` is a pure function. Counts 0 through 8 follow JAY-44 directly. For `n >= 9`, start with 6 General, 1 Goalkeeping, and 1 Fitness at slot 8. Each complete next block of eight adds Goalkeeping, Fitness, then six General. A remainder applies that same order. This produces 9 = 6/2/1, 10 = 6/2/2, 16 = 12/2/2, and 17 = 12/3/2 for General/Goalkeeping/Fitness.

General candidates have Preferred Job Coach. Fitness candidates have Preferred Job Fitness Coach and require `coach_fitness`. Goalkeeping candidates have Preferred Job Goalkeeping Coach and require `coach_goalkeeping`. Fitness and Goalkeeping groups rank by score descending and UID ascending and emit vacancies when their required count exceeds eligible remaining candidates.

General allocation uses a staff-private exact cardinality-first matching routine. For `g` General slots, every discipline has a required quota of `floor(g / 6)`, and exactly `g % 6` distinct disciplines receive one additional slot. Candidate-to-discipline edges exist only for available persisted scores. The objective first maximizes filled required General slots, then maximizes total applicable score, then chooses one lexicographically smallest complete assignment sequence. A required General node has identity `(cycle_number, canonical_discipline_rank)`: full cycles contain all six ranks, the partial cycle selects distinct ranks from all six, and repeated discipline quotas are ordered by earlier cycle before later cycle. The tie sequence orders nodes by that identity and compares `(cycle_number, canonical_discipline_rank, numeric_uid_or_vacancy)` with a vacancy sentinel after every numeric UID. This also fixes which repeated node receives a candidate and which ordered node becomes a vacancy. The concrete cardinality regression has required A and B slots, candidate X with A=100/B=0, and candidate Y with A=99 only: the result must fill both slots as Y→A and X→B for total 99 instead of choosing only X→A for total 100. The helper stays inside Staff rather than importing Planner's player-specific 11-lane matcher or adding a dependency.

One closed Rust `CoachRequirement` field travels on both Coaches recommendations and Coaches vacancies. Its canonical wire values are `attacking_technical`, `attacking_tactical`, `defending_technical`, `defending_tactical`, `possession_technical`, `possession_tactical`, `fitness`, and `goalkeeping`; non-Coaches slots carry null under the existing tagged DTO style. For a General vacancy, `StaffAssignmentEvidence` counts only Preferred Job Coach candidates for that exact persisted discipline score. For Fitness and Goalkeeping vacancies, it counts only the exact Preferred Job pool and exact `coach_fitness` or `coach_goalkeeping` score. Joined, eligible, and unavailable counts remain bounded aggregates, and a missing score contributes unavailable rather than zero. `StaffAssignmentResults` renders the same typed requirement for a recommendation and a vacancy. It also puts a ghost icon button in `Panel.actions`, exposes `aria-expanded`, uses a stable `aria-controls` target, and gives the current action a clear accessible name. The component keeps the result prop mounted as the same accepted value; only its body is conditional. A new optimization starts expanded, while toggling does not call `optimizeStaffAssignments`.

## Uncertainty register

### Known

- Linear JAY-44 and the developer's eight resolved decisions define the approved behavior.
- No planned spec and no active ledger exist for this feature.
- Main is clean, and the current staff assignment optimizer is present at migration v35.
- The Linear issue has no comments, children, attachments, or linked documents beyond the image embedded in its description.
- The current table schema can store the redesigned catalog without alteration.
- The current review graph is stale and CodeGraph is unavailable. This plan uses direct source, tests, migration code, and repository documents as authority.
- `coach_fitness`, `coach_goalkeeping`, and `recruitment_analyst` already exist in the persisted 21-role score catalog.
- Shared `Panel.actions` and existing accessible icon-button patterns can host the collapse control without changing the shared Panel API.

### Assumptions

- JAY-44's role lists and the developer's explicit scope decision fully specify the screenshot's behavior. The embedded Linear upload returned HTTP 401 to the available fetch tool, so no independent pixel-level inspection was possible.
- Team-specific roles keep the existing 0-through-50 contract because the approved decision limits the new one-slot cap specifically to club-wide leads.
- Typed Coach requirement evidence uses one closed `CoachRequirement` value on both recommendation and vacancy DTO variants and keeps only bounded aggregate counts; it does not return the candidate pool.

### Decisions

- Use one PR. Persistence, Rust allocation, Configure slots, and result collapse form one Staff Shortlist workflow and no independently valuable trunk publication boundary justifies a second PR.
- Apply migration v36 as `DELETE FROM staff_assignment_targets` with no table rebuild. This is the smallest reset that guarantees startup and first open cannot expose old disallowed pairs.
- Keep every club-wide target persisted under `club`; change only the Modal's presentation placement.
- Return section and maximum metadata from Rust instead of copying the role catalog into React.
- Keep Assistant Manager and every team-specific target at 0 through 50. Cap the eight club-wide lead roles at 1; keep Scout and Recruitment Analyst at 0 through 50.
- Replace direct one-label partitioning with explicit lead, ordinary, exact-role, and Coaches phases under one global UID set.
- Use a bounded Staff-private cardinality-first exact matcher for General specialization quotas: maximum filled slots, then maximum total score, then the exact lexicographic required-slot/discipline/UID order. Do not reuse Planner's player-specific matcher and do not add a solver crate.
- Reset the result to expanded only when a newly accepted optimization result arrives. Collapse state does not own or invalidate result data.
- Do not create an ADR. The change evolves the existing Rust-owned target, migration, Query, and local-presentation seams without establishing a new durable architecture boundary.
- Do not change BACKLOG. No accepted deferred scope is added, removed, or reclassified.

### Unknowns

- Native Tauri/WebView numeric-control density and focus behavior in the redesigned Modal cannot be proved by jsdom or the browser IPC stub.
- The screenshot itself could not be fetched through the available authenticated read path. The issue text and developer-supplied exact catalog remove the product ambiguity, but pixel comparison remains a manual review gap.
- Mutation testing remains unsupported by the repository command surface.

### Risks

- A migration registered after the catalog change but without the reset could leave the first open blocked by old persisted pairs.
- Frontend presentation could rewrite Club scope to Senior or duplicate Club controls when Senior is enabled.
- Local maximum validation could diverge from Rust and allow a lead count above one.
- Lead and ordinary pool overlap could assign one UID twice or let an ordinary slot consume the best lead candidate.
- HPA fallback could rank Performance Analyst candidates by the wrong score or let HPA-preferred staff fill ordinary slots.
- A coach composition boundary or remainder formula could shift Fitness and Goalkeeping counts.
- A score-first, fixed-prefix, or greedy General matcher could leave a fillable required slot vacant or lower the partial-cycle total.
- A collapse implementation could clear the result, trigger optimization, lose accessible state, or break the bounded Staff Shortlist layout.

## Walking skeleton

Upgrade one populated v35 database to zero redesigned targets, return one club-wide Head Performance Analyst target under `club`, present it inside Senior's Coaching section, save it at count 1, allocate the best eligible Head Performance Analyst or Performance Analyst by the HPA score before one Senior Performance Analyst slot, and render the accepted result. The remaining packets extend this same path to the complete catalog, exact Coaches composition and matching, all UI groups, and collapse behavior.

## Delivery plan

### PR 1 — Redesign staff assignment slots for FM26

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/staff-assignment-fm26-layout

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** strict required GitHub Actions `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(staff): redesign assignment slots for FM26`

**Purpose:** Deliver the complete FM26 staff-target catalog, safe one-time reset, Rust-authoritative allocation rules, Configure slots layout, and collapsible results in one reviewable Staff Shortlist workflow.

**Depends on:** Current `main` at planning time; no planned feature, active ledger, external service, or prior PR.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(staff): plan FM26 assignment layout`

**Work:** Commit the independently reviewed schema-2 ledger and TODO activation before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, migrations, executable configuration, generated files, BACKLOG changes, current-state documents, ADRs, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed ledger and TODO change after exact branch and base verification.

**Files and responsibilities:**

- `.wiki/features/active/staff-assignment-fm26-layout.md` — approved JAY-44 intent, architecture, delivery authority, implementation packets, validation, and risks.
- `.wiki/TODO.md` — move **Staff Assignment FM26 Layout** to Active and link this ledger while retaining the gender item in Next.
- `.wiki/BACKLOG.md` — deliberately unchanged because accepted deferred scope does not change.
- `.wiki/features/planned/` — deliberately unchanged because no planned spec exists.
- `.wiki/decisions/` — deliberately unchanged because no decision meets the ADR threshold.

**Behavior and data flow:**

- Record one reviewed source of feature truth and its exact delivery sequence before the separately authorized feature branch receives implementation.

**Ordered implementation steps:**

1. Verify `feature/staff-assignment-fm26-layout` is based on `main` without changing Git state during planning.
2. Confirm the worktree contains only the reviewed ledger and TODO planning diff.
3. Run the ledger classifier with the pending fingerprint placeholder before independent review.
4. After independent review clears and the orchestrator records the exact fingerprint, rerun both classifiers and confirm no packet or authority field changed.
5. Under a later authorized delivery workflow, stage only the two reviewed planning paths and obtain the normal independent checkpoint review.

**Tests and proof:**

- Not applicable — independently reviewed planning documents only. The ledger classifier proves schema, lifecycle state, one Active commit, active-work pointers, authority fields, and complete packets. No test fixture, mock, snapshot, or helper changes in this commit.

**Patterns to verify:**

- `.wiki/features/active/README.md`, `.wiki/TODO.md`, `.wiki/features/completed/staff-assignment-optimizer.md`, and the current repository PR authority.

**Constraints and non-goals:**

- Do not alter approved intent, packet order, authority fields, implementation, tests, BACKLOG, current-state docs, ADRs, or Git state outside the separately authorized delivery workflow.

**Dependencies and sequencing:**

- Requires a clear independent plan review, developer acceptance, orchestrator-recorded Delivery fingerprint, classifier success, and exact branch activation before commit authority exists.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/staff-assignment-fm26-layout.md`

**Stop conditions:** Stop on an uncleared review, classifier error, unreviewed path, substantive post-review change, fingerprint mismatch, wrong branch/base, or any changed path other than the ledger and TODO.

**Review mandate:** Verify every approved JAY-44 decision is present; one PR has exact authority; every later packet is executable; the fingerprint remains pending until orchestrator recording; only the ledger and TODO change; BACKLOG, current-state docs, and ADRs remain untouched.

#### Commit 2 — Reset and redefine assignment targets

**Status:** Completed

**Provisional commit:** `feat(staff): redefine FM26 assignment targets`

**Work:** Add the one-time v36 reset and replace the Rust-owned target catalog with the exact FM26 role, scope, section, order, and maximum contract.

**Size assessment:** About 220 changed non-test implementation lines. Exceeds the soft target because migration, catalog, DTO metadata, exact production result bound, and minimum target-side allocator recognition form one atomic catalog boundary. Splitting them can expose old rows as disallowed, leave a stale safety constant, let UI limits diverge from Rust, or make the exact legal catalog fail to emit all 1,108 result rows.

**Out of scope:**

- Recruitment Analyst candidate score loading, Preferred Job classification, successful Recruitment Analyst allocation, lead/ordinary allocation changes, Modal visual grouping, result collapse, and browser result behavior.

**Implementation packet:**

- Keep the v35 table, delete all saved targets once in migration v36, then make target reads and complete replacement use the redesigned 30-pair catalog. Add Rust-provided section and maximum metadata, add only the `recruitment_analyst` canonical target/group recognition needed to emit configured vacancies, set the production result cap to the exact legal maximum of 1,108, preserve Planner removal cleanup by semantic scope, and update every catalog-dependent optimizer/query fixture in the same commit so it remains trunk-safe.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — add and register migration v36 with the target reset; update latest-version and monotonic-registry assertions; add populated-v35 upgrade proof that saves and Planner teams remain while every target row is cleared.
- `src-tauri/src/features/staff/assignment_targets.rs` — replace `TEAM_JOBS` and `CLUB_JOBS` with the exact catalog; add section and maximum metadata; keep club-wide roles under `club`; enforce per-target 0/1 or 0/50 bounds; retain exact-completeness, compaction, stored-row rejection, transaction rollback, enabled-team ordering, and save-token checks.
- `src-tauri/src/features/staff/commands.rs::StaffAssignmentTargetDto` and serialization tests — expose `section` and `maxSlotCount` in camelCase without changing command names or input ownership.
- `src-tauri/src/features/planner/teams_tests.rs` — update target-removal fixtures to the new team-only catalog and prove team removal preserves every Club target.
- `src-tauri/src/features/staff/assignment_optimizer.rs::CANONICAL_JOB_IDS`, `canonical_job_index`, and candidate-group sizing — add `recruitment_analyst` as a recognized target group so configured slots emit ordered vacancies; do not add its Preferred Job mapping, score field, or successful candidate allocation in this commit.
- `src-tauri/src/features/staff/assignment_optimizer_tests.rs` — add a focused empty-candidate proof that a configured Recruitment Analyst target emits every requested vacancy with zero joined, eligible, and unavailable evidence rather than being skipped.
- `src-tauri/src/features/staff/assignment_optimizer_query.rs::MAX_STAFF_ASSIGNMENT_SLOTS` — replace the former 1,750 cap with the exact new legal maximum 1,108 while retaining the same pre-allocation and post-allocation boundary errors.
- `src-tauri/src/features/staff/assignment_optimizer_query_tests.rs::caps_the_ready_result_at_the_supported_slot_limit` and its catalog fixtures — replace retired team-scoped HPA/medical lead rows with the exact 20 legal team count pairs, 2 legal Club count pairs, and 8 legal Club lead pairs; keep its shortlist fixture free of Recruitment Analyst candidates; assert `configured_slot_count == 1_108` and `slots.len() == 1_108`; retain the bounded-query contract before later candidate/allocation packets.
- Existing migration, target, Planner, optimizer, optimizer-query, and command test helpers — modify only where the catalog/version changes; delete obsolete assertions that expect team-scoped lead rows or the 1,750 former maximum; retain unrelated migration, query-boundary, and Planner behavior tests.

**Behavior and data flow:**

- Database open applies v36 before feature reads. A prior target set becomes empty. `get_staff_assignment_targets` then resolves enabled teams and returns every new allowed pair at zero with Rust section and maximum metadata. Save validates every pair and its own maximum before one transactional replacement. Planner team removal sees only team-scoped ordinary rows and cannot delete Club rows. The allocator recognizes `recruitment_analyst` as a target-side group and emits one truthful vacancy per configured slot when no eligible group members exist; candidate recognition and scores remain absent until Commit 3, and successful assignment remains owned by Commit 4. The optimizer query rejects any configured or produced result above 1,108 and still permits the exact legal maximum.

**Ordered implementation steps:**

1. Add a RED populated-v35 migration test with old team lead rows and Club rows; expect v36, zero target rows, retained save, and retained Planner teams.
2. Add RED target tests for the exact role/scope catalog with all teams, Senior disabled, lead maximum 1, ordinary maximum 50, Senior Manager rejection, complete replacement, and persisted invalid-row rejection.
3. Add RED Planner tests that removing a team reports/deletes its new ordinary targets while preserving Club targets.
4. Add a RED pure allocator test showing a configured Recruitment Analyst target with no candidates is currently skipped instead of emitting the requested vacancies.
5. Rewrite the optimizer-query maximum fixture with only new legal pairs, assert the exact 1,108 boundary, and make it RED against both the stale 1,750 constant and missing Recruitment Analyst target-group recognition.
6. Register the reset and implement the minimum catalog, metadata, validation, `recruitment_analyst` canonical target/group entry, and `MAX_STAFF_ASSIGNMENT_SLOTS = 1_108` changes to make all proofs GREEN.
7. Update command DTO serialization and remove only assertions for the retired catalog or former maximum.
8. Refactor catalog lookup only while focused Rust proofs stay green.

**Tests and proof:**

- RED: v35 rows survive today and can become disallowed; current target reads expose 35 old pairs without section or maximum metadata; every count currently accepts 50.
- GREEN: migration tests prove the one-time reset and retained non-target state. Target tests prove all 30 pairs, club semantic scope, no duplication, Senior-disabled validity, exact payload validation, 0/1 and 0/50 bounds, rollback, and a valid first read after upgrade. Planner tests prove only removed-team targets are affected. The pure allocator test proves target-side Recruitment Analyst slots become explicit vacancies without candidates. The optimizer-query boundary test proves the exact legal 1,108-slot result remains accepted, fully emitted, and bounded.
- Add/modify: migration v35-to-v36 fixture, target catalog/service tests, command DTO serialization test, Planner removal tests, `assignment_optimizer_tests.rs` target-recognition vacancy proof, and `assignment_optimizer_query_tests.rs` catalog/boundary fixtures.
- Delete/rewrite: assertions and fixtures whose supported contract is the old team-scoped HPA/medical leads, six-role Club catalog, skipped Recruitment Analyst target, or 1,750 former maximum.
- Retain: save-token, forced rollback, duplicate/missing/extra payload, cascade, query pre/post-allocation bound errors, unrelated candidate mapping/allocation tests, unrelated migration, assignment removal, and focus/browser tests because they protect surviving contracts.

**Patterns to verify:**

- `STAFF_ASSIGNMENT_TARGETS_SQL`, migration registry tests, `allowed_pairs_for_teams`, `validate_complete_inputs`, `expand_targets`, `planner_team_removal_impacts`, `assignment_optimizer::CANONICAL_JOB_IDS`, `canonical_job_index`, `assignment_optimizer_query::MAX_STAFF_ASSIGNMENT_SLOTS`, `caps_the_ready_result_at_the_supported_slot_limit`, and the completed optimizer's transactional replacement tests.

**Constraints and non-goals:**

- Do not rebuild the table, preserve old counts, infer role aliases, store zero rows, initialize Planner teams during read-only optimize, or persist Club roles under Senior. Do not recognize Preferred Job Recruitment Analyst, load its score, or emit a successful Recruitment Analyst recommendation yet; this commit owns only target-side group recognition and vacancies. The migration must not delete saves, Planner teams, shortlist rows, staff, or scores.

**Dependencies and sequencing:**

- Depends on Commit 1. Commit 3 adds Recruitment Analyst candidate recognition and score loading to the target group created here; Commit 4 makes that group successfully allocatable. Commits 5 and 6 consume the new catalog and DTO metadata.

**Validation:** `./scripts/dev check-rust` then `./scripts/dev check`

**Stop conditions:** Stop if a plain target-row reset cannot make startup and first read valid, if target-side `recruitment_analyst` recognition cannot emit vacancies without pulling candidate work forward, if the exact legal fixture cannot emit 1,108 rows, if the new catalog requires a table-shape change, or if Planner cleanup cannot preserve Club rows.

**Review mandate:** Verify migration/reset and exact catalog; Club semantic persistence and per-target bounds; minimum Recruitment Analyst target/group recognition only; explicit vacancy emission before candidate support; exact 1,108 production/query bound; catalog-valid fully emitted query fixture; transactional replacement and Planner removal isolation; and no candidate score/classification or successful Recruitment Analyst allocation pulled forward.

#### Commit 3 — Load FM26 assignment candidate scores

**Status:** Completed

**Provisional commit:** `feat(staff): load FM26 assignment candidates`

**Work:** Extend the bounded current-snapshot candidate projection with Fitness, Goalkeeping, and Recruitment Analyst scores and the approved Preferred Job eligibility labels.

**Size assessment:** About 100 changed non-test implementation lines. Within the soft target; this is a focused SQL-to-domain contract change.

**Out of scope:**

- Lead priority, coach composition, specialization matching, target Modal layout, and result collapse.

**Implementation packet:**

- Starting from Commit 2's vacancy-capable `recruitment_analyst` target group, select the three existing persisted role scores that the optimizer currently omits. Replace one-label-to-one-target candidate recognition with an eligibility classification that can populate exact roles, the four lead/ordinary families, and the three Coaches types without performing successful Recruitment Analyst or lead allocation yet.

**Files and responsibilities:**

- `src-tauri/src/features/staff/assignment_optimizer.rs` — replace `canonical_job_id` as the candidate-recognition seam with a closed Preferred Job classification for Manager, Assistant Manager, Coach, Fitness Coach, Goalkeeping Coach, Set Piece Coach, Head Performance Analyst, Performance Analyst, HOYD, DoF, Technical Director, Loan Manager, Scout, Recruitment Analyst, Physio, and Sports Scientist.
- `src-tauri/src/features/staff/assignment_optimizer_query.rs::SCORE_ROLE_IDS`, `StaffAssignmentScoreSet`, `load_candidates`, and `set_score` — select and decode `coach_fitness`, `coach_goalkeeping`, and `recruitment_analyst` while preserving null as unavailable and the bounded current Shortlist join.
- `src-tauri/src/features/staff/assignment_optimizer_query_tests.rs` — prove each added persisted score reaches the domain model, wrong-snapshot or non-shortlisted rows remain excluded, and unsupported Preferred Job count follows the new closed labels.
- `src-tauri/src/features/staff/assignment_optimizer_tests.rs` — replace the obsolete 16-label mapping table with the approved label classification and rejected near-match coverage.
- Existing current-snapshot and shortlist fixtures — retain and extend; delete only fixtures for retired Preferred Job Chief Scout, Head Physio, or Head of Sports Science eligibility.

**Behavior and data flow:**

- The existing query joins current staff, shortlist metadata, and only the closed score IDs. Each row populates one candidate score set. Candidate recognition reports approved exact pools without assigning a target. Missing role rows and null values remain `None`, and unsupported labels remain counted but stay visible in the Shortlist table.

**Ordered implementation steps:**

1. Add RED query tests for the three omitted role IDs and exact new Preferred Jobs.
2. Add RED classification tests that accept the approved labels and reject old literal lead labels for Chief Scout, Head Physio, and Head of Sports Science plus near matches.
3. Extend the selected score list and score decoder without changing scoring formulas or ingest.
4. Implement the closed candidate classification and use it for unsupported counts.
5. Run focused Rust tests and refactor only while null and shortlist-boundary proofs stay green.

**Tests and proof:**

- RED: current SQL cannot load the three scores and treats Fitness Coach, Goalkeeping Coach, and Recruitment Analyst as unsupported.
- GREEN: query tests fail if any score ID is omitted, null becomes zero, an out-of-snapshot or non-shortlisted candidate enters, or an unapproved Preferred Job is recognized.
- Add/modify: candidate classification and optimizer-query integration tests.
- Delete/rewrite: old mapping assertions that encode one Preferred Job per target or accept displayed lead labels outside JAY-44's real pools.
- Retain: stale-context, current/recruitment classification, no-shortlist, no-current-snapshot, name-null, and bounded-pool tests.

**Patterns to verify:**

- `staff::scoring::STAFF_ROLES`, `assignment_optimizer_query::load_candidates`, `set_score`, current `SCORE_ROLE_IDS` parameterization, and Shortlist exact Preferred Job matching.

**Constraints and non-goals:**

- Do not recalculate scores, change ingest, use Club Job, apply Shortlist presentation filters, coerce missing values, add aliases, or widen the IPC pool.

**Dependencies and sequencing:**

- Depends on Commit 2's catalog and target-side Recruitment Analyst group recognition. Commit 3 adds only candidate classification and scores; Commits 4 and 5 consume them for allocation.

**Validation:** `./scripts/dev check-rust` then `./scripts/dev check`

**Stop conditions:** Stop if any required score is not already persisted, if a Preferred Job label remains product-ambiguous, if query parameter bounds no longer hold, or if score loading requires a scoring-model or ingest change.

**Review mandate:** Verify all three score IDs; exact approved labels and rejected old lead labels; current Shortlist/current-snapshot join; missing-score preservation; unsupported count; no scoring duplication; SQL parameter coverage; and regression-value of the updated fixtures.

#### Commit 4 — Allocate leads before ordinary roles

**Status:** Completed

**Provisional commit:** `feat(staff): prioritize assignment lead roles`

**Work:** Replace direct one-group allocation for non-Coaches targets with explicit lead-first and ordinary phases, including Recruitment Analyst and HPA fallback.

**Size assessment:** About 190 changed non-test implementation lines. Within the soft target; this isolates the cross-target candidate competition from the separate Coaches matcher.

**Out of scope:**

- Coaches composition and specialization matching, Configure slots presentation, result collapse, and persistence changes.

**Implementation packet:**

- Allocate the four lead/ordinary families before corresponding ordinary slots under one global UID set. Keep direct exact-role allocation for the remaining jobs and activate successful Recruitment Analyst allocation from Commit 2's target group and Commit 3's candidate score/classification.

**Files and responsibilities:**

- `src-tauri/src/features/staff/assignment_optimizer.rs::allocate_staff_assignments` and private helpers — introduce lead-family candidate views, lead score ranking, ordinary residual allocation, exact-role allocation, canonical scope/slot ordering, evidence, and the shared assigned-UID set.
- `src-tauri/src/features/staff/assignment_optimizer_tests.rs` — add focused lead fallback, score-source, allocation order, deterministic tie, missing-score, vacancy, and uniqueness cases; update old shared-score tests.
- `src-tauri/src/features/staff/assignment_optimizer_query_tests.rs` — prove an end-to-end ready result for Recruitment Analyst and each lead family from persisted scores.
- `src-tauri/src/features/staff/commands.rs` serialization tests — modify only if evidence or recommendation mapping needs a bounded field; retain command names and context tokens.
- Existing Coach tests — deliberately retain until Commit 5 replaces the old generic Coach behavior in the same pure module.

**Behavior and data flow:**

- The allocator reserves the highest HPA-score candidate from HPA or Performance Analyst for the club lead, then fills ordinary Performance Analyst slots only from remaining Performance Analyst candidates using their ordinary score. It applies the same lead-first reservation to Scout, Physio, and Sports Scientist pools for Chief Scout, Head Physio, and Head of Sports Science. Other exact roles, including Recruitment Analyst, allocate by required score in canonical scope and slot order. Every reservation shares one UID set.

**Ordered implementation steps:**

1. Add RED pure tests where allocating ordinary first would steal the best lead candidate.
2. Add RED HPA tests for both Preferred Jobs, HPA score ranking, PA-only ordinary eligibility, missing lead versus ordinary scores, and deterministic UID ties.
3. Add RED Scout, Physio, and Sports Scientist lead/ordinary tests plus Recruitment Analyst success/vacancy tests.
4. Implement lead-family phases and exact-role residual allocation with one global UID set.
5. Update query integration fixtures to prove persisted rows reach the correct result.
6. Refactor only while all focused allocation proofs stay green.

**Tests and proof:**

- RED: the current one-label partition cannot use ordinary labels for leads, cannot combine the HPA pool, and can allocate ordinary slots before leads.
- GREEN: pure and query tests fail if the wrong score is used, an HPA-preferred candidate fills ordinary PA, a lead UID repeats, missing becomes zero, Club lead priority is lost, or Recruitment Analyst remains unsupported.
- Add/modify: pure lead-family tests, query ready-result fixtures, and any bounded DTO assertion required by evidence.
- Delete/rewrite: tests that require Preferred Job Chief Scout, Head Physio, or Head of Sports Science to fill their displayed lead.
- Retain: exact-role Manager/Assistant Manager/Set Piece/HOYD/director/loan tests, canonical scope ordering, 50-slot boundary, classification, and context tests.

**Patterns to verify:**

- Current `allocate_staff_assignments`, `canonical_scope_rank`, `score_for_job`, stable score-descending/UID-ascending sort, and the existing global `assigned_uids` guard.

**Constraints and non-goals:**

- Do not let HPA candidates fill ordinary PA, let ordinary slots precede leads, add cross-job substitutions beyond the four approved families, change current-staff classification, or introduce a general solver for direct roles.

**Dependencies and sequencing:**

- Depends on Commit 3's candidate classes and score set. Commit 5 replaces only the Coaches portion while preserving these phases.

**Validation:** `./scripts/dev check-rust` then `./scripts/dev check`

**Stop conditions:** Stop if a lead pool needs another Preferred Job or score, if lead priority conflicts with an approved canonical ordering case, if one UID set cannot cover all phases, or if result evidence requires an unbounded candidate payload.

**Review mandate:** Verify lead-before-ordinary execution; exact candidate pools; HPA versus PA score use; PA-only ordinary eligibility; Recruitment Analyst; missing-score vacancies with deterministic ties; global uniqueness with canonical residual ordering; and tests that fail for ordinary-first allocation.

#### Commit 5 — Match the FM26 Coaches composition

**Status:** Active

**Provisional commit:** `feat(staff): match FM26 coach composition`

**Work:** Implement exact Coaches composition, cardinality-first General specialization matching, and typed requirement evidence for every Coaches recommendation and vacancy.

**Size assessment:** About 350 changed non-test implementation lines. Exceeds the soft target because the composition function, specialization quotas, cardinality-first matcher, global reservation integration, closed Rust/IPC/TypeScript requirement contract, and recommendation/vacancy rendering are one correctness unit. Splitting this packet would either expose Coaches rows without truthful typed requirements or land a matcher whose vacancy semantics cannot be reviewed end to end.

**Out of scope:**

- Target persistence, Modal layout, result collapse, new dependencies, and Planner optimizer refactoring.

**Implementation packet:**

- Replace highest-of-six generic Coach ranking with three exact Preferred Job pools and one staff-private cardinality-first exact matcher. Carry one closed Coach requirement through Rust allocation, vacancy evidence, command serialization, frontend types, rendering, mocks, and browser fixtures. Preserve lead/exact allocations and the global UID set from Commit 4.

**Files and responsibilities:**

- `src-tauri/src/features/staff/assignment_optimizer.rs::CoachRequirement`, `StaffAssignmentRecommendation`, `StaffAssignmentVacancy`, `StaffAssignmentEvidence`, `allocate_staff_assignments`, and private Coach helpers — own the eight-value closed requirement enum, pure count-to-composition function, General quota identities, cardinality-first matching objective, Fitness/Goalkeeping ranking, typed recommendation/vacancy evidence, and canonical squad integration.
- `src-tauri/src/features/staff/assignment_optimizer_tests.rs` — cover 0/1/2/3/8/9/10/16/17 and a 17+ remainder; six disciplines in every full quota; partial best-subset selection; maximum-cardinality before score; the X(A=100/B=0), Y(A=99 only) regression requiring Y→A and X→B; repeated-quota node and vacancy ordering; deterministic equal-score ties; all-null and partial-null scores; exact typed evidence counts; missing General/Fitness/GK vacancies; no substitution; cross-squad priority; and global one-UID uniqueness.
- `src-tauri/src/features/staff/assignment_optimizer_query_tests.rs` — prove persisted Fitness and Goalkeeping scores fill their typed requirements, each missing exact score remains a vacancy with pool-specific aggregate evidence, and the bounded result does not include candidate rows.
- `src-tauri/src/features/staff/commands.rs::StaffAssignmentEvidenceDto`, `StaffAssignmentSlotDto`, the replacement for `coach_discipline_name`, and DTO serialization tests — map the exact typed vacancy aggregates, serialize `coachRequirement` on both tagged slot variants with one of the eight canonical wire values, serialize null for non-Coaches, and remove the old recommendation-only `coachDiscipline` contract.
- `src/features/staff/types/staff-assignment.ts::CoachRequirement`, `StaffAssignmentRecommendation`, and `StaffAssignmentVacancy` — define the eight-value string union and `coachRequirement: CoachRequirement | null` on both slot variants.
- `src/features/staff/components/staff-assignment-results.tsx::StaffAssignmentResults` and `evidenceText` — render the canonical requirement label for both a Coaches recommendation and a Coaches vacancy; render exact bounded vacancy counts without deriving eligibility in React.
- `src/features/staff/components/staff-assignment-optimizer.test.tsx` — prove General, Fitness, and Goalkeeping recommendation/vacancy labels, null on non-Coaches, exact vacancy evidence text, and retained ScoreBadge/result behavior.
- `src/testing/staff-ipc-mock.ts` — update the shared optimization DTO fixtures to the closed `coachRequirement` field on both variants and remove `coachDiscipline`.
- `e2e/tauri-ipc-stub.ts` and the Staff Shortlist assignment scenario in `e2e/smoke.spec.ts` — return and display typed requirements for at least one recommendation and one vacancy while retaining bounded layout and current-snapshot clearing coverage.
- Existing Planner matcher — deliberately unchanged; use it only as an algorithmic analogue, not a cross-feature import.

**Behavior and data flow:**

- For each enabled squad in canonical order, Rust derives General, Fitness, and Goalkeeping counts. Fitness and Goalkeeping consume only their exact Preferred Job pool and score. General builds ordered `(cycle_number, canonical_discipline_rank)` requirement nodes and candidate edges from available six-role scores. It first maximizes filled nodes, then total score, then the exact lexicographic node/discipline/UID sequence. Repeated discipline nodes use cycle order; a vacancy sorts after numeric UIDs and remains attached to its ordered requirement node. Every recommendation and vacancy carries the same typed requirement. Selected UIDs join the global reserved set before the next squad.
- General vacancy evidence counts only Preferred Job Coach candidates for that requirement's persisted discipline score. Fitness and Goalkeeping vacancy evidence counts only their exact Preferred Job pool and exact persisted score. Joined candidates with a missing score increment unavailable, candidates with a score increment eligible, and no count is derived in React or expanded into an unbounded payload.

**Ordered implementation steps:**

1. Add RED table tests for every required count boundary and type count.
2. Add RED full-cycle and partial-cycle tests, including fixed-prefix and greedy lower-total fixtures plus the X(A=100/B=0), Y(A=99 only) case that must prefer two filled slots totaling 99 over one slot totaling 100.
3. Add RED tests for repeated discipline quota identity, lexicographic UID ties, deterministic vacancy placement, missing candidates, cross-squad priority, and global UID uniqueness.
4. Add RED pure/query tests for the eight typed requirements and exact pool-specific eligible/unavailable vacancy counts.
5. Implement the pure composition function and the smallest Staff-private cardinality-first matcher with explicit score and lexicographic tie phases.
6. Integrate all three coach types, `CoachRequirement`, and typed recommendation/vacancy evidence into the allocator.
7. Extend command DTO serialization, frontend slot types, `StaffAssignmentResults`, and shared IPC fixtures; remove the recommendation-only `coachDiscipline` contract.
8. Update focused component and browser proofs for recommendation and vacancy requirement rendering.
9. Run query integration tests for persisted Fitness/GK scores and refactor only while all focused Rust, frontend, and smoke proofs stay green.

**Tests and proof:**

- RED: current behavior accepts only Preferred Job Coach, chooses one highest discipline per person, and has no type composition or exact specialization coverage.
- GREEN: tests fail for any boundary count error, missing full-cycle discipline, score-first cardinality loss, fixed partial prefix, greedy lower-total assignment, unstable repeated-node/vacancy order, wrong typed requirement, wrong evidence pool, type substitution, null coercion, cross-squad priority change, or duplicate UID. The X/Y regression must return Y→A and X→B.
- Add/modify: pure composition/matching fixtures, query integration cases, command DTO serialization, frontend slot types, optimizer component result assertions, shared Staff IPC mock, e2e stub, and the existing Staff Shortlist smoke scenario.
- Delete/rewrite: the old highest-of-six Coach test and the recommendation-only `coachDiscipline` field/assertions/fixtures.
- Retain: lead allocation, exact-role allocation, context/query join, classification, 1,108 result bound, ScoreBadge behavior, context clearing, and unrelated staff scoring formula tests.

**Patterns to verify:**

- `planner::optimizer::MatchObjective` and `MatchGraph` as a local flow analogue while deliberately reversing its score/cardinality priority for this contract; current `highest_coaching_score` null semantics; six persisted General score IDs; tagged `StaffAssignmentSlotDto`; TypeScript slot unions; and existing result evidence rendering.

**Constraints and non-goals:**

- Do not import Planner-private types, add a crate, substitute coach types, use a fixed partial specialization order, recompute scores, assign one candidate twice, or optimize across later squads at the expense of canonical squad priority.

**Dependencies and sequencing:**

- Depends on Commits 3 and 4. Commit 6 presents the resulting target catalog; Commit 7 changes only result visibility.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-optimizer.test.tsx` then `./scripts/dev check-rust` then `./scripts/dev smoke` then `./scripts/dev check`

**Stop conditions:** Stop if maximum cardinality cannot precede score in a bounded deterministic matcher, if the exact lexicographic node/discipline/UID order cannot decide repeated nodes and vacancies, if typed Coach requirement evidence cannot traverse both slot variants, if a composition boundary conflicts with JAY-44, or if candidate volume makes the chosen bounded algorithm unsuitable.

**Review mandate:** Verify every boundary composition; cardinality before score with the X/Y regression; full and partial discipline quotas; exact repeated-node/UID/vacancy order; typed requirement on both slot variants; exact pool-specific missing-score evidence; canonical squad order with global uniqueness; and no new dependency or cross-feature solver coupling.

#### Commit 6 — Group Configure slots by FM26 section

**Status:** Pending

**Provisional commit:** `feat(staff): group FM26 assignment controls`

**Work:** Render Rust-provided Coaching, Recruitment, and Medical controls within enabled squad groups, with Club roles inside Senior or one standalone Club group.

**Size assessment:** About 170 changed non-test implementation lines. Within the soft target; target presentation, typed metadata, mock/stub fixtures, and its browser proof are one coherent UI outcome.

**Out of scope:**

- Rust allocation rules, result collapse, route structure, new shared components, and persisted scope changes.

**Implementation packet:**

- Extend frontend target types with Rust section and maximum metadata. Derive presentation groups without altering draft scope/job keys, render semantic nested fieldsets, and validate each input against its Rust maximum.

**Files and responsibilities:**

- `src/features/staff/types/staff-assignment.ts` — add the closed section type and `maxSlotCount` to target DTOs without changing save input shape.
- `src/features/staff/components/staff-assignment-target-modal.tsx::StaffAssignmentTargetModal` — derive enabled team presentation, insert Club targets into Senior or standalone Club, nest section fieldsets, keep exact labels and keys, use per-target maxima, and preserve save, pending, context reset, error, focus, and complete-payload behavior.
- `src/features/staff/components/staff-assignment-target-modal.test.tsx` — prove exact group/section placement with Senior on and off, one Club-role occurrence, absence of Doctor/Chief Doctor, Assistant Manager separation, Recruitment Analyst, Senior Manager absence, per-target max, full payload scope preservation, validation, pending lock, context discard, and trigger focus restoration.
- `src/testing/staff-ipc-mock.ts` — update the shared target catalog fixture with section/max metadata and the exact 30 allowed pairs.
- `src/features/staff/api/staff-assignment-targets-api.test.ts` — retain command/key proof and update the typed fixture only as needed.
- `src/app/routes/my-club-squad.test.tsx` — retain existing context and mounted-workspace tests; modify only shared-fixture assertions needed to prove Senior-disabled standalone Club through the route seam.
- `e2e/tauri-ipc-stub.ts` — update the browser target catalog and save behavior while preserving semantic `club` inputs.
- `e2e/smoke.spec.ts` Staff Shortlist assignment scenario — prove sections, Club placement inside First Team, lead max 1, ordinary max 50, Recruitment Analyst, no duplicate Club group, save/reopen values, and supported 1280×800 containment.
- Existing API, optimizer, route, and smoke helpers — retain unless the exact catalog makes an assertion obsolete.

**Behavior and data flow:**

- The target Query returns Rust metadata. The Modal creates presentation-only team/section groups. A Club target stays one draft keyed by `club:jobId` even when displayed below Senior. Save sends every exact Rust-provided pair once. Rust remains authoritative if frontend validation is bypassed.

**Ordered implementation steps:**

1. Add RED component tests for Senior-enabled nesting, Senior-disabled standalone Club, section labels, exact role visibility, one Club role occurrence, and target maxima.
2. Add a RED save assertion that a Club control shown under Senior still submits `scope: 'club'` and that all 30 pairs are present once.
3. Extend types and shared IPC fixture from the Rust contract.
4. Implement the minimum presentation derivation and nested semantic fieldsets.
5. Update browser stub and smoke scenario for the new layout, save/reopen, bounds, and containment.
6. Refactor only while focused component, route, and smoke proofs stay green.

**Tests and proof:**

- RED: the current Modal renders flat team fields and a separate Club group, repeats former team lead roles, has no section metadata, and hardcodes max 50.
- GREEN: component and smoke tests fail if Club duplicates, semantic scope changes, Senior-disabled Club disappears, a role enters the wrong section, lead max exceeds 1, Recruitment Analyst is absent, Doctor appears, or the complete payload changes.
- Add/modify: target Modal component tests, target types, shared Staff IPC fixture, optional route assertion, browser stub, and existing smoke scenario.
- Delete/rewrite: assertions for 35 controls, flat Club placement with Senior enabled, team-scoped lead controls, and universal max 50.
- Retain: API command/token tests, Modal pending/error/context/focus tests, optimizer context tests, Shortlist table behavior, and unrelated smoke workflows.

**Patterns to verify:**

- Existing `targetGroups`, `draftKey`, `draftFromTargets`, Modal nested fieldsets elsewhere in the app, shared icon/input focus classes, and the Staff Shortlist smoke containment loop.

**Constraints and non-goals:**

- Do not map Club scope to Senior in data, copy the role catalog into React, add controls not returned by Rust, omit zero entries, change Query ownership, or add responsive/mobile layouts outside the desktop contract.

**Dependencies and sequencing:**

- Depends on Commit 2's metadata contract. It can follow allocator commits without changing their behavior.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-target-modal.test.tsx src/features/staff/api/staff-assignment-targets-api.test.ts src/app/routes/my-club-squad.test.tsx` then `./scripts/dev smoke` then `./scripts/dev check`

**Stop conditions:** Stop if Rust metadata cannot express the complete grouping, if Senior-disabled placement needs a persisted scope change, if one draft key cannot support presentation relocation, or if the supported desktop Modal cannot contain the catalog accessibly.

**Review mandate:** Verify exact sections and roles; Senior/standalone Club placement with no duplication or scope rewrite; Assistant Manager and Manager rules; Recruitment Analyst and Doctor exclusions; per-target bounds; full payload; context, pending, and focus behavior; and browser containment.

#### Commit 7 — Collapse assignment results without rerun

**Status:** Pending

**Provisional commit:** `feat(staff): collapse assignment results`

**Work:** Add the accessible result-panel chevron while keeping the current accepted optimization result in memory.

**Size assessment:** About 60 changed non-test implementation lines. Within the soft target; this is one local interaction change.

**Out of scope:**

- Allocation, target configuration, result persistence, shared Panel changes, route state, and animation.

**Implementation packet:**

- Give `StaffAssignmentResults` local expanded state, render a ghost icon button through `Panel.actions`, and hide only the body. Ensure a new result expands while context and target resets continue to clear obsolete results through `StaffAssignmentOptimizer`.

**Files and responsibilities:**

- `src/features/staff/components/staff-assignment-results.tsx::StaffAssignmentResults` — add stable body ID, local expanded state keyed to the accepted result, accessible chevron action, `aria-expanded`, `aria-controls`, and conditional body without changing table content.
- `src/features/staff/components/staff-assignment-optimizer.test.tsx` — prove initial expansion, collapse/expand labels and state, hidden/visible body, no additional optimize IPC call, retained result after collapse, new-result re-expansion, and existing context/target clearing.
- `src/testing/staff-ipc-mock.ts` — add or retain optimizer invocation-count inspection needed to prove no rerun; do not duplicate result fixtures.
- `e2e/smoke.spec.ts` Staff Shortlist assignment scenario — collapse and expand the current result, assert accessible state and no loss, then retain the existing snapshot-replacement clearing proof.
- `e2e/tauri-ipc-stub.ts` — retain optimizer behavior; add a call counter only if Playwright cannot prove no rerun through stable visible data.
- Existing result table, ScoreBadge, vacancy, context, and setup-state tests — retain.

**Behavior and data flow:**

- An accepted ready result still lives in `StaffAssignmentOptimizer`. `StaffAssignmentResults` starts expanded for that result. The header button toggles only its local body visibility. Expanding renders the same result prop and performs no command. A new accepted result resets expanded to true; target or context changes still remove the component and obsolete result under the existing generation guards.

**Ordered implementation steps:**

1. Add a RED component test for the named chevron, `aria-expanded`, controlled body, retained content, and unchanged optimize invocation count.
2. Add RED coverage that a newly accepted result reopens after a prior result was collapsed.
3. Implement local expanded state and use `Panel.actions` with a Lucide chevron and shared ghost-button styling.
4. Extend the existing smoke scenario to collapse, expand, retain the same result, and then clear it on snapshot replacement.
5. Refactor only while focused accessibility and context proofs stay green.

**Tests and proof:**

- RED: the current result Panel has no control and cannot collapse.
- GREEN: tests fail if the button lacks a clear name or `aria-expanded`, collapse discards the result, expand invokes optimization, a new result remains collapsed, or context replacement stops clearing obsolete data.
- Add/modify: optimizer component interaction tests, optional IPC call counter, and the existing Staff Shortlist smoke scenario.
- Delete: none.
- Retain: all recommendation/vacancy rendering, ScoreBadge accessible names, setup/error states, pending suppression, target-save clearing, late-response generation guards, and layout containment proofs.

**Patterns to verify:**

- `Panel.actions`, `Button` ghost/icon behavior, `AppNavRail`'s `aria-expanded` toggle, existing Lucide chevrons, and the current accepted-result ownership in `StaffAssignmentOptimizer`.

**Constraints and non-goals:**

- Do not move result data into Zustand or Query, persist collapse state, rerun optimization, clear the accepted result on collapse, change Panel globally, or rely on icon direction as the only accessible state.

**Dependencies and sequencing:**

- Depends on Commits 4 through 6 for final result behavior and presentation. This is the final implementation commit; completion moves the feature to Validation before close-out.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-optimizer.test.tsx` then `./scripts/dev smoke` then `./scripts/dev check`

**Stop conditions:** Stop if result ownership must move, if collapse cannot retain the exact accepted result, if a new result cannot deterministically reopen, or if the shared Panel needs a breaking API change.

**Review mandate:** Verify accessible name and state attributes; chevron direction; same-result retention; no IPC rerun; new-result expansion; context and target clearing; keyboard focus with no shared Panel regression; and valuable component plus smoke proof.

## Active work

**PR:** PR 1 — Redesign staff assignment slots for FM26

**Commit:** Commit 5 — Match the FM26 Coaches composition

### RED or removal proof

Add exact composition-boundary and specialization-matching cases showing the generic Coach allocator cannot distinguish General, Fitness, and Goalkeeping requirements, can choose a score-first lower-cardinality assignment, cannot select the best partial discipline subset, and cannot carry one typed requirement through recommendation, vacancy, IPC, and UI rendering.

### Expected outcome

Rust derives the exact repeating FM26 Coaches composition, fills General requirements by maximum cardinality then total score then deterministic lexicographic order, allocates only exact Fitness and Goalkeeping pools, preserves global UID and squad priority, and exposes one closed Coach requirement on every Coaches recommendation and vacancy through the rendered result.

### Explicit exclusions

Target persistence or Configure slots grouping, result collapse, Planner matcher changes, new dependencies, coach-type substitution, scoring or ingest changes, current-state documentation, and unrelated cleanup.

## Discoveries and replanning

- The embedded JAY-44 screenshot URL returned HTTP 401 to the available fetch tool. The complete issue text and the developer's resolved exact catalog provide the planning contract; pixel-level screenshot comparison remains a manual validation gap.
- The v35 table needs no shape change. A one-statement v36 target-row reset is sufficient to prevent old semantic pairs from reaching `expand_targets` after the catalog changes.
- The existing Planner exact matcher is player- and 11-lane-specific. The feature needs the same deterministic objective pattern but not a cross-feature dependency.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Redesign staff assignment slots for FM26 | Commit 1 — Record the approved feature plan | 3ac8e21906986f929191fb2264fbc7ec4322adbd | Recorded the reviewed JAY-44 ledger and TODO Active link. | `ledger_state.py`; `delivery_state.py`; `git diff --cached --check` — passed. | Not applicable | Clear | 0 | None |
| PR 1 — Redesign staff assignment slots for FM26 | Commit 2 — Reset and redefine assignment targets | 67854bb776d4948c62fbb549c9f044a0278325ad | Added migration v36, the exact Rust-owned FM26 target catalog and limits, Club-safe Planner cleanup, Recruitment Analyst target vacancies, and the 1,108-slot bound. | `./scripts/dev check-rust`; `./scripts/dev check` — passed (667 Rust tests, 2 ignored). | Pass | Clear | 0 | None |
| PR 1 — Redesign staff assignment slots for FM26 | Commit 3 — Load FM26 assignment candidate scores | 89d447aac9079be3a2cca7aeeb2ad4c4388f18c9 | Loaded Fitness, Goalkeeping, and Recruitment Analyst scores and introduced the closed approved Preferred Job classification without changing allocation phases. | `./scripts/dev check-rust`; `./scripts/dev check` — passed (668 Rust tests, 2 ignored). | Pass | Clear | 0 | None |
| PR 1 — Redesign staff assignment slots for FM26 | Commit 4 — Allocate leads before ordinary roles | Pending record | Added lead-first HPA, Scout, Physio, and Sports Science allocation, ordinary residual phases, global UID reservation, and successful Recruitment Analyst assignment. | `./scripts/dev check-rust`; `./scripts/dev check` — passed (671 Rust tests, 2 ignored). | Pass | Clear | 0 | None |

## Final validation

Run after every implementation commit is completed and before feature review:

1. `./scripts/dev test`
2. `./scripts/dev check-rust`
3. `./scripts/dev smoke`
4. `./scripts/dev check`

Manual gap: inspect **Configure slots** and the result chevron in the supported native Tauri/WebView at 1280×800 and 1600×900 for focus order, numeric input behavior, section density, Club placement with Senior enabled and disabled, and collapse state. This manual check is required when the environment is available; otherwise report it as not run rather than passed.

Mutation testing is not run because `./scripts/dev mutate` is unsupported until mutation tooling is configured.

No release command, package build, version change, tag, or publication is part of this feature.

## Documentation impact

Complete during mandatory feature-close-out reconciliation. Update `.wiki/ARCHITECTURE.md` to record implemented migration v36 and its one-time target reset, the exact Rust-owned target catalog and 0/1 versus 0/50 bounds, the 1,108 result limit, non-exact lead eligibility, three-type Coaches eligibility, cardinality-first specialization allocation, typed Coach requirement evidence, and preserved Rust/IPC authority. Update `.wiki/DESIGN.md` to record the implemented FM26 Coaching/Recruitment/Medical Configure-slots sections, the rule that Club roles render inside Senior or one standalone Club group without changing persisted scope, per-target control bounds, and the accessible result-panel collapse behavior that retains the current result. Then move this complete ledger to `.wiki/features/completed/` and move the TODO item from Active to Completed. These current-state updates are required, not conditional. Do not create an ADR unless implementation discovers a new decision that meets the repository threshold.
