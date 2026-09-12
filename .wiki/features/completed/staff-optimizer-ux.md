# Staff Optimizer UX Improvements

## Status

Ready for final publication

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** a5e55328a2f7c8f159c13527252eda76c9753afeb1a121388ae808d1bfe602d7

## Intent

Make Staff Search assignment planning ready to act on: one clear optimization path, clear prerequisite guidance, usable staffing controls, and a compact result surface that explains vacancies without recreating Rust decisions.

## User-visible behavior

- Staff Search presents **Optimize assignments** as its single primary action. **Upload CSV** and **Configure staffing needs** are secondary actions. The page shows concise shortlist and configured-slot readiness before optimization and directs the user to a missing prerequisite.
- Optimize is disabled when zero slots are configured. The visible, accessible reason directs the user to configure staffing needs. Missing shortlist or managed-club context receives equivalent local recovery guidance; settled valid context permits optimization.
- The stable full-width status region below the action row reports target-save success and optimization/setup/query errors. It does not move when a button label or pending state changes.
- The configuration dialog is titled and triggered as **Configure staffing needs**. Each slot has compact minus, numeric input, and plus controls. Direct numeric entry and keyboard operation remain available. Controls respect the Rust-provided bounds and pending state.
- The dialog explains once that zero excludes a role from recommendations. It shows each enabled team total and section total only from the current locally valid draft; Rust remains the authoritative validator on save.
- A ready result first reports filled slots, vacancies, current staff, and recruits. Candidate and configured-slot totals remain supporting text.
- Vacancies use warning tokens, an icon and text, a plain-language reason, detailed optional evidence, and nearby **Adjust staffing needs** and **Review shortlist** recovery actions. React renders Rust evidence and does not calculate eligibility.
- Results group consecutive Rust-provided scope display names, show each group’s filled/configured count, and remove the repeated Scope column. The bounded result scroller has sticky headers.
- Each row shows one concise reason. An accessible optional disclosure contains Rust-provided bounded eligible, unavailable, and joined-candidate counts.
- The result-panel control displays a compact **Collapse** or **Expand** label as well as its chevron, keeps `aria-expanded` and `aria-controls`, retains the accepted result, and does not optimize again.

## Invariants

- Rust remains authoritative for target validation, current-context checks, candidate-pool semantics, eligibility, score/null meaning, allocation, vacancy evidence, scope display names, and result ordering. React does not recalculate eligibility or scores.
- The existing save/snapshot immutable-token, route-context, request-generation, pending-context suppression, shortlist-pool, and read-only recommendation contracts remain unchanged.
- Null or missing scores remain unavailable, never zero. Existing current-staff name styling and Current staff icon from `bb41169` remain visible on recommendation rows.
- The action row remains full width below the Staff Search shell work. The alignment and containment behavior from `0386fa5` remains intact.
- A target-save draft contains every Rust-provided target once. Local validation gives prompt feedback only; Rust remains authoritative for complete payload and bounds.
- Result groups use Rust `scopeDisplayName` in received order. React may count received tagged slots for presentation only; it does not map scope IDs, filter candidates, rank people, or infer eligibility.
- No result, collapse state, or recommendation history is persisted. Collapse changes local presentation only and never invokes optimization.
- The product remains offline, desktop-only, dependency-free for this feature, and uses existing design tokens and shared primitives.

## Non-goals

- Deferred formation comparison, best/worst candidate highlighting, gap recommendations, charts, candidate comparisons, recommendation persistence/history, backend allocation or business-rule changes, schema/migration changes, and new dependencies.
- Changes to Rust target catalog, candidate eligibility, shortlist replacement semantics, context guards, scoring, score/null semantics, allocation, result DTO meaning, or Football Manager writes.
- A route, URL state, global store, raw IPC call, frontend SQL, mobile layout, pixel-baseline gate, or native-WebView claim from Playwright evidence.
- Retroactively classifying `bb41169` or `0386fa5` as schema-2 completed feature work. The two reviewed precursor commits stay outside this ledger's nine-commit packet sequence.

## Current-state map

- `src/app/routes/staff.tsx::StaffSearchContent` composes Upload CSV, the Staff assignment controller, route context, shortlist state, and the full-width Staff Search workspace. It currently renders a visually hidden page heading and one action row.
- `src/features/staff/components/staff-assignment-optimizer.tsx::StaffAssignmentOptimizer` owns token-bound optimize mutation state, target-save clearing, local result acceptance, and the current Configure/Optimize controls. Its `contents` layout currently allows feedback to participate in the button-row flex layout.
- `staff-assignment-target-modal.tsx::StaffAssignmentTargetModal` owns a target Query, local complete draft, local 0-through-Rust-maximum feedback, target save mutation, and a numeric input per returned target. It currently uses **Configure Club Staff**, plain number inputs, and no totals or zero explanation.
- `staff-assignment-results.tsx::StaffAssignmentResults` owns only local expand state. It currently renders one table with a Scope column, a bounded non-sticky scroll area, verbose in-row evidence, and an icon-only collapse action. It already preserves the accepted result and current-staff icon/name treatment.
- `src/features/staff/types/staff-assignment.ts` defines the bounded Rust DTO. `StaffAssignmentSlot` carries the tagged recommendation/vacancy kind, Rust `scopeDisplayName`, classification, score, Coach requirement, and bounded vacancy evidence.
- Existing focused proof seams are `staff-assignment-optimizer.test.tsx`, `staff-assignment-target-modal.test.tsx`, `staff-assignment-targets-api.test.ts`, `src/app/routes/staff.test.tsx`, `src/testing/staff-ipc-mock.ts`, `e2e/tauri-ipc-stub.ts`, and the Staff Shortlist workflow in `e2e/smoke.spec.ts`.
- `e2e/ui-inspection.spec.ts` has populated initial-route captures only. `./scripts/dev inspect-ui` runs that spec and writes ignored PNGs beneath `.work/ui-inspection`; `CI=1 ./scripts/dev smoke` separately runs behavior proof.
- `scripts/dev` provides `test`, `check`, `check-app`, `check-rust`, `smoke`, and `inspect-ui`. `check` is the complete project gate. `.github/workflows/check.yml` runs frontend checks/tests and browser smoke for `src/**` or `e2e/**` changes; GitHub’s strict required status is `check`.
- `main` is `0312fd1b818435d136df36b1587489a482c239b1`. This worktree is on local-only `feature/staff-optimizer-tweaks` at `0386fa563a34ab968583e38a3f1811e0b468e815`; its only commits ahead of `main` are `bb41169f7a264e1ddf9c08a97d8becfbde2591ef` and then `0386fa563a34ab968583e38a3f1811e0b468e815`. Neither is on `main`, and the branch has no upstream or PR.

## Feature architecture

`StaffAssignmentOptimizer` becomes the Staff-feature composition owner for its action row, stable status region, controlled configuration dialog, and accepted result. `StaffSearchContent` continues to own route URL changes and gives it the existing Upload CSV action plus a narrow callback that turns the existing shortlist filter on for **Review shortlist**. This keeps the route responsible for route state and keeps assignment-local interaction state in the Staff feature.

The optimizer reads the existing Rust target Query once. It derives only presentation readiness from returned target counts and the existing shortlist/setup query state: configured-slot total, shortlist readiness, and the known recovery action. It passes that same query data and a controlled `open`/`onOpenChange` flow to `StaffAssignmentTargetModal`; the Modal no longer starts a duplicate target query. The optimizer opens the dialog from its secondary action and from a vacancy recovery action. Target saves invalidate the existing target key, reset the accepted result through the existing generation guard, and publish their success/error to the stable status region.

The primary Optimize action is enabled only when route context is settled, the target Query is usable, at least one slot is configured, and the existing prerequisites are ready. The status region remains mounted below the action controls, uses the appropriate live role for success/error/setup, and supplies the disabled reason through visible text and `aria-describedby`; it does not hide prerequisite direction behind hover.

`StaffAssignmentTargetModal` remains the local draft owner. A small modal-private control composes minus button, labelled numeric input, and plus button for each returned target. It changes only the local string draft by one bounded step, never clamps typed values, disables all controls while saving, and retains the current local/Rust validation split. The existing Rust target metadata remains the source for scope, section, labels, and maximums. Team and section totals sum only valid current draft values; an invalid affected group reports that its total is unavailable until corrected. The zero explanation appears once above the grouped controls.

`StaffAssignmentResults` receives Rust slots and two narrow UI callbacks: request configuration and review shortlist. It may derive display-only counts from the received tagged slots. It groups consecutive Rust `scopeDisplayName` values without a frontend scope map. Each group has a filled/configured summary. Vacancy display uses the existing warning token family, a non-color icon/text label, and a user-facing reason. The detailed Rust evidence remains bounded and appears in a native accessible disclosure; no candidate filtering or eligibility calculation moves to React. The table’s one bounded scroller owns sticky headers using the design z-index and surface tokens.

The persistent UI inspection extension lives in `e2e/ui-inspection.spec.ts`, not in smoke or product source. It adds two named Staff inspection states that use the existing populated `staffAssignment` and `staffShortlist` stub: an opened configuration dialog and an accepted assignment result. The state selector uses documented inspection-only URL fragments so `inspect-ui` can capture either exact state without a product route or temporary screenshot code. The same two cases remain part of the canonical inspection set. The test uses real accessible actions to reach each state; it does not recreate Rust allocation. Generated PNGs remain disposable `.work/ui-inspection` evidence.

## Uncertainty register

### Known

- The approved scope is all twelve review findings and the stated exclusions.
- Current source, tests, and `DESIGN.md` already provide the Rust DTO, Query, modal, result, action-row, design-token, smoke, and inspection seams needed for this frontend-only feature.
- `bb41169` added current-staff icon/name treatment, and `0386fa5` made the Staff action/result panel full-width aligned. They are reviewed precursor commits already on this PR branch and remain outside this ledger's packet sequence.
- No planned feature spec exists. BACKLOG and ADRs do not change under the approved path.
- GitHub, `.github/pull_request_template.md`, squash merges, and the strict GitHub Actions `check` status are repository evidence.

### Assumptions

- The existing shortlist setup query exposes enough settled state to distinguish a missing shortlist from a ready shortlist without a new command. If its result cannot express that state at the controller seam, stop rather than infer from rendered table rows.
- Native `<details>` provides the required keyboard-accessible optional evidence disclosure without a dependency or a new shared primitive.
- The bounded result order keeps equal `scopeDisplayName` rows consecutive because Rust already returns canonical scope order. If that is not true on the current branch, stop and replan instead of sorting in React.

### Decisions

- Use one PR with maximal atomic commits. The reviewed precursor tweaks and the planned improvements share one Staff Search interaction and result surface; no separately mergeable PR seam exists.
- Adopt local-only `feature/staff-optimizer-tweaks` as the PR branch. It remains based on `main` and contains the reviewed precursor sequence `bb41169` → `0386fa5`; no merge or synchronization prerequisite applies.
- Commit 1 is the first ledger-controlled feature commit and remains planning-only, although it follows the two completed precursor Git commits. Before Commit 1, delivery must verify that `main` is an ancestor of `HEAD`, `bb41169f7a264e1ddf9c08a97d8becfbde2591ef` is an ancestor of `0386fa563a34ab968583e38a3f1811e0b468e815`, `0386fa563a34ab968583e38a3f1811e0b468e815` is the captured starting HEAD, and `git rev-list --reverse main..HEAD` lists exactly those two commits. Afterward, only recorded ledger-controlled commits may extend that range. This preserves the reviewed tasks and prevents another unplanned commit from entering the PR.
- Do not rewrite history. The two local commits are already independently reviewed, validated behavioral work; preserving their order and hashes is lower risk than rewriting a local branch solely to make a later ledger sequence appear first. They are not added to Completed work rows.
- Keep readiness as frontend presentation of existing Rust/query state. Do not add a backend readiness, allocation, score, schema, or persistence rule.
- Make `StaffAssignmentOptimizer` the single target-query and controlled-configuration owner. Keep URL filter changes in `StaffSearchContent` via a narrow callback. This prevents duplicate query ownership and lets both action and vacancy recovery open the same dialog without a global store.
- Keep all action feedback in a stable status region owned by the optimizer composition, below its action row. Button rows contain controls only.
- Show totals only from the current valid local draft. This gives immediate form feedback while preserving Rust’s authoritative complete-payload and range validation.
- Use Rust-provided display names and tagged slots for all result grouping/count presentation. React can count already returned rows but never calculates candidate eligibility, scores, or evidence.
- No ADR is warranted. The feature uses existing React/Query/local-state, Rust DTO, and design-system boundaries without a durable architectural alternative.

### Unknowns

- Exact local class placement needed to keep the status region and result panel inside the Staff workspace’s bounded-height layout after the controller stops using `display: contents`. Resolve against the current branch and its route tests; do not change shared shell geometry without replanning.
- The exact readiness-marker wording/selector for the two inspection states in `e2e/ui-inspection.spec.ts`. It must be a stable, route-local test convention and must not enter product URL validation or `e2e/smoke.spec.ts`.
- Native Tauri/WebView feel for numeric step controls and disclosure geometry remains unavailable in WSL. Playwright is Chromium-stub evidence only.

### Risks

- An unplanned commit could enter the local PR range. Branch verification must require the exact two-commit precursor range before Commit 1 and only recorded ledger-controlled commits afterward.
- Duplicating target queries or configuration opening state could show stale counts, fail to reset recommendations, or make vacancy recovery open a separate dialog instance.
- A disabled primary action without visible/accessible recovery text could strand the user; an action row that owns feedback could shift hierarchy or wrap feedback beside controls.
- Step controls could remove direct entry, keyboard use, max/min bounds, pending locks, or Rust error handling.
- React grouping, summaries, or evidence changes could accidentally reimplement scope/eligibility logic, conceal null-score truth, or alter current-staff styling.
- Sticky headers, grouped rows, optional disclosure, or collapse text could break bounded overflow, focus reachability, or 1280×800 containment.
- A visual capture that starts at `/staff` alone cannot inspect the dialog or ready result. Inspection state support must use the existing stub and persistent spec, not manual screenshot code.

## Walking skeleton

On the current branch after Commit 1 records this plan, a ready Staff Search uses one target Query to show “0 configured slots” readiness, disables Optimize with a visible **Configure staffing needs** recovery action, opens the same controlled dialog, saves one valid slot, then enables the primary Optimize action and renders the existing Rust result. Later commits improve the dialog controls and result readability without changing that context, target, or allocation contract.

## Delivery plan

### PR 1 — Improve Staff Optimizer UX

**Status:** Ready for publication

**PR ref:** <https://github.com/JG1995/fm-valuescout/pull/154>

**Merge ref:** Not merged

**Branch:** feature/staff-optimizer-tweaks

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** strict required GitHub Actions `check`

**Feature close-out:** Current

**CI repair rounds:** 0

**Provisional PR title:** `feat(staff): improve optimizer experience`

**Purpose:** Deliver one reviewed Staff Search optimizer PR: the two reviewed precursor tweaks (current-staff treatment and full-width panel alignment) plus the ordered twelve-finding UX pass. Commits 1–9 are the ledger-controlled feature sequence; the precursor commits are part of the PR range but not Completed work. Final feature review, final validation, staged-diff/PR preparation, and the PR description must inspect and describe the complete `main...HEAD` range, including both precursors.

**Depends on:** The existing local precursor range only: `bb41169f7a264e1ddf9c08a97d8becfbde2591ef` followed by `0386fa563a34ab968583e38a3f1811e0b468e815` on `feature/staff-optimizer-tweaks`. No merge, publication, or synchronization prerequisite applies.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(staff): plan optimizer UX improvements`

**Work:** Commit the reviewed active ledger and TODO activation before implementation. This is the first ledger-controlled feature commit, but it follows the two completed precursor Git commits already on the PR branch.

**Atomicity:** The ledger and TODO link are one planning-state outcome; either file alone leaves no complete approved feature pointer, and the classifier is the meaningful proof. Keeping this planning-only commit after the reviewed precursor commits preserves their separate behavioral history without inventing feature-completion records.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:** Implementation, tests, executable configuration, BACKLOG, DESIGN, ARCHITECTURE, ADRs, planned specs, Git mutations outside authorized delivery, and retroactive schema-2 completion records for precursor commits.

**Implementation packet:** Preserve the independently reviewed planning diff only after verifying the exact precursor range on `feature/staff-optimizer-tweaks`. Record the fingerprint only after fresh complete plan review; do not alter the precursor commits or create Completed work rows for them.

**Files and responsibilities:**

- `.wiki/features/active/staff-optimizer-ux.md` — approved intent, boundaries, packets, risks, and delivery authority.
- `.wiki/TODO.md` — one Active link to this ledger while retaining Next and Completed content.
- `.wiki/BACKLOG.md` — deliberately unchanged; no deferred scope is promoted or reclassified.
- `.wiki/features/planned/` — deliberately unchanged; no planned specification exists to promote or remove.
- `.wiki/decisions/` — deliberately unchanged; no approved decision meets the ADR threshold.

**Behavior and data flow:** Record one durable active feature owner and make TODO point at it before code work begins.

**Ordered implementation steps:**

1. Capture the starting HEAD. Verify that `main` is its ancestor, the exact local refs `bb41169f7a264e1ddf9c08a97d8becfbde2591ef` and `0386fa563a34ab968583e38a3f1811e0b468e815` are ancestors in that order, `0386fa563a34ab968583e38a3f1811e0b468e815` is the starting HEAD, and `git rev-list --reverse main..HEAD` contains no other commits.
2. Confirm only the reviewed ledger and TODO paths are present.
3. Run the ledger classifier. After fresh complete review and fingerprint recording, run the delivery-state validator using the recorded fingerprint.
4. Inspect the exact planning diff before normal checkpoint review.

**Tests and proof:** Not applicable — this is planning-only. Prove classifier structure and exact path scope; no tests, fixtures, mocks, or screenshots change.

**Patterns to verify:** `.wiki/features/active/README.md`, `.wiki/TODO.md`, current completed Staff assignment records, and the repository PR template.

**Constraints and non-goals:** Do not alter approved scope or the precursor commits, add a fingerprint before review/orchestrator recording, retroactively classify precursor work, or edit other planning/current-state paths.

**Dependencies and sequencing:** Requires fresh complete plan-review clearance, developer acceptance, recorded fingerprint, exact branch activation, and verification of the two-commit precursor range. No merge or synchronization prerequisite applies.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/staff-optimizer-ux.md`; after review, `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/staff-optimizer-ux.md .`.

**Stop conditions:** Stop if the exact precursor refs are missing, out of order, or not the complete pre-Commit-1 `main..HEAD` range; on a classifier error, unreviewed path, fingerprint mismatch, or branch/base mismatch.

**Review mandate:** Verify only the two approved planning paths change; all twelve findings, exclusions, one-PR authority, exact precursor boundary, planning-only first ledger-controlled commit, and later packet contracts are preserved.

#### Commit 2 — Add optimizer readiness and stable feedback

**Status:** Completed

**Provisional commit:** `feat(staff): clarify optimizer readiness`

**Work:** Establish the one-primary-action Staff optimizer composition: readiness, prerequisite recovery, controlled configuration opening, and a stable status region below the action row.

**Atomicity:** The shared target-query/controller, primary-action availability, recovery actions, and status placement form one observable action contract; splitting them would leave either duplicate query ownership or a disabled action with no reliable recovery/proof.

**Size assessment:** About 160 non-test implementation lines. Within the soft target.

**Out of scope:** Numeric step controls/totals, result summary, vacancy treatment, grouping, sticky headers, evidence disclosure, collapse-label changes, Rust behavior, and persistence.

**Implementation packet:** Refactor the Staff assignment composition so it reads targets once, owns the controlled configuration dialog, receives the existing Upload action from the route, and places controls plus feedback in stable separate regions. Use existing query state only to present readiness; do not add IPC or derive domain eligibility.

**Files and responsibilities:**

- `src/app/routes/staff.tsx::StaffSearchContent` — pass the existing Upload CSV action and a narrow URL-filter-on callback; retain route context and bounded workspace ownership.
- `src/features/staff/components/staff-assignment-optimizer.tsx` — own one target Query, controlled modal opening, readiness, Optimize enablement, action hierarchy, and stable success/error/setup status region.
- `src/features/staff/components/staff-assignment-target-modal.tsx` — accept controller-provided target data and controlled open state; stop duplicating the target Query; rename its trigger/entry point to **Configure staffing needs** where it remains exposed.
- `src/features/staff/components/staff-assignment-optimizer.test.tsx`, `staff-assignment-target-modal.test.tsx`, and `src/app/routes/staff.test.tsx` — prove one primary action, disabled-reason/recovery behavior, query ownership, controlled opening, action/status separation, and retained context/reset guards.
- `src/testing/staff-ipc-mock.ts` — modify only fixtures/inspection helpers needed for the changed query ownership and status assertions.
- `e2e/smoke.spec.ts` — update every affected Configure-button assertion to **Configure staffing needs** and preserve or deliberately update the exact `Slot counts saved.` stable-status-region assertion.

**Behavior and data flow:** Rust target data enters one Staff Query. The controller derives configured-slot total and prerequisite state for display, opens the same modal from the secondary action, and resets results after target save. The route alone changes `shortlistOnly` for Review shortlist. Optimize remains primary but is disabled with visible/accessible reason for zero slots and other unsettled/missing prerequisites. Feedback is emitted below the controls, never as a flex item in the button row.

**Ordered implementation steps:**

1. Add RED component/route proofs that zero configured slots disables Optimize with a visible accessible Configure staffing needs recovery, and that feedback occupies a full-width region below controls.
2. Add RED proof that controller and modal share one target Query/data source and a recovery action opens the controlled modal.
3. Move the composition to the controller/route slot boundary; retain token, generation, pending, and save-reset behavior.
4. Make Upload and Configure secondary and Optimize primary without changing upload semantics.
5. Run focused tests, smoke, and the full gate; inspect the affected Staff initial state before and after at both required viewports.

**Tests and proof:** Observable behavior: a user can see why Optimize is unavailable and reach its prerequisite; successful target save and errors do not shift into the action row. Add/modify the named component/route tests and only needed IPC fixture support. Update every affected Configure-button assertion in `e2e/smoke.spec.ts` to **Configure staffing needs**. Preserve the exact `Slot counts saved.` assertion for the stable status region, or deliberately update its exact text if the approved region contract requires it. RED must fail because the current zero-slot state allows optimize and feedback shares action-row layout. `CI=1 ./scripts/dev smoke` remains browser behavior proof, not visual proof, and must pass so the rename and moved feedback remain trunk-safe.

**Patterns to verify:** Existing `StaffAssignmentOptimizer` request generation, `StaffAssignmentTargetModal` context reset, route `updateSearch`, action-row layout from `0386fa5`, and shared Button pending/accessibility behavior.

**Constraints and non-goals:** Do not change Rust setup states, target payload, query keys, route validation, result DTOs, current-staff styling, or URL ownership. Do not hide recovery guidance in a tooltip.

**Dependencies and sequencing:** Requires Commit 1 and the verified precursor range on the current branch. Commit 3 consumes the controlled modal seam; later result commits consume the recovery callbacks.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-optimizer.test.tsx src/features/staff/components/staff-assignment-target-modal.test.tsx src/app/routes/staff.test.tsx`; `CI=1 ./scripts/dev smoke`; `./scripts/dev check`; before/after and final-state evidence: `./scripts/dev inspect-ui '/staff?shortlistOnly=true' 1280 800` and `./scripts/dev inspect-ui '/staff?shortlistOnly=true' 1600 900`, opening each generated PNG with the image-capable reader.

**Stop conditions:** Stop if existing query data cannot distinguish readiness without inspecting rendered rows, if controlled opening requires a global store or duplicate target Query, if status cannot be separated without breaking the bounded workspace, or if prerequisite behavior needs a Rust contract change.

**Review mandate:** Verify one target-query owner; Optimize primary and secondaries correct; zero-slot and missing-prerequisite accessible recovery; route-only URL update; stable full-width status placement; token/generation/pending reset preservation; and focused tests that fail for stale feedback or no recovery.

#### Commit 3 — Improve staffing-needs controls

**Status:** Completed

**Provisional commit:** `feat(staff): improve staffing needs controls`

**Work:** Add compact accessible decrement/input/increment controls, one zero-exclusion explanation, and validated-draft team/section totals to the controlled configuration dialog.

**Atomicity:** Each control’s direct entry, bounded step actions, pending lock, validation, and totals consume the same local draft; splitting them would leave two conflicting ways to edit a target or totals that do not reflect the supported form state.

**Size assessment:** About 180 non-test implementation lines. Within the soft target.

**Out of scope:** Readiness/action hierarchy, target persistence/Rust validation, result presentation, new shared form primitives, and changes to target catalog metadata.

**Implementation packet:** Keep `StaffAssignmentTargetModal` as the local draft owner. Replace each plain field presentation with a modal-private compact control while retaining the actual labelled number input. Derive totals only from valid draft values and preserve current server-error, focus, close, and context-reset rules.

**Files and responsibilities:**

- `src/features/staff/components/staff-assignment-target-modal.tsx` — modal-private minus/input/plus control, direct entry, valid-draft totals by returned team and section, one zero explanation, pending behavior, labels, and error descriptions.
- `src/features/staff/components/staff-assignment-target-modal.test.tsx` — direct typing, keyboard access, decrement/increment bounds, disabled pending controls, valid/invalid totals, zero explanation, complete payload, Rust error, and focus/context behavior.
- `src/testing/staff-ipc-mock.ts` — retain Rust-shaped maximums; change only target fixture details needed to show varied counts/totals.
- `e2e/ui-inspection.spec.ts` — add the durable populated configuration-dialog inspection state reached through accessible UI actions and its readiness assertion.

**Behavior and data flow:** Returned target metadata seeds one string draft. A step button changes a valid numeric draft by exactly one only inside the returned 0-to-max bound; direct input remains uncoerced so existing local error feedback applies. Team and section totals read the current valid entries in their group; invalid input yields no misleading total. Save continues to send the complete draft to Rust.

**Ordered implementation steps:**

1. Add RED component tests for typed value retention, keyboard input, bounded minus/plus behavior, pending lock, and no premature clamping.
2. Add RED totals tests that use the current valid draft and become unavailable when affected input is invalid.
3. Implement the minimum dialog-private control and total presentation without copying the catalog.
4. Add the populated inspection case by opening the existing dialog against the current stub; do not modify smoke for screenshots.
5. Run focused, smoke, and complete checks, then capture/open configuration PNGs at both supported viewports.

**Tests and proof:** Observable behavior: the user can type or step each count, sees accurate valid-draft totals and the zero rule, and cannot mutate while save is pending. Modify the named modal test and fixture only. The inspection case must fail before it can find the opened dialog. Add no duplicate allocator or Rust-bound tests.

**Patterns to verify:** Current `slotCountError`, `draftKey`, `draftFromTargets`, shared Modal focus behavior, Button disabled semantics, and existing nested fieldsets.

**Constraints and non-goals:** Rust remains authoritative. Do not change requested target maxima, add a dependency/shared input component, use buttons without names, remove number-input keyboard operation, or calculate persisted/eligible recommendations.

**Dependencies and sequencing:** Requires Commit 2’s one-query controlled modal. Commit 8’s final visual inspection consumes this durable dialog state.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-target-modal.test.tsx src/features/staff/components/staff-assignment-optimizer.test.tsx`; `CI=1 ./scripts/dev smoke`; `./scripts/dev check`; before/after and final evidence: `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-modal' 1280 800` and `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-modal' 1600 900`, then open both PNGs with the image-capable reader.

**Stop conditions:** Stop if a returned target cannot retain direct numeric keyboard entry, if totals need new backend data or cannot be made truthful for invalid drafts, if a control changes target semantics, or if the dialog cannot contain the dense controls at the supported desktop sizes.

**Review mandate:** Verify real labelled number inputs remain; minus/plus bounds and pending locks; valid-draft-only totals; one zero explanation; complete Rust payload and error/focus/context contracts; durable interaction-based inspection coverage; and no new dependency or duplicated catalog.

#### Commit 4 — Summarize assignment results

**Status:** Completed

**Provisional commit:** `feat(staff): summarize assignment results`

**Work:** Add the concise ready-result summary for filled, vacancies, current staff, and recruits while retaining candidate/configured counts as supporting text.

**Atomicity:** The four metrics describe the same accepted Rust result and need one presentation-count proof; splitting them would create incomplete or inconsistent summary meaning with no independently useful outcome.

**Size assessment:** About 70 non-test implementation lines. Within the soft target.

**Out of scope:** Vacancy warning/recovery, table grouping, sticky headers, evidence disclosure, collapse label, Rust response changes, and candidate eligibility computation.

**Implementation packet:** Derive display-only counts from received tagged slots/classification in `StaffAssignmentResults`. Keep the existing dynamic Rust joined-candidate and configured-slot sentence contract as supporting text; with the current fixture, it says exactly `5 joined shortlisted candidates; 4 configured slots.`. Do not filter, score, or infer candidate eligibility.

**Files and responsibilities:**

- `src/features/staff/components/staff-assignment-results.tsx` — ready-result summary layout and presentation-only counts.
- `src/features/staff/components/staff-assignment-optimizer.test.tsx` — filled/vacancy/current/recruit summary assertions alongside existing result fixture/rendering proof.
- `src/testing/staff-ipc-mock.ts` — adjust only result fixtures needed to distinguish all four metrics.

**Behavior and data flow:** Rust returns slots with kind and classification. React counts those supplied tags for labels only, then retains `joinedCandidateCount` and `configuredSlotCount` below as the existing dynamic supporting sentence; with the current fixture, it renders exactly `5 joined shortlisted candidates; 4 configured slots.`.

**Ordered implementation steps:**

1. Add a RED result assertion with recommendations, recruitment rows, and vacancies.
2. Add the compact summary using supplied slot tags only.
3. Preserve empty-ready and setup-state rendering.
4. Run focused test, smoke, and project gate.

**Tests and proof:** Observable behavior: a ready result exposes all four summary measures and does not relabel candidate/configured supporting text as primary metrics. Modify the optimizer component test to retain the dynamic supporting-sentence contract, including the current fixture’s exact `5 joined shortlisted candidates; 4 configured slots.` text. Retain Rust/allocation tests because they protect the DTO’s truth.

**Patterns to verify:** Existing result summary paragraph, tagged `StaffAssignmentSlot` union, current-staff icon/name test, and Panel typography tokens.

**Constraints and non-goals:** Do not add result fields, traverse candidates, alter ScoreBadge/null behavior, or replace current-staff styling.

**Dependencies and sequencing:** Requires Commit 2’s stable result placement. Commit 5 adds recovery actions without changing summary ownership.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-optimizer.test.tsx`; `CI=1 ./scripts/dev smoke`; `./scripts/dev check`.

**Stop conditions:** Stop if the tags cannot distinguish each displayed count, if counts require an eligibility recomputation, or if summary placement breaks result containment.

**Review mandate:** Verify only received tagged slots drive summary counts; all four labels are clear; candidate/configured detail remains supporting; current-staff styling survives; and the test detects a wrong classification or vacancy count.

#### Commit 5 — Make vacancies recoverable

**Status:** Completed

**Provisional commit:** `feat(staff): make assignment vacancies actionable`

**Work:** Give vacancy rows warning treatment, user-facing reason, and adjacent configuration/shortlist recovery actions.

**Atomicity:** Warning semantics, reason, and the two local recovery actions are one actionable vacancy outcome; a warning without recovery or actions without a reason would not satisfy the reviewed user path.

**Size assessment:** About 130 non-test implementation lines. Within the soft target.

**Out of scope:** Grouping, sticky headers, optional detailed evidence, summary counts, target validation, and allocation changes.

**Implementation packet:** Render vacancy state with warning-container/token classes and a non-color icon/text cue. Expose **Adjust staffing needs** through the controller’s controlled dialog callback and **Review shortlist** through the route’s existing shortlist-filter callback. Keep only a concise in-row reason at this stage; Commit 7 owns optional detailed evidence.

**Files and responsibilities:**

- `src/features/staff/components/staff-assignment-optimizer.tsx` — pass narrow request-configuration/review-shortlist callbacks to result presentation.
- `src/features/staff/components/staff-assignment-results.tsx` — warning icon/text/tokens, concise vacancy reason, and nearby recovery actions.
- `src/app/routes/staff.tsx` — supply only the existing URL shortlist-filter-on callback; retain route state ownership.
- `src/features/staff/components/staff-assignment-optimizer.test.tsx` and `src/app/routes/staff.test.tsx` — prove warning non-color cue, recovery opening/filter behavior, and no new optimize call.

**Behavior and data flow:** Rust supplies an explicit vacancy plus bounded evidence. React displays that known state; Adjust opens the existing controlled modal and Review uses the route callback to turn on the saved shortlist view. Neither action changes allocation, target data, or recommendation ownership until the established save/optimize flow runs.

**Ordered implementation steps:**

1. Add RED component/route proofs for warning icon/text, concise reason, controlled dialog opening, and route-owned shortlist activation.
2. Implement warning token treatment without raw colors.
3. Wire callbacks without moving URL state or target Query ownership.
4. Verify keyboard reachability and no rerun on either recovery action.
5. Run focused, smoke, and complete checks.

**Tests and proof:** Observable behavior: a vacancy explains that no eligible shortlisted candidate filled the slot and gives both recovery paths. Extend focused component/route tests; no backend test changes. `CI=1 ./scripts/dev smoke` must retain the assembled Staff workflow.

**Patterns to verify:** `StatusChip` semantic icon/text rule, warning tokens in DESIGN, route `updateSearch`, controlled modal flow from Commit 2, and existing button focus behavior.

**Constraints and non-goals:** Do not calculate or expose candidate eligibility in React, replace Rust evidence, add raw colors/tooltips, change shortlist membership, or silently rerun optimization.

**Dependencies and sequencing:** Requires Commit 2 controlled opening and Commit 4 summary. Commit 7 moves the detailed evidence to the disclosure.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-optimizer.test.tsx src/app/routes/staff.test.tsx`; `CI=1 ./scripts/dev smoke`; `./scripts/dev check`.

**Stop conditions:** Stop if route callback cannot update the existing filter without new route state, if action placement cannot retain keyboard/focus behavior, or if a recovery action requires new backend information.

**Review mandate:** Verify warning token plus icon/text redundancy; plain vacancy reason; both narrow recovery flows; no result/eligibility duplication or rerun; existing context guards; and route/component tests at the correct seams.

#### Commit 6 — Group assignment results by scope

**Status:** Completed

**Provisional commit:** `feat(staff): group assignment results by scope`

**Work:** Replace repeated scope cells with consecutive Rust-display-name result groups and per-group filled/configured summaries.

**Atomicity:** Removing Scope and adding its group header/summary together preserve scope context for every row; either half alone loses information or duplicates it, so the grouped table is one coherent outcome.

**Size assessment:** About 140 non-test implementation lines. Within the soft target.

**Out of scope:** Sticky headers, evidence disclosure, collapse label, vacancy recovery, allocation order, or frontend scope mapping.

**Implementation packet:** Partition only adjacent received rows by exact `scopeDisplayName`, render a labelled group header and filled/configured count, and remove the Scope table column. Preserve received row order and existing selected/current-staff content.

**Files and responsibilities:**

- `src/features/staff/components/staff-assignment-results.tsx` — consecutive display-name grouping, per-group presentation count, column removal, semantic table/group markup.
- `src/features/staff/components/staff-assignment-optimizer.test.tsx` — custom Rust display-name grouping, no Scope header, group count, and preserved row order assertions.
- `src/testing/staff-ipc-mock.ts` — result fixture with multiple supplied group names if needed.
- `e2e/ui-inspection.spec.ts` — add the durable accepted-recommendations inspection state using existing populated stub actions and readiness assertion.
- `e2e/smoke.spec.ts` — update the existing Staff assignment row-count and Scope assertions to grouped headers, per-group counts, and preserved result order without weakening the scenario.

**Behavior and data flow:** Rust slot order and `scopeDisplayName` arrive bounded. React walks the array once to form only adjacent display-name groups, counts filled tagged rows for each group, and renders no raw scope ID or Scope column.

**Ordered implementation steps:**

1. Add RED component proof with Rust-provided custom team name, Club, and multiple group transitions.
2. Replace Scope cells with group headers/summaries while preserving body row order.
3. Add the durable populated results inspection state through accessible configuration/Optimize actions; do not duplicate the allocator in the stub.
4. Run focused tests, smoke, full gate, and inspect results at both viewports.

**Tests and proof:** Observable behavior: group names are Rust strings, each group has filled/configured summary, and no Scope header remains. Update the existing `e2e/smoke.spec.ts` Staff assignment row-count and Scope assertions to grouped headers, per-group counts, and preserved result order without weakening the scenario. The tests must fail if React maps scope IDs, reorders slots, or omits a group. The inspection state must fail before the ready result can be reached.

**Patterns to verify:** `scopeDisplayName` DTO contract, existing semantic table caption, row keys, and table token guidance.

**Constraints and non-goals:** Do not sort, map, or otherwise interpret scopes in React. Do not change Rust DTOs, group across nonconsecutive rows, or add a second table.

**Dependencies and sequencing:** Requires Commit 4 slot-summary seam. Commit 8 consumes this durable result inspection state for final visual evidence.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-optimizer.test.tsx`; `CI=1 ./scripts/dev smoke`; `./scripts/dev check`; before/after and final evidence: `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-results' 1280 800` and `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-results' 1600 900`, then open both PNGs with the image-capable reader.

**Stop conditions:** Stop if Rust display names are not consecutive in result order, if the DTO loses group context, or if preserving group context requires frontend scope semantics.

**Review mandate:** Verify Rust-provided names/order only; group filled/configured counts; Scope-column removal; semantic table/accessibility; current-staff treatment; persistent interaction-based inspection coverage; and tests that detect a raw ID/map/order regression.

#### Commit 7 — Disclose assignment evidence on demand

**Status:** Completed

**Provisional commit:** `feat(staff): disclose assignment evidence on demand`

**Work:** Reduce in-row evidence weight to one concise reason and place detailed bounded Rust counts in an accessible optional disclosure.

**Atomicity:** The concise row statement and its disclosure replace one evidence contract together; splitting them would either hide actionable evidence or retain the noisy presentation the commit removes.

**Size assessment:** About 100 non-test implementation lines. Within the soft target.

**Out of scope:** Vacancy recovery, grouping, sticky headers, allocation/evidence computation, and new details/popover dependencies.

**Implementation packet:** Use semantic native disclosure for detailed vacancy counts. Keep recommendation reason concise and preserve Coach requirement text where supplied. Render only DTO evidence values; never derive eligible/unavailable counts or candidate decisions.

**Files and responsibilities:**

- `src/features/staff/components/staff-assignment-results.tsx` — concise recommendation/vacancy reasons and accessible optional disclosure of bounded counts.
- `src/features/staff/components/staff-assignment-optimizer.test.tsx` — collapsed/expanded disclosure accessibility, exact Rust count text, and no eligibility recomputation assertions.
- `src/testing/staff-ipc-mock.ts` — retain/adjust bounded evidence fixtures only if required for distinct counts.

**Behavior and data flow:** The Rust vacancy DTO’s `eligibleScoreCount`, `unavailableScoreCount`, and `joinedCandidateCount` flow unchanged into a disclosure. The table row supplies its one-line reason; user expansion reveals the detailed counts without calling IPC.

**Ordered implementation steps:**

1. Add RED result test for concise default evidence and keyboard-reachable details with supplied values.
2. Replace verbose in-row evidence with native disclosure while preserving Coach requirement wording.
3. Confirm opening/closing detail neither optimizes nor changes result state.
4. Run focused test, smoke, and complete gate.

**Tests and proof:** Observable behavior: default scanning is concise, optional details expose exact bounded counts, and a wrong React calculation cannot pass. Modify focused optimizer component test only; retain Rust evidence tests as source-of-truth proof.

**Patterns to verify:** Existing `evidenceText`, native semantic disclosure behavior, DESIGN keyboard/no-hover rule, and DTO null/evidence semantics.

**Constraints and non-goals:** Do not use a custom popover, hide evidence from assistive technology, use hover-only content, compute eligibility, or change DTO shapes.

**Dependencies and sequencing:** Requires Commit 5 warning/reason presentation and Commit 6 grouped rows.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-optimizer.test.tsx`; `CI=1 ./scripts/dev smoke`; `./scripts/dev check`.

**Stop conditions:** Stop if native disclosure cannot retain accessible label/state in the table, if detailed text needs data absent from the DTO, or if implementation would recompute eligibility.

**Review mandate:** Verify concise default reason; disclosure semantics/keyboard access; exact Rust bounded counts; no rerun or frontend eligibility; Coach/null evidence truth; and focused test value.

#### Commit 8 — Keep result headers visible

**Status:** Completed

**Provisional commit:** `feat(staff): keep assignment headers visible`

**Work:** Make assignment table headers sticky inside the existing bounded results scroller.

**Atomicity:** Sticky header positioning, scroller ownership, and token/z-index proof are one containment outcome; a CSS-only partial without the bounded scroller check could cover rows or escape the panel.

**Size assessment:** About 40 non-test implementation lines. Within the soft target.

**Out of scope:** Column/group content, evidence disclosure, general Data Table refactors, global CSS, and new scrolling behavior outside results.

**Implementation packet:** Apply sticky header positioning only to the existing results scroller and use the documented recessed surface/z-index tokens. Preserve the current `max-h-80` bounded vertical/horizontal overflow owner.

**Files and responsibilities:**

- `src/features/staff/components/staff-assignment-results.tsx` — sticky header classes within the existing results scroll container.
- `src/features/staff/components/staff-assignment-optimizer.test.tsx` — structural class/attribute proof only where useful.
- `e2e/smoke.spec.ts` — extend the existing Staff assignment scenario with a bounded scroll/geometry assertion that a header remains visible after table scroll; do not add screenshot calls.

**Behavior and data flow:** No data changes. The results scroller retains its bounds; only the header sticks above its body rows at the documented sticky layer.

**Ordered implementation steps:**

1. Add a RED browser assertion that scrolls the existing assignment result scroller and checks the header stays in the scrollport.
2. Apply the minimal token-aligned sticky header classes.
3. Confirm scroll containment and focus/row visibility at 1280×800 and 1600×900.
4. Run smoke and the complete gate.

**Tests and proof:** Observable behavior: a user scrolling a long bounded result can still read headers. Extend the current Staff Shortlist smoke scenario; do not add duplicate allocation fixtures.

**Patterns to verify:** DESIGN Data Table sticky-header z-index/surface rules, existing result `max-h-80` scroller, and Smoke’s existing viewport geometry assertions.

**Constraints and non-goals:** Do not change row order/height, unbound the table, alter global table styles, or introduce a new scroll container.

**Dependencies and sequencing:** Requires Commit 6 grouped result table. Commit 9 changes only collapse control content.

**Validation:** `CI=1 ./scripts/dev smoke`; `./scripts/dev check`; final visual evidence after this and later visual commits: `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-results' 1280 800` and `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-results' 1600 900`, opening each PNG with the image-capable reader.

**Stop conditions:** Stop if sticky positioning cannot remain within the current bounded scroller, obscures focused rows, or requires shared/global table changes.

**Review mandate:** Verify exact scroller ownership; token/z-index use; header visibility after scroll; no document overflow or row/focus regression; and meaningful browser proof separate from visual inspection.

#### Commit 9 — Label the result collapse control

**Status:** Completed

**Provisional commit:** `feat(staff): label result collapse control`

**Work:** Replace the icon-only result-panel collapse affordance with compact visible Collapse/Expand text while retaining the existing no-rerun accessibility contract.

**Atomicity:** The visible label and existing ARIA/no-rerun state describe one control; splitting them could expose a label that no longer matches controlled content or remove a previously proved accessibility guard.

**Size assessment:** About 45 non-test implementation lines. Within the soft target.

**Out of scope:** Result ownership, grouping, evidence, animation, shared Panel API, persisted collapse preference, and optimize mutation changes.

**Implementation packet:** Keep `StaffAssignmentResults` local `expanded` state and result prop. Change only the header action anatomy to a compact visible label plus chevron; retain `aria-expanded`, stable `aria-controls`, keyboard operation, same-result retention, and new-result expansion.

**Files and responsibilities:**

- `src/features/staff/components/staff-assignment-results.tsx` — compact visible Collapse/Expand label alongside icon and existing ARIA state.
- `src/features/staff/components/staff-assignment-optimizer.test.tsx` — visible label, `aria-expanded`, `aria-controls`, retained body, no extra optimizer IPC call, and new-result expansion proof.
- `e2e/smoke.spec.ts` — update existing collapse/expand accessible-name assertion while retaining no-loss workflow proof.

**Behavior and data flow:** Toggling changes local visibility only. The same accepted result stays mounted; re-expanding uses the same prop and never calls optimize. A newly accepted result still begins expanded.

**Ordered implementation steps:**

1. Change the existing RED accessibility assertion to require visible Collapse/Expand text as well as ARIA state.
2. Apply the smallest shared-Button-compatible compact label/icon change.
3. Retain no-rerun/new-result/context-clear proof.
4. Run focused test, smoke, and full gate.

**Tests and proof:** Observable behavior: the control is visibly named, keyboard-accessible, preserves the exact accepted result, and does not invoke a second optimization. Modify the existing component and smoke tests; do not add a second collapse fixture.

**Patterns to verify:** Current `useId`, `Panel.actions`, Button variants, existing collapse tests, and DESIGN requirement that icon-only information has a visible keyboard path.

**Constraints and non-goals:** Do not change result data ownership, persist expansion, rerun optimize, animate, or modify Panel globally.

**Dependencies and sequencing:** Requires the existing collapse contract and follows all result presentation commits. This is the final implementation commit; completion moves the feature to Validation before close-out.

**Validation:** `./scripts/dev test src/features/staff/components/staff-assignment-optimizer.test.tsx`; `CI=1 ./scripts/dev smoke`; `./scripts/dev check`; final visual evidence: `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-modal' 1280 800`, `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-modal' 1600 900`, `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-results' 1280 800`, and `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-results' 1600 900`, opening each generated PNG with the image-capable reader.

**Stop conditions:** Stop if a visible label cannot fit without losing the ARIA relation, if a label change triggers optimization or clears results, or if it requires shared Panel changes.

**Review mandate:** Verify compact visible label and chevron; `aria-expanded`/`aria-controls`; keyboard operation; same-result/no-rerun/new-result behavior; current context clearing; and non-duplicative focused plus smoke proof.

## Discoveries and replanning

- Approved replan: adopt local-only `feature/staff-optimizer-tweaks` rather than requiring separate merges. `bb41169` and `0386fa5` are reviewed precursor commits on the eventual PR branch, outside this ledger's schema-2 packet sequence. Before Commit 1, their exact two-commit `main..HEAD` range must be verified; after that, delivery may build on the branch without synchronization.
- The existing controller and target Modal both own adjacent parts of target state. The accepted plan settles one target Query/controller plus controlled Modal flow rather than adding a second source of truth.
- Initial `/staff` inspection cannot reach the approved dialog/result states. The durable `ui-inspection.spec.ts` extension must reach each state through the existing populated stub and accessible controls; inspection PNGs remain evidence, not a baseline gate.
- Commit 2 correction review retained one MEDIUM advisory: standalone `StaffAssignmentTargetModal` defaults `targetsPending` to `false`, so its fallback query pending state does not disable the trigger. Production uses the controlled path and remains correct. The advisory does not block advancement and remains open for feature close-out unless explicitly delegated.
- Commit 5 review retained one MEDIUM proof advisory: the Adjust staffing needs test proves the controlled dialog opens but does not assert a seeded draft value. The implementation reuses `draftFromTargets`, and existing trigger-path tests prove that helper; the advisory remains open for feature close-out unless explicitly delegated.
- Commit 7 review retained one MEDIUM naming advisory: a pre-existing panel Collapse/Expand test was accidentally renamed to claim evidence-disclosure coverage while its body still proved panel collapse. Commit 9 resolved it within the owning test while adding the visible-label proof.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Improve Staff Optimizer UX | Commit 1 — Record the approved feature plan | `036471db667b0e4cafd6794726c07b0c77fa8e51` | Recorded the accepted schema-2 ledger and TODO Active link after the exact reviewed precursor range. | `ledger_state.py`; `delivery_state.py`; `git diff --cached --check` — passed. | Not applicable | Clear | 0 | None |
| PR 1 — Improve Staff Optimizer UX | Commit 2 — Add optimizer readiness and stable feedback | `990e47d50dd12251381d386522fec86c873cdbcd` | Added one-query controlled staffing configuration, explicit readiness and recovery, single-primary action hierarchy, and stable full-width feedback while preserving context/reset guards. | Focused component/route tests (74 passed before correction; latest focused optimizer/modal 21 passed); full frontend tests (1,025 passed); `CI=1 ./scripts/dev smoke` (62 passed); `./scripts/dev check` (875 Rust tests passed, 3 ignored); dual-viewport UI inspection; LSP and diff checks — passed. | Pass | Accepted findings — one MEDIUM standalone fallback-pending advisory retained for feature close-out; no CRITICAL or HIGH findings remain. | 1 | Initial review found three HIGH hierarchy, feedback-placement, and test-value defects; correction review verified all three fixed. |
| PR 1 — Improve Staff Optimizer UX | Commit 3 — Improve staffing-needs controls | `34ce800e7fe605aea06b7b15339f01224dc9e653` | Added explicitly labelled bounded step controls, direct numeric entry, pending locks, one zero rule, truthful valid-draft team/section totals, and a durable populated dialog inspection state. | Focused optimizer/modal tests (22 passed); `CI=1 ./scripts/dev smoke` (62 passed); `./scripts/dev check` (875 Rust tests passed, 3 ignored); dual-viewport modal inspection; LSP and diff checks — passed. | Pass | Clear | 1 | Initial review found invalid label-wrapped step buttons; correction used explicit collision-safe label/input association and added live Chromium step proof. |
| PR 1 — Improve Staff Optimizer UX | Commit 4 — Summarize assignment results | `6a762e1776a1f31d7695fbeb5c381cccabb7a792` | Added compact filled, vacancy, current-staff, and recruit metrics derived only from received slot tags while retaining candidate/configured supporting text. | Focused optimizer tests (16 passed); `CI=1 ./scripts/dev smoke` (62 passed); `./scripts/dev check` (875 Rust tests passed); LSP and diff checks — passed. | Pass | Clear | 0 | None |
| PR 1 — Improve Staff Optimizer UX | Commit 5 — Make vacancies recoverable | `81ed8e1e5c88e36802f76e6f393884d4e3419a09` | Added warning-token vacancy cues and reasons plus controlled staffing and route-owned shortlist recovery without rerunning optimization. | Focused optimizer/route tests (71 passed); `CI=1 ./scripts/dev smoke` (62 passed); `./scripts/dev check` (875 Rust tests passed, 3 ignored); LSP and diff checks — passed. | Pass | Accepted findings — one MEDIUM Adjust-path draft-content proof advisory retained for feature close-out; no CRITICAL or HIGH findings. | 0 | None |
| PR 1 — Improve Staff Optimizer UX | Commit 6 — Group assignment results by scope | `d6c2c74fb4b71d0ed1f974c48042eae8892a2720` | Grouped adjacent Rust display names in received order, added per-group filled/configured headers, removed Scope, and added durable populated result inspection. | Focused optimizer tests (16 passed); `CI=1 ./scripts/dev smoke` (62 passed); `./scripts/dev check` (875 Rust tests passed, 3 ignored); dual-viewport result inspection; LSP and diff checks — passed. | Pass | Clear | 0 | None |
| PR 1 — Improve Staff Optimizer UX | Commit 7 — Disclose assignment evidence on demand | `9aea5b4c7a8735a51d5df7f822fe671df99fc38a` | Kept row reasons concise and moved exact bounded Rust vacancy counts into native accessible disclosures without rerunning optimization. | Focused component/route tests (77 passed); `CI=1 ./scripts/dev smoke` (62 passed); `./scripts/dev check` (875 Rust tests passed, 3 ignored); LSP and diff checks — passed. | Pass | Accepted findings — one MEDIUM inaccurate collapse-test name retained for Commit 9; no CRITICAL or HIGH findings. | 0 | None |
| PR 1 — Improve Staff Optimizer UX | Commit 8 — Keep result headers visible | `64bfcce46da8306bb648fa28b1f6b7f49851fd6d` | Kept assignment column headers visible within the existing bounded result scroller using documented sticky surface/layer tokens. | `CI=1 ./scripts/dev smoke` (62 passed); `./scripts/dev check` (875 Rust tests passed, 3 ignored); dual-viewport result inspection; LSP and diff checks — passed. | Pass | Clear | 0 | None |
| PR 1 — Improve Staff Optimizer UX | Commit 9 — Label the result collapse control | `cb065374fab331959cbc2e44892f68522f3df50d` | Added compact visible Collapse/Expand text beside the chevron while preserving ARIA state, result retention, context clearing, new-result expansion, and no rerun. | Focused optimizer tests (16 passed); `CI=1 ./scripts/dev smoke` (62 passed); `./scripts/dev check` (875 Rust tests passed, 3 ignored); four final UI inspections; LSP and diff checks — passed. | Pass | Clear | 0 | Resolved Commit 7's inaccurate collapse-test name within the owning test. |

## Final validation

After all implementation commits complete and before feature review:

1. `./scripts/dev test`
2. `./scripts/dev check-rust`
3. `CI=1 ./scripts/dev smoke`
4. `./scripts/dev check`
5. `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-modal' 1280 800`
6. `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-modal' 1600 900`
7. `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-results' 1280 800`
8. `./scripts/dev inspect-ui '/staff?shortlistOnly=true#inspection-assignment-results' 1600 900`

Open every generated PNG with the image-capable reader. Compare before/after and final captures for action hierarchy, readability, modal density, clipping/overflow, sticky-header behavior where observable, responsive containment, warning/icon clarity, and collapse-label visibility. Before feature review and PR preparation, inspect `git diff --check main...HEAD`, `git diff --stat main...HEAD`, and the complete `git diff main...HEAD`; this range must include `bb41169` and `0386fa5` as well as ledger-controlled commits. Inspect the staged diff with that range during final PR preparation, and describe the two precursor tweaks plus the UX pass in the PR description. Remove `.work/ui-inspection` after inspection. `inspect-ui` is visual evidence only; keep `CI=1 ./scripts/dev smoke` as separate behavior proof.

Mutation testing is not run because `./scripts/dev mutate` is unsupported.

## Documentation impact

Planning changes only the active ledger and TODO. During feature reconciliation, update `.wiki/DESIGN.md` to describe the implemented Staff action hierarchy/readiness, stable status placement, staffing-needs controls/totals/zero rule, grouped warning/disclosed result presentation, sticky headers, and visible collapse label. Update `.wiki/ARCHITECTURE.md` only if implementation changes an implemented data-flow boundary; this plan expects no such change. Move this ledger and reconcile TODO during normal feature close-out. No BACKLOG, ADR, planned spec, release, schema, or migration documentation change is expected.

## Delivered behavior

- Staff Search presents one primary **Optimize assignments** action, secondary **Upload CSV** and **Configure staffing needs** actions, visible readiness and recovery guidance, and a stable status region.
- Staffing needs use bounded step controls with direct entry, a one-time zero rule, and valid-draft team and section totals.
- Results show four primary metrics with supporting counts, consecutive Rust-name groups without a Scope column, warning/icon/text vacancy treatment with **Adjust staffing needs** and **Review shortlist**, native evidence disclosure, sticky headers in the bounded scroller, current-staff steel/icon treatment, and visible **Collapse**/**Expand** labels without rerunning optimization.

## Final architecture

- React remains a presentation layer over the existing Rust assignment DTO, target validation, allocation, and evidence. The feature changes no IPC, persistence, or data-flow boundary.
- No ADR or debug report was needed. Existing boundaries and regression tests explain the implementation.

## Exact implementation refs

**Feature range:** `0312fd1b818435d136df36b1587489a482c239b1..cb065374fab331959cbc2e44892f68522f3df50d`

| Ref | Role |
| --- | --- |
| `0312fd1b818435d136df36b1587489a482c239b1` | Base |
| `bb41169f7a264e1ddf9c08a97d8becfbde2591ef` | Precursor |
| `0386fa563a34ab968583e38a3f1811e0b468e815` | Precursor |
| `036471db667b0e4cafd6794726c07b0c77fa8e51` | Planning |
| `990e47d50dd12251381d386522fec86c873cdbcd` | Content |
| `34ce800e7fe605aea06b7b15339f01224dc9e653` | Content |
| `6a762e1776a1f31d7695fbeb5c381cccabb7a792` | Content |
| `81ed8e1e5c88e36802f76e6f393884d4e3419a09` | Content |
| `d6c2c74fb4b71d0ed1f974c48042eae8892a2720` | Content |
| `9aea5b4c7a8735a51d5df7f822fe671df99fc38a` | Content |
| `64bfcce46da8306bb648fa28b1f6b7f49851fd6d` | Content |
| `cb065374fab331959cbc2e44892f68522f3df50d` | Content |

Correction ref: none. Close-out documentation ref: `d4d38559fa1226717de2ee4f33de63ffd0a0be8b`.

## Final publication

```yaml
status: ready_for_publication
pr_status: open
merge_status: not_merged
pr_ref: "https://github.com/JG1995/fm-valuescout/pull/154"
merge_ref: "Not merged"
branch: feature/staff-optimizer-tweaks
base_branch: main
base_ref: 0312fd1b818435d136df36b1587489a482c239b1
provisional_pr_title: "feat(staff): improve optimizer experience"
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_checks: strict_check
required_check_name: check
pr_count: 1
earlier_prs: none
feature_close_out: current
feature_review_blocking: false
feature_review_recommendation: accept
feature_review_critical: none
feature_review_high: none
feature_review_medium:
  - standalone StaffAssignmentTargetModal fallback pending state remains an unsupported-path advisory
  - Adjust staffing needs proof does not assert a seeded draft value
feature_review_nitpick: none
feature_review_action: skip
feature_review_correction_rounds: 0
ci_repair_rounds: 0
implementation_range: "0312fd1b818435d136df36b1587489a482c239b1..cb065374fab331959cbc2e44892f68522f3df50d"
feature_review_scope: "main...HEAD including precursors and all ledger content commits"
final_pr_commit_set:
  - bb41169f7a264e1ddf9c08a97d8becfbde2591ef
  - 0386fa563a34ab968583e38a3f1811e0b468e815
  - 036471db667b0e4cafd6794726c07b0c77fa8e51
  - 990e47d50dd12251381d386522fec86c873cdbcd
  - 34ce800e7fe605aea06b7b15339f01224dc9e653
  - 6a762e1776a1f31d7695fbeb5c381cccabb7a792
  - 81ed8e1e5c88e36802f76e6f393884d4e3419a09
  - d6c2c74fb4b71d0ed1f974c48042eae8892a2720
  - 9aea5b4c7a8735a51d5df7f822fe671df99fc38a
  - 64bfcce46da8306bb648fa28b1f6b7f49851fd6d
  - cb065374fab331959cbc2e44892f68522f3df50d
  - d4d38559fa1226717de2ee4f33de63ffd0a0be8b
correction_ref: none
close_out_documentation_ref: d4d38559fa1226717de2ee4f33de63ffd0a0be8b
publication_correction_evidence: none
project_fit: conforms

```

## Feature close-out

**State:** Current. Full frontend validation passed (83 files, 1,027 tests), Rust validation passed (875 tests, 3 ignored), smoke passed (62), check passed, four UI inspection captures were opened at both required viewports, and diff checks passed. Mutation testing was unsupported. Chromium/WSL evidence makes no native-WebView claim.

The feature review found no CRITICAL, HIGH, or NITPICK findings. The two implementation MEDIUM advisories remain non-blocking follow-up risks; the documentation MEDIUM is resolved. Test portfolio: Pass. Architecture and project fit: Conforms. No overengineering and zero feature correction rounds.

## Follow-up

- Consider a supported standalone fallback pending state and stronger seeded-draft proof for the Adjust staffing needs path.
- Generated `.work/ui-inspection` artifacts are absent; no removal is needed.
