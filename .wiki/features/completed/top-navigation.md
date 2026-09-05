# Top Navigation

## Status

Ready for final publication

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 33835f43cdbb06b97a1f11b0bb1ec547374018ade8d6816398ac933a410d7e70

## Intent

Replace the persistent left rail with top navigation per Linear JAY-54. Navigation becomes stable and navigation-only. Search, Moneyball, Squad, Planner, Tactic, My Staff, and Youth become direct destinations. The utility bar keeps global player search, save selection, snapshot freshness, Back/Forward, cap controls, and Load Data. Content reflows to recover rail width.

## User-visible behavior

- The shell shows a persistent utility bar first and a persistent navigation bar second. No left rail remains.
- At 1280×800 and above, every direct destination is visible as an icon-plus-label link in grouped sections with no dropdowns: Home holds Dashboard; Players holds Search and Moneyball; Staff holds Search and My Staff; Club holds Squad, Planner, Tactic, and Youth; Settings holds Settings.
- Destination groups use fine vertical separators. Each group shows a low-emphasis caption under its links. The active destination uses a distinct contained state with a reinforced label.
- Player Search General and Moneyball are navigation destinations. The local General/Moneyball page tabs in `/search` are gone. Player profile analysis tabs stay local.
- Managed-club Staff moves out of My Club and is labeled My Staff under Staff. `/staff?view=my-staff` is canonical. My Staff shows selected-club context and links to Club setup when no managed club is configured. The managed-club selector stays in the Club header. It is not duplicated and does not move to the utility bar.
- Club Squad, Planner, and Tactic are navigation destinations. The page-level My Club workspace tabs that duplicate them are gone. `/planner` replace-redirects to `/my-club?view=planner`.
- Youth is a navigation destination at `/academy`. Youth Overview, Graduates, and Class tabs stay local.
- Old `/my-club?view=staff&staffSort&staffDir` links replace-redirect to `/staff?view=my-staff&myStaffSort&myStaffDir`. Valid sort values carry over; invalid values fall back through the existing staff validators.
- Deep links and browser history land on the same destination with URL state intact after reload. Switching Search views from navigation applies exactly the current route view-transition semantics (the same reset/preserve behavior as the current tab `onClick`/`onKeyDown` `updateSearch`: view change resets comparison pool, sort, direction, and filters to the new view defaults). Do not promise that all Search state survives a view switch.

## Invariants

- Navigation is stable and navigation-only. No dynamic or current-record item appears in the navigation bar.
- The utility bar owns global player search, save selection, snapshot freshness, Back/Forward, cap controls, and Load Data. Navigation never absorbs them.
- The managed-club selector lives only in the Club header.
- Profile routes mark only the stable Players or Staff group caption with visible group-context treatment and `aria-current="location"`. No child destination receives `aria-current="page"` on profile routes. They never mark an individual Search, Moneyball, or My Staff destination current. Profile tabs stay local.
- Every supported direct destination marks exactly one current destination (`aria-current="page"`). Unknown and not-found routes mark no destination current.
- ValueScout tokens, the Lucide icon system, and the DESIGN.md accessibility rules stay authoritative. The Linear reference image supplies hierarchy and density direction only, not branding or exact visuals.
- Offline and desktop-only operation does not change. Minimum window stays 1280×800.
- No migration, IPC command, query key, stored table-layout identifier, or capability changes. Rust and SQLite ownership is unchanged.

## Non-goals

- New scoring models, role catalogs, attribute formulas, or data features.
- Changes to bridge scanning, snapshot selection, CSV enrichment, shortlist semantics, Planner allocation, Academy rules, Settings management, or Dashboard content.
- A light theme, mobile or narrow breakpoints, remote assets, or decorative imagery.
- Release preparation. An explicit release skill owns that separately.
- Editing `.wiki/ARCHITECTURE.md` or `.wiki/DESIGN.md` during planning. Those documents describe current state and are reconciled in the final delivery commit after implementation makes the new behavior true.

## Current-state map

- Relevant components:
  - `src/app/components/app-shell-layout.tsx::AppShellLayout` — composes `AppNavRail` plus `AppTopBar` with the content outlet.
  - `src/app/components/app-nav-rail.tsx::AppNavRail` — links Dashboard, Player Search, Staff Search, My Club, Youth Academy, and Settings with exact matching and a collapse toggle.
  - `src/app/components/app-top-bar.tsx::AppTopBar` — Back/Forward, `GlobalPlayerSearch`, save selector, freshness chip, cap controls, and Load Data. Behavior is unchanged by this feature.
  - `src/stores/use-layout-store.ts` — persists `railExpanded`. Consumers are `AppNavRail`, `src/app/routes/players.$uid.tsx`, and `src/app/routes/staff.$uid.tsx` through `profileWorkspaceClassName`.
  - `src/app/routes/search.tsx::SearchPageContent` — renders local General/Moneyball tabs and owns `view`, `comparisonPool`, and `shortlistOnly` URL state.
  - `src/app/routes/staff.tsx` — composes Staff Search only. `view=my-staff` replace-redirects to `/my-club`. Holds shortlist orchestration, assignment optimizer context, and `shortlistOnly` URL state.
  - `src/app/routes/my-club.tsx` — canonical Club route with the managed-club selector at `#managed-club` and Squad, Planner, Tactic, and Staff workspaces.
  - `src/features/my-club/components/my-club-workspace-tabs.tsx` — accessible Squad/Planner/Tactic/Staff tabs retired by this feature.
  - `src/features/staff/components/staff-search-results-panel.tsx` — supports `scope="my-staff"` for the managed-club cohort. Scope and query ownership stay.
  - `src/app/routes/planner.tsx` — replace-redirects to `/my-club` with workspace and Squad sort mapping.
  - `src/app/routes/academy.tsx` — `AcademyWorkspaceTabs` for Overview/Class/Graduates stay local.
  - `src/app/routes/players.$uid.tsx` and `src/app/routes/staff.$uid.tsx` — local analysis and attribute tabs stay local.
- Data model:
  - No data model change. Managed-club Staff reads, shortlist joins, Planner depth, and Academy memberships keep their owners.
- Persistence and migrations:
  - None. No migration in this feature.
- Existing behavioral assumptions:
  - The rail uses exact route matching, so profile routes currently mark nothing current. The new navigation must add stable Players/Staff group context there instead of leaving profiles unmarked.
  - Staff Search integrated shortlist behavior, including upload, Configure, Optimize, Preferred Job, and unemployment controls, stays in the Search destination untouched.
  - Completed records for My Club workspace, Staff workspace, and integrated shortlists stay valid except where this ledger explicitly moves or removes a view.
- Architectural seams:
  - Routes stay thin. Features stay composed in route files. No cross-feature imports. `src/lib/tauri-client.ts` remains the sole invoke wrapper.
  - New route files may be added only if view-param links cannot carry a destination cleanly. Prefer existing routes with explicit view params.
- Project validation commands:
  - `./scripts/dev test [target...]` for focused and affected tests.
  - `./scripts/dev check` as the full commit gate (Biome, TypeScript, secretlint, Rust format, Clippy, Rust tests).
  - `CI=1 ./scripts/dev smoke` for the Playwright product suite after Chromium install. This is the final browser and layout proof, with explicit `1280×800` viewport coverage (the default 1280×720 viewport does not count).
- Primary risks:
  - Ten icon-plus-label destinations plus group captions must fit 1280px without overflow or dropdowns.
  - Active-state derivation from pathname plus view params must stay correct across deep links, legacy redirects, and profile routes.
  - Reversing the My Staff redirect must preserve sort state and must not strand optimizer or assignment context.
  - Removing page-level tabs must not remove local state that has no navigation equivalent (comparison pool, shortlist toggle, sort state).

## Feature architecture

- React owns the new navigation bar, destination links, group structure, active-state derivation, page-tab removal, route redirect reversal, and content reflow. No new state library or routing framework.
- `AppTopBar` keeps its behavior and becomes the utility row. `src/app/components/app-nav-bar.tsx::AppNavBar` owns the destination row. `AppShellLayout` stacks utility bar first and navigation bar second above the content outlet.
- Destination identity derives from the current pathname plus view search params. Supported destinations and only these params select a destination: `/search?view=general|moneyball`; `/staff?view=search|my-staff` with `myStaffSort`/`myStaffDir`; `/my-club?view=squad|planner|tactic`; `/academy?view=overview|graduates|class&classId=...`. Profile pathnames map to their stable Players or Staff group caption only, with `aria-current="location"` on that caption and no `aria-current="page"` on any child destination. `/planner` replace-redirects to `/my-club?view=planner`. `/my-club?view=staff&staffSort&staffDir` replace-redirects to `/staff?view=my-staff&myStaffSort&myStaffDir`, preserving valid values and defaulting invalid values through the existing validators.
- Existing routes stay canonical: `/search?view=general|moneyball`, `/staff?view=search|my-staff`, `/my-club?view=squad|planner|tactic`, and `/academy?view=overview|graduates|class&classId=...`. `/planner` stays a compatibility replace-redirect to `/my-club?view=planner`. `/my-club?view=staff` becomes a replace redirect to `/staff?view=my-staff` with `staffSort`/`staffDir` mapped onto `myStaffSort`/`myStaffDir`.
- Contract removal ships with replacement behavior in the same commits. The rail, its collapse toggle, its persisted expansion state, and the duplicated page-level tabs retire only where the navigation bar covers them.
- Current docs still describe `AppNavRail`, local Search and My Club workspace tabs, and managed-club Staff in My Club. Reconciliation lands in the final delivery commit, not during planning.

## Uncertainty register

### Known

- The tree is clean at `b69b830`. No active ledger and no planned spec exist.
- The reference image is at `/tmp/linear-cli-images/9e082363bc5dbd13/image.png`. It shows a compact utility row above navigation, all icon-plus-label destinations visible in one row, fine vertical separators between groups, low-emphasis group captions under links, and a distinct contained active state with a reinforced label.
- `useLayoutStore.railExpanded` has three production consumers: `AppNavRail`, `players.$uid.tsx`, and `staff.$uid.tsx`. All three must be updated together when the rail retires.
- Rail width tokens live in `src/styles/global.css` (`--spacing-rail-width`, `--spacing-rail-width-expanded`) and `.wiki/DESIGN.md` (`rail-width`, `rail-width-expanded`). Token and doc cleanup belongs to the removal and reconciliation commits.
- No planned feature spec exists. BACKLOG is unchanged. No deferred item graduates or retires under this scope.

### Assumptions

- Recent PRs squash-merge into `main` with Conventional Commits titles under 72 characters. The ledger records squash and the repository PR template from that evidence.
- Publication uses GitHub, `.github/pull_request_template.md`, squash merge, and required strict status `check`. Ordinary feature work carries no release metadata.
- One PR suffices. Shell with rail retirement, Search, Staff, Club, and reconciliation commits share one navigation contract, so splitting would create multiple reviews of one seam rather than independent merges.

### Decisions

- Approved developer decisions in the dispatch are the decision record: utility bar first with navigation bar second; the exact grouped icon-plus-label destination map; profile group-only marking; and selector ownership in Club workspaces with My Staff context display plus a setup link.
- No ADR. The change reuses established router, query, app-layer, and styling boundaries. No durable structural choice with meaningful alternatives meets the ADR threshold.
- The reference image direction is hierarchy and density only. Tokens, icons, and accessibility rules stay authoritative.

### Unknowns

- None.

### Risks

- Ten destinations may not fit 1280px with icon-plus-label treatment plus captions. The shell commit must prove fit at 1280×800 or return for a developer decision before tab removal proceeds.
- Active-state logic could mark a destination current on profiles or mark nothing on a valid destination. Packets require group-only marking on profiles and exact destination marking elsewhere.
- The redirect reversal could drop sort state or break optimizer context. The Staff packet requires the `staffSort`/`staffDir` onto `myStaffSort`/`myStaffDir` mapping with existing-validator fallback and unchanged assignment context.
- Tab removal could strand comparison-pool, shortlist-toggle, or sort state. Each removal packet names the surviving owner of that state.

## Walking skeleton

The grouped top navigation bar lands first in the shell with utility bar ordering, correct links, and group-aware active state. Page-level tabs and legacy redirects keep working underneath. The shell commit retires the rail; later commits move each workspace onto its destination, reverse the My Staff redirect, remove duplicated tabs, and reconcile docs.

## Delivery plan

### PR 1 — feat(nav): replace left rail with top navigation

**Status:** Ready for publication

**PR ref:** https://github.com/JG1995/fm-valuescout/pull/116

**Merge ref:** Not merged

**Branch:** `feat/top-navigation`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub required strict status check

**Feature close-out:** Current

**CI repair rounds:** 0

**Provisional PR title:** `feat(nav): replace left rail with top navigation`

**Purpose:** The sole review and merge boundary. Shell with rail retirement, Search, Staff, Club, and doc reconciliation share one navigation contract, so one PR keeps the seam reviewable. Later commits depend on earlier merged work inside this PR only.

**Depends on:** None. Builds on completed My Club workspace, Staff workspace, integrated shortlists, and todo-ux-quality-pass records.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(nav): record approved feature plan`

**Work:** Commit the independently reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/top-navigation.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — active feature state with Linear JAY-54 context.

**Behavior and data flow:**

- Move planning truth into one reviewed active ledger and record the exact delivery sequence before implementation. No planned spec, BACKLOG update, or ADR exists in this change.

**Ordered implementation steps:**

1. Verify the active branch and base without changing Git state.
2. Confirm the worktree contains only the reviewed planning paths.
3. Run the ledger classifier and any repository documentation check.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The ledger classifier and documentation checks prove structural consistency.

**Patterns to verify:**

- The active-ledger template, current TODO/BACKLOG ownership rules, and relevant accepted ADR format.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, plan scope, packet order, or reviewed decisions.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/top-navigation.md` plus the repository documentation check when one exists.

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files.

#### Commit 2 — Replace the left rail with the grouped top navigation shell

**Status:** Completed

**Provisional commit:** `feat(nav): replace left rail with top navigation`

**Work:** One coherent, revertible shell outcome: the utility bar renders first, the grouped navigation bar renders second, the rail is gone from the shell, and content reflows to the recovered width. This commit owns all physical rail retirement: `AppNavRail` leaves shell composition and `app-nav-rail.tsx` is deleted; `use-layout-store.ts` is deleted; rail width tokens are removed; player/staff profile width branches are updated; all rail-dependent unit tests and the rail smoke assertion are updated; and an explicit 1280×800 nav-fit browser proof is added. Page-level tabs and legacy redirects keep working underneath. Any duplication between the new bar and old tabs is correct in this commit and owned by later commits.

**Size assessment:** Roughly 150 to 250 changed non-test implementation lines across one new navigation component, the shell layout, rail/store/token deletion, and profile width classes. May exceed the soft target; keep it together because the bar, shell order, and rail removal are one observable layout contract.

**Out of scope:**

- Page-level tab removal, redirect reversal, new routes, utility-bar behavior changes, and `.wiki/ARCHITECTURE.md` or `.wiki/DESIGN.md` reconciliation.

**Implementation packet:**

**Files and responsibilities:**

- `src/app/components/app-nav-bar.tsx::AppNavBar` — grouped icon-plus-label destination links for Dashboard, Search, Moneyball, Staff Search, My Staff, Squad, Planner, Tactic, Youth, and Settings; fine vertical separators; low-emphasis group captions; distinct contained active state with reinforced label; Lucide icons and ValueScout tokens only.
- Active-state helper colocated in `src/app/components/app-nav-bar.tsx` unless readability during implementation proves it must be a same-folder pure helper — derives the current destination from pathname plus view search params; maps `/players/$uid` to the Players group caption and `/staff/$uid` to the Staff group caption with `aria-current="location"` on that caption and no `aria-current="page"` on any child destination; keeps `aria-current="page"` on exact direct-destination links and neither mark on unknown routes.
- `src/app/components/app-shell-layout.tsx::AppShellLayout` — stacks the utility bar first, the navigation bar second, and the content outlet below. Removes `AppNavRail` from composition.
- `src/app/components/app-nav-rail.tsx` — delete in this commit. Verify no import remains.
- `src/stores/use-layout-store.ts` — delete in this commit. `railExpanded` has no navigation equivalent and no surviving consumer after the profile width updates below.
- `src/styles/global.css` — remove `--spacing-rail-width` and `--spacing-rail-width-expanded` in this commit once no component references them.
- `src/app/routes/players.$uid.tsx` and `src/app/routes/staff.$uid.tsx` — update `profileWorkspaceClassName` width branches for the rail-free layout and drop the `useLayoutStore` consumption.
- `src/app/routes/academy.tsx` — no behavior change. Verify the Youth destination link lands on `/academy` with local Overview/Class/Graduates tabs intact.
- Tests: extend `src/app/app-shell-routing.test.tsx` for bar order, all ten destinations, group structure, exact active marking, profile group-only marking, and history traversal. Retire rail collapse and expansion tests that assert the removed contract. Update every rail-dependent unit-test layout-store setup (`src/app/routes/search.test.tsx`, `src/app/routes/staff.test.tsx`, `src/app/routes/players.$uid.test.tsx`, `src/app/routes/staff.$uid.test.tsx`) for the deleted store. Retarget the `app-nav-rail` smoke assertion in `e2e/smoke.spec.ts` to the navigation destinations.

**Behavior and data flow:**

- Entry: every route renders through `AppShellLayout`. The shell renders the unchanged utility bar, then the navigation bar, then the outlet. Active state recomputes on pathname and view-param change. Every supported direct destination renders exactly one `aria-current="page"` on its link; unknown and not-found routes render neither mark; `/players/$uid` and `/staff/$uid` render only their Players/Staff group caption with `aria-current="location"` and no `aria-current="page"` on any child destination.

**Ordered implementation steps:**

1. Add the smallest RED proof: the shell renders utility bar before navigation bar; each destination link resolves; a profile route marks only its group.
2. Build the navigation component with grouped links, separators, captions, and active state.
3. Recompose the shell, remove the rail from composition, and reflow content and profile width classes.
4. Update shell tests and remove only rail-contract assertions with no surviving behavior.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Observable behavior: bar order, destination visibility, and active-state semantics.
- RED proof at the shell seam: every supported direct destination renders the bar with exactly one `aria-current="page"`; unknown and not-found routes render the bar with neither mark; `/players/$uid` and `/staff/$uid` render only their Players/Staff group caption with `aria-current="location"` and no `aria-current="page"` on any child destination.
- Layout proof: an explicit Playwright run at a `1280×800` viewport shows every destination without horizontal overflow. The default 1280×720 viewport does not count. Prove with the smoke suite at 1280×800 or a focused browser run; stop for a developer decision when destinations do not fit.
- Absence proof for the rail: assert `AppNavRail`, its toggle, and its expansion persistence are gone from the shell, and full-text search for `AppNavRail`, `app-nav-rail`, `railExpanded`, and `rail-width` returns only historical or explicitly retained references. Add it because rail reintroduction is plausible and observable.
- Do not inventory unchanged utility-bar, search, staff, club, academy, or snapshot tests.

**Patterns to verify:**

- `AppTopBar` for sticky bar geometry, `z-10` layering, and 56px header rhythm.
- DESIGN.md Nav Rail active-state rules (`aria-current="page"`, filled-versus-outline icon change, gold fill) adapted to the top bar with tokens only.
- Existing `Skip to content` and `#main-content` semantics preserved.

**Constraints and non-goals:**

- Utility-bar behavior is unchanged: search, save selector, freshness, caps, Load Data, and Back/Forward stay as-is.
- Keyboard reachability for every destination. Hover changes colour only. Focus ring follows the token rule.
- Do not change routes, redirects, tabs, stores beyond rail state, or current-state documents.

**Dependencies and sequencing:**

- Requires Commit 1 on the feature branch. Later commits build on this shell.

**Validation:** Focused `./scripts/dev test src/app/app-shell-routing.test.tsx`, then the affected app suite, then `./scripts/dev check`. Add/use an explicit 1280×800 Playwright project/test viewport in `e2e/smoke.spec.ts` or `playwright.config.ts` for nav-fit browser proof, then run `CI=1 ./scripts/dev smoke` before checkpoint when the tree supports it.

**Stop conditions:** Stop for a developer decision when ten icon-plus-label destinations plus captions do not fit 1280px, or when active-state derivation cannot cover deep links and profiles without new routes.

**Review mandate:** Verify bar order with no rail remnant; `app-nav-rail.tsx`, `use-layout-store.ts`, and rail width tokens deleted with no live consumer left behind; all ten destinations link to working targets; group separators and captions use tokens; active state is exact on destinations and group-only on profiles; keyboard and focus behavior intact; profile width classes correct; no utility-bar behavior changed; no route or redirect changed.

#### Commit 3 — Move Search and Moneyball tabs into top navigation

**Status:** Completed

**Provisional commit:** `feat(search): move search views into top navigation`

**Work:** One coherent, revertible Search outcome: Search and Moneyball become navigation destinations, the local General/Moneyball page tabs are removed, and view switching lives in the navigation bar. Comparison-pool state, shortlist toggle, filters, sorting, tactic columns, Club DNA, and profile analysis views are preserved under their surviving owners.

**Size assessment:** Roughly 100 to 200 changed non-test implementation lines in the search route and navigation wiring. Within the soft target.

**Out of scope:**

- Staff, Club, Youth, utility-bar, redirect, and current-state document changes.

**Implementation packet:**

**Files and responsibilities:**

- `src/app/routes/search.tsx::SearchPageContent` — remove the local General/Moneyball tablist and its arrow-key handling; read the navigation-selected view from `view=general|moneyball` route search state; keep `comparisonPool` and `shortlistOnly` URL ownership, filter bar, tactic toggles, upload modals, and results panel wiring. Navigation view switching applies exactly the current route view-transition semantics (the same reset/preserve behavior as the current tab `updateSearch`: view change resets comparison pool, sort, direction, and filters to the new view defaults).
- Navigation wiring from Commit 2 — Search links to the General view and Moneyball links to the Moneyball view, each with exact active marking for its view.
- `src/features/search/types/search-view.ts` and `src/features/search/utils/search-url-search.ts` — retain view parsing and legacy `/search?view=shortlist` normalization unchanged unless removal evidence supports a change.
- Tests: extend `src/app/routes/search.test.tsx` for navigation-driven view switching applying the existing tab transition behavior — view change resets comparison pool, sort, direction, and filters to the new view defaults and preserves `shortlistOnly` because the current patch does not replace it — plus legacy view URLs and deep-link/history URL-state preservation. Remove tab-specific tests that assert the removed tablist. Update the Search-tab smoke assertions (`tablist` named "Search view") in `e2e/smoke.spec.ts` and `e2e/tauri-ipc-stub.ts` only where they assert the retired tablist.

**Behavior and data flow:**

- Navigation selects the Search or Moneyball destination at `/search?view=general|moneyball`. The route renders that view without local tabs. URL state round-trips so deep links and reloads land on the same view with state intact; view switching itself follows the current route transition semantics and does not promise that all Search state survives the switch. Profile analysis tabs stay local and unchanged.

**Ordered implementation steps:**

1. Add the smallest RED proof: selecting Moneyball in navigation renders the Moneyball view with no local tablist; a deep link with `view=moneyball` lands there directly.
2. Remove the local tablist and wire view selection to navigation.
3. Preserve comparison-pool, shortlist, filter, sort, tactic-column, and upload state under their surviving owners.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Observable behavior: navigation-owned view switching with preserved Search state.
- RED proof at the route seam: `view=general` and `view=moneyball` URLs render the matching view; no `tablist` labelled Search view remains.
- Absence proof for the removed tablist: assert the local tabs are gone. Add it because tab reintroduction is plausible and observable.
- Do not inventory unchanged Moneyball scoring, staff, planner, or snapshot tests.

**Patterns to verify:**

- My Club workspace-tab removal analogues are not yet implemented; use the Commit 2 navigation wiring as the view-selection pattern.
- Existing `updateSearch` replace semantics and loader deps for view changes.

**Constraints and non-goals:**

- Player profile analysis tabs and the shared Moneyball default preference stay unchanged.
- Search filter, sort, tactic-column, Club DNA, shortlist, and upload behavior stays unchanged.
- Do not change staff code, club code, redirects, or current-state documents.

**Dependencies and sequencing:**

- Requires Commit 2 on the feature branch for the navigation shell and active-state wiring.

**Validation:** Focused `./scripts/dev test src/app/routes/search.test.tsx`, then the affected search suite, then `./scripts/dev check`.

**Stop conditions:** Stop for a developer decision when comparison-pool or shortlist state has no surviving owner under the settled `/search?view=general|moneyball` shape.

**Review mandate:** Verify navigation owns view switching; local tablist fully removed with no orphaned keyboard handling; deep links and reload land correctly; comparison-pool, shortlist, filter, sort, tactic, and upload state preserved; profile tabs untouched; no unrelated Search behavior changed.

#### Commit 4 — Make My Staff a canonical Staff destination

**Status:** Completed

**Provisional commit:** `feat(staff): promote my staff to a top destination`

**Work:** One coherent, revertible Staff outcome: `/staff?view=my-staff` becomes the canonical My Staff destination with selected-club context and a Club-setup link when unconfigured; old `/my-club?view=staff&staffSort&staffDir` links replace-redirect to `/staff?view=my-staff&myStaffSort&myStaffDir` with valid sort values preserved and invalid values defaulted through the existing staff validators; the managed-club selector stays in the Club header; Staff Search integrated shortlist behavior stays untouched.

**Size assessment:** Roughly 100 to 200 changed non-test implementation lines across the staff route, My Club route, and navigation wiring. Within the soft target.

**Out of scope:**

- Search, Club tab, Youth, utility-bar, optimizer semantics, CSV contracts, and current-state document changes.

**Implementation packet:**

**Files and responsibilities:**

- `src/app/routes/staff.tsx` — make `view=my-staff` canonical: render the managed-club cohort through the existing `StaffSearchResultsPanel` `scope="my-staff"` path; show selected-club context; link to Club setup (`/my-club#managed-club`) when no managed club is configured; never duplicate the selector and never move it to the utility bar. Remove the `view=my-staff` to `/my-club` redirect.
- `src/app/routes/my-club.tsx` — add a replace redirect for `view=staff` to `/staff?view=my-staff`, mapping `staffSort`/`staffDir` onto `myStaffSort`/`myStaffDir` (valid values preserved, invalid values defaulted through the existing staff validators); remove the Staff workspace from the Club surface once the redirect owns it.
- `src/features/my-club/components/my-club-workspace-tabs.tsx` — in this commit only narrow the workspace type and tab rendering to Squad/Planner/Tactic (remove the Staff tab); the file itself is deleted in the Club commit after the surviving type/parser/panel helpers move.
- Navigation wiring from Commit 2 — Staff Search links to `/staff?view=search` and My Staff links to `/staff?view=my-staff`, each with exact active marking.
- Tests: extend `src/app/routes/staff.test.tsx` for canonical My Staff rendering, unconfigured-club setup link, selector absence in Staff, and sort preservation; rewrite `src/app/routes/legacy-club-routes.test.tsx` My Staff expectations for the reversed direction; update `src/app/routes/my-club-squad.test.tsx` Staff workspace expectations for the removed workspace. Update the My Staff/My Club Staff smoke assertions (`/my-club?view=staff` navigations) in `e2e/smoke.spec.ts` and `e2e/tauri-ipc-stub.ts` only where they assert the retired Club Staff surface.

**Behavior and data flow:**

- Navigation selects Staff Search or My Staff. My Staff reads the managed-club cohort for the active save and snapshot. A save, snapshot, or managed-club change invalidates through the existing staff and managed-club query keys. Assignment optimizer context carries over unchanged from the current Staff orchestration. Legacy Club Staff links replace-redirect without adding history.

**Ordered implementation steps:**

1. Add the smallest RED proof: `/staff?view=my-staff` renders the managed-club cohort; `/my-club?view=staff` replace-redirects to it with sort preserved.
2. Reverse the redirect and render My Staff canonically with club context and setup link.
3. Remove the Staff workspace from the Club surface and the obsolete redirect.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Observable behavior: canonical My Staff destination with club context and legacy redirect compatibility.
- RED proof at the route seams: unconfigured club shows the setup link and no selector; configured club shows cohort rows with context; legacy URL replace-normalizes with no back entry.
- Absence proof: assert the managed-club selector does not render in Staff. Add it because selector duplication is plausible and observable.
- Do not inventory unchanged search, planner, academy, or snapshot tests.

**Patterns to verify:**

- Existing `beforeLoad` replace-redirect shapes in `staff.tsx`, `my-club.tsx`, and `planner.tsx` for history-safe normalization.
- Existing shortlist orchestration, assignment context keys, and context-guard semantics in `staff.tsx` carried over unchanged.

**Constraints and non-goals:**

- Staff Search shortlist upload, Configure, Optimize, Preferred Job, and unemployment behavior stays unchanged.
- Selector ownership, recovery targets, and downstream invalidation in the Club header stay unchanged.
- Do not change search code, club tabs beyond Staff removal, or current-state documents.

**Dependencies and sequencing:**

- Requires Commit 2 on the feature branch. Independent of Commit 3 except for shared PR sequencing.

**Validation:** Focused `./scripts/dev test src/app/routes/staff.test.tsx src/app/routes/legacy-club-routes.test.tsx`, then the affected staff and club suites, then `./scripts/dev check`.

**Stop conditions:** Stop for a developer decision when sort state cannot transfer across the redirect, or when optimizer context cannot carry over without a wider refactor.

**Review mandate:** Verify canonical My Staff rendering with club context; setup link present only when unconfigured; no selector in Staff or utility bar; redirect maps `staffSort`/`staffDir` onto `myStaffSort`/`myStaffDir` with validator fallback and no history entry; optimizer and assignment context unchanged; Staff Search behavior untouched; Club Staff workspace fully removed.

#### Commit 5 — Move Club Squad, Planner, and Tactic into top navigation

**Status:** Completed

**Provisional commit:** `feat(club): move club workspaces into top navigation`

**Work:** One coherent, revertible Club outcome: Squad, Planner, and Tactic become navigation destinations; the page-level My Club workspace tabs are removed; the managed-club selector, Club DNA action, Squad boosts, CSV actions, Planner matrix, and Tactic editor stay under their surviving owners; `/planner` still redirects to the Club Planner destination.

**Size assessment:** Roughly 100 to 200 changed non-test implementation lines across the Club route, workspace tabs, and navigation wiring. Within the soft target.

**Out of scope:**

- Search, Staff, Youth, utility-bar, redirect-reversal beyond `/planner` mapping, and current-state document changes.

**Implementation packet:**

**Files and responsibilities:**

- `src/app/routes/my-club.tsx` — remove `MyClubWorkspaceTabs` usage; derive the active Club workspace from `view=squad|planner|tactic` route search state selected by navigation; keep the managed-club selector at `#managed-club`, Club DNA wiring, Squad overview with boosts and CSV actions, Planner depth matrix, Tactic editor, sort-state ownership, and query invalidation. Before deleting the tab component, move `MyClubWorkspace` (narrowed to Squad/Planner/Tactic) and `parseMyClubWorkspace` into `src/app/routes/my-club.tsx`, exporting both for `planner.tsx`. Do not retain or move `myClubWorkspacePanelProps`: retire its tab-specific `role="tabpanel"`, `aria-labelledby`, and `my-club-workspace-panel-*` / `my-club-workspace-tab-*` IDs with the page-level tabs. Keep the hidden-mounted Planner/Tactic panels so drafts and local state survive destination changes via three exact workspace-specific direct `hidden` props (`hidden={activeWorkspace !== "squad"}` on the Squad workspace `<div>`, `hidden={activeWorkspace !== "planner"}` on the Planner workspace `<div>`, `hidden={activeWorkspace !== "tactic"}` on the Tactic workspace `<div>`); no app-route-local panel helper is added because the current source spreads the helper onto plain `<div>`s where a direct boolean `hidden` prop is clearer and needs no indirection.
- `src/features/my-club/components/my-club-workspace-tabs.tsx` — delete in this commit once nothing imports it, including its keyboard handling and `myClubWorkspacePanelProps` (deleted with no move; its tab IDs, `role="tabpanel"`, and `aria-labelledby` retire with it).
- `src/app/routes/planner.tsx` — retarget the compatibility replace-redirect to `/my-club?view=planner`, preserving workspace and Squad sort mapping; retarget its `MyClubWorkspace` / `parseMyClubWorkspace` import to `src/app/routes/my-club.tsx` once the tab file is deleted.
- Navigation wiring from Commit 2 — Squad, Planner, and Tactic link to their Club destinations with exact active marking.
- Tests: extend `src/app/routes/my-club-squad.test.tsx` for navigation-driven workspace switching with preserved sort and selector state; verify the nav link selects each workspace, only the selected workspace is exposed while hidden Planner/Tactic panels remain mounted with drafts intact, and no obsolete `tabpanel` role or `aria-labelledby` reference remains; update `src/app/routes/legacy-club-routes.test.tsx` Planner expectations for `/my-club?view=planner`; update the Club-tab smoke assertions (`tablist` named "My Club workspaces") in `e2e/smoke.spec.ts` and `e2e/tauri-ipc-stub.ts` only where they assert the retired tabs.

**Behavior and data flow:**

- Navigation selects a Club destination. The Club route renders that workspace without local tabs. Managed-club selection, Club DNA, boosts, imports, depth, and tactic mutations keep their owners and invalidation. Deep links with workspace and sort params land directly.

**Ordered implementation steps:**

1. Add the smallest RED proof: each Club destination renders its workspace with no local tablist; a deep link with workspace and sort params lands there directly.
2. Remove the workspace tabs and wire workspace selection to navigation.
3. Retarget the `/planner` redirect and preserve sort mapping.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Observable behavior: navigation-owned Club workspace switching with preserved Club state.
- RED proof at the route seam: workspace URLs render the matching workspace with the matching nav link selected and only that workspace exposed; hidden Planner/Tactic panels remain mounted with drafts and local state intact; no `tablist` labelled My Club workspaces remains and no `role="tabpanel"` or `aria-labelledby` pointing to deleted tab IDs remains.
- Absence proof for the removed tabs: assert the tablist, its keyboard handling, and the `myClubWorkspacePanelProps` tab semantics (tab-panel IDs, `tabpanel` role, `aria-labelledby`) are gone. Add it because tab reintroduction is plausible and observable.
- Do not inventory unchanged planner allocation, tactic validation, boost, or snapshot tests.

**Patterns to verify:**

- Commit 3 Search tab-removal shape for tab-to-navigation migration.
- Existing direct-`hidden`-prop mounting semantics on plain workspace `<div>`s where Planner/Tactic drafts or layouts must survive switching (no `tabpanel` role, no `aria-labelledby`, no tab/panel IDs).

**Constraints and non-goals:**

- Managed-club selector, Club DNA, boosts, CSV actions, depth, and tactic behavior stays unchanged.
- My Staff redirect ownership from Commit 4 stays unchanged.
- Do not change search code, staff code, academy code, or current-state documents.

**Dependencies and sequencing:**

- Requires Commits 2 and 4 on the feature branch. Commit 4 must own the Staff workspace removal first.

**Validation:** Focused `./scripts/dev test src/app/routes/my-club-squad.test.tsx src/app/routes/legacy-club-routes.test.tsx`, then the affected club suite, then `./scripts/dev check`.

**Stop conditions:** Stop for a developer decision when mounted-panel draft or layout state has no surviving owner under the settled `/my-club?view=squad|planner|tactic` shape.

**Review mandate:** Verify navigation owns workspace switching; local tablist, keyboard handling, and `myClubWorkspacePanelProps` fully removed with no move (no `tabpanel` role, `aria-labelledby`, or tab/panel IDs remain); hidden Planner/Tactic panels stay mounted via direct `hidden` props with drafts intact; selector, DNA, boosts, imports, depth, and tactic behavior preserved; `/planner` redirect lands on `/my-club?view=planner` with sort mapping; deep links and reload land correctly; no unrelated Club behavior changed.

#### Commit 6 — Reconcile navigation documentation

**Status:** Completed

**Provisional commit:** `docs(nav): reconcile navigation documentation`

**Work:** One coherent documentation outcome: current-state docs describe the implemented top navigation, destination map, redirect compatibility, and Staff ownership exactly as Commits 2 through 5 built them. No behavior change.

**Size assessment:** Documentation only. No implementation code.

**Out of scope:**

- Implementation, tests, executable configuration, and behavior changes.

**Implementation packet:**

**Files and responsibilities:**

- `.wiki/ARCHITECTURE.md` — replace the `AppNavRail` plus `AppTopBar` shell description with utility bar plus navigation bar; update route and workspace ownership for Search, Moneyball, Staff, My Staff, Club, and Youth destinations; update redirect compatibility (`/my-club?view=staff&staffSort&staffDir` to `/staff?view=my-staff&myStaffSort&myStaffDir`, `/planner` to `/my-club?view=planner`); remove rail-expansion persistence claims.
- `.wiki/DESIGN.md` — replace the Nav Rail component contract with the top navigation contract: grouped icon-plus-label destinations, separators, group captions, contained active state, accessibility rules, and the 1280×800 fit claim as implemented.
- Record any smaller doc touch-ups the implementation disproved, each traced to its owning commit diff.

**Behavior and data flow:**

- Documentation follows implementation. Every statement traces to the merged implementation diff in this PR.

**Ordered implementation steps:**

1. Diff the implementation commits against current-state docs.
2. Update the narrowest owning sections only.
3. Verify no proposed behavior is described as current state beyond what this PR implements.
4. Run planning validators and the repository documentation check.

**Tests and proof:**

- Not applicable — this commit changes prose only. The ledger classifier and documentation checks prove structural consistency.

**Patterns to verify:**

- The technical-writing skill: short sentences, active verbs, consistent terms, no hard breaks mid-sentence.

**Constraints and non-goals:**

- Do not change implementation, tests, or executable configuration.
- Do not duplicate ledger detail into current-state documents. State what is true, not the delivery history.

**Dependencies and sequencing:**

- Requires Commits 2 through 5 complete and reviewed on the feature branch.

**Validation:** Ledger classifier plus the repository documentation check when one exists. No test or gate rerun beyond what those checks require.

**Stop conditions:** Stop on any doc statement the implementation diff cannot support, or any behavior change smuggled into a prose commit.

**Review mandate:** Verify every changed doc statement traces to the implementation diff; no stale rail, tab, or Staff-ownership claim remains; no implementation or unrelated doc file changed.

## Active work

**PR:** PR 1 — feat(nav): replace left rail with top navigation

**Active work:** None — feature close-out

**Commit:** None — feature close-out

### RED or removal proof

Not applicable — every planned packet, full feature validation, feature review, correction review, and documentation reconciliation is complete.

### Expected outcome

The reviewed feature is ready for final PR publication.

### Explicit exclusions

Release preparation, unrelated implementation or documentation, and branch cleanup.

## Discoveries and replanning

- Commit 2: removing the 56px rail changed the effective content width of two out-of-contract 900px Planner smoke scenarios enough to cross the matrix layout threshold. Their viewport changed to 844px to preserve the same pre-removal content geometry and tabbed-mode contract; the supported 1280×800 shell and navigation fit has separate direct coverage.
- Commit 3 review recorded and omitted one NITPICK: `src/app/routes/search.test.tsx` contains a duplicate mismatched-snapshot empty-state assertion. It does not weaken coverage or behavior and does not justify a correction round.
- Feature review found missing visible profile group context, missing proof that Search preserves `combine`, and two stale rail references. Correction commit `eb72201bf2c079c04bf5513cfc6f235f6bb01e4b` added token-based profile caption treatment and focused proof, strengthened the Search transition test, and removed the stale prose. Correction review found no remaining CRITICAL, HIGH, or MEDIUM issue; test portfolio passed and project fit conformed.
- Feature close-out passed `./scripts/dev test` with 822 frontend tests, `./scripts/dev check` with 766 Rust tests passed and 2 ignored, and `CI=1 ./scripts/dev smoke` with 54 browser tests including the explicit 1280×800 navigation-fit proof.
- Native Tauri/WebView rendering remains an honest manual validation gap. The feature changes no IPC, capability, persistence, migration, or native integration contract.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — feat(nav): replace left rail with top navigation | Commit 1 — Record the approved feature plan | 911ea310f31485936dabf431a9a1eb73d485d398 | Recorded the independently reviewed schema 2 ledger and active JAY-54 TODO entry. | `ledger_state.py`, `delivery_state.py`, `git diff --cached --check`, and `./scripts/dev check` passed; 766 Rust tests passed and 2 were ignored. | Not applicable | Clear | 0 | None |
| PR 1 — feat(nav): replace left rail with top navigation | Commit 2 — Replace the left rail with the grouped top navigation shell | 786e1f7dfdee02d7524038db1ea36a15ea0032d6 | Added the grouped top navigation beneath the utility bar; retired the rail, expansion store, width tokens, and profile width branches; added exact active-state and 1280×800 fit proof. | Focused shell and affected route tests passed; `./scripts/dev check` passed with 766 Rust tests and 2 ignored; `CI=1 ./scripts/dev smoke` passed 54/54; LSP and staged diff checks passed. | Pass | Clear | 1 | Two 900px Planner smoke viewports changed to 844px to preserve their prior effective content width after rail retirement. |
| PR 1 — feat(nav): replace left rail with top navigation | Commit 3 — Move Search and Moneyball tabs into top navigation | 6bedba8c08b760ddb2ceb66d16778ef0b04c631f | Removed the local Search view tablist and moved General/Moneyball switching into top navigation while preserving deep-link state and the prior reset/preserve transition contract. | Focused Search tests passed 72/72; affected Search and shell tests passed 144/144; `./scripts/dev check` passed with 766 Rust tests and 2 ignored; LSP and staged diff checks passed. | Pass | Clear | 0 | None |
| PR 1 — feat(nav): replace left rail with top navigation | Commit 4 — Make My Staff a canonical Staff destination | 12d51ab98615f67e033d1725a1daebd70646bb5d | Made My Staff canonical under Staff, reversed the legacy Club redirect with validated sort mapping, removed the Club Staff workspace, and isolated My Staff from Staff Search-only queries. | Focused Staff, legacy, Club, and shell tests passed 205/205; `./scripts/dev check` passed with 766 Rust tests and 2 ignored; `CI=1 ./scripts/dev smoke` passed 54/54; LSP and staged diff checks passed. | Pass | Clear | 3 | None |
| PR 1 — feat(nav): replace left rail with top navigation | Commit 5 — Move Club Squad, Planner, and Tactic into top navigation | 0c550937ab489cd7f6e4a504bed0459d3d012d08 | Removed the Club workspace tabs and obsolete tab semantics, moved route parsing ownership, preserved mounted Planner/Tactic state and Squad sort state, and normalized `/planner` to the Planner destination. | Focused Club, legacy, and shell tests passed 165/165; `./scripts/dev check` passed with 766 Rust tests and 2 ignored; `CI=1 ./scripts/dev smoke` passed 54/54; LSP and staged diff checks passed. | Pass | Clear | 1 | None |
| PR 1 — feat(nav): replace left rail with top navigation | Commit 6 — Reconcile navigation documentation | e9916dc0eeaea9db341fcbea92075998a93623ee | Updated architecture and design current state for the utility and navigation bars, destination ownership, compatibility routes, URL/history behavior, accessibility, and retired rail/tab contracts. | `ledger_state.py`, Markdown, whitespace, and staged diff checks passed; implementation and route tests supplied the documentation evidence. | Not applicable | Clear | 2 | None |

## Final validation

- Focused `./scripts/dev test ...` for each commit target, affected suites per packet, full `./scripts/dev test`, and `./scripts/dev check` as the commit gate.
- `CI=1 ./scripts/dev smoke` as the final browser and layout proof, with an explicit `1280×800` viewport proving destination visibility and redirect coverage. The default 1280×720 viewport does not count.
- Manual native Tauri/WebView verification only as an honest remaining gap where automation cannot prove it. Automation evidence decides; confidence does not.
- `./scripts/dev mutate` remains unsupported and is never reported as passed.

## Documentation impact

- Reconciliation is complete in `.wiki/ARCHITECTURE.md`, `.wiki/DESIGN.md`, and `.wiki/TODO.md` for the implemented shell, destinations, route ownership, redirects, state transitions, accessibility, and removed rail/tab contracts.
- `.wiki/CONCEPT.md` and `.wiki/BACKLOG.md` require no change. No ADR meets the project threshold. No release metadata applies.
- This complete ledger moves to `.wiki/features/completed/top-navigation.md` in the reviewed close-out commit.
