# Todo UX Quality Pass

## Status

Active

**Ledger schema:** 2

> **Owner:** Main implementation session
> **Created:** 2026-08-22
> **Updated:** 2026-08-22
> **Linear:** [JAY-35](https://linear.app/jaycount/issue/JAY-35), [JAY-36](https://linear.app/jaycount/issue/JAY-36), [JAY-37](https://linear.app/jaycount/issue/JAY-37), [JAY-38](https://linear.app/jaycount/issue/JAY-38), [JAY-39](https://linear.app/jaycount/issue/JAY-39), [JAY-40](https://linear.app/jaycount/issue/JAY-40), [JAY-41](https://linear.app/jaycount/issue/JAY-41)

## Summary

Deliver the seven current Linear Todo issues as four sequential pull requests. The work fixes My Club containment, adds session navigation controls, refines shared table presentation, and makes Moneyball profile comparisons position-specific without mixing unrelated review surfaces in one PR.

The feature starts from clean `main` at `889a9803eb70a92117688d3619ec0c09c4f419db`, synchronized with `origin/main`. Each later PR begins only after the previous PR has merged and local `main` has been synchronized, even though the four product slices have no functional dependency on one another.

## User-visible outcome

- My Club Staff and Staff Shortlist scroll inside their workspaces instead of growing the document, and managed-club search and save controls share one responsive row.
- The app header exposes accessible Back and Forward controls beside global player search for the current session's route history.
- Nationality cells distinguish primary and secondary flags, and player tables use a stacked player identity while retaining configurable metrics.
- Moneyball Player Profile percentiles and role scores use natural-position peers, explain their comparison basis, and keep General and Moneyball header geometry stable.

## Scope

### In scope

- JAY-35 through JAY-41 exactly as recorded above.
- Frontend behavior, Rust query behavior, IPC/type changes, focused regression coverage, browser evidence, and intrinsic current-state documentation reconciliation.
- One-time persisted player-table layout migration for the stacked identity presentation.

### Out of scope

- Moneyball Search percentile population changes.
- Profile or Academy nationality text replacement.
- Staff-table stacked identity rows.
- Persisted route history, custom navigation shortcuts, or explicit page/table scroll restoration.
- A virtualization rewrite, variable-height virtual rows, new dependencies, database migrations, or capability changes.
- Redesign beyond the issue references and the decisions recorded here.

## Invariants

- The Rust layer remains authoritative for Moneyball percentile and role-score calculations.
- Natural position means familiarity exactly `20`; partial familiarity never enters the profile cohort key.
- A player with several natural positions uses the deduplicated union of all matching peers.
- Existing percentile rules remain intact: null samples stay excluded, ties share rank, lower-is-better metrics stay inverted, and a singleton comparable sample remains neutral at `50`.
- Search filters, sorting, bounded page fetching, configurable columns, row activation, keyboard focus, and fixed `40px` virtual row height remain operational.
- The primary nationality stays first. Duplicate names are removed without reordering the first occurrence.
- Router history remains the single navigation source of truth.
- No implementation commit weakens or deletes existing tests to make the change pass.

## Decisions

1. **One ledger, four PRs.** The developer requested one planning record for every current Todo issue while keeping unrelated work separate. Four PRs are warranted because My Club, global navigation, shared player tables, and Moneyball profiles are independent review surfaces with different failure modes and implementation seams.
2. **Sequential publication, no functional dependency.** Only one PR is active at a time. A later PR waits for its predecessor to merge and for synchronized `main`; this is a workflow dependency, not a product dependency.
3. **Natural-position comparison basis is visible.** Moneyball profiles show the subject's matching natural positions and the deduplicated comparison-player count. If the subject has no exact-20 position, raw imported metrics remain visible while percentiles and role scores are explicitly unavailable.
4. **Stable profile footprint.** General and Moneyball summaries occupy the same two-band header footprint. Existing information and toggle focus behavior remain; inaccessible Linear image uploads are not grounds to invent additional design detail.
5. **Session-only history.** Back and Forward operate on TanStack Router history for pathname, search parameters, and hash. They do not persist across restart or own transient component/scroll state.
6. **Forward availability is derived minimally.** Because the installed history exposes `canGoBack` but not `canGoForward`, the app subscribes to history and tracks current and maximum session indices. A new push after navigating back resets the forward boundary.
7. **Secondary flags use the shared table cell.** Exact duplicate nationalities are removed while preserving order. The first flag keeps current emphasis; all later flags are smaller and lower emphasis across every existing shared flag table.
8. **Club and division move into player identity.** Search, Moneyball Search, and Squad show name above `club · division`. A one-time Zustand store-version migration removes the now-duplicate visible Club and Division columns while retaining those metrics in the picker and preserving all other saved order and widths.
9. **Staff scrolling is a containment fix.** The staff tables already use the shared virtualizer. The fix restores a bounded flex chain in both My Club tab panels and proves bounded rendering; it does not replace table infrastructure.
10. **No ADR.** These changes refine existing component, router, persistence, IPC, and calculation seams without introducing a durable architectural choice that is not already governed by current documents.

## Reference evidence

- JAY-36 and JAY-41 contain product reference images. Their upload URLs returned HTTP `401` in the planning environment, so implementation must use the issue acceptance criteria and current component contracts rather than guessing unseen pixels.
- JAY-39 references [`JG1995/fm-valuescout-react`](https://github.com/JG1995/fm-valuescout-react). Its `src/features/scouting-view/components/player-table.tsx` and `docs/images/demo-scouting.png` establish the name-first, club/division-second visual grouping. Current FM ValueScout behavior and design tokens remain authoritative for interaction and accessibility.
- Current-state owners: [ARCHITECTURE.md](../../ARCHITECTURE.md) and [DESIGN.md](../../DESIGN.md).
- Relevant completed records: [Moneyball views](../completed/moneyball-views.md), [Moneyball role scores](../completed/moneyball-role-scores.md), [configurable player tables](../completed/configurable-player-tables.md), [player profile information controls](../completed/player-profile-information-controls.md), [player position familiarity](../completed/player-position-familiarity.md), [My Club workspace](../completed/my-club-workspace.md), [managed club settings](../completed/settings-managed-club.md), [staff workspace](../completed/staff-workspace.md), and [staff shortlist](../completed/staff-shortlist.md).

## Delivery plan

### PR 1 — Contain My Club workspaces and compact club setup

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `fix/my-club-ux-containment`

**Base branch:** `main`

**Provisional PR title:** `fix(club): contain staff tables and compact club setup`

**Purpose:** Fix the two My Club layout defects as one independently reviewable and revertible route-level change.

**Depends on:** None.

**Release intent:** Provisionally `patch` (`0.11.2` from current `v0.11.1`); reclassify from the latest reachable tag and complete unreleased range at publication.

- **Branch:** `fix/my-club-ux-containment`
- **Base branch:** `main`
- **Base ref:** `889a9803eb70a92117688d3619ec0c09c4f419db`
- **Head ref:** Pending branch creation
- **Publication:** GitHub pull request to `JG1995/fm-valuescout`
- **Merge strategy:** Squash
- **Required check:** `check` with strict up-to-date enforcement
- **Release intent:** Provisionally `patch`; expected `0.11.2` from `v0.11.1`, subject to publication-time range inspection.
- **Provisional title:** `fix(club): contain staff tables and compact club setup`
- **Template:** `.github/pull_request_template.md`
- **Issues:** JAY-40, JAY-41
- **Why this boundary:** Both changes are local to the My Club route/header composition and share the same route-level regression surface. They can be reviewed and reverted without touching the global shell, player-table presentation, or Moneyball calculation.
- **Activation dependency:** None; this is the first PR.
- **Close-out owner:** No.

#### Commit 1 — Record the active Todo UX delivery plan

**Status:** Completed

**Provisional commit:** `docs(plan): schedule Todo UX quality pass`

**Work:** Create the active delivery ledger and register it in `.wiki/TODO.md`.

**Out of scope:** Runtime, test, configuration, Linear, and Git-history changes.

**Implementation packet:** Apply the detailed documentation-only packet below.

**Files and responsibilities:** This ledger owns feature intent and delivery; `.wiki/TODO.md` owns the active queue entry.

**Behavior and data flow:** Documentation only; no executable path changes.

**Ordered implementation steps:** Write the two planning files, classify the ledger, inspect the complete diff, and correct deterministic gaps.

**Tests and proof:** The schema classifier must report schema 2 with zero errors and state `runnable`; `git diff --check` must pass for exactly the two planning paths.

**Patterns to verify:** Follow `.wiki/features/active/README.md` schema 2 and existing active-ledger conventions.

**Constraints and non-goals:** Do not edit implementation, current-state docs, BACKLOG, ADRs, or Git state.

**Dependencies and sequencing:** Clean synchronized `main` at the recorded base ref; this packet precedes implementation.

**Validation:** Run the classifier and focused diff checks recorded below.

**Stop conditions:** Stop on classifier errors, base drift, another active ledger, or an expanded path set.

**Review mandate:** Verify complete issue coverage, exact active state, coherent PR boundaries, executable packets, and correct documentation ownership.

- **Provisional subject:** `docs(plan): schedule Todo UX quality pass`
- **Outcome:** Create this active ledger and register the batch in `.wiki/TODO.md` before implementation begins.
- **Files and symbols:**
  - Add `.wiki/features/active/todo-ux-quality-pass.md` as the sole feature intent and delivery owner.
  - Update `.wiki/TODO.md` so `Active` links this ledger while retaining the existing `Next` item.
- **Behavior and data flow:** Documentation only. No runtime, test, configuration, Linear, or Git-history behavior changes.
- **RED:** Not applicable; this is the planning artifact required before the first build commit.
- **GREEN:** Run the schema classifier and inspect the complete documentation diff.
- **Test assets:** Added: this ledger. Modified: `.wiki/TODO.md`. Deleted: none. Retained: all implementation and test files.
- **Patterns and constraints:** Follow `.wiki/features/active/README.md`; do not describe proposed behavior as implemented in current-state documents.
- **Dependencies:** Clean synchronized `main` at the recorded base ref.
- **Soft-size assessment:** Documentation-only and below the normal production-code target.
- **Targeted validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/todo-ux-quality-pass.md`
- **Affected validation:** `git diff --check -- .wiki/TODO.md .wiki/features/active/todo-ux-quality-pass.md`
- **Commit gate:** Not run for the planning artifact until checkpoint; the checkpoint still requires `./scripts/dev check`.
- **Stop conditions:** Stop if the classifier is not schema 2, error-free, and `runnable`; if the worktree base changed; if another active ledger appeared; or if the diff includes any path outside the two approved planning files.
- **Review concerns:**
  - All seven Todo issues are present once.
  - PR and commit states identify exactly one active commit.
  - Later PRs are gated by predecessor merge, not falsely described as functionally dependent.
  - Every implementation outcome has an executable packet and documentation owner.

#### Commit 2 — Restore bounded Staff workspace scrolling

**Status:** Active

**Provisional commit:** `fix(club): contain staff table scrolling`

**Work:** Restore a bounded flex chain for Staff and Staff Shortlist so their existing virtualizers own scrolling.

**Out of scope:** Virtualizer, row-height, page-size, and Staff query rewrites.

**Implementation packet:** Execute the route-containment RED→GREEN packet below.

**Files and responsibilities:** `my-club.tsx` owns tabpanel containment; `my-club-squad.test.tsx` owns route-level regression proof.

**Behavior and data flow:** Active Staff tabpanels bound the existing results panel and virtualized table without changing query behavior.

**Ordered implementation steps:** Add failing containment/bounded-row tests, add the minimum flex class correction, then run affected validation.

**Tests and proof:** Prove both workspaces render a bounded subset for totals larger than the viewport and scroll internally.

**Patterns to verify:** Copy the established bounded flex chain from other My Club workspaces.

**Constraints and non-goals:** Preserve hidden-panel, query, paging, and shared-table behavior.

**Dependencies and sequencing:** Commit 1 complete.

**Validation:** Run the focused route test, shared table contract, and full commit gate recorded below.

**Stop conditions:** Stop if the RED disproves containment as root cause or two bounded corrections fail.

**Review mandate:** Verify both panels, bounded DOM size, internal scrolling, hidden-panel behavior, and absence of shared virtualizer changes.

- **Provisional subject:** `fix(club): contain staff table scrolling`
- **Outcome:** Keep My Club Staff and Staff Shortlist tables within their available workspace height so their existing virtualizers own scrolling.
- **Files and symbols:**
  - `src/app/routes/my-club.tsx`: Staff and Staff Shortlist tabpanel wrapper class chains around `MyClubStaffWorkspace` and `MyClubStaffShortlistWorkspace`.
  - `src/app/routes/my-club-squad.test.tsx`: route-level containment and bounded-row regression scenarios for both workspaces.
  - `.wiki/DESIGN.md`: Staff/My Club scrolling statement, only if the existing current-state text needs correction after the implementation is true.
- **Behavior and data flow:** The active tabpanel becomes a real `flex min-h-0 flex-1 flex-col` container. The existing `StaffSearchResultsPanel` and `ConfigurableVirtualizedTable` then receive a bounded ancestor height and continue to fetch/render only the visible page range.
- **RED:** Add route tests that activate Staff and Staff Shortlist with totals larger than a viewport, assert the tabpanel/table has the containment contract, and assert the DOM contains a bounded subset rather than every result. Run the focused test and confirm failure because the tabpanel lacks the flex containment.
- **GREEN:** Add only the missing flex containment classes. Re-run both new scenarios, then the complete route test.
- **Test assets:** Modified: `src/app/routes/my-club-squad.test.tsx`. Added/deleted: none. Retained: virtualizer contract tests and Staff query behavior.
- **Patterns and constraints:** Reuse the existing Squad/Planner bounded flex chain and existing table test helpers. Do not change `ConfigurableVirtualizedTable`, row height, page size, or Staff data queries unless the RED evidence disproves the diagnosed ancestor break.
- **Dependencies:** Commit 1 complete.
- **Soft-size assessment:** Expected to be a very small production diff plus focused regression coverage; split only if evidence reveals a separate virtualizer defect.
- **Targeted validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx`
- **Affected validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx src/components/player-table/configurable-table-contract.test.tsx`
- **Commit gate:** `./scripts/dev check`
- **Stop conditions:** Stop and replan if bounded rows still fail after two containment corrections, if either Staff surface bypasses `ConfigurableVirtualizedTable`, or if the fix requires variable-height rows or query changes.
- **Review concerns:**
  - Both Staff and Staff Shortlist tabpanels establish the full `min-h-0` flex chain.
  - Hidden tabpanels do not affect layout or fetch ownership.
  - Rendering remains bounded for large totals.
  - The document no longer becomes the table scroll container.
  - No shared virtualization behavior changes incidentally.

#### Commit 3 — Place managed-club save inline with search

**Status:** Pending

**Provisional commit:** `style(club): align managed club save action`

**Work:** Place managed-club search and save controls on one responsive row with feedback below.

**Out of scope:** Picker internals, suggestion ownership, submit logic, and message copy.

**Implementation packet:** Execute the focused responsive-layout packet below.

**Files and responsibilities:** `managed-club-selector.tsx` owns form composition; the My Club route test owns integration proof.

**Behavior and data flow:** Form state and submission remain unchanged while only control layout and wrapping change.

**Ordered implementation steps:** Add a failing structure/order proof, apply the minimum wrapper/classes, and verify route plus browser behavior.

**Tests and proof:** Prove control order, feedback placement, and retained pending/disabled semantics.

**Patterns to verify:** Reuse repository form, responsive flex, spacing, and popover-anchor conventions.

**Constraints and non-goals:** Preserve autocomplete positioning, keyboard order, Enter submit, and narrow-width usability.

**Dependencies and sequencing:** Commit 2 complete; no functional dependency beyond the shared route surface.

**Validation:** Run the focused route test, full gate, and browser evidence recorded below.

**Stop conditions:** Stop if inline composition breaks anchoring, focus order, or feedback visibility.

**Review mandate:** Verify desktop alignment, responsive fallback, picker width, popover anchoring, tab order, and state behavior.

- **Provisional subject:** `style(club): align managed club save action`
- **Outcome:** Put the managed-club picker and save button on one responsive control row while feedback remains below.
- **Files and symbols:**
  - `src/features/managed-club/components/managed-club-selector.tsx`: form layout around `ManagedClubPicker`, `Button`, and feedback paragraphs.
  - `src/app/routes/my-club-squad.test.tsx`: layout semantics, tab order, pending/disabled, and feedback placement scenario.
  - `.wiki/DESIGN.md`: managed-club control composition if current-state text needs the new arrangement recorded.
- **Behavior and data flow:** Form submission and selection state remain unchanged. Only the control wrapper becomes a responsive row at supported widths; the picker retains its positioned suggestion anchor and messages remain block content beneath the row.
- **RED:** Add a focused test asserting picker then submit control order inside a shared control group, with status/alert content outside that row. Preserve existing pending and validation assertions. Confirm failure against the vertical form.
- **GREEN:** Introduce the smallest wrapper/class change that supports inline desktop layout and narrow-width fallback. Re-run route tests and browser smoke.
- **Test assets:** Modified: `src/app/routes/my-club-squad.test.tsx`. Added/deleted: none. Retained: existing managed-club blur, duplicate-save, and suggestion behavior coverage.
- **Patterns and constraints:** Use existing Tailwind tokens and form semantics. Do not change picker internals, suggestion positioning ownership, submit logic, or messages.
- **Dependencies:** Commit 2 complete; no functional dependency beyond the shared route test surface.
- **Soft-size assessment:** Small layout-only production change with one regression scenario.
- **Targeted validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx`
- **Affected validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx`
- **Commit gate:** `./scripts/dev check`
- **Browser evidence:** `./scripts/dev smoke`, supplemented by manual desktop and narrow-width inspection if the smoke fixture does not expose the setup state.
- **Stop conditions:** Stop if an inline wrapper breaks popover anchoring, reorders keyboard focus, obscures feedback, or requires a picker rewrite.
- **Review concerns:**
  - Picker and button align without compressing the field below its usable width.
  - Autocomplete suggestions remain anchored and unobstructed.
  - Narrow layouts wrap predictably.
  - Pending/disabled and Enter-submit behavior are unchanged.
  - Warning and error messages remain associated and visible below the controls.

#### Commit 4 — Prepare the My Club patch release

**Status:** Pending

**Provisional commit:** `chore(release): prepare 0.11.2`

**Work:** Prepare the repository's durable release metadata for PR 1's compatible user-visible fixes.

**Out of scope:** Publishing, tagging, GitHub release creation, unrelated changelog entries, and feature implementation.

**Implementation packet:** At publication time, inspect the latest reachable tag and complete unreleased range. If `patch` remains correct and the latest tag is `v0.11.1`, propose `0.11.2`, the publication date, exact changelog text, and files before editing; obtain the required checkpoint approval, then update the seven release metadata paths.

**Files and responsibilities:** `package.json`, `src-tauri/Cargo.toml`, root `app` entry in `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, and `bridge/FmDataBridge.csproj` must agree on `0.11.2`; `CHANGELOG.md` keeps `Unreleased` and adds the complete dated fix entry; `release-preparation.json` records matching version/intent and increments its positive sequence.

**Behavior and data flow:** Metadata authorizes verified-main release automation for the exact merged source; it does not publish locally.

**Ordered implementation steps:** Reclassify from the complete range; run RED metadata validation; obtain approval; edit exact owners without regenerating Cargo.lock; run GREEN metadata validation and the gate.

**Tests and proof:** `./scripts/dev release-metadata v0.11.1 patch` must fail before preparation for the expected version mismatch and pass afterward with machine-readable `0.11.2` evidence.

**Patterns to verify:** Follow `.agents/skills/create-pr/SKILL.md`, Keep a Changelog ordering, and the prior release-preparation commit.

**Constraints and non-goals:** Stop rather than force `patch` if the complete range requires `minor` or `major`; change only the root Cargo lock entry.

**Dependencies and sequencing:** Commits 2 and 3 validated and reviewed; latest reachable tag still `v0.11.1` or the packet is replanned.

**Validation:** `./scripts/dev release-metadata v0.11.1 patch && ./scripts/dev check`

**Stop conditions:** Latest tag/version drift, ambiguous compatibility, incomplete changelog range, or missing approval.

**Review mandate:** Verify intent classification, all five version owners, root-only lock edit, changelog completeness/date, release-preparation sequence, metadata output, and absence of publication side effects.

### PR 2 — Add app history controls

**Status:** Awaiting prior PR merge

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/app-history-controls`

**Base branch:** `main`

**Provisional PR title:** `feat(navigation): add back and forward controls`

**Purpose:** Add global session-history controls at the app-shell seam without coupling them to route-specific work.

**Depends on:** PR 1 merged and synchronized `main` recorded as the base ref.

**Release intent:** Provisionally `minor` (`0.12.0` after expected `v0.11.2`); reclassify from the latest reachable tag and complete unreleased range at publication.

- **Branch:** `feature/app-history-controls`
- **Base branch:** `main`
- **Base ref:** Pending PR 1 merge and synchronized `main`
- **Head ref:** Pending branch creation
- **Publication:** GitHub pull request to `JG1995/fm-valuescout`
- **Merge strategy:** Squash
- **Required check:** `check` with strict up-to-date enforcement
- **Release intent:** Provisionally `minor`; expected `0.12.0` after PR 1 release, subject to publication-time range inspection.
- **Provisional title:** `feat(navigation): add back and forward controls`
- **Template:** `.github/pull_request_template.md`
- **Issue:** JAY-37
- **Why this boundary:** Global route-history state belongs to the app shell and has no shared implementation seam with My Club layout, player-table rendering, or Moneyball calculation.
- **Activation dependency:** PR 1 merged; record its merge commit as this PR's base ref before build.
- **Close-out owner:** No.

#### Commit 1 — Expose session Back and Forward controls

**Status:** Pending

**Provisional commit:** `feat(navigation): add back and forward controls`

**Work:** Add accessible Back and Forward controls synchronized with current-session TanStack Router history.

**Out of scope:** Restart persistence, custom shortcuts, explicit scroll restoration, and a parallel URL stack.

**Implementation packet:** Execute the subscribed-history RED→GREEN packet below.

**Files and responsibilities:** `app-top-bar.tsx` owns controls/state; top-bar and shell-routing tests own behavioral proof.

**Behavior and data flow:** Installed router history emits state; current/max indices derive availability; controls delegate to history back/forward.

**Ordered implementation steps:** Add failing transition/integration tests, implement minimal subscribed state and controls, then run affected validation.

**Tests and proof:** Cover initial state, A→B→C traversal, URL parameters/hash, branching PUSH, labels, focus, and cleanup.

**Patterns to verify:** Follow existing top-bar controls and TanStack router-history test harnesses.

**Constraints and non-goals:** Router history remains authoritative; preserve global search and responsive header behavior.

**Dependencies and sequencing:** PR 1 merged for workflow activation only.

**Validation:** Run focused top-bar/routing tests, affected search tests, gate, and smoke below.

**Stop conditions:** Stop if installed history events cannot distinguish branch-resetting PUSH behavior or state remains stale after two fixes.

**Review mandate:** Verify disabled states, forward truncation, exact URL restoration, subscription cleanup, accessibility, and search regression safety.

- **Provisional subject:** `feat(navigation): add back and forward controls`
- **Outcome:** Add accessible Back and Forward controls immediately after global player search, with availability synchronized to current-session router history.
- **Files and symbols:**
  - `src/app/components/app-top-bar.tsx`: controls, labels/tooltips, history subscription, and session index boundary ownership; extract a small hook beside the component only if tests or hook rules require it.
  - `src/app/app-top-bar.test.tsx`: initial disabled state, push/back/forward transitions, forward reset after a new push, click behavior, labels, and focus.
  - `src/app/app-shell-routing.test.tsx`: route pathname/search/hash restoration through actual app navigation.
  - `.wiki/DESIGN.md`: app-header control order and session-history scope after implementation.
- **Behavior and data flow:** Subscribe to the installed TanStack history. Read the current history index, retain the highest reachable index for the active branch, and derive Back/Forward disabled states. Button activation delegates to `history.back()` and `history.forward()`; normal route navigation continues to push into the same history and truncates the derived forward boundary when it branches.
- **RED:** Add top-bar unit tests and one routing integration path covering A → B → C → Back → Back → Forward, then Back → new route with Forward disabled. Include a URL with search parameters and hash. Confirm failures because controls and forward availability do not exist.
- **GREEN:** Add the minimal subscribed state and two existing-design-system icon buttons in the required order. Re-run focused tests and smoke.
- **Test assets:** Modified: `src/app/app-top-bar.test.tsx`, `src/app/app-shell-routing.test.tsx`. Added/deleted: none unless a focused local hook test is demonstrably clearer. Retained: GlobalPlayerSearch and existing route-history tests.
- **Patterns and constraints:** Use TanStack history as the sole source of navigation actions. Unsubscribe on cleanup. Preserve global-search behavior and top-bar responsive layout. Do not add localStorage, a second URL stack, shortcut listeners, or scroll restoration.
- **Dependencies:** PR 1 merge for workflow activation only.
- **Soft-size assessment:** One coherent frontend commit; expected production code remains well below the soft target.
- **Targeted validation:** `./scripts/dev test src/app/app-top-bar.test.tsx src/app/app-shell-routing.test.tsx`
- **Affected validation:** `./scripts/dev test src/app/app-top-bar.test.tsx src/app/app-shell-routing.test.tsx src/features/search/components/global-player-search.test.tsx`
- **Commit gate:** `./scripts/dev check`
- **Browser evidence:** `./scripts/dev smoke`, plus manual button-state inspection if smoke cannot deterministically seed forward history.
- **Stop conditions:** Stop if the installed history index cannot distinguish POP from a branch-resetting PUSH, if Hash/Search restoration bypasses router history, or if two corrections still leave stale button state.
- **Review concerns:**
  - Initial Back/Forward disabled states reflect the real session stack.
  - A new push after Back cannot expose stale forward entries.
  - Subscriptions are cleaned up and do not duplicate under React Strict Mode.
  - Route parameters and hash return exactly.
  - Controls are adjacent to search, labelled, keyboard operable, and visibly disabled.
  - GlobalPlayerSearch focus and responsive header behavior remain intact.

#### Commit 2 — Prepare the navigation minor release

**Status:** Pending

**Provisional commit:** `chore(release): prepare 0.12.0`

**Work:** Prepare durable release metadata for the compatible Back/Forward capability.

**Out of scope:** Publishing, tagging, GitHub release creation, unrelated changelog entries, and navigation implementation.

**Implementation packet:** At publication time, inspect the latest reachable tag and complete unreleased range. If `minor` remains correct and the expected prior release is `v0.11.2`, propose `0.12.0`, date, exact changelog, and files; obtain approval; update all durable owners and release authorization.

**Files and responsibilities:** The five version owners (`package.json`, `src-tauri/Cargo.toml`, root `app` Cargo.lock entry, `src-tauri/tauri.conf.json`, `bridge/FmDataBridge.csproj`) agree on `0.12.0`; `CHANGELOG.md` records the complete range; `release-preparation.json` matches and increments sequence.

**Behavior and data flow:** Metadata authorizes verified-main release automation after merge; no local publication occurs.

**Ordered implementation steps:** Reclassify; run RED metadata validation; obtain approval; edit exact owners; run GREEN validation and gate.

**Tests and proof:** `./scripts/dev release-metadata v0.11.2 minor` fails before preparation and passes afterward with `0.12.0` evidence.

**Patterns to verify:** Follow create-pr, prior release metadata, and Keep a Changelog conventions.

**Constraints and non-goals:** Do not regenerate Cargo.lock, invent a tag, or proceed on ambiguous `major` compatibility.

**Dependencies and sequencing:** Commit 1 validated/reviewed and PR 1 release/tag available; otherwise replan tag/version arguments.

**Validation:** `./scripts/dev release-metadata v0.11.2 minor && ./scripts/dev check`

**Stop conditions:** Latest tag drift, intent ambiguity, incomplete range, or missing approval.

**Review mandate:** Verify range-based intent, all owners, lock scope, changelog, authorization sequence, validator evidence, and no publication side effects.

### PR 3 — Refine shared player-table presentation

**Status:** Awaiting prior PR merge

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/player-table-presentation`

**Base branch:** `main`

**Provisional PR title:** `feat(tables): stack player identity and nationality flags`

**Purpose:** Deliver the related shared player-row visual refinements and persistence migration in one table-focused review boundary.

**Depends on:** PR 2 merged and synchronized `main` recorded as the base ref.

**Release intent:** Provisionally `minor` (`0.13.0` after expected `v0.12.0`); reclassify from the latest reachable tag and complete unreleased range at publication.

- **Branch:** `feature/player-table-presentation`
- **Base branch:** `main`
- **Base ref:** Pending PR 2 merge and synchronized `main`
- **Head ref:** Pending branch creation
- **Publication:** GitHub pull request to `JG1995/fm-valuescout`
- **Merge strategy:** Squash
- **Required check:** `check` with strict up-to-date enforcement
- **Release intent:** Provisionally `minor`; expected `0.13.0` after PR 2 release, subject to publication-time range inspection.
- **Provisional title:** `feat(tables): stack player identity and nationality flags`
- **Template:** `.github/pull_request_template.md`
- **Issues:** JAY-38, JAY-39
- **Why this boundary:** Both issues change shared row presentation and its table contracts. They can be validated across Search, Squad, and existing Staff nationality consumers without involving the app shell or profile calculation.
- **Activation dependency:** PR 2 merged; record its merge commit as this PR's base ref before build.
- **Close-out owner:** No.

#### Commit 1 — Distinguish secondary nationality flags

**Status:** Pending

**Provisional commit:** `feat(tables): distinguish secondary nationality flags`

**Work:** Deduplicate nationality names and visually distinguish all secondary flags in every shared flag-table consumer.

**Out of scope:** Profile/Academy nationality text, sorting, capping, and row-identity changes.

**Implementation packet:** Execute the shared nationality-cell RED→GREEN packet below.

**Files and responsibilities:** `nationality-cell.tsx` owns dedupe/presentation; its tests and shared table contract own proof.

**Behavior and data flow:** Stable first-occurrence dedupe feeds existing flag rendering with index-based primary/secondary emphasis.

**Ordered implementation steps:** Add failing component cases, implement stable dedupe/styles, then run component and table validation.

**Tests and proof:** Cover single, multiple, duplicate, order, unknown, empty, and accessible-name behavior.

**Patterns to verify:** Reuse existing flag source, row sizing, titles, and table-cell conventions.

**Constraints and non-goals:** Do not change adapters, Staff row layout, asset source, or fixed row height.

**Dependencies and sequencing:** PR 2 merged for workflow activation only.

**Validation:** Run nationality and shared table tests plus the full gate below.

**Stop conditions:** Stop on fixed-row overflow, incompatible nationality identifiers, or application outside flag tables.

**Review mandate:** Verify stable dedupe, primary hierarchy, secondary accessibility, Staff scope, and unchanged empty/unknown behavior.

- **Provisional subject:** `feat(tables): distinguish secondary nationality flags`
- **Outcome:** Render a primary nationality flag at current emphasis followed by every unique secondary flag at reduced size and emphasis in all shared flag-table consumers.
- **Files and symbols:**
  - `src/components/player-table/nationality-cell.tsx`: stable deduplication, primary/secondary presentation, accessible labels/titles.
  - `src/components/player-table/nationality-cell.test.tsx`: single, multiple, duplicate, and order-preservation cases.
  - `src/components/player-table/configurable-table-contract.test.tsx`: retain or extend consumer contract only where needed to prove shared integration.
  - `.wiki/DESIGN.md`: nationality-cell visual hierarchy and shared table scope after implementation.
- **Behavior and data flow:** The component receives the existing nationality-name array, removes later exact duplicates, maps each retained name through the existing flag source, and styles index zero as primary and later entries as secondary. All current consumers inherit the behavior without adapter changes.
- **RED:** Add component tests proving duplicate removal, stable order, unchanged primary emphasis, reduced secondary emphasis, and readable accessible names. Confirm the current identical rendering fails the hierarchy assertions.
- **GREEN:** Apply stable first-occurrence deduplication and index-based styles in the shared component. Re-run its tests and the shared table contract.
- **Test assets:** Modified: `src/components/player-table/nationality-cell.test.tsx`; optionally modified: configurable table contract if it adds meaningful integration proof. Added/deleted: none. Retained: existing flag lookup and empty-state tests.
- **Patterns and constraints:** Keep the current flag asset/emoji mechanism and row-height budget. Do not cap secondary flags, reorder by country, or change profile/Academy text surfaces.
- **Dependencies:** PR 2 merge for workflow activation only.
- **Soft-size assessment:** Small shared-component commit.
- **Targeted validation:** `./scripts/dev test src/components/player-table/nationality-cell.test.tsx`
- **Affected validation:** `./scripts/dev test src/components/player-table/nationality-cell.test.tsx src/components/player-table/configurable-table-contract.test.tsx`
- **Commit gate:** `./scripts/dev check`
- **Stop conditions:** Stop if secondary flags overflow the fixed row height, current data can contain non-name identifiers that break exact deduplication, or shared application changes a non-table surface.
- **Review concerns:**
  - Exact duplicates collapse without changing first-occurrence order.
  - The primary flag remains visually and semantically primary.
  - Secondary emphasis remains legible and accessible.
  - Staff consumers inherit only the flag treatment, not player-row grouping.
  - Empty and unknown-nationality behavior remains unchanged.

#### Commit 2 — Stack player identity and migrate visible layouts

**Status:** Pending

**Provisional commit:** `feat(tables): stack player identity records`

**Work:** Stack name over club/division in three player tables and migrate duplicate visible identity columns without losing other preferences.

**Out of scope:** Staff row grouping, variable heights, metric removal, server sorting/filtering changes, and unrelated layout resets.

**Implementation packet:** Execute the identity-rendering and versioned-layout migration packet below atomically.

**Files and responsibilities:** Search/Squad panels own rendering; the Zustand store owns defaults/migration; store, Search-route, My Club Squad-route, and table-contract tests own proof.

**Behavior and data flow:** Existing DTO identity fields render in one fixed-height cell; persisted hydration removes only duplicate visible Club/Division entries.

**Ordered implementation steps:** Add failing migration/render/activation tests, implement minimal render and migration changes, then validate persistence and virtualization.

**Tests and proof:** Prove custom order/width preservation and picker availability in the store; prove name-first grouping, clean missing-value formatting, and one activation in both Search modes and Squad; keep fixed-height and bounded-row assertions in the generic table contract.

**Patterns to verify:** Match existing two-line row tokens, configurable-column sanitizer, and the old-app visual grouping.

**Constraints and non-goals:** Keep Club/Division registered and sortable; preserve every unrelated saved layout and interaction.

**Dependencies and sequencing:** Commit 1 complete.

**Validation:** Run store/table/nationality tests, gate, smoke, and visual comparison below.

**Stop conditions:** Stop if DTO projection depends on visible columns, safe prior-version migration is ambiguous, row height overflows, or activation duplicates.

**Review mandate:** Verify migration scope, preference preservation, metric availability, missing values, cross-table consistency, virtualization, sorting/filtering, and whole-row interaction.

- **Provisional subject:** `feat(tables): stack player identity records`
- **Outcome:** Show player name above club and division in Search, Moneyball Search, and Squad while preserving configurable metric behavior and removing duplicate visible identity columns once.
- **Files and symbols:**
  - `src/features/search/components/search-results-panel.tsx`: General and Moneyball name-cell renderers receive and present club/division.
  - `src/features/squad/components/squad-overview-panel.tsx`: Squad name-cell renderer uses the same two-line identity pattern.
  - `src/stores/use-player-table-store.ts`: new default visible lists and versioned migration for `search`, `moneyball-search`, and `squad` only.
  - `src/stores/use-player-table-store.test.ts`: old-version migration fixtures, preserved custom order/widths, removed duplicate visible identity columns, and picker availability.
  - `src/app/routes/search.test.tsx`: General and Moneyball Search identity grouping, missing-value formatting, and whole-row activation.
  - `src/app/routes/my-club-squad.test.tsx`: Squad identity grouping, missing-value formatting, and whole-row activation.
  - `src/components/player-table/configurable-table-contract.test.tsx`: fixed-height virtualization, whole-row focus, and bounded rendered rows.
  - `.wiki/DESIGN.md`: Data Table identity hierarchy and configuration behavior.
  - `.wiki/ARCHITECTURE.md`: persisted table-layout version/current defaults if its current-state description becomes stale.
- **Behavior and data flow:** Existing page DTOs continue to carry name, club, and division. The Name renderer displays the three fields as one fixed-height identity cell. Store hydration detects the prior persisted version, removes `club` and `division` from only the three player-table visible arrays, removes their stored widths, preserves all other IDs/order/widths, and leaves the metrics valid for later picker re-addition. New defaults start in the same nonduplicated state.
- **RED:** Add migration fixtures for default-like and custom layouts. Add route-level assertions for General Search, Moneyball Search, and Squad covering two-line identity, absent/null club or division, and one activation from either line. Keep generic table assertions for one `40px` row, whole-record focus, and bounded rendering. Confirm failures against current separate-column rendering/version.
- **GREEN:** Update the identity renderers, defaults, and one-time migration with the smallest shared presentation helper only if three call sites would otherwise diverge. Re-run store, table, and smoke coverage.
- **Test assets:** Modified: `src/stores/use-player-table-store.test.ts`, `src/app/routes/search.test.tsx`, `src/app/routes/my-club-squad.test.tsx`, and `src/components/player-table/configurable-table-contract.test.tsx`. Added: a small identity-cell component test only if a shared component is introduced. Deleted: none. Retained: dynamic column, sorting, query paging, and virtualizer tests.
- **Patterns and constraints:** Follow the existing two-line row token and the referenced visual grouping. Do not make row height variable, remove Club/Division from metric registries, reset unrelated saved layouts, or migrate Staff tables.
- **Dependencies:** Commit 1 complete so final table visuals can be reviewed together.
- **Soft-size assessment:** Expected near the soft limit because presentation and persisted migration must land atomically. Split migration tests from rendering only if either interim commit remains independently correct and user-safe.
- **Targeted validation:** `./scripts/dev test src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx src/components/player-table/configurable-table-contract.test.tsx`
- **Affected validation:** `./scripts/dev test src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx src/components/player-table/configurable-table-contract.test.tsx src/components/player-table/nationality-cell.test.tsx`
- **Commit gate:** `./scripts/dev check`
- **Browser evidence:** `./scripts/dev smoke`, plus visual inspection against the old-app reference for General Search, Moneyball Search, and Squad.
- **Stop conditions:** Stop if current DTOs omit club/division when their visible columns are absent, migration cannot distinguish prior layouts safely, a two-line cell exceeds fixed row height, or one activation causes duplicate navigation.
- **Review concerns:**
  - Migration touches only three player-table layouts and preserves every unrelated column and width.
  - Club and Division remain available and sortable when re-added.
  - Missing club/division renders cleanly without dangling separators.
  - General, Moneyball, and Squad share the same identity hierarchy.
  - Fixed-height virtualization and bounded page fetching remain correct.
  - Sorting and filtering continue to target underlying fields, not rendered combined text.
  - Hover, focus, and click treat both lines as one player record.

#### Commit 3 — Prepare the player-table minor release

**Status:** Pending

**Provisional commit:** `chore(release): prepare 0.13.0`

**Work:** Prepare durable release metadata for the compatible player-table presentation capability.

**Out of scope:** Publishing, tagging, GitHub release creation, unrelated changelog entries, and table implementation.

**Implementation packet:** At publication time, inspect the latest reachable tag and complete unreleased range. If `minor` remains correct and the expected prior release is `v0.12.0`, propose `0.13.0`, date, exact changelog, and files; obtain approval; update all durable owners and release authorization.

**Files and responsibilities:** The five version owners (`package.json`, `src-tauri/Cargo.toml`, root `app` Cargo.lock entry, `src-tauri/tauri.conf.json`, `bridge/FmDataBridge.csproj`) agree on `0.13.0`; `CHANGELOG.md` records nationality, stacked identity, and layout migration accurately; `release-preparation.json` matches and increments sequence.

**Behavior and data flow:** Metadata authorizes verified-main release automation after merge; no local publication occurs.

**Ordered implementation steps:** Reclassify; run RED metadata validation; obtain approval; edit exact owners; run GREEN validation and gate.

**Tests and proof:** `./scripts/dev release-metadata v0.12.0 minor` fails before preparation and passes afterward with `0.13.0` evidence.

**Patterns to verify:** Follow create-pr, prior release metadata, and Keep a Changelog conventions.

**Constraints and non-goals:** Do not regenerate Cargo.lock, omit the persisted-layout behavior from the changelog, or proceed on ambiguous compatibility.

**Dependencies and sequencing:** Commits 1 and 2 validated/reviewed and PR 2 release/tag available; otherwise replan.

**Validation:** `./scripts/dev release-metadata v0.12.0 minor && ./scripts/dev check`

**Stop conditions:** Latest tag drift, intent ambiguity, incomplete range, or missing approval.

**Review mandate:** Verify intent, all owners, lock scope, complete user-visible changelog, authorization sequence, validator evidence, and no publication side effects.

### PR 4 — Refine Moneyball profile comparisons and header

**Status:** Awaiting prior PR merge

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/moneyball-profile-refinements`

**Base branch:** `main`

**Provisional PR title:** `feat(profile): refine Moneyball comparisons and layout`

**Purpose:** Deliver the Rust cohort contract and its complete Player Profile presentation as one profile-focused review boundary.

**Depends on:** PR 3 merged and synchronized `main` recorded as the base ref.

**Release intent:** Provisionally `minor` (`0.14.0` after expected `v0.13.0`); reclassify from the latest reachable tag and complete unreleased range at publication.

- **Branch:** `feature/moneyball-profile-refinements`
- **Base branch:** `main`
- **Base ref:** Pending PR 3 merge and synchronized `main`
- **Head ref:** Pending branch creation
- **Publication:** GitHub pull request to `JG1995/fm-valuescout`
- **Merge strategy:** Squash
- **Required check:** `check` with strict up-to-date enforcement
- **Release intent:** Provisionally `minor`; expected `0.14.0` after PR 3 release, subject to publication-time range inspection.
- **Provisional title:** `feat(profile): refine Moneyball comparisons and layout`
- **Template:** `.github/pull_request_template.md`
- **Issues:** JAY-35, JAY-36
- **Why this boundary:** Natural-position cohort calculation, IPC metadata, profile score availability, and the shared profile header are one Player Profile review surface. They are independent of list-table and shell behavior.
- **Activation dependency:** PR 3 merged; record its merge commit as this PR's base ref before build.
- **Close-out owner:** Yes; after every planned PR has merged, this PR's close-out packet triggers full validation, feature-complete review, documentation reconciliation, and ledger archival.

#### Commit 1 — Compute natural-position profile cohorts

**Status:** Pending

**Provisional commit:** `feat(moneyball): compare profile metrics by natural position`

**Work:** Recompute profile percentiles/role scores over deduplicated exact-natural-position peers and expose basis metadata.

**Out of scope:** Search cohorts, persisted recalculation, schema migration, new scoring rules, and frontend basis rendering.

**Implementation packet:** Execute the Rust query/IPC/type RED→GREEN packet below.

**Files and responsibilities:** Rust query owns cohort/calculation; command DTO and TS types own the additive boundary contract; architecture owns implemented truth.

**Behavior and data flow:** For an existing subject Moneyball row, exact-20 positions select a current-snapshot peer union; existing percentile logic recomputes scores; an explicit available or unavailable-no-natural-position basis crosses IPC without persistence writes. The existing no-data response remains authoritative when the subject has no Moneyball row.

**Ordered implementation steps:** Add failing Rust cohort/state tests, implement one bounded load and recomputation, update additive contracts, then run Rust/frontend validation.

**Tests and proof:** Cover one/multiple positions, overlap dedupe, partial exclusion, snapshot isolation, null/tie/inversion/singleton rules, and no-natural unavailable output.

**Patterns to verify:** Reuse positions parsing, parameterized query patterns, `calculate_percentiles`, role explanation logic, and additive IPC serialization.

**Constraints and non-goals:** Rust remains authoritative; load the comparison set once; preserve Search and SQLite semantics.

**Dependencies and sequencing:** PR 3 merged for workflow activation only.

**Validation:** Run Rust checks, affected frontend contract tests, and the full gate below.

**Stop conditions:** Stop on ambiguous familiarity data, cohort identity, snapshot ownership, or need for persisted-schema changes.

**Review mandate:** Verify exact familiarity, union dedupe, snapshot scope, reused percentile rules, Search isolation, unavailable semantics, and bounded query shape.

- **Provisional subject:** `feat(moneyball): compare profile metrics by natural position`
- **Outcome:** Recompute a profiled player's Moneyball percentiles and role scores from the active imported snapshot's deduplicated natural-position peer union, and expose the comparison basis over IPC.
- **Files and symbols:**
  - `src-tauri/src/features/moneyball/query.rs`: `get_player_moneyball`, targeted row loaders/helpers, and inline query tests.
  - `src-tauri/src/features/moneyball/percentile.rs`: reuse `calculate_percentiles`; change only if a test proves a reusable input contract is missing.
  - `src-tauri/src/features/moneyball/commands.rs`: additive profile response metadata for natural positions, cohort count, and score availability.
  - `src/features/moneyball/types/moneyball-profile.ts` and `src/features/player-profile/types/player-moneyball.ts`: mirror the additive IPC contract.
  - `.wiki/ARCHITECTURE.md`: replace the full-import Profile percentile claim with current natural-position cohort behavior after implementation.
- **Behavior and data flow:** After the existing query confirms a subject Moneyball row, load the subject's exact-20 positions from `positions_json`. If nonempty, query current-snapshot Moneyball rows joined to players whose parsed familiarity map contains at least one of those positions; deduplicate by player identity before building metric samples. Feed each metric sample through the existing percentile function, then derive role explanations/scores from those recomputed percentiles. Return an explicit available basis with positions and unique cohort count. If that existing subject row has no natural positions, return raw imported metrics with absent percentile/role-score values and an explicit unavailable-no-natural-position basis. Preserve the current no-data response when no subject Moneyball row exists; distinguishing no import from a missing subject row is not part of JAY-35.
- **RED:** Add Rust tests for one natural position, overlapping two-position union without duplicate peers, exclusion of partial-familiarity players, lower-is-better and null preservation through the new population, snapshot/import isolation, singleton neutrality, and no-natural-position unavailable output. Confirm failures because the query returns persisted full-import percentiles.
- **GREEN:** Add the smallest cohort loader/recomputation path and additive response fields, reusing parser and percentile functions. Update TypeScript contracts without rendering the new metadata yet. Run Rust and TypeScript gates.
- **Test assets:** Modified: inline tests in `query.rs`. Added/deleted: none unless a small fixture helper is needed. Retained: `percentile.rs` tie/null/inversion tests and Moneyball Search query tests, which must continue proving full Search population behavior.
- **Patterns and constraints:** Keep SQL parameterized and snapshot-scoped. Do not write recalculated profile values back to SQLite. Avoid one query per player/metric; load the bounded comparison set once and compute in memory using existing logic.
- **Dependencies:** PR 3 merge for workflow activation only.
- **Soft-size assessment:** Likely near or above the soft production target because query, DTO, and contracts form one atomic behavior. Keep helper extraction minimal and split only if the additive contract remains backward-compatible and each commit is testable.
- **Targeted validation:** `./scripts/dev check-rust`
- **Affected validation:** `./scripts/dev check-rust && ./scripts/dev test src/features/moneyball/components/moneyball-profile-panel.test.tsx src/features/moneyball/components/moneyball-role-fit-panel.test.tsx src/app/routes/players.\$uid.test.tsx`
- **Commit gate:** `./scripts/dev check`
- **Stop conditions:** Stop if `positions_json` semantics differ from exact familiarity maps, imports do not provide a stable active-snapshot cohort key, deduplication identity is ambiguous, or the solution requires persisted-schema changes.
- **Review concerns:**
  - Cohort membership requires exact familiarity `20` and at least one shared subject position.
  - Multi-position matches cannot count one comparison player twice.
  - The subject and peers stay inside the active snapshot/import contract.
  - Existing null, tie, inversion, and neutral rules are reused rather than reimplemented.
  - Moneyball Search remains on its current population.
  - No-natural-position output cannot masquerade as a neutral score.
  - Query count and memory shape remain bounded by one imported snapshot.

#### Commit 2 — Explain profile comparison basis and unavailable scores

**Status:** Pending

**Provisional commit:** `feat(profile): explain Moneyball comparison basis`

**Work:** Render natural positions/cohort count and honest unavailable percentile/role states while retaining raw metrics.

**Out of scope:** Browser-side cohort calculation, fallback to persisted scores, header geometry, and Search UI changes.

**Implementation packet:** Execute the Moneyball panel/route RED→GREEN packet below.

**Files and responsibilities:** Moneyball panels own basis/unavailable presentation; the player route owns composition; tests own state distinctions.

**Behavior and data flow:** Additive IPC metadata flows through the route; valid cohorts show basis/scores, while no-natural cohorts show raw values with scores unavailable.

**Ordered implementation steps:** Add failing valid/unavailable panel and route tests, render through existing patterns, then run affected validation.

**Tests and proof:** Cover single/multiple positions, deduped count, raw metric retention, no score leakage, and distinct loading/no-data states.

**Patterns to verify:** Reuse existing profile status, metric, role explanation, typography, and data-honesty conventions.

**Constraints and non-goals:** Do not calculate basis in React. Preserve the existing no-data state for a missing subject Moneyball row; distinguish only that no-data state from an existing row whose scores are unavailable because it has no natural position.

**Dependencies and sequencing:** Commit 1 additive contract complete.

**Validation:** Run Moneyball panel/role/route tests, check-app, gate, and browser evidence below.

**Stop conditions:** Stop if the additive basis cannot distinguish an existing raw-metric row with no natural position from the existing no-data response.

**Review mandate:** Verify backend/UI basis agreement, raw-value retention, unavailable score integrity, role explanation source, state distinction, and toggle regression safety.

- **Provisional subject:** `feat(profile): explain Moneyball comparison basis`
- **Outcome:** Show natural positions and comparison-player count in Moneyball Player Profile, and clearly mark percentile/role scores unavailable when no natural cohort exists while retaining raw metrics.
- **Files and symbols:**
  - `src/features/moneyball/components/moneyball-profile-panel.tsx` and `.test.tsx`: comparison-basis label, raw metric rendering, unavailable percentile state.
  - `src/features/moneyball/components/moneyball-role-fit-panel.tsx` and `.test.tsx`: unavailable role-score state and retained explanation behavior for valid cohorts.
  - `src/app/routes/players.$uid.tsx` and `src/app/routes/players.$uid.test.tsx`: route composition and mode-level integration.
  - `.wiki/DESIGN.md`: Moneyball profile data-basis and unavailable-state language.
- **Behavior and data flow:** The route passes additive IPC metadata into existing profile panels. A valid cohort renders a concise basis such as natural positions plus unique player count. An unavailable cohort renders raw imported metric values but replaces percentile bars/ranks and role scores with an explicit unavailable message; it never displays stale persisted percentiles.
- **RED:** Add panel and route tests for one/multiple natural positions, cohort count, valid percentile/role rendering, no-natural-position raw metrics, and unavailable scores. Confirm current panels cannot render basis or unavailable state.
- **GREEN:** Render the new metadata through existing typography, metric, and status patterns. Keep valid-cohort score explanations unchanged apart from the clarified basis.
- **Test assets:** Modified: `moneyball-profile-panel.test.tsx`, `moneyball-role-fit-panel.test.tsx`, `players.$uid.test.tsx`. Added/deleted: none. Retained: profile mode toggle, loading, no-data, and role explanation tests.
- **Patterns and constraints:** Use existing data-honesty wording and status components. Do not synthesize a cohort count in the browser or fall back to persisted import-wide scores.
- **Dependencies:** Commit 1 additive contract complete.
- **Soft-size assessment:** One focused frontend integration commit below the soft target.
- **Targeted validation:** `./scripts/dev test src/features/moneyball/components/moneyball-profile-panel.test.tsx src/features/moneyball/components/moneyball-role-fit-panel.test.tsx src/app/routes/players.\$uid.test.tsx`
- **Affected validation:** Same focused set plus `./scripts/dev check-app`.
- **Commit gate:** `./scripts/dev check`
- **Browser evidence:** `./scripts/dev smoke`, plus manual valid-cohort and no-natural-position profile inspection if smoke data cannot represent both.
- **Stop conditions:** Stop if additive contract fields cannot distinguish an existing subject Moneyball row with no natural position from the current no-data response for a missing row.
- **Review concerns:**
  - Basis text matches the backend cohort, including deduplicated count.
  - Raw metrics remain visible when scores are unavailable.
  - No stale or neutral-looking percentile/role score leaks into the unavailable state.
  - Valid role explanations still use the recomputed percentile values.
  - Existing loading/no-data behavior remains intact, while an existing raw-metric row with no natural position is a distinct unavailable-score state.
  - Toggle and keyboard behavior remain unchanged.

#### Commit 3 — Stabilize General and Moneyball header geometry

**Status:** Pending

**Provisional commit:** `fix(profile): stabilize analysis header layout`

**Work:** Give General and Moneyball profile summaries one stable two-band footprint and toggle position.

**Out of scope:** Information removal, hard-coded content offsets, table layout, and unseen reference-image invention.

**Implementation packet:** Execute the shared profile-composition RED→GREEN packet below.

**Files and responsibilities:** The player route/shared overview own geometry; the route test and browser evidence own stability proof.

**Behavior and data flow:** Both modes use the same outer bands while only their inner summary content changes; toggle focus remains stable.

**Ordered implementation steps:** Add failing shared-landmark/focus tests and visual proof, refactor minimal composition, then run profile validation.

**Tests and proof:** Prove invariant order/landmarks, retained data, focus, long-content wrapping, and equal supported-viewport geometry.

**Patterns to verify:** Reuse current profile cards, grids, spacing tokens, and accessible analysis-tab conventions.

**Constraints and non-goals:** Preserve all current data and avoid duplicated full summaries or route-specific pixel patches.

**Dependencies and sequencing:** Commit 2 complete so final Moneyball basis participates in the footprint.

**Validation:** Run route and Moneyball panel tests, gate, smoke, and manual side-by-side evidence below.

**Stop conditions:** Stop if equality requires information loss, focus still moves after two corrections, or composition duplicates whole summaries.

**Review mandate:** Verify stable toggle position, complete information, long/narrow layout, focus semantics, and no spillover into list tables.

- **Provisional subject:** `fix(profile): stabilize analysis header layout`
- **Outcome:** Give General and Moneyball Player Profile overview modes one stable two-band footprint so the analysis toggle no longer shifts.
- **Files and symbols:**
  - `src/app/routes/players.$uid.tsx`: shared profile-header composition around `GeneralPlayerProfile`, `MoneyballPlayerProfile`, and `PlayerAnalysisTabs`.
  - `src/features/player-profile/components/player-overview-panel.tsx`: General/Moneyball summary-grid geometry only if the shared route wrapper cannot establish equal footprint cleanly.
  - `src/app/routes/players.$uid.test.tsx`: stable landmarks/order, retained information, toggle focus, and both-mode states.
  - `.wiki/DESIGN.md`: common two-band profile header geometry after implementation.
- **Behavior and data flow:** Both profile modes render the same outer header bands and reserve the same summary footprint. Mode-specific content changes inside those bands; the toggle remains in one DOM/layout position and keeps focus when switching.
- **RED:** Extend route tests to assert shared header landmarks and invariant control order across modes, retained General and Moneyball information, and focused toggle after mode change. Add browser visual evidence for equal geometry because DOM tests alone do not prove no movement.
- **GREEN:** Refactor only the shared composition/spacing required to produce the fixed footprint. Re-run profile tests and smoke.
- **Test assets:** Modified: `src/app/routes/players.$uid.test.tsx`. Added/deleted: none. Retained: General profile, Moneyball panel, information controls, and routing tests.
- **Patterns and constraints:** Follow current `PlayerOverviewPanel` card and spacing tokens. Do not hide existing data, hard-code content-specific pixel offsets, or infer inaccessible reference-image detail.
- **Dependencies:** Commit 2 complete so the final Moneyball basis content participates in the stable footprint.
- **Soft-size assessment:** Small-to-medium presentation refactor below the soft target.
- **Targeted validation:** `./scripts/dev test src/app/routes/players.\$uid.test.tsx`
- **Affected validation:** `./scripts/dev test src/app/routes/players.\$uid.test.tsx src/features/moneyball/components/moneyball-profile-panel.test.tsx src/features/moneyball/components/moneyball-role-fit-panel.test.tsx`
- **Commit gate:** `./scripts/dev check`
- **Browser evidence:** `./scripts/dev smoke` and manual side-by-side General/Moneyball inspection at the supported viewport.
- **Stop conditions:** Stop if equal footprint requires dropping information, if focus moves on toggle after two corrections, or if the current component boundary would duplicate whole profile summaries.
- **Review concerns:**
  - Toggle position is stable between modes.
  - Both modes retain every current datum plus the new Moneyball basis.
  - Shared height works for long names and multiple natural positions.
  - Focus stays on the activated toggle and accessible tab semantics remain correct.
  - Narrow layout wraps without overlap or clipped content.
  - The change does not leak profile-specific spacing into list tables.

#### Commit 4 — Prepare the Moneyball profile minor release

**Status:** Pending

**Provisional commit:** `chore(release): prepare 0.14.0`

**Work:** Prepare durable release metadata for natural-position comparisons and stable profile layout.

**Out of scope:** Publishing, tagging, GitHub release creation, unrelated changelog entries, implementation, and feature-ledger archival.

**Implementation packet:** At publication time, inspect the latest reachable tag and complete unreleased range. If `minor` remains correct and the expected prior release is `v0.13.0`, propose `0.14.0`, date, exact changelog, and files; obtain approval; update all durable owners and release authorization.

**Files and responsibilities:** The five version owners (`package.json`, `src-tauri/Cargo.toml`, root `app` Cargo.lock entry, `src-tauri/tauri.conf.json`, `bridge/FmDataBridge.csproj`) agree on `0.14.0`; `CHANGELOG.md` records cohort basis, unavailable behavior, and profile geometry; `release-preparation.json` matches and increments sequence.

**Behavior and data flow:** Metadata authorizes verified-main release automation after merge; close-out documentation remains a separate final workflow action.

**Ordered implementation steps:** Reclassify; run RED metadata validation; obtain approval; edit exact owners; run GREEN validation and gate.

**Tests and proof:** `./scripts/dev release-metadata v0.13.0 minor` fails before preparation and passes afterward with `0.14.0` evidence.

**Patterns to verify:** Follow create-pr, prior release metadata, and Keep a Changelog conventions.

**Constraints and non-goals:** Do not regenerate Cargo.lock, archive the ledger early, or proceed on ambiguous compatibility.

**Dependencies and sequencing:** Commits 1–3 validated/reviewed and PR 3 release/tag available; otherwise replan.

**Validation:** `./scripts/dev release-metadata v0.13.0 minor && ./scripts/dev check`

**Stop conditions:** Latest tag drift, intent ambiguity, incomplete range, or missing approval.

**Review mandate:** Verify intent, all owners, lock scope, complete changelog, authorization sequence, validator evidence, close-out separation, and no publication side effects.

## Validation contract

### Per commit

1. Run the packet's focused RED test and confirm it fails for the expected missing behavior.
2. Make the minimum coherent GREEN change.
3. Run the packet's affected tests.
4. Run `./scripts/dev format <changed frontend paths>` for frontend paths and `./scripts/dev format` when Rust is changed.
5. Run LSP/diagnostic checks on edited source before the build gate.
6. Run `./scripts/dev check` before checkpoint.
7. Obtain a fresh read-only Sol Medium review for every non-trivial staged change.

### Per PR

- Inspect `git status`, complete diff, `git diff --check`, and staged diff/stat.
- Run `./scripts/dev check-app` for frontend-only PRs; PR 4 also runs `./scripts/dev check-rust`.
- Run `./scripts/dev smoke` for the route/layout behavior in every PR.
- Use `.agents/skills/create-pr` and `.github/pull_request_template.md` for human-authored publication.
- Reclassify release intent from the latest reachable tag and complete unreleased user-visible range; obtain approval, prepare all durable version/changelog/authorization owners, and run `./scripts/dev release-metadata <latest-tag> <intent>` for every release-bearing PR.
- Wait for required strict status `check` before squash merge.

### Feature close-out

After PR 4 is merged and `main` is synchronized:

1. Confirm every acceptance criterion and all four merge commits in this ledger.
2. Run full `./scripts/dev test`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke`.
3. Run the feature-complete fresh Sol xhigh review.
4. Dispatch documentation reconciliation so ARCHITECTURE/DESIGN describe only implemented truth, TODO no longer marks the feature Active, and this ledger moves to `.wiki/features/completed/`.
5. Run the ledger classifier in terminal state and verify no temporary planning artifacts remain.

## Documentation impact

- `.wiki/TODO.md`: planning activation now; removal/next-state reconciliation only at feature completion.
- `.wiki/ARCHITECTURE.md`: update in the commit that changes persisted player-table layout defaults/version and in the commit that changes profile percentile query behavior.
- `.wiki/DESIGN.md`: update alongside each implemented UI contract: My Club containment/setup controls, app-header history controls, nationality hierarchy, stacked player identity, Moneyball basis/unavailable state, and stable profile header.
- Each PR's final release-preparation packet updates `package.json`, `src-tauri/Cargo.toml`, the root `app` entry in `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `bridge/FmDataBridge.csproj`, `CHANGELOG.md`, and `release-preparation.json` after publication-time intent validation and approval.
- No `.wiki/CONCEPT.md`, `.wiki/BACKLOG.md`, ADR, or debug report change is planned.

## Risks and mitigations

- **False virtualization fix:** The visible symptom could include a second ancestor constraint. The RED test must prove the diagnosed tabpanel break before production changes; stop rather than rewrite the virtualizer if the test disproves it.
- **History index drift:** POP/PUSH semantics can desynchronize a hand-built stack. Track only current/max indices from installed history events and integration-test branching after Back.
- **Persisted layout loss:** A broad migration could reset user choices. Fixtures must prove only Club/Division removal in three player tables and preservation of all other order/widths.
- **Row-height overflow:** Secondary flags and stacked identity must stay inside the existing `40px` row. Browser evidence and bounded-row tests are required.
- **Cohort double counting:** SQL joins across several natural positions can duplicate peers. Deduplicate by stable player identity before percentile samples and prove overlap in Rust tests.
- **Misleading unavailable values:** Reusing persisted percentiles for no-natural-position players would violate data honesty. The backend contract and frontend tests must distinguish raw metric availability from score availability.
- **Reference-image ambiguity:** Two Linear uploads are inaccessible. Acceptance criteria and existing design contracts govern; do not infer unseen spacing or controls.

## Uncertainties and stop conditions

- Confirm the exact TanStack history event shape during PR 2 RED. If it does not expose sufficient index/action information, stop for a bounded technical replan rather than adding a parallel router.
- Confirm the player-table DTO always carries club and division independent of visible columns during PR 3 RED. If server projection depends on columns, update the packet before migration.
- Confirm Moneyball `positions_json` is the same familiarity map used by Player Profile position displays. If not, stop because cohort semantics would be ambiguous.
- Confirm the additive comparison-basis contract distinguishes an existing raw-metric row with no natural position from the existing no-data response. Distinguishing no import from a missing subject row remains outside this feature.
- No database, authentication, security, public API, layer-boundary, or dependency decision remains open at planning time.

## Discoveries and replanning

- The feature-scoped planner failed closed repeatedly. At the developer's direction, the main session authored the same approved schema-2 artifact; an independent fresh-context review found three issues, all were corrected, and the focused re-review returned clear.
- Publication review established that every user-visible PR requires release-bearing intent and its own approved release-preparation packet. The delivery plan now records provisional versions and mandatory publication-time reclassification.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Contain My Club workspaces and compact club setup | Commit 1 — Record the active Todo UX delivery plan | Pending record | Added the schema-2 ledger and activated it in `.wiki/TODO.md` | Classifier schema 2, zero errors, `runnable`; `git diff --check` passed | Not applicable | Clear | 1 | Main-session authorship after planner transport failures; no scope change |

## Active work

**PR:** PR 1 — Contain My Club workspaces and compact club setup

**Commit:** Commit 2 — Restore bounded Staff workspace scrolling

### RED proof

Add route-level scenarios for both Staff workspaces with totals larger than the viewport. They must fail against the current tabpanels because their `flex-col` class does not establish a bounded flex container for the existing virtualized table.

### Expected outcome

Staff and Staff Shortlist each retain a bounded DOM row set and scroll inside the active My Club workspace.

### Explicit exclusions

No shared virtualizer, row-height, page-size, Staff query, unrelated My Club, or release-metadata change.

**Next action:** Invoke `$workflow-build` for PR 1 Commit 2 and execute its recorded RED→GREEN packet.

## Progress log

- 2026-08-22 — Confirmed seven Linear Todo issues, clean synchronized `main`, no active ledger, and no overlapping planned specification.
- 2026-08-22 — Inspected current query, profile, app-shell, table, Staff, My Club, persistence, tests, completed records, and the old React table reference.
- 2026-08-22 — Developer accepted natural-position basis details, stacked identity migration, session-only history, and shared all-table secondary nationality treatment.
- 2026-08-22 — Planned four sequential PRs because the batch spans four independent review surfaces.
- 2026-08-22 — Developer accepted the reviewed plan; activated `fix/my-club-ux-containment` and advanced Active work to PR 1 Commit 2.

## Acceptance checklist

- [ ] JAY-40 Staff and Staff Shortlist scroll internally with bounded rendered rows.
- [ ] JAY-41 managed-club picker and save action align inline with feedback below.
- [ ] JAY-37 Back/Forward controls reflect and traverse session route history.
- [ ] JAY-38 all shared flag tables distinguish deduplicated secondary nationalities.
- [ ] JAY-39 General Search, Moneyball Search, and Squad use stacked player identity without losing configurable metrics.
- [ ] JAY-35 Moneyball Player Profile uses deduplicated exact-natural-position cohorts and explains or withholds scores honestly.
- [ ] JAY-36 General and Moneyball overview headers keep one stable footprint and toggle position.
- [ ] All four PRs pass their focused tests, release-metadata validation, `./scripts/dev check`, required `check`, review, and publication contracts.
- [ ] Final validation and documentation reconciliation complete; this ledger is archived.
