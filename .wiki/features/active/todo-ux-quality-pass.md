# Todo UX Quality Pass

## Status

Active

**Ledger schema:** 2

> **Owner:** Main implementation session
> **Created:** 2026-08-22
> **Updated:** 2026-08-23
> **Linear:** [JAY-35](https://linear.app/jaycount/issue/JAY-35), [JAY-36](https://linear.app/jaycount/issue/JAY-36), [JAY-37](https://linear.app/jaycount/issue/JAY-37), [JAY-38](https://linear.app/jaycount/issue/JAY-38), [JAY-39](https://linear.app/jaycount/issue/JAY-39), [JAY-40](https://linear.app/jaycount/issue/JAY-40), [JAY-41](https://linear.app/jaycount/issue/JAY-41)

## Delivery authorization

**Delivery fingerprint:** 4246559c54a277c98780c607212cdf7215ed5896f72cbf146504c6cf6fe7e38f

## Release

**Release intent:** minor

**Release target:** 0.12.0

**Release command:** `bash -lc 'set -euo pipefail; sha=$(git rev-parse HEAD); for _ in {1..120}; do run=$(gh run list --repo JG1995/fm-valuescout --workflow Release --branch main --event workflow_run --limit 20 --json databaseId,headSha --jq ".[] | select(.headSha == \"$sha\") | .databaseId" | head -n1); if [[ -n "$run" ]]; then gh run watch "$run" --repo JG1995/fm-valuescout --exit-status; exit; fi; sleep 10; done; exit 1'`

**Release verification:** `bash -lc 'set -euo pipefail; version=0.12.0; tag=v$version; sha=$(git rev-parse HEAD); tmp=$(mktemp -d); trap '\''rm -rf "$tmp"'\'' EXIT; test "$(gh api repos/JG1995/fm-valuescout/git/ref/tags/$tag --jq .object.sha)" = "$sha"; gh release view "$tag" --repo JG1995/fm-valuescout --json name,body,tagName,targetCommitish,isDraft,isPrerelease,assets > "$tmp/release.json"; gh release download "$tag" --repo JG1995/fm-valuescout --pattern "FM-ValueScout_${version}_x64-setup.exe.sha256" --dir "$tmp"; node --input-type=module -e '\''import { readFileSync } from "node:fs"; import { extractDatedSection } from "./scripts/release-metadata.mjs"; const [sha, version, releasePath, checksumPath] = process.argv.slice(1); const release = JSON.parse(readFileSync(releasePath, "utf8")); const tag = "v" + version; const installer = "FM-ValueScout_" + version + "_x64-setup.exe"; const checksum = installer + ".sha256"; const names = release.assets.map((asset) => asset.name).sort(); const binary = release.assets.find((asset) => asset.name === installer); const notes = extractDatedSection(readFileSync("CHANGELOG.md", "utf8"), version); const checksumText = readFileSync(checksumPath, "utf8").trim(); const valid = release.tagName === tag && release.name === "FM ValueScout " + tag && release.body === notes && release.targetCommitish === sha && release.isDraft === false && release.isPrerelease === false && names.length === 2 && names[0] === installer && names[1] === checksum && /^sha256:[0-9a-f]{64}$/.test(binary?.digest ?? "") && checksumText === binary.digest.slice(7) + " *" + installer; if (!valid) process.exit(1);'\'' "$sha" "$version" "$tmp/release.json" "$tmp/FM-ValueScout_${version}_x64-setup.exe.sha256"'`

Intermediate PRs use release intent `none`. The final PR prepares the complete compatible capability range as `0.12.0`; verified `main` starts publication automatically, and the release command waits for that exact workflow run.

## Intent

Deliver the seven current Linear Todo issues as four sequential pull requests. The work fixes My Club containment, adds session navigation controls, refines shared table presentation, and makes Moneyball profile comparisons position-specific without mixing unrelated review surfaces in one PR.

The feature branch is rebased onto clean `main` at `76cc9f8dbd82504f966038583da4e332e9839c0e`, synchronized with `origin/main`. Each later PR begins only after the previous PR has merged and local `main` has been synchronized, even though the four product slices have no functional dependency on one another.

The feature covers JAY-35 through JAY-41, frontend and Rust behavior, additive IPC/type changes, focused regression coverage, browser evidence, intrinsic current-state documentation, and one persisted player-table layout migration.

## User-visible behavior

- My Club Staff and Staff Shortlist scroll inside their workspaces instead of growing the document, and managed-club search and save controls share one responsive row.
- The app header exposes accessible Back and Forward controls beside global player search for the current session's route history.
- Nationality cells distinguish primary and secondary flags, and player tables use a stacked player identity while retaining configurable metrics.
- Moneyball Player Profile percentiles and role scores use natural-position peers, explain their comparison basis, and keep General and Moneyball header geometry stable.

## Invariants

- The Rust layer remains authoritative for Moneyball percentile and role-score calculations.
- Natural position means familiarity exactly `20`; partial familiarity never enters the profile cohort key.
- A player with several natural positions uses the deduplicated union of all matching peers.
- Existing percentile rules remain intact: null samples stay excluded, ties share rank, lower-is-better metrics stay inverted, and a singleton comparable sample remains neutral at `50`.
- Search filters, sorting, bounded page fetching, configurable columns, row activation, keyboard focus, and fixed `40px` virtual row height remain operational.
- The primary nationality stays first. Duplicate names are removed without reordering the first occurrence.
- Router history remains the single navigation source of truth.
- No implementation commit weakens or deletes existing tests to make the change pass.

## Non-goals

- Change the Moneyball Search percentile population.
- Replace Profile or Academy nationality text.
- Add stacked identity rows to Staff tables.
- Persist route history, add custom navigation shortcuts, or add explicit page/table scroll restoration.
- Rewrite virtualization, add variable-height virtual rows or dependencies, or migrate the database.
- Redesign beyond the issue references and the decisions recorded here.

## Current-state map

- **Relevant components:** `src/app/routes/my-club.tsx`, `src/features/managed-club/components/managed-club-selector.tsx`, `src/app/components/app-top-bar.tsx`, shared player-table components and store, `src/app/routes/players.$uid.tsx`, and the Rust Moneyball query/percentile modules.
- **Data model:** Moneyball profiles read the active imported snapshot. Player-table configuration persists in the Zustand store. Route history is session-only TanStack Router state.
- **Persistence and migrations:** JAY-39 needs one Zustand layout-version migration. No SQLite schema migration or persisted Moneyball recalculation is planned.
- **Existing behavioral assumptions:** Staff tables already use the shared virtualizer; player DTOs carry club and division; position familiarity maps use exact value `20` for natural positions; the installed router history reports enough index/action state to derive a forward boundary.
- **Architectural seams:** My Club owns workspace containment, the app shell owns global history controls, shared table cells/store own row presentation and migration, and Rust owns profile cohort calculations exposed through additive IPC types.
- **Project validation commands:** `./scripts/dev test`, `./scripts/dev check-app`, `./scripts/dev check-rust`, `./scripts/dev check`, `./scripts/dev smoke`, `./scripts/dev release-metadata v0.11.1 none` for intermediate PRs, and `./scripts/dev release-metadata v0.11.1 minor` for the final PR.
- **Primary risks:** A false containment diagnosis, router index drift, persisted-layout loss, fixed-row overflow, cohort double counting, and misleading unavailable scores.

Reference evidence includes JAY-36 and JAY-41 image uploads that returned HTTP `401`; implementation must use their acceptance criteria and current component contracts rather than invent unseen pixels. JAY-39 also references [`JG1995/fm-valuescout-react`](https://github.com/JG1995/fm-valuescout-react), whose player table and demo image establish the name-first, club/division-second grouping. Current FM ValueScout behavior and design tokens remain authoritative.

Current-state owners are [ARCHITECTURE.md](../../ARCHITECTURE.md) and [DESIGN.md](../../DESIGN.md). Relevant completed records are [Moneyball views](../completed/moneyball-views.md), [Moneyball role scores](../completed/moneyball-role-scores.md), [configurable player tables](../completed/configurable-player-tables.md), [player profile information controls](../completed/player-profile-information-controls.md), [player position familiarity](../completed/player-position-familiarity.md), [My Club workspace](../completed/my-club-workspace.md), [managed club settings](../completed/settings-managed-club.md), [staff workspace](../completed/staff-workspace.md), and [staff shortlist](../completed/staff-shortlist.md).

## Feature architecture

The four PRs keep independent review surfaces separate. PR 1 changes My Club route composition only. PR 2 derives navigation availability from TanStack Router history inside the app shell. PR 3 changes shared player-table presentation and its persisted layout contract. PR 4 changes Rust-owned Moneyball profile cohorts, carries their basis through additive IPC types, and presents the result in the profile route. Later PRs start from synchronized `main`; they do not depend on unmerged branches.

## Uncertainty register

### Known

- The latest reachable release tag is `v0.11.1`, and the complete feature adds compatible capabilities, so the single release target is `0.12.0`.
- Verified `main` automatically starts the Release workflow after the required Check succeeds.
- JAY-36 and JAY-41 reference images are unavailable to the planning environment.

### Assumptions

- TanStack history events expose enough index and action information to detect a branch-resetting PUSH.
- Player-table DTOs carry club and division independently of visible-column selection.
- Moneyball `positions_json` uses the same familiarity map as Player Profile position displays.

### Decisions

1. **One ledger, four PRs.** Four PRs separate My Club, global navigation, shared player tables, and Moneyball profiles because they have independent review surfaces and failure modes.
2. **Sequential publication, no functional dependency.** A later PR waits for its predecessor to merge and synchronized `main`; this is a workflow dependency only.
3. **One feature release.** Intermediate PRs use release intent `none`. The final PR prepares and triggers the complete `0.12.0` outcome required by the current ledger contract.
4. **Natural-position comparison basis is visible.** Profiles show matching natural positions and the deduplicated comparison-player count. Without an exact-20 position, raw metrics remain visible while percentile and role scores are unavailable.
5. **Stable profile footprint.** General and Moneyball summaries use the same two-band footprint without removing information or moving toggle focus.
6. **Session-only history.** Back and Forward use TanStack Router history for pathname, search parameters, and hash. They do not persist or own component/scroll state.
7. **Forward availability is derived minimally.** Track current and maximum session indices because installed history has no `canGoForward`; a PUSH after Back resets the forward boundary.
8. **Secondary flags use the shared table cell.** Stable exact-name deduplication preserves order; later flags use reduced emphasis across existing shared flag tables.
9. **Club and division move into player identity.** Search, Moneyball Search, and Squad show name above `club · division`; one Zustand migration removes duplicate visible columns while retaining the metrics and other preferences.
10. **Staff scrolling is a containment fix.** Restore the bounded flex chain around the existing virtualizer instead of replacing table infrastructure.
11. **No ADR.** The work refines current component, router, persistence, IPC, and calculation seams without a new durable architectural boundary.

### Unknowns

- The exact TanStack history event shape must be confirmed during PR 2 RED.
- DTO projection independence from visible columns must be confirmed during PR 3 RED.
- Familiarity-map and additive unavailable-state semantics must be confirmed during PR 4 RED.

### Risks

- The visible Staff symptom could include another ancestor constraint; RED must prove the diagnosed tabpanel break.
- A hand-built forward boundary could drift from router POP/PUSH semantics.
- A broad migration could reset saved table preferences.
- Secondary flags or stacked identity could overflow the fixed `40px` row.
- Multi-position joins could count a Moneyball peer more than once.
- Persisted percentiles could leak a misleading score into the no-natural-position state.

## Walking skeleton

Restore bounded Staff and Staff Shortlist scrolling through the existing My Club route and shared virtualizer, then prove the result with route-level bounded-row tests. This is the smallest end-to-end path and the first implementation commit.

## Delivery plan

### PR 1 — Contain My Club workspaces and compact club setup

**Status:** Merged

**PR ref:** https://github.com/JG1995/fm-valuescout/pull/77

**Merge ref:** 2f7bba077b8907a7671021bfedc36392c8d1804c

**Branch:** `fix/my-club-ux-containment`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** check

**Feature close-out:** Not required

**CI repair rounds:** 0

**Provisional PR title:** `fix(club): contain staff tables and compact club setup`

**Purpose:** Fix JAY-40 and JAY-41 as one reviewable My Club route change. Publish this intermediate PR with release intent `none`.

**Depends on:** None. Base ref: `76cc9f8dbd82504f966038583da4e332e9839c0e`.

#### Commit 1 — Record the active Todo UX delivery plan

**Status:** Completed

**Provisional commit:** `docs(plan): schedule Todo UX quality pass`

**Work:** Create the active delivery ledger and register it in `.wiki/TODO.md`.

**Size assessment:** Estimated 0 changed non-test implementation lines; planning-only commit.

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

#### Commit 2 — Adopt fingerprinted feature delivery

**Status:** Completed

**Provisional commit:** `docs(workflow): align feature release delivery`

**Work:** Migrate the accepted feature plan to the current PI_SETUP ledger structure and its one-release contract.

**Size assessment:** Estimated 0 changed non-test implementation lines; ledger-only planning change.

**Out of scope:** Product behavior, tests, executable release automation, repository-wide workflow guidance, version owners, release metadata, Git publication, and Linear changes.

**Implementation packet:** Preserve JAY-35 through JAY-41 behavior and the accepted four-PR boundaries. Add the Delivery authorization and single Release blocks, complete every PR authority field, consolidate release preparation in the final PR, and permit intermediate user-visible PRs to use `none` only under an unchanged fingerprinted ledger that assigns the release to its final PR.

**Files and responsibilities:** `.wiki/features/active/todo-ux-quality-pass.md` owns the migrated feature authority and packets. The rebased commit does not change repository-wide policy: `AGENTS.md`, `.pi/skills/create-pr/SKILL.md`, `.wiki/notes/early-alpha-release-runbook.md`, and `.github/pull_request_template.md` arrive from synchronized `main` and remain the governing sources.

**Behavior and data flow:** The ledger's fingerprint binds all four PRs, remaining commit packets, one `0.12.0` Release block, and final close-out. Intermediate PRs retain unchanged release owners and use intent `none`; the final PR prepares the complete range for verified-`main` publication.

**Ordered implementation steps:** Compare the active ledger with the current PI_SETUP schema 2 template and synchronized repository guidance; preserve accepted feature intent and active state; add required authority fields and section order; remove intermediate release packets; add the final `0.12.0` packet and complete release evidence; run both classifiers; obtain independent review; record the exact Delivery fingerprint; rerun validation.

**Tests and proof:** `ledger_state.py` must report schema 2, `runnable`, and no errors. `delivery_state.py` must report the recorded fingerprint, `build`, the four exact PRs, nine remaining commits, and no errors. Markdown diagnostics, `git diff --check`, and `./scripts/dev check` must pass.

**Patterns to verify:** Current PI_SETUP active-ledger template, `workflow-plan-feature`, `workflow-deliver-feature`, ADR 0003, repository release automation, local create-PR procedure, and release runbook.

**Constraints and non-goals:** Do not change issue scope, product decisions, implementation order, branch names, merge method, required check, executable release automation, or the four-PR boundary approved by the developer.

**Dependencies and sequencing:** Commit 1 is complete. The developer approved preserving four PRs and assigning the single release to the final PR.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/todo-ux-quality-pass.md && python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/todo-ux-quality-pass.md . && git diff --check && ./scripts/dev check`

**Stop conditions:** Stop if either classifier reports another structural gap, repository release automation cannot support one final release, accepted product scope changes, the required PR boundary changes, or review finds an unresolved policy conflict.

**Review mandate:** Verify template and authority-field conformance, unchanged product intent, four-PR sequencing, intermediate `none` safeguards, final `0.12.0` ownership, command validity, release automation compatibility, and no executable or implementation drift.

#### Commit 3 — Reconcile the active ledger with current PI_SETUP

**Status:** Completed

**Provisional commit:** `docs(plan): reconcile Todo UX delivery ledger`

**Work:** Record the reviewed post-rebase schema 2 corrections before implementation resumes.

**Size assessment:** Estimated 0 changed non-test implementation lines; planning-only commit.

**Out of scope:** Product behavior, tests, executable configuration, PR boundaries, release intent or target, Linear changes, and unrelated documentation.

**Implementation packet:** Commit only the reviewed ledger correction that aligns current branch evidence, release authority, packet ownership, validation commands, sizing, and delivery sequencing with synchronized `main` and current PI_SETUP.

**Files and responsibilities:** `.wiki/features/active/todo-ux-quality-pass.md` owns this corrected feature authority and execution plan. `.wiki/TODO.md` remains unchanged and continues to own the active queue entry. `.wiki/BACKLOG.md`, planned specs, ADRs, implementation, tests, and repository-wide workflow guidance remain unchanged.

**Behavior and data flow:** Documentation only. The correction replaces the prior fingerprint with reviewed authority for one planning commit followed by the unchanged JAY-35 through JAY-41 implementation outcomes.

**Ordered implementation steps:** Verify the rebased branch and base; inspect the complete two-path branch diff; correct stale history, release commands, packet ownership, validation, size, and sequencing; run deterministic validation; obtain independent plan review; record the exact Delivery fingerprint; stage only this ledger after developer approval.

**Tests and proof:** No implementation behavior changes. `ledger_state.py` and `delivery_state.py` must report schema 2, `runnable`, the exact recorded fingerprint, and no errors. The release commands must exactly match the instantiated runbook forms and pass shell syntax validation. Markdown diagnostics, relative-link checks, `git diff --check`, `release-metadata v0.11.1 none`, and the full repository gate must pass.

**Patterns to verify:** `.wiki/features/active/README.md`, the current PI_SETUP schema 2 template and classifiers, ADR 0003, `.pi/skills/create-pr/SKILL.md`, and `.wiki/notes/early-alpha-release-runbook.md`.

**Constraints and non-goals:** Preserve every accepted product decision, issue, PR boundary, branch name, merge method, required check, release target, and implementation order. Do not edit `.wiki/TODO.md` again or change executable files.

**Dependencies and sequencing:** Commits 1–2 are complete. The branch is rebased onto synchronized `main` at `76cc9f8dbd82504f966038583da4e332e9839c0e`, and the developer requested structural reconciliation before implementation resumes.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/todo-ux-quality-pass.md && python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/todo-ux-quality-pass.md . && ./scripts/dev release-metadata v0.11.1 none && git diff --check && ./scripts/dev check`

**Stop conditions:** Stop on changed product scope or PR authority, a classifier or release-command mismatch, another changed path, an uncleared review finding, base drift, or missing developer approval to commit the correction.

**Review mandate:** Verify current history evidence, exact two-path branch scope, complete packet structure, release command parity, current Pi paths and roles, status consistency, unchanged product intent, and executable next action.

#### Commit 4 — Restore bounded Staff workspace scrolling

**Status:** Completed

**Provisional commit:** `fix(club): contain staff table scrolling`

**Work:** Restore a bounded flex chain for Staff and Staff Shortlist so their existing virtualizers own scrolling.

**Size assessment:** Estimated 5 changed non-test implementation lines. Within the soft target.

**Out of scope:** Virtualizer, row-height, page-size, and Staff query rewrites.

**Implementation packet:** Execute the route-containment RED→GREEN packet below.

**Files and responsibilities:** `my-club.tsx` owns tabpanel containment; `my-club-squad.test.tsx` owns route-level regression proof.

**Behavior and data flow:** Active Staff tabpanels bound the existing results panel and virtualized table without changing query behavior.

**Ordered implementation steps:** Add failing containment/bounded-row tests, add the minimum flex class correction, then run affected validation.

**Tests and proof:** Prove both workspaces render a bounded subset for totals larger than the viewport and scroll internally.

**Patterns to verify:** Copy the established bounded flex chain from other My Club workspaces.

**Constraints and non-goals:** Preserve hidden-panel, query, paging, and shared-table behavior.

**Dependencies and sequencing:** Commit 3 complete.

**Validation:** Run the focused route test, shared table contract, and full commit gate recorded below.

**Stop conditions:** Stop if the RED disproves containment as root cause or two bounded corrections fail.

**Review mandate:** Verify both panels, bounded DOM size, internal scrolling, hidden-panel behavior, and absence of shared virtualizer changes.

- **Provisional subject:** `fix(club): contain staff table scrolling`
- **Outcome:** Keep My Club Staff and Staff Shortlist tables within their available workspace height so their existing virtualizers own scrolling.
- **Files and symbols:**
  - `src/app/routes/my-club.tsx`: Staff and Staff Shortlist tabpanel wrapper class chains around `StaffSearchResultsPanel` and `MyClubStaffShortlistWorkspace`.
  - `src/app/routes/my-club-squad.test.tsx`: route-level containment and bounded-row regression scenarios for both workspaces.
  - `.wiki/DESIGN.md`: retain the existing full-height table and no-page-scroll contract unchanged; the implementation restores that documented behavior.
- **Behavior and data flow:** The active tabpanel becomes a real `flex min-h-0 flex-1 flex-col` container. The existing `StaffSearchResultsPanel` and `ConfigurableVirtualizedTable` then receive a bounded ancestor height and continue to fetch/render only the visible page range.
- **RED:** Add route tests that activate Staff and Staff Shortlist with totals larger than a viewport, assert the tabpanel/table has the containment contract, and assert the DOM contains a bounded subset rather than every result. Run the focused test and confirm failure because the tabpanel lacks the flex containment.
- **GREEN:** Add only the missing flex containment classes. Re-run both new scenarios, then the complete route test.
- **Test assets:** Modified: `src/app/routes/my-club-squad.test.tsx`. Added/deleted: none. Retained: virtualizer contract tests and Staff query behavior.
- **Patterns and constraints:** Reuse the existing Squad/Planner bounded flex chain and existing table test helpers. Do not change `ConfigurableVirtualizedTable`, row height, page size, or Staff data queries unless the RED evidence disproves the diagnosed ancestor break.
- **Dependencies:** Commit 3 complete.
- **Soft-size assessment:** Estimated 5 changed non-test implementation lines. Within the soft target.
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

#### Commit 5 — Place managed-club save inline with search

**Status:** Completed

**Provisional commit:** `style(club): align managed club save action`

**Work:** Place managed-club search and save controls on one responsive row with feedback below.

**Size assessment:** Estimated 15 changed non-test implementation lines. Within the soft target.

**Out of scope:** Picker internals, suggestion ownership, submit logic, and message copy.

**Implementation packet:** Execute the focused responsive-layout packet below.

**Files and responsibilities:** `managed-club-selector.tsx` owns form composition; the My Club route test owns integration proof.

**Behavior and data flow:** Form state and submission remain unchanged while only control layout and wrapping change.

**Ordered implementation steps:** Add a failing structure/order proof, apply the minimum wrapper/classes, and verify route plus browser behavior.

**Tests and proof:** Prove control order, feedback placement, and retained pending/disabled semantics.

**Patterns to verify:** Reuse repository form, responsive flex, spacing, and popover-anchor conventions.

**Constraints and non-goals:** Preserve autocomplete positioning, keyboard order, Enter submit, and narrow-width usability.

**Dependencies and sequencing:** Commit 4 complete; no functional dependency beyond the shared route surface.

**Validation:** Run the focused route test, full gate, and browser evidence recorded below.

**Stop conditions:** Stop if inline composition breaks anchoring, focus order, or feedback visibility.

**Review mandate:** Verify desktop alignment, responsive fallback, picker width, popover anchoring, tab order, and state behavior.

- **Provisional subject:** `style(club): align managed club save action`
- **Outcome:** Put the managed-club picker and save button on one responsive control row while feedback remains below.
- **Files and symbols:**
  - `src/features/managed-club/components/managed-club-selector.tsx`: form layout around `ManagedClubPicker`, `Button`, and feedback paragraphs.
  - `src/app/routes/my-club-squad.test.tsx`: layout semantics, tab order, pending/disabled, and feedback placement scenario.
  - `.wiki/DESIGN.md`: record the managed-club picker/save control row and feedback placement after implementation.
- **Behavior and data flow:** Form submission and selection state remain unchanged. Only the control wrapper becomes a responsive row at supported widths; the picker retains its positioned suggestion anchor and messages remain block content beneath the row.
- **RED:** Add a focused test asserting picker then submit control order inside a shared control group, with status/alert content outside that row. Preserve existing pending and validation assertions. Confirm failure against the vertical form.
- **GREEN:** Introduce the smallest wrapper/class change that supports inline desktop layout and narrow-width fallback. Re-run route tests and browser smoke.
- **Test assets:** Modified: `src/app/routes/my-club-squad.test.tsx`. Added/deleted: none. Retained: existing managed-club blur, duplicate-save, and suggestion behavior coverage.
- **Patterns and constraints:** Use existing Tailwind tokens and form semantics. Do not change picker internals, suggestion positioning ownership, submit logic, or messages.
- **Dependencies:** Commit 4 complete; no functional dependency beyond the shared route test surface.
- **Soft-size assessment:** Estimated 15 changed non-test implementation lines. Within the soft target.
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

### PR 2 — Add app history controls

**Status:** Merged

**PR ref:** https://github.com/JG1995/fm-valuescout/pull/78

**Merge ref:** 6797dc0cba262f4de251b241bf686a4537f92918

**Branch:** `feature/app-history-controls`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** check

**Feature close-out:** Not required

**CI repair rounds:** 0

**Provisional PR title:** `feat(navigation): add back and forward controls`

**Purpose:** Deliver JAY-37 at the app-shell seam without coupling it to route-specific work. Publish this intermediate PR with release intent `none`.

**Depends on:** PR 1 merged and synchronized `main`; record PR 1's immutable merge ref before activation.

#### Commit 1 — Expose session Back and Forward controls

**Status:** Completed

**Provisional commit:** `feat(navigation): add back and forward controls`

**Work:** Add accessible Back and Forward controls synchronized with current-session TanStack Router history.

**Size assessment:** Estimated 80 changed non-test implementation lines. Within the soft target.

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
  - `src/app/components/app-top-bar.tsx`: controls, labels/tooltips, history subscription, cleanup, and session index boundary ownership; keep this state beside the component without adding another URL-history abstraction.
  - `src/app/app-top-bar.test.tsx`: initial disabled state, push/back/forward transitions, forward reset after a new push, click behavior, labels, and focus.
  - `src/app/app-shell-routing.test.tsx`: route pathname/search/hash restoration through actual app navigation.
  - `.wiki/DESIGN.md`: app-header control order and session-history scope after implementation.
- **Behavior and data flow:** Subscribe to the installed TanStack history. Read the current history index, retain the highest reachable index for the active branch, and derive Back/Forward disabled states. Button activation delegates to `history.back()` and `history.forward()`; normal route navigation continues to push into the same history and truncates the derived forward boundary when it branches.
- **RED:** Add top-bar unit tests and one routing integration path covering A → B → C → Back → Back → Forward, then Back → new route with Forward disabled. Include a URL with search parameters and hash. Confirm failures because controls and forward availability do not exist.
- **GREEN:** Add the minimal subscribed state and two existing-design-system icon buttons in the required order. Re-run focused tests and smoke.
- **Test assets:** Modified: `src/app/app-top-bar.test.tsx`, `src/app/app-shell-routing.test.tsx`. Added/deleted: none. Retained: GlobalPlayerSearch and existing route-history tests.
- **Patterns and constraints:** Use TanStack history as the sole source of navigation actions. Unsubscribe on cleanup. Preserve global-search behavior and top-bar responsive layout. Do not add localStorage, a second URL stack, shortcut listeners, or scroll restoration.
- **Dependencies:** PR 1 merge for workflow activation only.
- **Soft-size assessment:** Estimated 80 changed non-test implementation lines. Within the soft target.
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

### PR 3 — Refine shared player-table presentation

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/player-table-presentation`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** check

**Feature close-out:** Not required

**CI repair rounds:** 0

**Provisional PR title:** `feat(tables): stack player identity and nationality flags`

**Purpose:** Deliver JAY-38 and JAY-39 as one shared player-row and persistence review boundary. Publish this intermediate PR with release intent `none`.

**Depends on:** PR 2 merged and synchronized `main`; record PR 2's immutable merge ref before activation.

#### Commit 1 — Distinguish secondary nationality flags

**Status:** Completed

**Provisional commit:** `feat(tables): distinguish secondary nationality flags`

**Work:** Deduplicate nationality names and visually distinguish all secondary flags in every shared flag-table consumer.

**Size assessment:** Estimated 25 changed non-test implementation lines. Within the soft target.

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
  - `src/components/player-table/configurable-table-contract.test.tsx`: retain its generic caller-owned catalog, action-cell, and row-activation contract unchanged.
  - `.wiki/DESIGN.md`: nationality-cell visual hierarchy and shared table scope after implementation.
- **Behavior and data flow:** The component receives the existing nationality-name array, removes later exact duplicates, maps each retained name through the existing flag source, and styles index zero as primary and later entries as secondary. All current consumers inherit the behavior without adapter changes.
- **RED:** Add component tests proving duplicate removal, stable order, unchanged primary emphasis, reduced secondary emphasis, and readable accessible names. Confirm the current identical rendering fails the hierarchy assertions.
- **GREEN:** Apply stable first-occurrence deduplication and index-based styles in the shared component. Re-run its tests and the shared table contract.
- **Test assets:** Modified: `src/components/player-table/nationality-cell.test.tsx`. Added/deleted: none. Retained: `src/components/player-table/configurable-table-contract.test.tsx` plus existing flag lookup and empty-state tests.
- **Patterns and constraints:** Keep the current flag asset/emoji mechanism and row-height budget. Do not cap secondary flags, reorder by country, or change profile/Academy text surfaces.
- **Dependencies:** PR 2 merge for workflow activation only.
- **Soft-size assessment:** Estimated 25 changed non-test implementation lines. Within the soft target.
- **Targeted validation:** `./scripts/dev test src/components/player-table/nationality-cell.test.tsx`
- **Affected validation:** `./scripts/dev test src/components/player-table/nationality-cell.test.tsx`
- **Commit gate:** `./scripts/dev check`
- **Stop conditions:** Stop if secondary flags overflow the fixed row height, current data can contain non-name identifiers that break exact deduplication, or shared application changes a non-table surface.
- **Review concerns:**
  - Exact duplicates collapse without changing first-occurrence order.
  - The primary flag remains visually and semantically primary.
  - Secondary emphasis remains legible and accessible.
  - Staff consumers inherit only the flag treatment, not player-row grouping.
  - Empty and unknown-nationality behavior remains unchanged.

#### Commit 2 — Stack player identity and migrate visible layouts

**Status:** Active

**Provisional commit:** `feat(tables): stack player identity records`

**Work:** Stack name over club/division in three player tables and migrate duplicate visible identity columns without losing other preferences.

**Size assessment:** Estimated 160 changed non-test implementation lines. Within the soft target; presentation and persisted migration remain atomic.

**Out of scope:** Staff row grouping, variable heights, metric removal, server sorting/filtering changes, and unrelated layout resets.

**Implementation packet:** Execute the identity-rendering and versioned-layout migration packet below atomically.

**Files and responsibilities:** Search/Squad panels own rendering; the Zustand store owns defaults/migration; store, Search-route, My Club Squad-route, and table-contract tests own proof.

**Behavior and data flow:** Existing DTO identity fields render in one fixed-height cell; persisted hydration removes only duplicate visible Club/Division entries.

**Ordered implementation steps:** Add failing migration/render/activation tests, implement minimal render and migration changes, then validate persistence and virtualization.

**Tests and proof:** Prove custom order/width preservation and picker availability in the store. In the Search and My Club route suites, prove name-first grouping, clean missing-value formatting, one activation in both Search modes and Squad, fixed `40px` identity rows, keyboard focus, and bounded virtual rendering.

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
  - `src/components/player-table/configurable-table-contract.test.tsx`: retain its generic caller-owned catalog, action-cell, and row-activation contract unchanged; route tests own identity layout, fixed-height, focus, and bounded-rendering proof.
  - `.wiki/DESIGN.md`: Data Table identity hierarchy and configuration behavior.
  - `.wiki/ARCHITECTURE.md`: update the persisted table-layout version and the Search, Moneyball Search, and Squad default visible columns.
- **Behavior and data flow:** Existing page DTOs continue to carry name, club, and division. The Name renderer displays the three fields as one fixed-height identity cell. Store hydration detects the prior persisted version, removes `club` and `division` from only the three player-table visible arrays, removes their stored widths, preserves all other IDs/order/widths, and leaves the metrics valid for later picker re-addition. New defaults start in the same nonduplicated state.
- **RED:** Add migration fixtures for default-like and custom layouts. Add route-level assertions for General Search, Moneyball Search, and Squad covering two-line identity, absent/null club or division, one `40px` row, whole-record focus, bounded rendering, and one activation from either line. Confirm failures against current separate-column rendering/version.
- **GREEN:** Update both existing name renderers, defaults, and the one-time migration directly. Do not add a shared identity component. Re-run store, table, and smoke coverage.
- **Test assets:** Modified: `src/stores/use-player-table-store.test.ts`, `src/app/routes/search.test.tsx`, and `src/app/routes/my-club-squad.test.tsx`. Added/deleted: none. Retained: `src/components/player-table/configurable-table-contract.test.tsx` and existing dynamic-column, sorting, query-paging, and virtualizer tests.
- **Patterns and constraints:** Follow the existing two-line row token and the referenced visual grouping. Do not make row height variable, remove Club/Division from metric registries, reset unrelated saved layouts, or migrate Staff tables.
- **Dependencies:** Commit 1 complete so final table visuals can be reviewed together.
- **Soft-size assessment:** Estimated 160 changed non-test implementation lines. Within the soft target; presentation and persisted migration remain atomic.
- **Targeted validation:** `./scripts/dev test src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx src/components/player-table/configurable-table-contract.test.tsx`
- **Affected validation:** `./scripts/dev test src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx src/components/player-table/nationality-cell.test.tsx`
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

### PR 4 — Refine Moneyball profile comparisons and header

**Status:** Awaiting prior PR merge

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/moneyball-profile-refinements`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** check

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(profile): refine Moneyball comparisons and layout`

**Purpose:** Deliver JAY-35 and JAY-36 as one Rust-to-profile review boundary, prepare the single feature release, and own final close-out.

**Depends on:** PR 3 merged and synchronized `main`; record PR 3's immutable merge ref before activation.

#### Commit 1 — Compute natural-position profile cohorts

**Status:** Pending

**Provisional commit:** `feat(moneyball): compare profile metrics by natural position`

**Work:** Recompute profile percentiles/role scores over deduplicated exact-natural-position peers and expose basis metadata.

**Size assessment:** Estimated 220–280 changed non-test implementation lines. This may exceed the soft target because the query behavior, additive IPC contract, and frontend type/mock boundary must land atomically.

**Out of scope:** Search cohorts, persisted recalculation, schema migration, new scoring rules, and frontend basis rendering.

**Implementation packet:** Execute the Rust query/IPC/type RED→GREEN packet below.

**Files and responsibilities:** Rust query owns cohort/calculation; command DTO and TS types own the additive boundary contract; architecture owns implemented truth.

**Behavior and data flow:** For an existing subject Moneyball row, exact-20 positions select a current-snapshot peer union; existing percentile logic recomputes scores; an explicit available or unavailable-no-natural-position basis crosses IPC without persistence writes. The existing no-data response remains authoritative when the subject has no Moneyball row.

**Ordered implementation steps:** Add failing Rust cohort/state tests, implement one bounded load and recomputation, update additive contracts, then run Rust/frontend validation.

**Tests and proof:** Cover one/multiple positions, overlap dedupe, partial exclusion, snapshot isolation, null/tie/inversion/singleton rules, and no-natural unavailable output. Add inline `commands.rs` conversion and serialization tests for available and unavailable comparison-basis responses, including exact camelCase field names.

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
  - `src-tauri/src/features/moneyball/percentile.rs`: retain `calculate_percentiles` and its existing tie, null, inversion, and singleton rules unchanged.
  - `src-tauri/src/features/moneyball/commands.rs`: add profile response metadata for natural positions, cohort count, and score availability.
  - `src/features/moneyball/types/moneyball-profile.ts`: mirror the additive IPC contract as the sole frontend type owner.
  - `src/testing/moneyball-ipc-mock.ts`: extend the ready-profile fixture with the comparison basis and unavailable-score state.
  - `.wiki/ARCHITECTURE.md`: replace the full-import Profile percentile claim with current natural-position cohort behavior after implementation.
- **Behavior and data flow:** After the existing query confirms a subject Moneyball row, load the subject's exact-20 positions from `positions_json`. If nonempty, query current-snapshot Moneyball rows joined to players whose parsed familiarity map contains at least one of those positions; deduplicate by player identity before building metric samples. Feed each metric sample through the existing percentile function, then derive role explanations/scores from those recomputed percentiles. Return an explicit available basis with positions and unique cohort count. If that existing subject row has no natural positions, return raw imported metrics with absent percentile/role-score values and an explicit unavailable-no-natural-position basis. Preserve the current no-data response when no subject Moneyball row exists; distinguishing no import from a missing subject row is not part of JAY-35.
- **RED:** Add Rust query tests for one natural position, overlapping two-position union without duplicate peers, exclusion of partial-familiarity players, lower-is-better and null preservation through the new population, snapshot/import isolation, singleton neutrality, and no-natural-position unavailable output. Add `commands.rs` tests that fail until available and unavailable basis metadata converts and serializes with the exact camelCase fields. Confirm query failures because the current path returns persisted full-import percentiles and boundary failures because the additive DTO fields do not exist.
- **GREEN:** Add the smallest cohort loader/recomputation path and additive response fields, reusing parser and percentile functions. Update TypeScript contracts without rendering the new metadata yet. Run Rust and TypeScript gates.
- **Test assets:** Modified: inline tests in `src-tauri/src/features/moneyball/query.rs`, inline conversion/serialization tests in `src-tauri/src/features/moneyball/commands.rs`, and the shared ready-profile fixture in `src/testing/moneyball-ipc-mock.ts`. Added/deleted: none. Retained: `percentile.rs` tie/null/inversion tests and Moneyball Search query tests, which continue to prove full Search population behavior.
- **Patterns and constraints:** Keep SQL parameterized and snapshot-scoped. Do not write recalculated profile values back to SQLite. Avoid one query per player/metric; load the bounded comparison set once and compute in memory using existing logic.
- **Dependencies:** PR 3 merge for workflow activation only.
- **Soft-size assessment:** Estimated 220–280 changed non-test implementation lines. The atomic Rust query, IPC DTO, frontend type, and shared mock contract justify exceeding the soft target.
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

**Size assessment:** Estimated 90 changed non-test implementation lines. Within the soft target.

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
- **Soft-size assessment:** Estimated 90 changed non-test implementation lines. Within the soft target.
- **Targeted validation:** `./scripts/dev test src/features/moneyball/components/moneyball-profile-panel.test.tsx src/features/moneyball/components/moneyball-role-fit-panel.test.tsx src/app/routes/players.\$uid.test.tsx`
- **Affected validation:** `./scripts/dev test src/features/moneyball/components/moneyball-profile-panel.test.tsx src/features/moneyball/components/moneyball-role-fit-panel.test.tsx src/app/routes/players.\$uid.test.tsx && ./scripts/dev check-app`.
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

**Size assessment:** Estimated 80 changed non-test implementation lines. Within the soft target.

**Out of scope:** Information removal, hard-coded content offsets, table layout, and unseen reference-image invention.

**Implementation packet:** Execute the shared profile-composition RED→GREEN packet below.

**Files and responsibilities:** The player route/shared overview own geometry; the route test and browser evidence own stability proof.

**Behavior and data flow:** Both modes use the same outer bands while only their inner summary content changes; toggle focus remains stable.

**Ordered implementation steps:** Add failing shared-landmark/focus tests and visual proof, refactor minimal composition, then run profile validation.

**Tests and proof:** Prove invariant order/landmarks, retained data, focus, long-content wrapping, and equal supported-viewport geometry. Manually compare General and Moneyball at 1280×800 and 1600×900 at normal scale and actual 200% browser zoom; the header, toggle, and content must remain usable without clipping or overlap.

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
  - `src/features/player-profile/components/player-overview-panel.tsx`: align General and Moneyball summary-grid bands while preserving each mode's content.
  - `src/app/routes/players.$uid.test.tsx`: stable landmarks/order, retained information, toggle focus, and both-mode states.
  - `.wiki/DESIGN.md`: common two-band profile header geometry after implementation.
- **Behavior and data flow:** Both profile modes render the same outer header bands and reserve the same summary footprint. Mode-specific content changes inside those bands; the toggle remains in one DOM/layout position and keeps focus when switching.
- **RED:** Extend route tests to assert shared header landmarks and invariant control order across modes, retained General and Moneyball information, and focused toggle after mode change. Add browser visual evidence for equal geometry because DOM tests alone do not prove no movement.
- **GREEN:** Refactor only the shared composition/spacing required to produce the fixed footprint. Re-run profile tests and smoke.
- **Test assets:** Modified: `src/app/routes/players.$uid.test.tsx`. Added/deleted: none. Retained: General profile, Moneyball panel, information controls, and routing tests.
- **Patterns and constraints:** Follow current `PlayerOverviewPanel` card and spacing tokens. Do not hide existing data, hard-code content-specific pixel offsets, or infer inaccessible reference-image detail.
- **Dependencies:** Commit 2 complete so the final Moneyball basis content participates in the stable footprint.
- **Soft-size assessment:** Estimated 80 changed non-test implementation lines. Within the soft target.
- **Targeted validation:** `./scripts/dev test src/app/routes/players.\$uid.test.tsx`
- **Affected validation:** `./scripts/dev test src/app/routes/players.\$uid.test.tsx src/features/moneyball/components/moneyball-profile-panel.test.tsx src/features/moneyball/components/moneyball-role-fit-panel.test.tsx`
- **Commit gate:** `./scripts/dev check`
- **Browser evidence:** `./scripts/dev smoke`, then manual side-by-side General/Moneyball inspection at 1280×800 and 1600×900 at normal scale and actual 200% browser zoom.
- **Stop conditions:** Stop if equal footprint requires dropping information, focus moves on toggle after two corrections, either mode clips or overlaps at 200% zoom or 1280×800, or the current component boundary would duplicate whole profile summaries.
- **Review concerns:**
  - Toggle position is stable between modes.
  - Both modes retain every current datum plus the new Moneyball basis.
  - Shared height works for long names and multiple natural positions.
  - Focus stays on the activated toggle and accessible tab semantics remain correct.
  - The 1280×800 layout and actual 200% browser zoom remain usable without overlap, clipping, or toggle movement.
  - The change does not leak profile-specific spacing into list tables.

#### Commit 4 — Prepare the Todo UX minor release

**Status:** Pending

**Provisional commit:** `chore(release): prepare 0.12.0`

**Work:** Prepare durable release metadata for the complete Todo UX Quality Pass.

**Size assessment:** Estimated 10 changed non-test configuration lines, excluding documentation and the lockfile. Within the soft target.

**Out of scope:** Publishing, tagging, GitHub release creation, unrelated changelog entries, implementation, and feature-ledger archival.

**Implementation packet:** After Commits 1–3 are complete, inspect the latest reachable tag and the complete unreleased range across all four feature PRs. If `minor` remains correct and the latest tag is `v0.11.1`, propose `0.12.0`, the publication date, exact changelog text, and exact files before editing. Then update the durable version, changelog, and release-authorization owners in the final PR. Native Windows, installed-app, recovery, bridge, live-FM, and product-flow evidence runs after feature close-out on the exact final PR head.

**Files and responsibilities:** `package.json`, `src-tauri/Cargo.toml`, the root `app` entry in `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, and `bridge/FmDataBridge.csproj` agree on `0.12.0`; `CHANGELOG.md` keeps `Unreleased` and records the complete JAY-35 through JAY-41 user-visible range; `release-preparation.json` records `0.12.0`, intent `minor`, and the next positive sequence.

**Behavior and data flow:** The final reviewed PR carries exact-SHA authorization for verified-`main` publication. Intermediate PRs leave release metadata unchanged and use release intent `none`.

**Ordered implementation steps:** Confirm `v0.11.1` remains the latest reachable tag; inspect the full unreleased range; run RED metadata validation; propose exact metadata changes; update only the seven owners; run GREEN metadata validation and the full gate. Defer native Windows packaging and every installed-app runbook item to post-close-out exact-head validation.

**Tests and proof:** `./scripts/dev release-metadata v0.11.1 minor` must fail before preparation for the expected version mismatch and pass afterward with machine-readable `0.12.0` evidence and the exact dated changelog section. Retain `scripts/release-metadata.test.ts`, `scripts/release-workflow.test.ts`, and `scripts/release-publication-policy.test.ts` unchanged because this packet changes release inputs, not release logic. Add, modify, or delete no test files.

**Patterns to verify:** Follow `.pi/skills/create-pr/SKILL.md`, the early-alpha release runbook, Keep a Changelog ordering, and the prior release-preparation commit.

**Constraints and non-goals:** Do not regenerate Cargo.lock, change a non-root package lock entry, prepare separate releases for intermediate PRs, archive the ledger early, or proceed if the complete range requires another intent or target.

**Dependencies and sequencing:** Commits 1–3 and all earlier PRs are validated, reviewed, merged, and represented in synchronized `main`; the latest reachable tag remains `v0.11.1`.

**Validation:** `./scripts/dev release-metadata v0.11.1 minor && ./scripts/dev check`.

**Stop conditions:** Stop on latest-tag drift, intent or target ambiguity, an incomplete changelog range, missing release-owner evidence, metadata validation failure, or a changed publication contract.

**Review mandate:** Verify range-based intent, all five version owners, root-only lock edit, complete JAY-35 through JAY-41 changelog, authorization sequence, metadata output, Release command and verification parity with the workflow, and no publication side effects.

## Active work

**PR:** PR 3 — Refine shared player-table presentation

**Commit:** Commit 2 — Stack player identity and migrate visible layouts

### RED or removal proof

Add store migration and route tests that fail because Club and Division remain duplicate visible columns and Search, Moneyball Search, and Squad do not group player name over club and division.

### Expected outcome

Search, Moneyball Search, and Squad render one fixed-height stacked identity cell, while a one-time layout migration removes duplicate visible Club and Division columns without losing unrelated preferences or metric availability.

### Explicit exclusions

No Staff identity grouping, variable row heights, metric removal, query/filter/sort behavior change, unrelated layout reset, or shared identity abstraction.

## Discoveries and replanning

- The feature-scoped planner failed closed repeatedly. At the developer's direction, the main session authored the same approved schema-2 artifact; an independent fresh-context review found three issues, all were corrected, and the focused re-review returned clear.
- The PI_SETUP delivery contract changed after the planning commit. One fingerprint now covers every PR, packet, close-out action, and one post-merge release outcome. This migration adds the required authority fields, removes three intermediate release-preparation commits, and consolidates release preparation in the final PR without changing JAY-35 through JAY-41 behavior.
- Intermediate PRs now use release intent `none`. The final PR prepares `0.12.0` from `v0.11.1`, then verified `main` publishes the one feature release.
- The initial planning commit preceded Delivery fingerprints. This migration records its immutable Git ref and authorizes only the remaining packets.
- 2026-08-22 — Confirmed seven Linear Todo issues, clean synchronized `main`, no active ledger, and no overlapping planned specification.
- 2026-08-22 — Inspected current query, profile, app-shell, table, Staff, My Club, persistence, tests, completed records, and the old React table reference.
- 2026-08-22 — Developer accepted natural-position basis details, stacked identity migration, session-only history, and shared all-table secondary nationality treatment.
- 2026-08-22 — Planned four sequential PRs because the batch spans four independent review surfaces.
- 2026-08-22 — Developer accepted the reviewed plan and activated `fix/my-club-ux-containment`.
- 2026-08-22 — Developer chose to preserve four PRs under the updated one-release contract and approved the narrow intermediate-`none` release-guidance exception.
- 2026-08-23 — Rebased `fix/my-club-ux-containment` onto synchronized `main` at `76cc9f8dbd82504f966038583da4e332e9839c0e`. Repository-wide Pi and release-policy changes now arrive from main; this branch retains only `.wiki/TODO.md` and this ledger.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Contain My Club workspaces and compact club setup | Commit 1 — Record the active Todo UX delivery plan | 54552a7cc285c7232b8a2fddb2a254e276d51762 | Added the schema-2 ledger and activated it in `.wiki/TODO.md` | Classifier schema 2, zero errors, `runnable`; post-rebase `git diff --check` and `./scripts/dev check` passed | Not applicable | Clear | 1 | Main-session authorship after planner transport failures; rebased without changing feature behavior |
| PR 1 — Contain My Club workspaces and compact club setup | Commit 2 — Adopt fingerprinted feature delivery | 5b657e51b84c9420639a7a64f7f2a25f8c8e6ac5 | Migrated the ledger to fingerprinted four-PR delivery with one final `0.12.0` release | Both classifiers reported `runnable`; release commands passed shell syntax validation; diagnostics, `git diff --check`, and `./scripts/dev check` passed after rebase | Not applicable | Clear | 3 | Repository-wide Pi and release guidance landed separately in main commit `76cc9f8dbd82504f966038583da4e332e9839c0e`; the rebased commit changes only this ledger |
| PR 1 — Contain My Club workspaces and compact club setup | Commit 3 — Reconcile the active ledger with current PI_SETUP | a1815e0f4ffab67733be09045e8c001ffd42182e | Reconciled the rebased ledger with current PI_SETUP authority, packet, validation, and release requirements | Both classifiers reported `runnable` with the recorded fingerprint; `release-metadata v0.11.1 none`, `git diff --check`, and `./scripts/dev check` passed | Not applicable | Clear | 1 | None |
| PR 1 — Contain My Club workspaces and compact club setup | Commit 4 — Restore bounded Staff workspace scrolling | aa8355c32851672f1377f5d7592634b4486a912d | Added the missing flex containment to Staff and Staff Shortlist tabpanels so their existing virtualized tables own scrolling | RED failed on both missing flex wrappers; focused route tests passed 108 tests, affected route and shared-table tests passed 111 tests, and `./scripts/dev check` passed | Pass | Clear | 0 | None |
| PR 1 — Contain My Club workspaces and compact club setup | Commit 5 — Place managed-club save inline with search | 1be1e3dc426f8cc69e49c7ce621c0487aa456378 | Grouped the managed-club picker and save action in one responsive row with feedback below | RED failed on the missing managed-club control group; focused route tests passed 109 tests, `./scripts/dev check` passed, and `./scripts/dev smoke` passed 48 browser tests | Pass | Clear | 0 | None |
| PR 2 — Add app history controls | Commit 1 — Expose session Back and Forward controls | d50b4ecf5b46b1cd8a2128e7c095627d21aafd21 | Added accessible Back and Forward controls backed by subscribed TanStack Router session history with branch-aware availability | RED failed because controls were absent; focused tests passed 27 tests, affected tests passed 33 tests, `./scripts/dev check` passed, and `./scripts/dev smoke` passed 48 browser tests | Pass | Clear | 1 | Added Strict Mode cleanup proof and corrected scroll-restoration documentation after review |
| PR 3 — Refine shared player-table presentation | Commit 1 — Distinguish secondary nationality flags | Pending record | Stable-deduplicated shared nationality cells and reduced every unique secondary flag's size and emphasis | RED failed on duplicate retention and identical emphasis; focused nationality tests passed 6 tests and `./scripts/dev check` passed | Pass | Clear | 0 | None |

## Final validation

### Per commit

1. Run the packet's focused RED test and confirm that it fails for the expected missing behavior.
2. Make the minimum coherent GREEN change.
3. Run the packet's affected tests.
4. Run LSP and diagnostics on edited source before the build gate.
5. Run `./scripts/dev check` before checkpoint.
6. Obtain a fresh `commit-reviewer` pass for every non-trivial staged change.

### Per PR

- Inspect `git status`, the complete diff, `git diff --check`, and staged diff/stat.
- Run `./scripts/dev check-app` for frontend-only PRs; PR 4 also runs `./scripts/dev check-rust`.
- Run `./scripts/dev smoke` for route and layout behavior in every PR.
- Use `.pi/skills/create-pr/SKILL.md` and `.github/pull_request_template.md` for publication.
- Validate intermediate PRs with `./scripts/dev release-metadata v0.11.1 none`; leave all release owners unchanged.
- Validate the final PR with `./scripts/dev release-metadata v0.11.1 minor`; prepare exactly `0.12.0` and the complete dated JAY-35 through JAY-41 changelog range.
- Wait for required strict status `check` before each squash merge and record each predecessor's immutable merge ref before activating the next PR.

### Acceptance

- [ ] JAY-40 Staff and Staff Shortlist scroll internally with bounded rendered rows.
- [ ] JAY-41 managed-club picker and save action align inline with feedback below.
- [ ] JAY-37 Back/Forward controls reflect and traverse session route history.
- [ ] JAY-38 all shared flag tables distinguish deduplicated secondary nationalities.
- [ ] JAY-39 General Search, Moneyball Search, and Squad use stacked player identity without losing configurable metrics.
- [ ] JAY-35 Moneyball Player Profile uses deduplicated exact-natural-position cohorts and explains or withholds scores honestly.
- [ ] JAY-36 General and Moneyball overview headers keep one stable footprint and toggle position at 1280×800, 1600×900, and actual 200% browser zoom.

### Feature close-out and release

Before the final PR merges:

1. Confirm all acceptance criteria and the immutable merge refs for PRs 1–3.
2. Run `./scripts/dev test`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke`.
3. Run a fresh `feature-reviewer` pass and resolve accepted findings.
4. Reconcile ARCHITECTURE and DESIGN, update TODO, move this complete ledger to `.wiki/features/completed/`, create the reviewed close-out commit, and set the final PR's Feature close-out to `Current`.
5. From that exact post-close-out final PR head, run native Windows `./scripts/dev package-windows` and every installed-app, recovery, bridge, live-FM, and supported product-flow check in `.wiki/notes/early-alpha-release-runbook.md`. Any later head change invalidates this evidence and requires the complete checklist again.
6. Validate the completed ledger and unchanged Delivery fingerprint without changing the head, then publish and merge the final PR.

After synchronized `main` reaches the final immutable merge ref, run the recorded Release verification. If `v0.12.0` is not yet the matching published release, run the recorded Release command once and then repeat Release verification.

## Documentation impact

- `AGENTS.md`, `.pi/skills/create-pr/SKILL.md`, `.wiki/notes/early-alpha-release-runbook.md`, and `.github/pull_request_template.md`: inherit fingerprinted delivery authority, direct-subagent guidance, and the intermediate-PR release exception from synchronized `main`; this feature does not modify them.
- `.wiki/TODO.md`: remove the active item and select the next state during feature reconciliation.
- `.wiki/ARCHITECTURE.md`: update with the persisted player-table layout version/defaults and natural-position profile cohort behavior when each implementation becomes true.
- `.wiki/DESIGN.md`: retain the existing My Club containment contract, then update managed-club setup controls, app-header history controls, nationality hierarchy, stacked player identity, Moneyball basis/unavailable state, and stable profile header as each new UI contract becomes true.
- The final release-preparation commit updates `package.json`, `src-tauri/Cargo.toml`, the root `app` entry in `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `bridge/FmDataBridge.csproj`, `CHANGELOG.md`, and `release-preparation.json`.
- No `.wiki/CONCEPT.md`, `.wiki/BACKLOG.md`, ADR, or debug report change is planned.
