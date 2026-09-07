# Player Profile Workspace Redesign

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** c1c4f5ba874753a4a5ffa0c28b59be58601aba5ec11740ec8ef4d1701d8eea0a

## Intent

The Player Profile currently presents identity, ability, value, and tactical-fit data at roughly equal visual priority in a horizontal header plus side-by-side Attributes and Role Fit panels, so attributes and tactical-role analysis compete for space with little hierarchy between identity, summary data, and deeper analysis.

This feature rebuilds the profile as a structured analysis workspace with a persistent Player Identity rail and a main workspace containing four sections: Overview, Attributes, Role Fit, and Moneyball.

All information and functionality currently available on the profile must remain accessible after the redesign.

Linear issue: JAY-62 (`Redesign Player Profile as structured analysis workspace`).

Directional design reference: `.wiki/features/assets/jay-62-player-profile-mockup.png` (layout treatment only; portrait and crest slots are placeholders; the mockup is not a pixel-perfect specification and functionality shown there but not required by JAY-62 is out of scope).

## User-visible behavior

- The profile shows a persistent Player Identity rail on the left with name, club, competition, nationality, age, date of birth, height, preferred foot, and market value, plus reserved neutral placeholder slots for a future player portrait and club logo without loading any images.
- The main workspace exposes four sections: Overview, Attributes, Role Fit, and Moneyball.
- Switching sections retains the currently viewed player.
- The profile keeps a stable visual identity across all four sections.
- Overview answers what kind of player this is and how good they can become: prominent but not dominating CA, PA, and market value, plus best current tactical fit per possession phase shown as same-role current → potential pairs (for example `66 → 88`).
- Overview contains no condensed attribute summary.
- Attributes is a dedicated full-width workspace with local Outfield, Goalkeeping, Hidden, and Personality controls using the existing attribute-layout system, tabular numerals, aligned numeric values, and existing semantic value colouring.
- Role Fit is a dedicated workspace with a position navigator (supported positions, familiarity values where supported, muted unfamiliar positions per the invariant definition, selected position obvious without colour alone, keyboard operable) and a comparison-oriented role list showing role name, IP/OOP phase indicator with readable text, and vertically aligned current and potential scores.
- Moneyball keeps all existing Moneyball profile functionality and calculations unchanged, presented through the same identity rail, navigation, panel hierarchy, current/potential visual language, typography, and spacing.
- Current/potential comparisons use one consistent `Current → Potential` paired presentation across the profile.
- Boost CA and Wonderkid Mentality remain available under a secondary Modify Player disclosure, visually distinguished from navigation and analysis.
- Show/hide hidden information remains available as a view/display control with unchanged behavior, separate from player-modifying actions.
- All section navigation, position selection, and modification controls are keyboard operable with appropriate current-state semantics.
- Existing direct links, Back/Forward navigation, and player-selection behavior continue to work through the legacy URL compatibility mapping recorded in the invariants.
- The layout stays usable at the minimum supported desktop size (1280×800) and scales to 3440×1440 with a bounded identity rail and no indefinitely stretched cards or controls.

## Invariants

- Frontend presentation redesign only.
- No Rust, SQLite, migration, scoring, or IPC changes.
- No change to the meaning or calculation of CA, PA, attribute values, attribute potential values, position familiarity, IP role scores, OOP role scores, potential role scores, Moneyball scores or data, market value, hidden attributes, or personality data.
- Canonical URL state is `section=overview|attributes|role-fit|moneyball`.
- `tab` stays as Attributes-local state with its current parsing (`parseProfileTab`, legacy `technical|mental|physical` normalization, goalkeeper defaults); it is not top-level profile navigation.
- Legacy URL mapping: an explicit valid canonical `section` always wins; otherwise `view=moneyball` selects Moneyball, `view=general` with an explicit valid attribute tab selects Attributes, and ordinary `view=general` selects Overview; an invalid `section` or invalid `view` is treated as absent and falls through the saved default analysis view (General default maps to Overview, Moneyball default maps to Moneyball); with neither `section` nor `view` present, the saved default analysis view applies.
- Search route General/Moneyball semantics are unchanged.
- Best tactical fit selects the best current IP role and the best current OOP role, and each summary shows that same role's current → potential score.
- Never select a separate best-potential role.
- Standard Role Fit scoring and Moneyball scoring stay distinct; no potential Moneyball scores are invented.
- Portrait and club crest use neutral bundled placeholders or monograms; no image loading and no network fetching.
- Concealment behavior, boost behavior, all currently exposed data, Moneyball details, position familiarity values, keyboard operation, direct links, browser Back/Forward, current-player retention, and supported 1280×800 through 3440×1440 behavior are preserved.
- Colour is never the sole indicator of meaning (selected position, IP/OOP phase, scores, and states always pair colour with text, shape, or semantics).
- Unfamiliar positions are exactly unrecorded positions (non-numeric familiarity) and numeric familiarity below `PLAYABLE_POSITION_FAMILIARITY` (15); familiarity at or above 15 never mutes; selected-position styling overrides muted styling; familiarity values and the threshold are unchanged.
- Market value appears in both the identity rail and the Overview ability summary by design; the no-duplication rule applies to all other identity facts.
- One PR on branch `feature/player-profile-workspace-redesign` against base `main`.

## Non-goals

- Actual player-face or club-logo loading and rendering (layout reserves space only).
- The optional condensed Overview attribute summary from the issue (explicitly omitted).
- Scoring, persistence, migration, IPC, bridge, or Rust changes.
- Search route changes beyond preserving its existing General/Moneyball semantics.
- New shared UI primitives unless repository evidence during implementation proves one is currently justified (the Modify Player disclosure is feature-local by default).
- Pixel-perfect reproduction of the mockup; mockup-only functionality not required by JAY-62.
- Mobile, narrow-viewport, or light-theme support (the product is desktop-only and dark-only per `.wiki/DESIGN.md`).
- Release preparation or publication (delivery ends at final PR merge and base synchronization).

## Current-state map

- Relevant components:
  - `src/app/routes/players.$uid.tsx` owns the profile route, `validateSearch` (`tab`, `view`), the loader (snapshot, player, conditional Moneyball prefetch keyed on the saved default analysis view), `GeneralPlayerProfile`, `MoneyballPlayerProfile`, `PlayerProfileHeader`, `PlayerAnalysisTabs` (General/Moneyball tablist with arrow-key handling and focus restoration), `profileWorkspaceClassName` (two-column Attributes beside Role Fit), and the player/moneyball query and mutation wiring.
  - `src/features/player-profile/components/player-overview-panel.tsx` owns `PlayerOverviewPanel` (identity facts, `BestRoleSummary` for Current IP, Current OOP, Potential IP, Potential OOP as four separately selected roles via `bestRoleScore` and `bestPotentialRoleScore`, CA/PA/value facts, hidden-information toggle, and the boost action slot).
  - `src/features/player-profile/components/player-attributes-panel.tsx` owns `PlayerAttributesPanel` with `PlayerProfileTabs` (Outfield, Goalkeeping, Hidden, Personality) and `Current → Potential` paired rows.
  - `src/features/player-profile/components/player-roles-panel.tsx` owns `PlayerRolesPanel` (position-filtered, sortable Current/Potential table with `RoleSortHeader`, phase shown as small text via `rolePhaseLabel`).
  - `src/features/player-profile/components/player-development-boosts-panel.tsx` owns `PlayerDevelopmentActions` (Boost CA and Wonderkid Mentality buttons with confirmation modal).
  - `src/features/player-profile/components/player-profile-tabs.tsx` owns the attribute tablist keyboard pattern to reuse for section navigation.
  - `src/components/profile-position-picker/profile-position-picker.tsx` owns `ProfilePositionPicker` (pitch-grid position buttons with familiarity, `aria-pressed`, tier classes).
  - `src/features/moneyball/components/moneyball-profile-panel.tsx` and `moneyball-role-fit-panel.tsx` own the Moneyball panels reused inside the Moneyball section.
  - `src/stores/use-moneyball-preferences.ts` owns the saved `defaultAnalysisView` (General or Moneyball) used when the URL omits a view.
- Data model:
  - `PlayerDetail` in `src/features/player-profile/types/player-detail.ts` (identity, CA/PA, value, attributes, potential attributes, hidden attributes, personality, positions familiarity map, role scores with current and potential, concealment flag).
  - No model change is planned.
- Persistence and migrations:
  - None.
  - Hidden-information visibility is a server-backed save-scoped mutation (`set-hidden-information-revealed.ts`); its contract is unchanged.
- Existing behavioral assumptions:
  - `tab` is a URL search param parsed by `parseProfileTab` with legacy `technical|mental|physical` → `outfield` normalization and goalkeeper-first tab order (`profile-tab.ts`).
  - `view` is a URL search param (`general|moneyball`) defaulting to the saved preference; unknown views normalize to General.
  - The player query is currently subscribed twice on the General path (once in `PlayerProfileContent`, once in `GeneralPlayerProfile`).
  - Mutations (boosts, hidden-information toggle) live inline in `GeneralPlayerProfile`.
  - Header best-role summaries select best-current and best-potential roles independently (`bestRoleScore` plus `bestPotentialRoleScore`).
  - Moneyball mode reuses `PlayerOverviewPanel` in `mode="moneyball"` with Moneyball IP/OOP summaries.
- Architectural seams:
  - Route composes panels; panels own their display logic; `utils/position-families.ts` owns role selection, phase filtering, playable-position filtering, and position defaulting; `utils/role-phase.ts` owns phase labels; `utils/attribute-groups.ts` owns attribute organization.
  - `src/components/ui/` holds Button, Modal, Panel, ScoreBadge (table, card, hero, muted variants), and related primitives; it holds no disclosure, accordion, or collapsible primitive, which justifies the feature-local Modify Player disclosure default.
- Project validation commands:
  - `./scripts/dev test <targets>` for focused and affected tests.
  - `./scripts/dev check` as the full commit gate.
  - `./scripts/dev smoke` for Playwright product evidence (`e2e/smoke.spec.ts`, Chromium) for the geometry commits.
- Primary risks:
  - URL compatibility regressions for saved bookmarks, Search entry, and Back/Forward behavior.
  - Concealment leaks (potential values, PA, boost actions, Potential columns) during restructuring.
  - Splitting the shared overview panel so Moneyball and standard scoring stay distinct.
  - Geometry regressions at 1280×800 or 3440×1440.
  - Selecting a separate best-potential role by accident instead of the same-role current → potential pair.

## Feature architecture

- The route owns only URL parsing (`section`, legacy `view`, Attributes-local `tab`), player retention across section switches, data prefetching, and composition.
- One profile query owner and one mutation owner serve all sections; sections receive data and callbacks and own no fetching.
- An isolated section-state owner (`src/features/player-profile/utils/profile-section.ts`) maps canonical `section` plus legacy `view`/`tab` to the active section with the saved-default fallback; a navigation UI owner (`src/features/player-profile/components/player-profile-navigation.tsx`) exposes one keyboard-operable section tablist with current-state semantics; the route only parses search params, wires callbacks, and composes sections.
- A player-identity presenter owns descriptive facts plus neutral portrait and crest placeholders; a persistent rail composes it across all sections.
- A hidden-information display control owner stays separate from the Modify Player disclosure that groups the two data-changing boost actions.
- A best-current tactical-fit selector owns the approved selection rule (best current IP role, best current OOP role, each paired with its own potential score) and replaces the independent best-potential selection in summaries.
- Overview, Attributes, Role Fit, and Moneyball sections own only their presentation; Role Fit owns the position navigator plus the comparison list with an explicit possession-phase field; Moneyball owns hierarchy alignment without touching calculations and preserves raw metric values plus percentile semantics with no potential scores.
- Geometry owners prove minimum-desktop containment and ultrawide bounds last, after all content is in place.

## Uncertainty register

### Known

- The profile route, panels, selection helpers, tests, design tokens, and supported window range are all cited above from current repository evidence.
- `src/components/ui/` contains no disclosure primitive, so a feature-local Modify Player disclosure is the default unless implementation evidence proves a shared primitive is currently justified.
- The mockup binary exists untracked at `.wiki/features/assets/jay-62-player-profile-mockup.png` and is referenced, not modified, by this plan.

### Assumptions

- The TanStack Router search-param pattern in `players.$uid.tsx` extends cleanly to a `section` param with `replace` navigation for section switches that must preserve Back/Forward entries the way current `view` switches do.
- Existing test doubles in `src/testing/player-ipc-mock.ts` and `src/testing/moneyball-ipc-mock.ts` extend to the new section components without harness changes.
- Playwright smoke (`e2e/smoke.spec.ts`, Chromium) asserts workspace containment at 1280×800 and rail bounds at 3440×1440 across all four sections; native WebView inspection, where unavailable, is a recorded additional gap, never a substitute for the smoke evidence.

### Decisions

- Canonical URL state is `section=overview|attributes|role-fit|moneyball`, with `tab` preserved as Attributes-local state, because section navigation is the new primary hierarchy while attribute tabs stay local evidence controls.
- Legacy mapping sends `view=moneyball` to Moneyball, `view=general` with an explicit valid attribute tab to Attributes, and ordinary General to Overview, with a valid canonical section winning, invalid `section` or `view` values treated as absent, and absent state falling back to the saved default (General default to Overview, Moneyball default to Moneyball), because existing bookmarks, Search entry, and Back/Forward entries must keep working.
- Best tactical fit selects the best current IP and OOP roles and shows each same role's current → potential score, with no separate best-potential role, because the issue requires the relationship between role, phase, current score, and potential score to read as one pair.
- Standard Role Fit and Moneyball scoring stay distinct with no potential Moneyball scores, because Moneyball percentiles are a separate calculation without a potential basis.
- Neutral bundled placeholders or monograms stand in for portrait and crest with no image loading, because `.wiki/DESIGN.md` mandates text-first identity, bundled assets only, and no network loading.
- The condensed Overview attribute summary is omitted, because Overview must remain a summary without internal scrolling through the complete dataset.
- The Modify Player grouping is a feature-local disclosure, because no shared disclosure primitive exists in `src/components/ui/` today.
- One PR carries all 21 commits, because the work is one coherent presentation redesign with no independently mergeable trunk-safe seam and no backend boundary.

### Unknowns

None. The independent plan review accepted the packet order and confirmed that Playwright is the required geometry proof. Native WebView inspection remains an additional gap, not a substitute.

### Risks

- URL mapping mistakes break saved profile links or Search-to-profile entry; mitigation is explicit legacy-mapping tests in commit 5 plus preserved Back/Forward tests.
- Restructuring leaks concealed potential data into the Overview, Attributes, or Role Fit surfaces; mitigation is keeping the existing concealment tests green in every commit and strengthening the same-role pairing tests in commit 10.
- Moneyball and standard scoring get mixed during panel separation; mitigation is keeping both score sources side by side in commits 5, 18, and 19 with distinct labels and tests.
- Geometry work stretches cards or the rail at the extremes; mitigation is dedicated containment commits with required smoke evidence last.

## Walking skeleton

- Section navigation renders four keyboard-operable sections for one player with the identity rail visible, legacy `view`/`tab` URLs resolve to the right section, and `./scripts/dev check` passes.
- Every later commit thickens one slice of that skeleton without changing its URL or ownership contracts.

## Delivery plan

### PR 1 — Player profile workspace redesign

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/player-profile-workspace-redesign`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub required `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(profile): redesign the player analysis workspace`

**Purpose:** This PR delivers the complete JAY-62 presentation redesign as one reviewable unit: query and mutation ownership cleanup, isolated section navigation with legacy URL compatibility, the four-section workspace with persistent identity rail, separated display and modification controls, same-role current → potential summaries, dedicated Attributes and Role Fit workspaces, aligned Moneyball presentation, and proven minimum and ultrawide geometry.

**Depends on:** None.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(profile): record workspace redesign plan`

**Work:** Commit the independently reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome.
- Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/player-profile-workspace-redesign.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — JAY-62 active entry linking to the ledger.
- `.wiki/features/assets/jay-62-player-profile-mockup.png` — approved untracked directional mockup, referenced and included unmodified.

**Behavior and data flow:**

- Move planning truth into one reviewed active ledger and record the exact delivery sequence before implementation.

**Ordered implementation steps:**

1. Verify the active branch is `feature/player-profile-workspace-redesign` with base `main` without changing Git state.
2. Confirm the worktree contains only the reviewed planning paths plus the unmodified mockup binary.
3. Run the ledger classifier and `git diff --check`.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning documents only.
- The ledger classifier (`ledger_state`) and `git diff --check` prove structural consistency; checkpoint review of the staged diff additionally requires `git diff --cached --check`.

**Patterns to verify:**

- The active-ledger template, current TODO ownership rules, and accepted planning-path boundaries from the dispatch.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, BACKLOG, ADRs, plan scope, packet order, or reviewed decisions.
- Do not modify the mockup binary.
- Do not stage or commit until the worker flow owns that step; this packet only defines the commit.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/player-profile-workspace-redesign.md` plus `git diff --check`; checkpoint review of the staged planning diff additionally requires `git diff --cached --check`.

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files; verify the mockup binary is byte-identical to the approved artifact; verify TODO changes touch only the JAY-62 active entry.

#### Commit 2 — Remove duplicate profile queries

**Status:** Completed

**Provisional commit:** `refactor(profile): centralize profile query ownership`

**Work:** Give one route-level owner to the player and snapshot query subscriptions so every section reads from the same data without duplicate subscriptions.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- New sections, navigation changes, visual changes, and mutation restructuring.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/app/routes/players.$uid.tsx` — remove the duplicate `useSuspenseQuery(getPlayerQueryOptions(uid))` subscription (currently in both `PlayerProfileContent` and `GeneralPlayerProfile`); pass `snapshot` and `player` down as props.
- `src/app/routes/players.$uid.test.tsx` — keep all existing query-behavior tests green; no new behavior to prove beyond single-subscription equivalence.

**Behavior and data flow:**

- Route subscribes once to `currentSnapshotQueryOptions` and `getPlayerQueryOptions(uid)` and passes resolved `snapshot` and `player` into `GeneralPlayerProfile`, `MoneyballPlayerProfile`, and `PlayerProfileContent`.
- Rendered output is byte-equivalent in behavior; only subscription ownership changes.

**Ordered implementation steps:**

1. Confirm the duplicate subscriptions and their props flow by reading the route.
2. Hoist the single subscription pair to the narrowest common owner and thread props through.
3. Remove the now-redundant subscription without changing loading, empty, or not-found branches.
4. Run focused, affected, and gate validation in order.

**Tests and proof:**

- Behavior-preserving proof: the full existing `players.$uid.test.tsx` suite stays green with no test changes, proving no observable change.
- Focused targets cover overview identity, snapshot empty state, and unknown-uid handling.

**Patterns to verify:**

- The existing prop-threading of `uid`, `tab`, and `view` in the same route file.

**Constraints and non-goals:**

- Do not change loading skeletons, empty states, prefetch behavior, or mutation wiring.
- Do not alter Moneyball query prefetch conditions.

**Dependencies and sequencing:**

- Depends on commit 1 (planning artifact recorded).
- Required before commits 3–5, which build on single-owner data flow.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if any existing test needs more than mechanical prop updates, which signals a behavior change that needs replanning; stop if the loader prefetch contract must change.

**Review mandate:** Verify exactly one player and snapshot subscription remains on each path; verify loading, empty, and not-found branches are untouched; verify no prop drilling leaks player data into components that did not receive it before; verify the diff touches only the route file.

#### Commit 3 — Centralize profile mutations

**Status:** Completed

**Provisional commit:** `refactor(profile): centralize profile mutation ownership`

**Work:** Move boost and hidden-information mutation ownership to one route-level owner that passes stable callbacks into sections.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- Visual regrouping of the actions (commits 8–9 own that); behavior and placement stay identical.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/app/routes/players.$uid.tsx` — hoist the `hiddenInformation` and `boost` `useMutation` wiring plus the `boostContextIsCurrent` and `hiddenInformationContextIsCurrent` guards to the single query owner from commit 2; pass callbacks and pending, error, and result state down.
- `src/features/player-profile/components/player-overview-panel.tsx` — receive callbacks as props; no visual change.
- `src/app/routes/players.$uid.test.tsx` — existing boost, mentality, visibility, and failure tests stay green unchanged.

**Behavior and data flow:**

- Mutations invalidate the same query keys (`playerKeys`, `staffKeys`, `snapshotKeys`, `searchKeys`, `plannerKeys`, `academyKeys`) under the same staleness guards.
- `PlayerOverviewPanel` keeps receiving the same props values from the new owner.

**Ordered implementation steps:**

1. Hoist both mutations to the commit-2 query owner without changing mutation functions or invalidation sets.
2. Thread callbacks and state through existing props.
3. Remove the inline mutation wiring from `GeneralPlayerProfile`.
4. Run focused, affected, and gate validation in order.

**Tests and proof:**

- Behavior-preserving proof: existing boost preview, confirm, error, cross-player outcome, visibility toggle, failure, and save-change tests stay green unchanged.
- No new tests; placement and labels are provably untouched.

**Patterns to verify:**

- The current invalidation sets and context guards, copied exactly.

**Constraints and non-goals:**

- Do not change mutation functions, invalidation keys, confirmation modal behavior, tooltips, or button order.
- Do not move the hidden-information toggle or boost buttons visually.

**Dependencies and sequencing:**

- Depends on commit 2.
- Required before commits 8–9, which regroup the same callbacks visually.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if invalidation behavior must change to keep tests green; stop if callback identity changes break the confirmation modal flow.

**Review mandate:** Verify mutation functions and invalidation sets are character-identical in behavior; verify context guards still drop stale save feedback; verify no visual or prop-value change in the overview panel; verify the diff touches only ownership lines.

#### Commit 4 — Extract existing analysis navigation

**Status:** Completed

**Provisional commit:** `refactor(profile): isolate analysis navigation state`

**Work:** Isolate the current General/Moneyball view navigation (state, keyboard handling, focus restoration) into one navigation owner without changing its behavior.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- The four-section model (commit 5 owns it); the visible tabs stay General and Moneyball.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/features/player-profile/components/player-profile-navigation.tsx` — new navigation UI owner holding the extracted `PlayerAnalysisTabs` state transitions, `onViewChange`, and focus-restoration handling with identical rendered tablist.
- `src/app/routes/players.$uid.test.tsx` — arrow-key, direct-click, delayed-navigation focus, and view-persistence tests stay green unchanged.

**Behavior and data flow:**

- Same `view` search param, same `replace` navigation, same `tab` preservation across view switches, same roving `tabIndex` and focus restoration; the route only parses, wires, and composes.

**Ordered implementation steps:**

1. Extract the navigation state and handlers without changing the rendered tablist.
2. Keep all IDs (`player-analysis-tab-*`, `player-analysis-panel`) and ARIA wiring identical.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- Behavior-preserving proof: existing analysis-view tests (arrow keys, click focus, delayed focus, explicit-view precedence, unknown-view normalization) stay green unchanged.

**Patterns to verify:**

- The `PlayerProfileTabs` keyboard pattern in `player-profile-tabs.tsx` as the analogue for any shared handler shape.

**Constraints and non-goals:**

- Do not rename tab IDs, roles, or labels.
- Do not change `replace` versus push navigation semantics.

**Dependencies and sequencing:**

- Depends on commit 3.
- Required before commit 5, which extends this owner to four sections.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if focus-restoration timing changes observably; stop if Back/Forward semantics would need to change.

**Review mandate:** Verify IDs, roles, labels, and navigation mode are unchanged; verify focus restoration still works after delayed Moneyball resolution; verify no visual change; verify `player-profile-navigation.tsx` owns only navigation UI and the route only parses, wires, and composes.

#### Commit 5 — Introduce four-section vertical slice with legacy URL compatibility

**Status:** Completed

**Provisional commit:** `feat(profile): add profile workspace sections`

**Work:** Introduce `section=overview|attributes|role-fit|moneyball` as canonical URL state with a keyboard-operable four-section tablist, the complete canonical/legacy/default URL resolution, and the Moneyball loader condition, reusing the existing panels behind each section without redesigning them yet. This vertical slice cannot split further: sections without resolution ship dead UI unreachable by URL, resolution without sections ships navigation that promises sections that do not exist, and a detached loader condition leaves the Moneyball section flashing empty or stale data — any split produces a lying or broken trunk state.

**Size assessment:** Medium; within the soft target (route plus two feature modules; panels untouched).

**Out of scope:**

- Identity rail extraction (commits 6–7); any panel redesign.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/features/player-profile/utils/profile-section.ts` — new section state and resolution owner: section parsing plus `resolveProfileSection({ section, view, tab, defaultAnalysisView })` with the exact precedence (valid canonical `section` wins; else `view=moneyball` to Moneyball; else `view=general` with an explicit valid attribute tab to Attributes; else ordinary General to Overview; invalid `section` or invalid `view` treated as absent; absent `section` and `view` to the saved default with General default to Overview and Moneyball default to Moneyball), with tests.
- `src/features/player-profile/components/player-profile-navigation.tsx` — extend the commit-4 navigation UI to four sections with `aria-current` or `aria-selected` semantics and the same keyboard pattern as `PlayerAnalysisTabs`.
- `src/app/routes/players.$uid.tsx` — only parses search params (`section`, legacy `view`, Attributes-local `tab`), wires the resolver into `validateSearch` defaults and the loader prefetch condition (Moneyball prefetch exactly when the resolved section is Moneyball), renders the matching existing panel composition per section, and retains the current player across switches.
- `src/app/routes/players.$uid.test.tsx` — section-switch retention, keyboard operation, current-state semantics, legacy URL mapping branch, Back/Forward-compatible navigation, and unchanged Search semantics tests (no Search file changes; assert profile-side only).

**Behavior and data flow:**

- Entry parses legacy params once into the canonical section; all downstream rendering reads the canonical section.
- Omitted or invalid `section` resolves through the legacy/`view`/`tab`/saved-default chain; an invalid `view` is treated as absent and falls through to the saved default.
- Section switches use the same `replace` navigation discipline as current view switches and keep `uid`, `tab`, and data subscriptions intact.
- `tab` parsing (`parseProfileTab`, legacy normalization) is reused unchanged and stays Attributes-local.

**Ordered implementation steps:**

1. Add the section parser and resolver with RED tests for valid values, unknown values, omitted state, every mapping branch including precedence (section beats view), invalid-section fallback, invalid-`view` fallthrough to the saved default, and the saved-default fallback.
2. Extend the navigation UI to four sections with the same keyboard pattern as `PlayerAnalysisTabs`.
3. Render existing panel compositions behind each section (Overview shows the current header; Attributes shows the attributes panel; Role Fit shows the roles panel; Moneyball shows the Moneyball composition).
4. Wire the resolver into search validation and the loader Moneyball-prefetch condition.
5. Add section retention, keyboard, semantics, mapping, and Back/Forward tests GREEN.
6. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: new parser tests fail on unknown `section` before normalization is implemented; new route tests fail to find the four-section tablist before the navigation extension; each new mapping test fails before the resolver exists (for example `?view=moneyball` does not select Moneyball; `?view=general&tab=hidden` does not select Attributes).
- GREEN assertions: switching sections retains the player heading; each tab exposes selected state; arrow keys move between all four sections; all mapping branches resolve per the precedence including invalid-value fallthrough; the loader prefetches Moneyball exactly when the resolved section is Moneyball.
- The existing unknown-view test is rewritten to assert saved-default fallthrough (invalid `view` with a Moneyball saved default selects Moneyball; with a General saved default selects Overview) since the four-section tablist intentionally supersedes the General-tab assertion; all other view, tab, and concealment tests stay green unchanged.

**Patterns to verify:**

- `PlayerAnalysisTabs` keyboard and focus-restoration pattern; `parseProfileTab` and `parsePlayerProfileView` normalization patterns in the same route.

**Constraints and non-goals:**

- Do not redesign any panel in this commit.
- Do not change scoring, concealment, or mutation behavior.
- Do not change Search route files or Search URL semantics.
- Do not change `replace` navigation discipline.
- Do not invent new query params.

**Dependencies and sequencing:**

- Depends on commit 4.
- Required before commits 6–7 and all section-content commits, which assume stable section resolution.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx src/features/player-profile/utils` then `./scripts/dev check`.

**Stop conditions:** Stop if the four-section model cannot preserve the current player without refetching; stop if keyboard semantics conflict with the existing tablist pattern (needs a developer decision on ARIA structure); stop if any currently passing direct-link or Back/Forward test cannot be preserved under the mapping (report rather than reordering packets); stop if Search semantics would need to change.

**Review mandate:** Verify section parsing rejects unknown values to the mapped default; verify every mapping branch in the invariants has a test including invalid-`view` fallthrough to the saved default; verify canonical-section precedence; verify loader prefetches Moneyball exactly when the resolved section needs it; verify player retention without extra queries; verify keyboard operation across all four sections; verify current-state semantics are exposed; verify panels are byte-equivalent compositions; verify no Search file changed; verify Back/Forward entries still work; verify no scoring or concealment change; verify `profile-section.ts` owns only state and resolution, `player-profile-navigation.tsx` owns only navigation UI, and the route only parses, wires, and composes.

#### Commit 6 — Extract identity presentation

**Status:** Completed

**Provisional commit:** `refactor(profile): isolate player identity presentation`

**Work:** Extract the descriptive identity facts (name, club, competition, nationality, age/DOB, height, foot, value, flags) into one identity presenter with no analytical values and no visual change.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- The persistent rail layout (commit 7 owns it); removing CA/PA/role summaries from the header (happens in commits 7 and 11–12 as sections take ownership).

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- New `src/features/player-profile/components/player-identity.tsx` — `PlayerIdentity` presenter receiving `player`, rendering the facts currently in the top band of `PlayerOverviewPanel` with identical formatting helpers (`formatPlayerDob`, `formatPreferredFoot`, `formatMoney`, `formatMissable`, `NationalityCell`) and flag rendering.
- `src/features/player-profile/components/player-overview-panel.tsx` — compose the presenter; no visual change.
- Tests: existing overview identity tests stay green; add presenter-level tests only if a realistic regression needs them (existing route coverage is the stronger seam).

**Behavior and data flow:**

- Same `player` in, same facts out; analytical values (CA, PA, role summaries) stay where they are for now.

**Ordered implementation steps:**

1. Extract the identity markup into the presenter with identical classes and helpers.
2. Compose it in the overview panel.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- Behavior-preserving proof: existing identity tests (heading, club/division, DOB, nationality flags, height, foot, flags) stay green unchanged.

**Patterns to verify:**

- `SummaryFact` micro-label pattern and DESIGN.md truncation rules (`truncate` plus `title`).

**Constraints and non-goals:**

- Do not add portrait or crest slots yet.
- Do not move or restyle any analytical value.
- Do not change accessible names of the summary region.

**Dependencies and sequencing:**

- Depends on commit 5.
- Required before commit 7.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if identity facts cannot be separated without duplicating formatting logic (prefer reuse over duplication per minimalism).

**Review mandate:** Verify the presenter contains zero analytical values; verify formatting helpers are reused, not copied; verify accessible names and truncation are unchanged; verify no visual diff.

#### Commit 7 — Install persistent identity rail

**Status:** Completed

**Provisional commit:** `feat(profile): add persistent player identity rail`

**Work:** Render the persistent left identity rail (identity presenter plus neutral portrait and crest placeholder slots, including market value) across all four sections with a bounded width.

**Size assessment:** Medium; within the soft target.

**Out of scope:**

- Section content redesign; geometry proof at the extremes (commits 20–21 own it).

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/app/routes/players.$uid.tsx` — compose the rail beside the section workspace on every section.
- Identity presenter from commit 6 — add reserved portrait and crest slots using neutral placeholders or monograms on `surface-container-high` per DESIGN.md text-first identity; no image loading, no network calls, no new dependencies.
- `src/app/routes/players.$uid.test.tsx` — rail persistence across section switches, placeholder presence without images, and identity/analysis separation tests.

**Behavior and data flow:**

- Rail receives `player`; workspace receives section content; both share the single query owner.

**Ordered implementation steps:**

1. Add RED tests: rail visible on all four sections, including market value; no `img` loads portrait or crest assets; identity region and analysis region are distinct landmarks or labelled regions.
2. Build the rail composition GREEN with bounded width and placeholder slots.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: rail persistence test fails (rail absent on at least one section) before composition; market-value test fails while the rail omits value; placeholder test fails if any remote or bundle image is referenced.
- GREEN assertions: same player identity and market value on every section; placeholders render neutral content with accessible labels; CA, PA, and role suitability are absent from the rail.

**Patterns to verify:**

- DESIGN.md text-first identity (initials monogram on `surface-container-high` or omitted slot) and Panel hairline conventions.

**Constraints and non-goals:**

- Do not load, fetch, or bundle portrait or crest images.
- Do not present CA, PA, or role suitability inside the rail; market value stays in the rail by design.
- Do not change the analysis content of any section yet.

**Dependencies and sequencing:**

- Depends on commit 6.
- Required before commits 11–12 and 17–18, which assume the rail owns identity.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if the rail cannot persist without remounting player state on section switches; stop if placeholder styling needs a new shared primitive (prefer the feature-local slot per the approved direction).

**Review mandate:** Verify the rail persists with identical identity and market value on all sections; verify CA, PA, and role suitability are absent from the rail; verify no image loading or network surface; verify placeholders carry honest accessible names; verify heading order stays correct.

#### Commit 8 — Separate hidden-information display control

**Status:** Completed

**Provisional commit:** `refactor(profile): separate hidden information controls`

**Work:** Move the show/hide hidden-information toggle out of the modification action group into its own view/display control position with identical behavior.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- Grouping the boost actions (commit 9 owns that); behavior changes.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/app/routes/players.$uid.tsx` and `src/features/player-profile/components/player-overview-panel.tsx` — relocate the toggle rendering to a display-control slot; reuse the commit-3 callback, pending, and error props exactly.
- `src/app/routes/players.$uid.test.tsx` — existing visibility tests stay green; update only selectors that name the old grouping, including the action-row order test which now asserts the toggle is separate from modification actions.

**Behavior and data flow:**

- Same mutation, same `aria-pressed`, same pending and error rendering; only ownership of placement changes.

**Ordered implementation steps:**

1. Relocate the toggle to the display-control slot.
2. Update affected selectors minimally.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- Behavior-preserving proof with one deliberate selector update: all concealment tests (conceal values, PA removal, boost-action hiding, failure alert, save-change reset) stay green; the action-row order test is rewritten to assert the toggle is no longer grouped with Boost actions.

**Patterns to verify:**

- Button secondary variant and `aria-pressed` usage already in the panel.

**Constraints and non-goals:**

- Do not group the toggle with Modify Player actions.
- Do not change toggle behavior, labels, or accessible names.

**Dependencies and sequencing:**

- Depends on commits 3 and 7.
- Required before commit 9, which groups what remains.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if concealment tests fail for any reason beyond mechanical selector updates.

**Review mandate:** Verify the toggle is structurally separate from modification actions; verify behavior, labels, and `aria-pressed` are unchanged; verify concealment still hides PA, potentials, boost actions, and Potential columns; verify the rewritten order test asserts the new structure.

#### Commit 9 — Group modification actions

**Status:** Active

**Provisional commit:** `feat(profile): group player modification actions`

**Work:** Group Boost CA and Wonderkid Mentality under a secondary feature-local Modify Player disclosure, visually distinguished from navigation and analysis, with unchanged boost behavior and accessible labels.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- Shared primitive creation; behavior changes to boosts or confirmation.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- New feature-local disclosure in `src/features/player-profile/components/player-development-boosts-panel.tsx` — native `<details>`/`<summary>` or a minimal button-disclosure reusing Button and Modal patterns; keyboard operable with `aria-expanded` and clear labels.
- `src/app/routes/players.$uid.test.tsx` — boost tests updated for the disclosure step (open Modify Player, then Boost CA); assertions on preview, confirm, error, and outcome text unchanged.

**Behavior and data flow:**

- Same callbacks, previews, confirmation modal, outcomes, and invalidations; only the buttons sit behind one disclosure.

**Ordered implementation steps:**

1. Add RED tests: boost actions are hidden behind the Modify Player control initially and reachable by keyboard; hidden-information toggle is outside the disclosure.
2. Build the feature-local disclosure GREEN reusing existing `PlayerDevelopmentActions` content.
3. Update existing boost tests for the extra open step.
4. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: new disclosure tests fail (actions visible without opening, or unreachable by keyboard) before grouping.
- GREEN assertions: disclosure opens and closes by keyboard; boost flows complete from inside; toggle stays outside.
- All existing boost and mentality assertions (previews, caps, eligibility, errors) stay green with mechanical open-step updates.

**Patterns to verify:**

- Button secondary/ghost usage and Modal confirmation flow already in the boosts panel; no shared disclosure primitive exists, so feature-local is correct.

**Constraints and non-goals:**

- Do not create or extend a shared UI primitive.
- Do not change boost eligibility, previews, confirmation copy, or outcomes.
- Keep the disclosure visually secondary per the hierarchy (identity, section, analysis, metrics, secondary actions).

**Dependencies and sequencing:**

- Depends on commit 8.
- Required before final visual review of the rail and Overview.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if repository evidence during implementation proves a shared disclosure primitive is currently justified (record and seek a decision; do not silently promote it); stop if keyboard or focus behavior of the confirmation modal regresses.

**Review mandate:** Verify the disclosure is feature-local with no new shared API; verify keyboard operation and `aria-expanded`; verify boost behavior is identical; verify the hidden-information control is outside; verify accessible labels are clear.

#### Commit 10 — Define best-current tactical-fit selection

**Status:** Pending

**Provisional commit:** `feat(profile): select best current tactical fits`

**Work:** Replace the independent best-potential selection with the approved rule: best current IP role and best current OOP role, each paired with its own potential score.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- Overview layout (commits 11–12 own it); Moneyball summaries (commit 18 owns them).

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/features/player-profile/utils/position-families.ts` — add a best-current-pair selector `bestCurrentRolePair` built on `rolesForPlayablePositions`, `rolesForPhase`, and `bestRoleScore`, returning each best-current role with its own `potentialScore`; keep `bestPotentialRoleScore` for its remaining consumers or remove it only with its tests if it becomes unused.
- `src/features/player-profile/utils/position-families.test.ts` — new selection-rule tests.
- `src/features/player-profile/components/player-overview-panel.tsx` — consume the new selector for the standard summaries.
- `src/app/routes/players.$uid.test.tsx` — update the phase-specific summary test to the same-role expectation.

**Behavior and data flow:**

- Input role scores and playable-position filtering are unchanged; only the pairing rule changes from best-potential role to same-role potential.

**Ordered implementation steps:**

1. Add RED selector tests: tie-breaking keeps the earlier catalog entry; unplayable positions are excluded; each pair carries the same role's current and potential scores; null potential stays null.
2. Implement the selector GREEN.
3. Switch the overview panel to the selector.
4. Update the obsolete separate-potential test expectations.
5. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: new selector tests fail before the rule exists (for example a fixture where the best-potential role differs from the best-current role returns the wrong role).
- GREEN assertions: Overview shows best-current role names with `current → potential` pairs; unavailable potentials render the neutral `—` without a badge.
- Obsolete tests to rewrite: the phase-specific summary test asserting separate Potential IP/OOP role names.

**Patterns to verify:**

- `bestRoleScore` tie and null handling as the analogue.

**Constraints and non-goals:**

- Do not touch Moneyball scoring or summaries.
- Do not invent potential Moneyball scores.
- Respect concealment: hidden potentials still render concealed when unrevealed.

**Dependencies and sequencing:**

- Depends on commit 6 (identity extraction keeps the summary area clean).
- Required before commit 12, which presents the pairs.

**Validation:** `./scripts/dev test src/features/player-profile/utils/position-families.test.ts src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if any consumer still needs the independent best-potential role (record it rather than removing `bestPotentialRoleScore` silently).

**Review mandate:** Verify the pairing rule matches the dispatch exactly; verify playable-position filtering still excludes unplayable roles; verify tie-breaking and null handling; verify Moneyball paths are untouched; verify concealment still gates potentials.

#### Commit 11 — Build Overview ability summary

**Status:** Pending

**Provisional commit:** `feat(profile): add overview ability summaries`

**Work:** Build the Overview ability summary: a distinct ability region presenting CA, PA, and market value with a clear card or region hierarchy below the section title, owned by the Overview section and separated from identity. CA, PA, and value fact rows already exist inside the shared header summary today; this packet owns only the distinct Overview ability region and its hierarchy, not the values themselves.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- Tactical-fit summary (commit 12 owns it); attribute summary (omitted by decision).

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/features/player-profile/components/player-overview-panel.tsx` — ability summary block with its own labelled region `data-testid="overview-ability"` presenting CA, PA (concealment-gated), and market value using existing formatters, tabular numerals, and `—` for missing values.
- `src/app/routes/players.$uid.test.tsx` — ability summary tests (distinct ability region and prominence hierarchy, market value present in both the rail and the Overview region, concealment gating, missing-value rendering).

**Behavior and data flow:**

- Same `player` values, same concealment flag; presentation-only change.

**Ordered implementation steps:**

1. Add RED tests: Overview exposes a distinct ability region with its own hierarchy below the section title showing CA, PA, and market value; concealed PA is absent with hidden info hidden; missing value renders `—`; market value renders in both the rail and the Overview region.
2. Build the summary GREEN reusing `SummaryFact` styling and formatters, moving the values into the distinct region.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: new region tests fail while CA/PA/value render only inside the shared header summary without a distinct Overview ability region (the values exist today; the region and hierarchy are the missing outcome).
- GREEN assertions: CA/PA/market value present in the distinct region and correctly gated; market value also present in the rail by design; no duplication of other identity facts.
- Existing CA/PA concealment tests stay green.

**Patterns to verify:**

- DESIGN.md micro-label pattern and mono usage for hero metrics.

**Constraints and non-goals:**

- Do not reproduce the complete profile or add an attribute summary.
- Do not let the summary dominate the workspace (hierarchy: identity, section, summary, metrics, secondary actions).
- Market value in both the rail and the Overview region is required; the no-duplication rule applies to all other identity facts.

**Dependencies and sequencing:**

- Depends on commits 7 and 10.
- Required before final Overview review.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if ability values cannot reuse existing formatters without duplication.

**Review mandate:** Verify CA/PA/value correctness and concealment gating; verify a distinct ability region with visual hierarchy (prominent, not dominating); verify market value renders in both the rail and the Overview region; verify no other identity fact duplicates; verify missing values use `—`.

#### Commit 12 — Build Overview tactical-fit summary

**Status:** Pending

**Provisional commit:** `feat(profile): add overview tactical fit summaries`

**Work:** Build the Overview tactical-fit summary: best IP role and best OOP role as two clearly related summaries using the commit-11 pairs with the standard `current → potential` visual language.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- Attribute summary (omitted); Moneyball summaries (commit 18 owns them).

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- Overview section from commit 11 (`player-overview-panel.tsx`) — IP and OOP summary cards or rows consuming the commit-10 selector, with screen-reader-friendly current/potential text plus the compact visual pair.
- `src/app/routes/players.$uid.test.tsx` — tactical-fit summary tests (role, phase, current, potential immediately understandable; concealed and unavailable states).

**Behavior and data flow:**

- Selector pairs in, paired presentation out; concealment gates potentials as before.

**Ordered implementation steps:**

1. Add RED tests: best IP and OOP roles show with same-role current → potential; concealed potentials render concealed; unavailable potentials render `—` without a badge.
2. Build the summaries GREEN with aligned values and readable phase labels.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: new summary tests fail before the paired presentation exists.
- GREEN assertions: role-to-phase-to-scores relationship reads without verbose repeated labels; numbers stay the facts with colour as reinforcement.

**Patterns to verify:**

- Existing `BestRoleSummary` and ScoreBadge hero variant; attributes panel `Current → Potential` paired pattern with `sr-only` text.

**Constraints and non-goals:**

- Do not select a separate best-potential role.
- Do not repeat verbose labels around every value.
- Do not rely on colour alone.

**Dependencies and sequencing:**

- Depends on commits 10 and 11.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if the paired presentation cannot stay understandable without verbose labels (needs a design decision, not silent label repetition).

**Review mandate:** Verify same-role pairing; verify phase readability; verify concealed and unavailable states; verify colour is never the sole indicator; verify hierarchy below identity and section title.

#### Commit 13 — Expand Attributes workspace

**Status:** Pending

**Provisional commit:** `feat(profile): expand the attributes workspace`

**Work:** Give Attributes its dedicated full-width geometry and main-workspace scroll ownership, reusing the existing attribute-layout system, local category controls, paired current → potential presentation, tabular numerals, and semantic colouring. Isolation of Attributes behind its own section already arrives in the commit-5 vertical slice; this packet owns only the still-missing width rules and scroll container, which are an independent reviewable outcome (no competing Role Fit panel, content-owned scrolling).

**Size assessment:** Medium; within the soft target.

**Out of scope:**

- Attribute data or grouping changes; concealment changes.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/features/player-profile/components/player-attributes-panel.tsx` — full-width layout rules and the section scroll container, existing `PlayerProfileTabs` kept as local controls, existing `AttributeSection` grouping (Technical, Mental, Physical, Set Pieces, goalkeeping, hidden, personality) reused.
- `src/app/routes/players.$uid.test.tsx` plus `attribute-groups.test.ts` — existing attribute tests stay green; add workspace layout tests (full-width section rules, section-owned scrolling, local tab ownership).

**Behavior and data flow:**

- Same `player`, `tab`, `onTabChange`, and `hiddenInformationRevealed` inputs; only the section container and width change.

**Ordered implementation steps:**

1. Add RED tests: Attributes section applies full-width rules with section-owned scrolling and no competing Role Fit panel; category tabs remain local and keyboard operable.
2. Apply the width and scroll-ownership rules GREEN reusing existing sections and paired rows.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: workspace tests fail while the Attributes section still inherits the shared two-column workspace grid without its own width rules or scroll container (section isolation itself already arrives in commit 5 and is not re-proved here).
- GREEN assertions: full-width section with section-owned scrolling; all four categories reachable; current → potential pairs aligned with tabular numerals; hidden and personality gating unchanged.
- Existing attribute rendering, goalkeeper ordering, legacy tab normalization, and arrow-key tests stay green.

**Patterns to verify:**

- Existing `AttributeSection` subgroups and `profileTabPanelProps` hidden-panel pattern.

**Constraints and non-goals:**

- Do not change attribute groups, values, colouring rules, or concealment.
- Keep attribute names vertically scannable with consistently aligned numbers; the section owns scrolling so the main workspace does not scroll twice.

**Dependencies and sequencing:**

- Depends on commit 5 (section model).
- Independent of commits 11–12 except for shared route composition.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx src/features/player-profile/utils/attribute-groups.test.ts` then `./scripts/dev check`.

**Stop conditions:** Stop if full-width layout forces attribute regrouping (regrouping is out of scope; report instead).

**Review mandate:** Verify width rules and scroll ownership are the only layout change; verify all categories and groupings are byte-equivalent in data; verify local tab ownership; verify paired presentation, tabular numerals, and colouring preserved; verify no competing panel; verify exactly one vertical scroll owner for the section.

#### Commit 14 — Mute unfamiliar positions

**Status:** Pending

**Provisional commit:** `feat(profile): mute unfamiliar position choices`

**Work:** Restyle unfamiliar positions in the Role Fit navigator with muted styling so low familiarity values no longer compete visually, while keeping all familiarity values available. Unfamiliar means exactly unrecorded (non-numeric familiarity) or numeric familiarity below `PLAYABLE_POSITION_FAMILIARITY` (15); familiarity at or above 15 never mutes; selected-position styling overrides muted styling; values and the threshold are unchanged.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- Selection indication changes (commit 15 owns them); playable thresholds (unchanged).

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/components/profile-position-picker/profile-position-picker.tsx` — muted classes exactly for unrecorded positions and numeric familiarity below `PLAYABLE_POSITION_FAMILIARITY` (15); familiarity text (`—` or value) always rendered; selected styling wins over muted styling.
- `src/app/routes/players.$uid.test.tsx` — muted-styling tests (unrecorded muted, below-15 muted, at-or-above-15 not muted, selected overrides muted) plus retained familiarity-value assertions.

**Behavior and data flow:**

- Same `positions`, `selectedPosition`, and `onSelectPosition`; class-only change plus preserved accessible names.

**Ordered implementation steps:**

1. Add RED tests: unrecorded and below-15 positions carry muted classes; at-or-above-15 positions carry none; the selected position keeps selected styling even when unfamiliar; every familiarity value remains in the accessible tree.
2. Apply muted styling GREEN.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: muted-class assertions fail before restyling.
- GREEN assertions: unrecorded and below-15 values are present but visually de-emphasized; 15 and above never mute; selected styling overrides muted styling; playable thresholds (`PLAYABLE_POSITION_FAMILIARITY`) unchanged.
- Existing familiarity, SW-omission, and tier tests stay green unless the muted treatment intentionally supersedes a tier emission (record any such update explicitly).

**Patterns to verify:**

- Current tier-class pattern in the picker; DESIGN.md muted `on-surface-variant` usage.

**Constraints and non-goals:**

- Do not change playable thresholds, position sets, or selection behavior.
- Do not remove familiarity values.
- Do not mute familiarity at or above 15; do not let muted styling override the selected position.

**Dependencies and sequencing:**

- Depends on commit 5.
- Required before commit 15, which layers selection reinforcement on top.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if muting requires changing the tier semantics used elsewhere (tiers are shared meaning; report instead of redefining).

**Review mandate:** Verify exactly the unrecorded and below-15 positions are muted without losing values; verify 15 and above read clearly; verify the selected position keeps selected styling; verify accessible names unchanged; verify thresholds untouched.

#### Commit 15 — Mark selected positions without colour alone

**Status:** Pending

**Provisional commit:** `feat(profile): reinforce selected position state`

**Work:** Make the selected Role Fit position obvious through more than colour alone (shape, border, indicator, or label reinforcement alongside the existing gold treatment).

**Size assessment:** Small; within the soft target.

**Out of scope:**

- Familiarity muting (commit 14 owns it); behavior changes.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/components/profile-position-picker/profile-position-picker.tsx` — non-colour selection indicator reinforcing `aria-pressed` (for example a visible ring plus weight or marker change, reusing the existing selected classes as the base).
- `src/app/routes/players.$uid.test.tsx` — selection-indicator tests.

**Behavior and data flow:**

- Same selection state and callbacks; presentation reinforcement only.

**Ordered implementation steps:**

1. Add RED tests: the selected position exposes a non-colour indicator (assert the marker element or reinforced attribute beyond class colour).
2. Build the indicator GREEN.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: indicator assertions fail before reinforcement.
- GREEN assertions: selected position identifiable with colour removed (structural marker plus `aria-pressed`); keyboard selection still works.

**Patterns to verify:**

- DESIGN.md rule that selected states pair tint with an indicator plus semantics (table-row 2px gold indicator plus `aria-selected` is the analogue).

**Constraints and non-goals:**

- Keep the gold selected treatment; add to it rather than replacing it.
- Do not rely on colour alone anywhere in the new indicator.

**Dependencies and sequencing:**

- Depends on commit 14.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if the indicator needs a new icon asset (prefer CSS or text markers from the bundle; no new dependencies).

**Review mandate:** Verify the selection reads without colour; verify `aria-pressed` still correct; verify keyboard operation; verify muted unfamiliar styling from commit 14 is preserved.

#### Commit 16 — Expose possession phase as comparison field

**Status:** Pending

**Provisional commit:** `feat(profile): expose role possession phases`

**Work:** Give every Role Fit row an explicit readable IP/OOP phase chip inside the role cell next to the role name, with readable text and accessible names instead of small inferred text. Column geometry stays untouched here; commit 17 owns the aligned columns.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- Geometry alignment (commit 17 owns it); scoring changes.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/features/player-profile/components/player-roles-panel.tsx` — readable phase chip inside the role cell beside the role name using `rolePhaseLabel` with `IP`/`OOP` text and full phase accessible names; the role cell keeps role name plus chip, and the table keeps its current columns.
- `src/features/player-profile/utils/role-phase.ts` plus `role-phase.test.ts` — extend only if a compact label helper is needed.
- `src/app/routes/players.$uid.test.tsx` — phase-field tests.

**Behavior and data flow:**

- Same roles, scores, and sort; one additional explicit field per row.

**Ordered implementation steps:**

1. Add RED tests: every role row exposes an IP or OOP chip inside the role cell with readable text and an accessible name.
2. Build the phase chip GREEN.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: phase-chip assertions fail while phase is only small inferred text.
- GREEN assertions: readable chips with text inside the role cell; screen-reader names state the full phase; sorting, filtering, and column structure unaffected.

**Patterns to verify:**

- `rolePhaseLabel` mapping; Status Chip text-plus-icon rule as the analogue for never relying on abbreviation alone.

**Constraints and non-goals:**

- Do not require users to infer phase from role names.
- Keep indicators compact so the comparison columns keep their alignment.

**Dependencies and sequencing:**

- Depends on commit 5.
- Required before commit 17, which aligns the full row geometry.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx src/features/player-profile/utils/role-phase.test.ts` then `./scripts/dev check`.

**Stop conditions:** Stop if the phase field crowds out role names at 1280×800 (defer to commit 17 geometry rather than truncating names here).

**Review mandate:** Verify every row carries a readable IP/OOP chip inside the role cell; verify accessible names; verify the table columns are otherwise untouched; verify no scoring or sort change.

#### Commit 17 — Complete Role Fit geometry

**Status:** Pending

**Provisional commit:** `feat(profile): align role fit comparisons`

**Work:** Align the Role Fit comparison geometry into Role | Phase | Current | Potential columns: position navigator beside a comparison list with vertically aligned current and potential columns, a dedicated phase column carrying the commit-16 chips, preserved sort headers, and concealment-gated Potential column.

**Size assessment:** Medium; within the soft target.

**Out of scope:**

- Extreme-viewport proof (commits 20–21 own it); scoring or sort behavior changes.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/features/player-profile/components/player-roles-panel.tsx` — navigator plus comparison-list grid or table geometry with Role, Phase, Current, and Potential columns (colgroup widths, tabular numerals, sticky headers where already present, concealment-gated Potential column); the commit-16 chip moves from the role cell into the Phase column with its readable text and accessible names intact.
- `src/app/routes/players.$uid.test.tsx` — geometry and alignment tests (column structure, aligned values, sort preservation, concealment gating).

**Behavior and data flow:**

- Same selected position, roles, sort state, and familiarity data; layout-only change.

**Ordered implementation steps:**

1. Add RED tests: Role, Phase, Current, and Potential columns exist with aligned values; the phase chip renders in the Phase column; sort headers keep working; concealed mode omits the Potential column.
2. Align the geometry GREEN.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: column assertions fail before the geometry change (no dedicated Phase column; values not vertically aligned).
- GREEN assertions: quick vertical comparison across roles; phase readable per row without crowding role names; sort ascending and descending from either header; empty-position state preserved.
- Existing sort, filter-by-position, labelled-badge, and full-catalog tests stay green.

**Patterns to verify:**

- DESIGN.md numeric alignment (right-aligned tabular figures) and the shared analysis table geometry conventions.

**Constraints and non-goals:**

- Do not change role catalog, scoring, sort behavior, or concealment rules.
- Do not compress role names or comparison values until unreadable.

**Dependencies and sequencing:**

- Depends on commits 14–16.
- Required before commit 21 spot-checks Role Fit at ultrawide.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if alignment needs virtualization or a shared table migration (out of scope; report instead).

**Review mandate:** Verify Role | Phase | Current | Potential column structure with vertical alignment of both score columns; verify sort behavior identical; verify the phase chip fits without crowding role names; verify concealment gating; verify empty states preserved.

#### Commit 18 — Relocate Moneyball tactical summaries

**Status:** Pending

**Provisional commit:** `feat(moneyball): relocate profile role summaries`

**Work:** Move the Moneyball IP/OOP tactical summaries out of the shared overview header into the Moneyball section so standard and Moneyball scoring never share one summary block.

**Size assessment:** Small; within the soft target.

**Out of scope:**

- Moneyball calculation or data changes; full Moneyball hierarchy alignment (commit 19 owns it).

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/features/moneyball/components/moneyball-role-summary.tsx` — new Moneyball tactical summary owner rendering Moneyball IP/OOP summaries from the ready profile's `roleScores` with distinct Moneyball labels; the route only composes it into the Moneyball section.
- `src/features/player-profile/components/player-overview-panel.tsx` — remove the `mode="moneyball"` summary branch or narrow it to identity only; keep the standard summary path intact.
- `src/app/routes/players.$uid.test.tsx` — Moneyball summary relocation tests; standard summary tests unchanged.

**Behavior and data flow:**

- Moneyball role scores flow only into `moneyball-role-summary.tsx` inside the Moneyball section; standard role scores flow only into standard sections; the route only composes.

**Ordered implementation steps:**

1. Add RED tests: Moneyball section shows Moneyball IP/OOP summaries; the shared header no longer mixes Moneyball scoring into standard views.
2. Relocate the summaries GREEN with distinct Moneyball labels.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: relocation tests fail while Moneyball summaries render only in the shared header.
- GREEN assertions: Moneyball IP/OOP names, scores, and unavailable states render in the Moneyball section; standard views show no Moneyball roles.
- Existing Moneyball no-data, no-natural-position, and ready-workspace tests stay green with mechanical selector updates only.

**Patterns to verify:**

- The current `mode="moneyball"` branch as the exact code being separated.

**Constraints and non-goals:**

- Do not alter Moneyball calculations, comparison basis, or exposed details.
- Do not invent potential Moneyball scores.
- Keep standard and Moneyball labels unmistakably distinct.

**Dependencies and sequencing:**

- Depends on commits 5, 7, and 10.
- Required before commit 19.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx src/features/moneyball/components/moneyball-profile-panel.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if any Moneyball detail is only reachable through the shared header (preserve access first, then relocate).

**Review mandate:** Verify no scoring path is shared or mixed; verify labels distinguish Moneyball from standard summaries; verify unavailable and no-data states still render; verify standard summaries are untouched; verify `moneyball-role-summary.tsx` owns the summaries and the route only composes.

#### Commit 19 — Integrate Moneyball hierarchy

**Status:** Pending

**Provisional commit:** `feat(moneyball): align the profile workspace`

**Work:** Align the Moneyball section with the workspace hierarchy: same identity rail, same section navigation, same panel hierarchy, same current/potential visual language, typography, and spacing, so moving between sections feels like one coherent workspace. The rail and navigation already persist from commits 5 and 7; this packet owns only the still-missing Moneyball-specific panel and scroll hierarchy, and preserves raw metric values plus percentile semantics with no potential scores.

**Size assessment:** Medium; within the soft target.

**Out of scope:**

- Moneyball calculation, data, or filter changes.

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- `src/features/moneyball/components/moneyball-profile-panel.tsx` and `moneyball-role-fit-panel.tsx` — hierarchy, spacing, and current/potential language alignment; reuse rail and navigation from commits 5 and 7.
- `src/app/routes/players.$uid.test.tsx` — cross-section coherence tests (rail plus navigation plus panel order identical across sections).

**Behavior and data flow:**

- Same Moneyball profile data and comparison basis; presentation alignment only.

**Ordered implementation steps:**

1. Add RED tests: Moneyball section shares the rail, navigation order, and Moneyball panel hierarchy with matching spacing and scroll ownership, while raw metric values and percentile semantics render unchanged.
2. Align the Moneyball presentation GREEN without touching calculations.
3. Run focused, affected, and gate validation in order.

**Tests and proof:**

- RED proof: coherence tests fail while the Moneyball panels lack the workspace panel hierarchy and scroll ownership (the rail and navigation already persist from commits 5 and 7 and are not re-proved here).
- GREEN assertions: identical rail and navigation; matching Moneyball panel hierarchy and visual language; raw metric values and percentile semantics fully preserved with no potential scores; Moneyball details fully preserved.
- All existing Moneyball tests stay green.

**Patterns to verify:**

- Overview, Attributes, and Role Fit section patterns from commits 12–14 and 18 as the alignment target.

**Constraints and non-goals:**

- Do not change Moneyball calculations or remove exposed Moneyball information.
- Do not merge standard and Moneyball scoring visuals.
- Do not invent potential Moneyball scores; raw metric values and percentile semantics stay exactly as calculated.

**Dependencies and sequencing:**

- Depends on commit 18.
- Required before commit 21 checks Moneyball at ultrawide.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx src/features/moneyball/components/moneyball-profile-panel.test.tsx` then `./scripts/dev check`.

**Stop conditions:** Stop if alignment would remove Moneyball-specific context (for example comparison-basis copy); preserve it instead.

**Review mandate:** Verify one coherent workspace across all four sections; verify all Moneyball details preserved; verify raw metric values and percentile semantics unchanged with no potential scores; verify scoring paths stay distinct; verify typography and spacing match the workspace system.

#### Commit 20 — Prove minimum desktop containment

**Status:** Pending

**Provisional commit:** `feat(profile): contain the workspace at minimum desktop size`

**Work:** Prove the redesigned workspace stays usable at 1280×800: all sections reachable, identity accessible, panels reflowing vertically where needed, with no unreadable compression of attributes, role names, or comparison values.

**Size assessment:** Small; geometry and overflow rules only; within the soft target.

**Out of scope:**

- Content changes; ultrawide behavior (commit 21 owns it).

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- Route and section styles from commits 5, 7, 13, and 17 — minimum-width rules, reflow breakpoints, rail-to-stacked behavior, and overflow containment at 1280×800.
- `src/app/routes/players.$uid.test.tsx` — containment assertions where jsdom can express them (no horizontal document overflow, rail present, sections reachable).
- `e2e/smoke.spec.ts` — Playwright browser assertions covering all four final sections (Overview, Attributes, Role Fit, Moneyball) at exactly 1280×800.

**Behavior and data flow:**

- No data or navigation change; CSS containment and reflow only.

**Ordered implementation steps:**

1. Capture 1280×800 evidence per section and record failures.
2. Add containment and reflow rules GREEN.
3. Add runnable containment assertions where the harness supports them.
4. Re-capture evidence and run focused, affected, gate, and smoke validation in order.

**Tests and proof:**

- Behavior-preserving proof: full existing route suite stays green.
- Geometry proof: exact `./scripts/dev smoke` with Playwright browser assertions per section (Overview, Attributes, Role Fit, Moneyball) at exactly 1280×800 showing reachable sections, accessible identity, and readable metrics; manual or native-WebView inspection is an additional recorded gap, never a substitute.

**Patterns to verify:**

- DESIGN.md minimum window (1280×800) and existing `lg:` reflow patterns in the route.

**Constraints and non-goals:**

- Do not compress attributes, role names, or comparison values until difficult to read.
- Do not hide sections or identity at minimum size.

**Dependencies and sequencing:**

- Depends on all content commits (5–19).
- Independent of commit 21 except for shared style touchpoints; coordinate to avoid conflicting rules.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check` then exact `./scripts/dev smoke`.

**Stop conditions:** Stop if any section cannot be contained at 1280×800 without content removal (report rather than silently dropping content); stop if smoke cannot run (record the gap and stop; manual evidence alone never passes this packet).

**Review mandate:** Verify per-section 1280×800 Playwright evidence for all four sections; verify all sections reachable; verify identity accessible; verify readability of attributes, role names, and values; verify no content removed; verify `./scripts/dev smoke` ran exact; verify any manual or native-WebView gap is recorded, not passed.

#### Commit 21 — Bound ultrawide geometry

**Status:** Pending

**Provisional commit:** `feat(profile): bound the ultrawide workspace`

**Work:** Bound the workspace at 3440×1440: bounded identity rail, majority width to the analysis workspace, no indefinitely stretched cards or controls, and extra width used to expose more analysis simultaneously (Attributes breadth, Role Fit navigator plus comparison list without excessive empty space).

**Size assessment:** Small; geometry rules only; within the soft target.

**Out of scope:**

- Content changes; minimum-size behavior (commit 20 owns it).

**Implementation packet:** The handoff below gives the worker the complete execution detail for this commit.

**Files and responsibilities:**

- Route and section styles from commits 5, 7, 13, 17, and 19 — max-width bounds on the rail and cards, workspace growth rules, and multi-column analysis exposure at ultrawide.
- `e2e/smoke.spec.ts` — Playwright browser assertions covering all four final sections (Overview, Attributes, Role Fit, Moneyball) at exactly 3440×1440.

**Behavior and data flow:**

- No data or navigation change; CSS bounds only.

**Ordered implementation steps:**

1. Capture 3440×1440 Playwright evidence per section and record stretch or empty-space failures.
2. Add rail and card bounds plus analysis growth rules GREEN.
3. Re-capture evidence and run focused, affected, gate, and smoke validation in order.

**Tests and proof:**

- Behavior-preserving proof: full existing route suite stays green.
- Geometry proof: exact `./scripts/dev smoke` with Playwright browser assertions per section (Overview, Attributes, Role Fit, Moneyball) at exactly 3440×1440 showing a bounded rail, unstretched controls, and effective Attributes and Role Fit use of width; manual or native-WebView inspection is an additional recorded gap, never a substitute.

**Patterns to verify:**

- DESIGN.md content rules (no `content-max-width` clamp on tables that need width; bounded chrome instead) and shared analysis table width conventions.

**Constraints and non-goals:**

- Do not stretch individual cards or controls indefinitely.
- Do not bound tables that legitimately need the width; bound the rail and chrome.

**Dependencies and sequencing:**

- Depends on all content commits (5–19); coordinate with commit 20 on shared rules.

**Validation:** `./scripts/dev test src/app/routes/players.$uid.test.tsx` then `./scripts/dev check` then exact `./scripts/dev smoke`.

**Stop conditions:** Stop if bounding breaks minimum-size containment from commit 20 (resolve the conflict explicitly); stop if smoke cannot run (record the gap and stop; manual evidence alone never passes this packet).

**Review mandate:** Verify per-section 3440×1440 Playwright evidence for all four sections; verify bounded rail and unstretched controls; verify Attributes breadth and Role Fit side-by-side without excessive empty space; verify commit-20 containment still holds; verify `./scripts/dev smoke` ran exact; verify any manual or native-WebView gap is recorded, not passed.

## Active work

**PR:** PR 1 — Player profile workspace redesign

**Commit:** Group modification actions

### RED or removal proof

New route tests must fail while Boost CA and Wonderkid Mentality are visible without opening a Modify Player disclosure. They must prove keyboard reachability and that the hidden-information control remains outside.

### Expected outcome

A feature-local keyboard-operable Modify Player disclosure contains both unchanged boost flows and exposes clear expanded state, while the hidden-information display control remains structurally separate.

### Explicit exclusions

Shared disclosure primitives, boost eligibility or mutation changes, confirmation-copy changes, hidden-control movement, and section-content redesign.

## Discoveries and replanning

The independent plan review accepted the 21-commit order with no findings. The developer accepted the reviewed plan and Delivery fingerprint before delivery began.

The planning-artifact checkpoint review found stale prose that still described review acceptance as unknown. This commit corrects that record without changing scope, packet order, or delivery authority.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Player profile workspace redesign | Commit 1 — Record the approved feature plan | 74e6797d2ea9a7592464881f8a05f3ef7d4c7266 | Recorded the reviewed schema 2 ledger, JAY-62 TODO activation, and approved directional mockup on the authorized feature branch. | `ledger_state.py` and `delivery_state.py` reported runnable; `git diff --check` and `git diff --cached --check` passed; the mockup SHA-256 matched the approved artifact; the pre-commit gate passed. | Not applicable | Clear | 1 | Checkpoint review corrected stale review-status prose; a fresh plan review and developer acceptance authorized replacement fingerprint `c1c4f5ba874753a4a5ffa0c28b59be58601aba5ec11740ec8ef4d1701d8eea0a`. |
| PR 1 — Player profile workspace redesign | Commit 2 — Remove duplicate profile queries | 7114fe414d2af575ed010d7240a1d8dcddae17d7 | Centralized the snapshot and player Query subscriptions in `PlayerProfileContent` and passed non-null resolved data into the existing General composition without changing rendering or prefetch behavior. | Focused Player Profile route tests passed 54/54; `./scripts/dev check` passed with 805 Rust tests and 2 ignored; TypeScript diagnostics and diff checks were clean. | Pass | Clear | 0 | None |
| PR 1 — Player profile workspace redesign | Commit 3 — Centralize profile mutations | 2ceda2b304269e71ddf27716900337cc5d350d7d | Hoisted hidden-information and boost mutation ownership into `PlayerProfileContent`, passed effective state and callbacks into the presenter, and restored the prior feedback lifetime when leaving General. | The feedback round-trip proof failed before correction and passed after it; focused Player Profile route tests passed 55/55; `./scripts/dev check` passed with 805 Rust tests and 2 ignored; TypeScript diagnostics and diff checks were clean. | Pass | Clear | 1 | Initial review found route-owned mutation feedback resurfaced after a General → Moneyball → General round trip; correction resets both observers when General is left and adds direct regression proof. |
| PR 1 — Player profile workspace redesign | Commit 4 — Extract existing analysis navigation | b915ee165ee384691d64c23c700b0b94a8c0f7e8 | Moved the existing General/Moneyball parser and keyboard-operable tablist with delayed focus restoration into the Player Profile feature module while leaving route URL wiring and rendering behavior unchanged. | Focused Player Profile route tests passed 55/55 unchanged; `./scripts/dev check` passed with 805 Rust tests and 2 ignored; TypeScript diagnostics and diff checks were clean. | Pass | Clear | 0 | None |
| PR 1 — Player profile workspace redesign | Commit 5 — Introduce four-section vertical slice with legacy URL compatibility | f86a49471aba41d04cab749ef70af5a0a182feaa | Added canonical four-section resolution and keyboard navigation, complete legacy/default URL mapping, exact Moneyball loader prefetch, section-specific reuse of existing panels, current-player retention, and replace-history behavior. | Resolver and route RED proofs failed before implementation; focused Player Profile and utility tests passed 96/96; `./scripts/dev check` passed with 805 Rust tests and 2 ignored; deterministic TypeScript and diff checks were clean. | Pass | Clear | 1 | Initial review required positive loader-prefetch and user-triggered replace-history proofs; correction added both. One advisory MEDIUM to update the stale Player Profile URL description in `.wiki/ARCHITECTURE.md` is deferred to feature close-out. |
| PR 1 — Player profile workspace redesign | Commit 6 — Extract identity presentation | 6982817b87c1aa6d42621d3bcf6740de3a9467d2 | Extracted name, club/division, flags, age/DOB, nationality, height, preferred foot, and market-value presentation into feature-local identity components while preserving their original General and Moneyball visibility and layout. | The market-value placement proof failed before correction and passed after it; focused Player Profile route tests passed 64/64; `./scripts/dev check` passed with 805 Rust tests and 2 ignored; TypeScript diagnostics and diff checks were clean. | Pass | Clear | 1 | Initial review found market value moved into Moneyball and changed General geometry; correction restored the four-fact identity grid and original General-only value position with direct regression proof. |
| PR 1 — Player profile workspace redesign | Commit 7 — Install persistent identity rail | e47173f48cf182cfb05363dbecabdd6c35114ba6 | Added one bounded all-section identity rail with market value and neutral text placeholders, distinct analysis landmark ownership, and removal of duplicate descriptive identity and heading copies from the analysis workspace. | Rail RED proofs failed before implementation and the duplicate-heading proof failed before correction; focused Player Profile route tests passed 68/68; `./scripts/dev check` passed with 805 Rust tests and 2 ignored; TypeScript diagnostics and diff checks were clean. | Pass | Clear | 1 | Initial review found descriptive identity remained duplicated in every analysis header; correction made the rail the sole identity owner while preserving the explicit market-value exception and existing analysis behavior. |
| PR 1 — Player profile workspace redesign | Commit 8 — Separate hidden-information display control | Pending record | Moved the unchanged hidden-information toggle and error feedback into a dedicated display-control slot separate from the existing boost action slot. | The structural separation proof failed before implementation; focused Player Profile route tests passed 68/68; `./scripts/dev check` passed with 805 Rust tests and 2 ignored; TypeScript diagnostics and diff checks were clean. | Pass | Clear | 0 | None |

## Final validation

- `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/player-profile-workspace-redesign.md` passes after plan review.
- `git diff --check` passes for the touched planning paths; checkpoint review of the staged planning diff additionally requires `git diff --cached --check`.
- Per implementation commit: focused `./scripts/dev test` targets listed in the packet, then `./scripts/dev check`.
- Commits 20–21 additionally require `./scripts/dev smoke` with Playwright browser assertions for all four sections at the exact viewport; manual or native-WebView inspection is a recorded additional gap, never a substitute.
- Feature completion additionally requires the full feature gate, bounded feature review, and documentation reconciliation owned by feature close-out (Not run in this PR per the delivery plan).

## Documentation impact

Recorded during reconciliation by feature close-out, not in implementation commits.

Expected impact: `.wiki/ARCHITECTURE.md` and `.wiki/DESIGN.md` only if implementation changes a documented contract or convention; no ADR is warranted (no durable consequential choice with meaningful alternatives beyond the recorded JAY-62 direction); no debug report is expected.

Reconciliation is explicitly not an implementation packet in this ledger; feature close-out owns final docs.
