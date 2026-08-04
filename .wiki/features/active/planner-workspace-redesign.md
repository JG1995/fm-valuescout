# Planner Workspace Redesign

## Status

Active

## Intent

Make Squad Planner a focused desktop workspace instead of one long page of equally weighted setup, tactic, and squad panels. The redesign puts the selected squad and its Optimize action first for configured saves while keeping tactic and club-family editing close at hand.

## User-visible behavior

- `/planner` has three URL-backed workspaces: **Squad**, **Tactic**, and **Club setup**.
- A configured save opens the Squad workspace when the URL does not select a workspace. A loaded save without a primary club opens Club setup. A valid explicit workspace in the URL wins over either default.
- The page header identifies the configured primary club when one exists. Only the active workspace is visible and exposed to assistive technology.
- Arrow keys, Home, and End move between workspace tabs. Tab changes replace the current URL search state so the selected workspace survives reload without adding noisy browser-history entries.
- Switching workspaces preserves unsaved club-family and tactic drafts, selected tactic lane, and selected Planner team. Switching the active app save keeps the existing reset and refresh behavior.
- The Tactic workspace shows the IP, OOP, or Both pitch view beside one selected-lane inspector. The inspector contains the lane's score weight, importance rank, preferred-foot rule, and the visible phase position and role controls.
- In Both view, IP and OOP pitches remain side by side at the supported desktop widths. The editor has one clear primary action: **Save tactic**.
- The Squad workspace puts Senior, Reserves, and Youth selection, **Optimize squads**, and selected-team **Clear Squad** in one compact toolbar above the matrix.
- Squad matrix rows use a compact two-line lane summary and align each player name with the combined score. The matrix owns overflow when its strings or rows exceed the available workspace instead of pushing unrelated workspaces down the page.
- Successful squad actions use one compact latest-status region. Errors remain visible near the affected control and retain their existing accessible alert behavior.

## Invariants

- Rust continues to own Planner persistence, validation, candidate scope, score calculation, optimizer allocation, mutation semantics, and returned DTOs. This feature changes React presentation and route state only.
- IPC command names, query keys, cache reconciliation, SQLite schema, migrations, tactic payloads, and Planner DTOs do not change.
- Senior, Reserves, and Youth remain the fixed teams. Strings remain ordered and unlimited, and each team keeps at least one string.
- One save-scoped tactic with eleven stable linked lanes remains shared by all teams.
- Manual and optimized assignment provenance, save-wide player uniqueness, selected-team clearing, and manual-assignment precedence remain unchanged.
- Missing phase scores render as `—`. Outside-pool and unresolved assignments remain occupied and visibly warned.
- Tactic and club-family drafts survive workspace changes. Failed saves retain the draft. An active-save change cannot carry a draft into the next save.
- All current keyboard paths, focus restoration, labelled controls, tab semantics, modal behavior, and mutation feedback remain available.
- The global top bar remains the owner of active-save and snapshot-freshness context. The Planner does not duplicate Load Data.
- The existing dark visual system, token-backed Tailwind utilities, IBM Plex typography, and Lucide icons remain unchanged.

## Non-goals

- No optimizer algorithm, eligibility rule, score calculation, preferred-foot behavior, ranking behavior, or gap recommendation changes.
- No tactic library, formation naming, custom string names, string reordering, drag-and-drop interaction, or new Planner data.
- No backend, IPC, persistence, migration, dependency, design-token, app-shell, or global-state change.
- No mobile or narrow-window design. The product remains desktop-only at a 1280×800 minimum and a 1600×900 design viewport.
- No generic shared tabs or workspace framework. Planner follows the existing profile-tab pattern within its feature boundary.
- No accordion fallback or permanent three-pane cockpit.

## Current-state map

- Relevant components: `src/app/routes/planner.tsx` loads all Planner queries and vertically renders `PlannerClubFamilyPanel`, `PlannerTacticEditor`, then `PlannerDepthMatrix`; `src/app/components/app-shell-layout.tsx` gives the main region page-level vertical scrolling.
- Tactic presentation: `src/features/planner/components/planner-tactic-editor.tsx` owns the draft, phase view, selected lane, global lane controls, validation, and save mutation. `planner-tactic-pitch.tsx` renders each pitch and duplicates a phase-specific lane-control card beside it.
- Squad presentation: `planner-depth-matrix.tsx` owns team selection, mutations, picker and menu state, and separate success and error messages. `planner-depth-table.tsx` renders sticky lane labels, horizontal overflow, four-line lane headers, and vertically stacked assignment content.
- Club-family presentation: `planner-club-family-panel.tsx` owns a local draft and invalidates the Planner query tree after save.
- Existing analogue: `/players/$uid` validates a `tab` search parameter, replaces URL search state on tab changes, and uses an accessible roving-tabindex `PlayerProfileTabs` component with hidden tab panels.
- Data model: save-scoped club settings, tactic lanes, strings, and assignments remain unchanged.
- Persistence and migrations: SQLite migrations v4 through v10 and all Planner Rust services are outside this feature.
- Existing behavioral assumptions: the tactic editor is keyed by active save to prevent cross-save draft leakage; Planner workspace components already own their transient interaction state; query invalidation after saves and mutations is established.
- Tests: `src/app/routes/planner.test.tsx` covers Planner route behavior and interactions; `e2e/smoke.spec.ts` covers no-snapshot, first-use setup, tactic save, string management, and optimizer paths through stubbed IPC.
- Project validation commands: `./scripts/dev test [target...]`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` is unsupported and is not acceptance evidence.
- Primary risks: losing local drafts when workspaces hide, creating nested-scroll traps, compressing warnings or long role names beyond legibility, and preserving accessible focus and announcements while controls move.

## Feature architecture

The existing `/planner` route remains the only Planner page and continues to load the current snapshot plus all Planner queries. It owns a validated `view` search parameter, effective first-use default selection, the page header, and three labelled tab panels. Planner-local workspace navigation follows the established player-profile tab behavior and does not introduce a shared abstraction.

All three workspace components stay mounted while their tab panels use the native `hidden` attribute. This keeps current component-owned draft and selection state without a new store or lifted state. The existing active-save key continues to reset the tactic editor at the save boundary.

`PlannerTacticEditor` remains the draft and mutation owner. Pitch rendering becomes presentation-only, while one Planner-local selected-lane inspector renders the global lane settings and the controls for the phase or phases visible in the selected tactic view. No tactic state or validation moves across the frontend/backend boundary.

`PlannerDepthMatrix` remains the squad interaction coordinator. It composes one compact team/action/status toolbar and gives `PlannerDepthTable` the available workspace. The table keeps semantic table markup, the sticky lane column, string header menus, cell accessible names, and focus behavior while reducing row content to a consistent compact layout.

The implementation uses existing React, TanStack Router, Tailwind, Panel, Button, field, Modal, and ScoreBadge patterns. It adds no dependency and does not require an ADR because it applies established route-state and feature-boundary decisions.

## Uncertainty register

### Known

- The populated Planner screenshot supplied on 2026-08-04 shows club setup, the full dual-pitch tactic editor, and the squad matrix stacked in one long document. The squad output begins below both configuration workspaces.
- The app targets a 1280×800 minimum and a 1600×900 design viewport.
- PR [#30](https://github.com/JG1995/fm-valuescout/pull/30) merged Planner optimizer preferences into `main` as `1c4ec088246d6563e1ff05636af8928b4f5a290f` on 2026-08-04.
- The repository has no active feature ledger and no planned spec for this redesign.
- The removed UI-agent command is unavailable. Browser smoke uses stubbed IPC and does not prove native WebView or live SQLite behavior.

### Assumptions

- Squad is the frequent-use workspace after initial configuration. Club setup is low-frequency maintenance, and tactic editing is an occasional supporting workflow.
- A URL search parameter is the correct owner for the visible workspace because the state is reload-relevant and the profile route already establishes the pattern.
- Keeping hidden workspaces mounted is acceptable for three already-loaded Planner components and is simpler than a new client store or draft-lifting boundary.
- Existing design tokens and primitives are sufficient. The problem is hierarchy and composition, not branding.

### Decisions

- Deliver the redesign in one PR with three atomic commits. The commits share one route, one test surface, and one visual acceptance pass, so a second PR would add publication overhead without an independent merge boundary.
- Use `squad`, `tactic`, and `clubs` as the URL values and **Squad**, **Tactic**, and **Club setup** as the visible labels.
- Use `replace: true` for workspace changes, matching the player-profile tab route.
- Keep inactive workspaces mounted inside hidden tab panels to preserve transient state.
- Keep the tactic inspector visible beside the pitch area. Both view shows two pitch boards and one inspector, not one inspector per pitch.
- Let the squad matrix own horizontal overflow and bounded vertical overflow. Do not constrain the entire route with an arbitrary pixel height.
- Update the relevant current-state `DESIGN.md` and `ARCHITECTURE.md` text only in the implementation or reconciliation step that makes each statement true.
- Do not create an ADR. The feature uses accepted React, Router, Tailwind, and Planner boundaries.
- For this experiment, use Luna Max for every implementation profile in this ledger. Keep Sol High for every review profile, including the final feature-complete review. This overrides the repository defaults for this ledger only.

### Unknowns

- The final compact row height and inspector width need populated visual inspection at both target viewports. These measurements do not block the workspace-navigation commit.
- Native Tauri viewport inspection remains manual unless a supported repository command is added by separate, explicitly approved tooling work.

### Risks

- Conditional rendering could unmount an editor and discard a draft. The tab-panel contract and tests must prove preservation.
- Hiding all but one workspace changes existing tests and smoke paths that assume all controls are visible at once.
- A bounded matrix can create nested scrolling or obscure focused rows if sticky headers and focus scrolling are not verified together.
- Compact lane and assignment rows can hide long role or player names. Truncation must retain `title` and accessible names.
- Moving success and error regions can create duplicate or missed screen-reader announcements.
- Two pitches plus one inspector can overflow when the navigation rail is expanded at 1280×800.

## Walking skeleton

Commit 1 replaces vertical workspace stacking with URL-backed, keyboard-operable Squad, Tactic, and Club setup tabs while preserving the existing three workspace components and their behavior. This proves the information architecture before the two dense workspaces are recomposed.

## Delivery plan

### PR 1 — Redesign Squad Planner workspace

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(planner): redesign squad planner workspace`

**Purpose:** Deliver the complete Planner information-architecture and density redesign in one review surface without changing Planner domain behavior.

**Depends on:** Planner optimizer preferences PR #30, merged as `1c4ec088246d6563e1ff05636af8928b4f5a290f`. Start implementation from a refreshed `main` on a new short-lived branch.

#### Commit 1 — Add Planner workspace navigation

**Status:** Completed

**Provisional commit:** `feat(planner): add workspace navigation`

**Work:** Add validated URL-backed workspace state, accessible Planner workspace tabs, configured and first-use defaults, primary-club header context, and hidden mounted tab panels. Update route tests, smoke paths, and the implemented workspace contract in `DESIGN.md`.

**Out of scope:**

- Tactic pitch or inspector recomposition.
- Squad toolbar, row-density, overflow, or feedback changes.
- Any Planner data, mutation, cache, or backend change.

**Implementation packet:**

- Owners and files: `src/app/routes/planner.tsx`; a Planner-local workspace tab component or small view utility under `src/features/planner/`; `src/app/routes/planner.test.tsx`; `e2e/smoke.spec.ts`; `e2e/tauri-ipc-stub.ts` only if configured-save coverage needs it; the Squad Planner section in `.wiki/DESIGN.md`; this ledger for progress evidence.
- Existing patterns to verify: `src/app/routes/players.$uid.tsx`, `src/features/player-profile/components/player-profile-tabs.tsx`, and their tests for validated search state, `replace: true`, roving tabindex, arrow/Home/End keys, and hidden panels.
- Constraints and invariants: one active visible workspace; valid explicit URL wins; configured default is Squad; unconfigured default is Club setup; invalid or missing search values normalize safely; components stay mounted; active-save reset, no-snapshot EmptyState, loaders, Suspense, and query invalidation remain unchanged.
- Dependencies and ordering: refresh `main` through merged PR #30 before starting; this commit establishes the composition seam required by commits 2 and 3.

**Implementation profile:** Luna Max — the route and analogue are established, but default selection, hidden mounted state, active-save behavior, and accessible keyboard navigation require local integration judgment.

**Review profile:** Sol High — review must cover URL state, lifecycle preservation, accessibility, and existing Planner paths that broad route tests currently exercise together.

**Validation:**

- RED then GREEN: `./scripts/dev test src/app/routes/planner.test.tsx`
- Browser control paths: `./scripts/dev smoke`
- Commit gate: `./scripts/dev check`
- Manual browser evidence at 1280×800 and 1600×900: only one workspace is visible; tabs and primary action remain visible with collapsed and expanded navigation rail; the page has no horizontal overflow.

**Stop conditions:** Stop and replan if preserving drafts requires a new global store or moving mutation ownership; if route search state conflicts with active-save switching; if the workspace boundary requires backend or query-contract changes; or if implementation cannot start from refreshed `main` after PR #30.

**Review mandate:**

- Verify valid, invalid, absent, configured, and first-use workspace selection paths.
- Verify URL replacement and direct navigation without browser-history spam.
- Verify roving tabindex, arrow/Home/End behavior, labelled tab panels, and focus visibility.
- Verify inactive workspaces are hidden from assistive technology but remain mounted so drafts and selections persist.
- Verify active-save changes still reset tactic state and no-snapshot behavior stays unchanged.
- Reject a new shared tab framework, client store, dependency, backend change, or unrelated layout cleanup.

#### Commit 2 — Unify tactic lane editing

**Status:** Completed

**Provisional commit:** `feat(planner): unify tactic lane editing`

**Work:** Recompose the Tactic workspace into a pitch area plus one selected-lane inspector that owns global lane settings and the visible phase controls. Keep IP, OOP, and Both behavior, validation, save payloads, and failed-draft retention unchanged. Update focused tests and the implemented tactic-layout contract in `DESIGN.md`.

**Out of scope:**

- Squad depth toolbar, table, status, or overflow changes.
- Club-family form changes beyond its workspace placement from commit 1.
- New tactic behavior, formations, drag-and-drop, or persistence.

**Implementation packet:**

- Owners and files: `src/features/planner/components/planner-tactic-editor.tsx`, `planner-tactic-pitch.tsx`, one Planner-local inspector component if it keeps the editor readable, `src/app/routes/planner.test.tsx`, `e2e/smoke.spec.ts`, the tactic bullets in `.wiki/DESIGN.md`, and this ledger for progress evidence.
- Existing patterns to verify: current selected-lane draft helpers in `planner-tactic-editor.tsx`; `SelectField`, native range behavior, Panel actions, phase-view controls, and the existing lane accessible names.
- Constraints and invariants: exactly one selected lane; one instance of score weight, importance rank, and foot rule; phase-compatible position and role options; Both shows two pitches plus IP and OOP controls; single-phase views show one pitch plus that phase's controls; save and refresh behavior remains byte-for-byte compatible at the API boundary.
- Dependencies and ordering: commit 1 provides the dedicated Tactic workspace and visible area. Do not change the route workspace contract here.

**Implementation profile:** Luna Max — the domain contract is fixed, but component ownership, responsive composition, and dense control hierarchy require design judgment.

**Review profile:** Sol High — deterministic tests cover behavior, while responsive fit, accessible labelling, and draft preservation need careful review and visual evidence.

**Validation:**

- RED then GREEN: `./scripts/dev test src/app/routes/planner.test.tsx`
- Browser control paths: `./scripts/dev smoke`
- Commit gate: `./scripts/dev check`
- Manual browser evidence at 1280×800 and 1600×900: Both shows two usable pitches and one inspector without horizontal page overflow; IP and OOP views keep all controls and Save tactic visible; keyboard focus follows a logical order.

**Stop conditions:** Stop and replan if the inspector requires lifting tactic drafts out of `PlannerTacticEditor`; if both pitches cannot remain usable at 1280×800 without changing the supported viewport; if labels or validation would need a public tactic contract change; or if one inspector cannot expose both phases without duplicate form-control IDs or ambiguous accessible names.

**Review mandate:**

- Verify every existing tactic field changes the same lane property and produces the same save payload.
- Verify one global lane-control set and one visible control set per phase, with unique IDs and labels.
- Verify selected-lane emphasis and accessible names stay synchronized across both pitches.
- Verify validation, disabled Save tactic states, pending feedback, failed-save draft retention, successful cache invalidation, and active-save reset.
- Verify Both, IP, and OOP layouts at both supported viewports and with the expanded navigation rail.
- Reject new tactic behavior, duplicated derived state, raw design values, and unrelated component abstraction.

#### Commit 3 — Compact the squad depth workspace

**Status:** Active

**Provisional commit:** `feat(planner): compact squad depth workspace`

**Work:** Place team selection and squad actions in one toolbar, compact lane and assignment rows, give the matrix bounded workspace overflow with sticky context, and consolidate successful squad-action feedback. Preserve all matrix, picker, string, Optimize, Clear Squad, warning, and focus behavior. Update route tests, populated smoke coverage where useful, and the implemented squad-layout contract in `DESIGN.md`.

**Out of scope:**

- Optimizer allocation, assignment persistence, candidate sorting, or score changes.
- New gap highlighting, recommendation data, filters, or matrix virtualization.
- Changes to Tactic or Club setup workspaces.

**Implementation packet:**

- Owners and files: `src/features/planner/components/planner-depth-matrix.tsx`, `planner-depth-table.tsx`, existing Planner-local controls only where toolbar composition requires it, `src/app/routes/planner.test.tsx`, `e2e/smoke.spec.ts`, `e2e/tauri-ipc-stub.ts` if representative populated coverage is added, the squad bullets in `.wiki/DESIGN.md`, and this ledger for progress evidence.
- Existing patterns to verify: current team roving tabs, Panel actions, sticky first column, string header menus, assignment-cell accessible names, ScoreBadge, Modal focus restoration, and app-shell overflow ownership.
- Constraints and invariants: Optimize remains the primary action and still applies to all squads; Clear Squad remains selected-team and destructive; unlimited strings keep horizontal scrolling; all eleven lanes remain in stable order; long names truncate visually but remain available through `title` and accessible names; warning and unknown-score states stay truthful; focused cells and headers remain visible when the matrix scrolls.
- Dependencies and ordering: commit 1 supplies the isolated Squad workspace. Commit 2 is behaviorally independent but lands first so final visual validation covers the finished Planner composition once.

**Implementation profile:** Luna Max — the mutation behavior is established, but table density, sticky overflow, status announcements, and many existing interaction paths make this a broad UI integration change.

**Review profile:** Sol High — review must combine deterministic mutation coverage with accessibility, focus, overflow, and populated-state visual checks that jsdom cannot prove.

**Validation:**

- RED then GREEN: `./scripts/dev test src/app/routes/planner.test.tsx`
- Browser control paths: `./scripts/dev smoke`
- Commit gate: `./scripts/dev check`
- Manual populated-state evidence at 1280×800 and 1600×900: team tabs and Optimize are visible on entry; 1600×900 shows the complete eleven-lane first-string matrix without document scrolling; 1280×800 keeps a useful matrix area and visible internal scroll cues; horizontal string overflow, sticky lane/header context, long names, warnings, and keyboard focus remain usable.

**Stop conditions:** Stop and replan if compact rows must omit lane roles, player names, scores, or warning states; if WebView sticky behavior fails inside the bounded overflow region; if focus can move behind clipped content without scrolling into view; if status consolidation changes mutation semantics; or if the layout needs a new global app-shell contract.

**Review mandate:**

- Verify Optimize, selected-team Clear Squad, team tabs, string menus, assignment picker, move and clear confirmations, and cache reconciliation are behaviorally unchanged.
- Verify lane order, IP/OOP role context, player names, scores, unresolved and outside-pool warnings, and unknown scores remain visible and accessible.
- Verify row density is consistent and long content truncates without losing `title` or accessible context.
- Verify vertical and horizontal overflow, sticky headers and lane labels, and focus scrolling at 1280×800 and 1600×900.
- Verify one latest success announcement does not suppress errors or create duplicate live-region output.
- Reject optimizer, persistence, backend, new-data, virtualization, and unrelated styling changes.

## Active work

**PR:** PR 1 — Redesign Squad Planner workspace

**Commit:** Commit 3 — Compact the squad depth workspace

### RED proof

Add focused tests that render the configured Squad workspace and expect one compact team/action toolbar, bounded matrix overflow, and consolidated success feedback without changing assignment or optimizer results. The current depth matrix distributes team actions and feedback around a dense unbounded table.

### Expected outcome

The Squad workspace keeps team selection and primary actions together above a compact, bounded matrix. Existing Optimize, Clear Squad, picker, string, assignment, warning, and focus behavior remain intact while the latest successful action is announced once.

### Explicit exclusions

- Do not change optimizer allocation, assignment persistence, candidate sorting, or score calculation.
- Do not add gap highlighting, recommendation data, filters, virtualization, or new Planner data.
- Do not change the Tactic or Club setup workspaces, route workspace contract, Rust, IPC, queries, cache behavior, persistence, or dependencies.

## Discoveries and replanning

- Planning confirmed that PR #30 is merged, but the local `origin/main` ref is stale. Refresh `main` before creating the redesign branch; do not stack this feature on the merged feature branch.
- A prior uncommitted workspace-tab attempt used `clubs`, `tactic`, and `squad` and passed focused route tests, but it was not accepted because only an empty save was available. The populated screenshot supplied for this plan now confirms the vertical-stacking problem. Treat the old attempt as historical evidence, not source code.
- No current runtime command provides the former native UI-agent inspection. Keep native populated-state verification as explicit manual evidence and do not report browser smoke as native proof.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Add Planner workspace navigation | Pending record | Added validated workspace search state, accessible Planner tabs, configured and first-use defaults, primary-club context, hidden mounted panels, route/smoke coverage, and current-state documentation. | Sol High approved after one fix round; focused route suite 37/37. | Native Tauri viewport evidence remains open because the former UI-agent runtime is unavailable; no scope deviations. |
| PR 1 | Commit 2 — Unify tactic lane editing | Pending record | Replaced per-pitch lane controls with one selected-lane inspector for shared settings and visible IP/OOP phase controls; preserved tactic state, save lifecycle, and route boundaries; updated tests and current-state docs. | Sol High approved; focused route suite 38/38, full suite 155/155, repository gate, and browser smoke 12/12. | Native Tauri viewport evidence remains open because the former UI-agent runtime is unavailable; no scope deviations. |

## Final validation

- `./scripts/dev test src/app/routes/planner.test.tsx`
- `./scripts/dev test`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Native Tauri inspection with a representative configured and populated save at 1280×800 and 1600×900. Capture evidence for Squad, Tactic Both, Tactic single-phase, and Club setup with the navigation rail collapsed and expanded.
- Keyboard-only pass: workspace tabs, phase controls, pitch lanes, inspector fields, team tabs, matrix cells, string menus, picker, confirmations, Save tactic, Optimize, and Clear Squad.
- Confirm workspace changes preserve drafts and selections; active-save changes reset or refresh them at the existing boundaries.
- Confirm no document-level horizontal overflow, no content hidden behind the top bar, visible matrix scroll cues, and visible focus throughout bounded overflow.
- Confirm active save and snapshot age stay visible in the global top bar and all score and warning states remain truthful.
- Fresh Sol High feature-complete review over the exact recorded implementation commits and final PR ref.
- `./scripts/dev mutate` remains unsupported and must not be reported as passed.

## Documentation impact

During implementation and reconciliation:

- Update `.wiki/DESIGN.md` as each new workspace, tactic, and squad layout becomes implemented.
- Update the Planner composition paragraphs in `.wiki/ARCHITECTURE.md` after the implemented route and component ownership change.
- Keep `.wiki/CONCEPT.md` unchanged because product purpose and scope do not change.
- Create no ADR unless implementation crosses an accepted routing, feature, or app-shell boundary.
- On completion, condense this ledger into `.wiki/features/completed/planner-workspace-redesign.md` and move the TODO entry to Completed.
