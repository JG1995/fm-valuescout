# Planner Workspace Redesign

## Status

Active

## Intent

Make Squad Planner a focused desktop workspace instead of one long page of equally weighted setup, tactic, and squad panels. The redesign puts squad decisions first, describes linked tactical positions in football terms instead of internal lane terminology, and uses available desktop width to show the three teams together when the matrix remains readable.

## User-visible behavior

- `/planner` has three URL-backed workspaces: **Squad**, **Tactic**, and **Club setup**.
- A configured save opens the Squad workspace when the URL does not select a workspace. A loaded save without a primary club opens Club setup. A valid explicit workspace in the URL wins over either default.
- The page header identifies the configured primary club when one exists. Only the active workspace is visible and exposed to assistive technology.
- Arrow keys, Home, and End move between workspace tabs. Tab changes replace the current URL search state so the selected workspace survives reload without adding noisy browser-history entries.
- Switching workspaces preserves unsaved club-family and tactic drafts, selected tactic lane, and selected Planner team. Switching the active app save keeps the existing reset and refresh behavior.
- The Tactic workspace shows the IP, OOP, or Both pitch view beside one selected-position inspector. The inspector contains the linked position's score weight, importance rank, preferred-foot rule, and the visible phase position and role controls.
- In Both view, IP and OOP pitches remain side by side at the supported desktop widths. The editor has one clear primary action: **Save tactic**.
- The Squad workspace keeps Senior, Reserves, and Youth presentation controls with **Optimize squads** in one compact toolbar above the matrix.
- Squad matrix rows use a compact two-line position-and-role summary and align each player name with the combined score. The matrix owns overflow when its strings or rows exceed the available workspace instead of pushing unrelated workspaces down the page.
- Successful squad actions use one compact latest-status region. Errors remain visible near the affected control and retain their existing accessible alert behavior.
- User-facing Planner copy does not expose lane IDs, lane numbers, or static labels such as **Left winger**. Tactic pitches, the inspector, the squad matrix, player-assignment flows, confirmations, validation, and accessible names describe the current linked IP/OOP positions and roles, with a spatial qualifier only when it is needed to distinguish positions.
- Focusing, hovering, or selecting a tactical position on either pitch emphasizes its linked counterpart without relying on a shared number.
- When the Squad workspace can preserve readable position and assignment widths, one semantic matrix shows Senior, Reserves, and Youth as grouped columns with their ordered strings beneath them. When the available Planner width or string count cannot preserve those widths, the existing team tabs show one team at a time.
- The matrix keeps position context and player scores close to the data they describe. Team boundaries and row focus remain visible across the combined view.
- **Optimize squads** remains global. **Clear Squad** is placed in the affected team's table header and always names that team before confirmation in both combined and single-team layouts.

## Invariants

- Rust continues to own Planner persistence, validation, candidate scope, score calculation, optimizer allocation, mutation semantics, and returned DTOs. This feature changes React presentation and route state only.
- IPC command names, query keys, cache reconciliation, SQLite schema, migrations, tactic payloads, and Planner DTOs do not change.
- Senior, Reserves, and Youth remain the fixed teams. Strings remain ordered and unlimited, and each team keeps at least one string.
- One save-scoped tactic with eleven stable linked lanes remains shared by all teams.
- Stable lane IDs and lane order remain internal Planner identities. User-facing labels derive from the current tactic draft or returned tactic and do not redefine persistence identity.
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
- No user-defined display labels, persisted position names, manual matrix-layout preference, or new tactic-link identifier.
- No rename of internal `TacticLane`, `laneId`, Rust fields, database columns, or optimizer terminology that does not appear in the UI.

## Current-state map

- Relevant components: `src/app/routes/planner.tsx` loads all Planner queries and owns URL-backed Squad, Tactic, and Club setup workspaces. It keeps `PlannerClubFamilyPanel`, `PlannerTacticEditor`, and `PlannerDepthMatrix` mounted in labelled hidden tab panels; `src/app/components/app-shell-layout.tsx` gives the main region page-level vertical scrolling.
- Tactic presentation: `src/features/planner/components/planner-tactic-editor.tsx` owns the draft, phase view, selected lane ID, linked highlight, validation, and save mutation. `planner-tactic-pitch.tsx` renders each pitch with current position-and-role buttons and linked counterpart emphasis, while `planner-tactic-inspector.tsx` renders one selected-position inspector. `src/features/planner/utils/tactic-editor.ts` derives descriptions and deterministic spatial qualifiers from the current tactic without exposing stable lane IDs.
- Squad presentation: `planner-depth-matrix.tsx` owns selected-team state, container-fit mode, mutations, picker and menu state, and one latest squad-action status. `planner-depth-table.tsx` renders one semantic grouped table when the current strings fit the matrix container and keeps hidden non-selected team panels mounted for the constrained tabbed mode. Both presentations keep sticky position and string headers, bounded two-axis overflow, compact rows, explicit team context, and current IP/OOP position-and-role descriptions. `planner-slot-fit-picker.tsx` receives the current tactic and options so assignment locations and confirmations use the same descriptions.
- Club-family presentation: `planner-club-family-panel.tsx` owns a local draft and invalidates the Planner query tree after save.
- Existing analogue: `/players/$uid` validates a `tab` search parameter, replaces URL search state on tab changes, and uses an accessible roving-tabindex `PlayerProfileTabs` component with hidden tab panels.
- Data model: save-scoped club settings, tactic lanes, strings, and assignments remain unchanged.
- Persistence and migrations: SQLite migrations v4 through v10 and all Planner Rust services are outside this feature.
- Existing behavioral assumptions: the tactic editor is keyed by active save to prevent cross-save draft leakage; Planner workspace components already own their transient interaction state; query invalidation after saves and mutations is established.
- Tests: `src/app/routes/planner.test.tsx` covers Planner route behavior and interactions; `e2e/smoke.spec.ts` covers no-snapshot, first-use setup, tactic save, string management, and optimizer paths through stubbed IPC.
- Project validation commands: `./scripts/dev test [target...]`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` is unsupported and is not acceptance evidence.
- Primary risks: keeping number-free tactical-position names truthful and distinguishable; preserving linked pitch emphasis and accessible names; avoiding focus loss when a string mutation changes responsive matrix mode; keeping complex team/string headers understandable; and preventing nested-scroll or sticky-header failures.

## Feature architecture

The existing `/planner` route remains the only Planner page and continues to load the current snapshot plus all Planner queries. It owns a validated `view` search parameter, effective first-use default selection, the page header, and three labelled tab panels. Planner-local workspace navigation follows the established player-profile tab behavior and does not introduce a shared abstraction.

All three workspace components stay mounted while their tab panels use the native `hidden` attribute. This keeps current component-owned draft and selection state without a new store or lifted state. The existing active-save key continues to reset the tactic editor at the save boundary.

`PlannerTacticEditor` remains the draft and mutation owner. Pitch rendering stays presentation-only, while one Planner-local selected-position inspector renders the shared settings and the controls for the phase or phases visible in the selected tactic view. A Planner-local display helper derives user-facing descriptions from the current IP/OOP positions and roles and adds a deterministic spatial qualifier when positions would otherwise be ambiguous. Stable lane IDs continue to link both pitches, assignments, and save payloads but do not appear in user-facing copy. Focus, hover, and selection use the existing linked identity to emphasize the same tactical position on both pitches.

`PlannerDepthMatrix` remains the squad interaction coordinator. It composes the compact team/action/status area, derives whether all teams fit from the matrix's available width and current string counts, and gives `PlannerDepthTable` one active presentation mode. The table uses one semantic grouped-header model: wide mode shows team column groups with ordered strings beneath them, while constrained mode shows the selected team and its tabs. Both modes keep one sticky position column, string header menus, explicit team-scoped Clear Squad actions, cell accessible names, and focus restoration. The layout does not duplicate interactive tables or add global state.

The implementation uses existing React, TanStack Router, Tailwind, Panel, Button, field, Modal, and ScoreBadge patterns. It adds no dependency and does not require an ADR because it applies established route-state and feature-boundary decisions.

## Uncertainty register

### Known

- The original populated Planner screenshot supplied on 2026-08-04 showed club setup, the full dual-pitch tactic editor, and the squad matrix stacked in one long document. Commits 1 through 3 replaced that composition with workspaces and a compact bounded matrix.
- A second populated screenshot supplied on 2026-08-04 shows the implemented Squad workspace at about 1920px wide. The first column expands across roughly half the matrix, role and score context sit far apart, and the static **Left winger** label conflicts with lane 9's current AMC position.
- The app targets a 1280×800 minimum and a 1600×900 design viewport.
- PR [#30](https://github.com/JG1995/fm-valuescout/pull/30) merged Planner optimizer preferences into `main` as `1c4ec088246d6563e1ff05636af8928b4f5a290f` on 2026-08-04.
- The feature branch is `feat/planner-workspace-redesign`. It is published at `origin/feat/planner-workspace-redesign`, remains unmerged, and has no PR ref recorded.
- The removed UI-agent command is unavailable. Browser smoke uses stubbed IPC and does not prove native WebView or live SQLite behavior.

### Assumptions

- Squad is the frequent-use workspace after initial configuration. Club setup is low-frequency maintenance, and tactic editing is an occasional supporting workflow.
- A URL search parameter is the correct owner for the visible workspace because the state is reload-relevant and the profile route already establishes the pattern.
- Keeping hidden workspaces mounted is acceptable for three already-loaded Planner components and is simpler than a new client store or draft-lifting boundary.
- Existing design tokens and primitives are sufficient. The problem is hierarchy and composition, not branding.
- Current phase positions, roles, and stable linked identity contain enough information to produce truthful user-facing descriptions without a persisted display label.
- The common two-string-per-team case can show all three teams at the supplied wide viewport once the position and assignment columns stop absorbing unused width. Constrained widths and larger string counts should retain the single-team presentation.

### Decisions

- Deliver the redesign in one PR with five atomic commits. The two added commits remain on the unpublished and unmerged feature branch, share the same Planner components, tests, and visual acceptance surface, and do not create an independent merge boundary.
- Use `squad`, `tactic`, and `clubs` as the URL values and **Squad**, **Tactic**, and **Club setup** as the visible labels.
- Use `replace: true` for workspace changes, matching the player-profile tab route.
- Keep inactive workspaces mounted inside hidden tab panels to preserve transient state.
- Keep the tactic inspector visible beside the pitch area. Both view shows two pitch boards and one inspector, not one inspector per pitch.
- Let the squad matrix own horizontal overflow and bounded vertical overflow. Do not constrain the entire route with an arbitrary pixel height.
- Keep stable lane IDs and order as internal persistence and linking contracts. Remove the word **lane**, lane numbers, and static lane-ID labels from normal Planner copy. Use current IP/OOP positions and roles, plus a deterministic spatial qualifier only when needed to distinguish linked positions.
- Use linked focus, hover, and selection emphasis across the IP and OOP pitches instead of visible numeric correspondence.
- Show all teams only when the Planner matrix container can preserve the existing readable minimum widths for the current strings. Otherwise show the selected team through the existing tabs. Do not add a manual layout toggle or persisted layout preference.
- Put each destructive Clear Squad action in the affected team's table header in both display modes. Keep Optimize squads as the one global primary action.
- Update the relevant current-state `DESIGN.md` and `ARCHITECTURE.md` text only in the implementation or reconciliation step that makes each statement true.
- Do not create an ADR. The feature uses accepted React, Router, Tailwind, and Planner boundaries.
- For this experiment, use Luna Max for every implementation profile in this ledger. Keep Sol High for every review profile, including the final feature-complete review. This overrides the repository defaults for this ledger only.

### Unknowns

- The exact fit threshold for combined-team mode needs populated visual inspection with representative one-, two-, and three-string teams at 1280×800, 1600×900, and the supplied wide viewport. This does not block the terminology commit because the threshold must preserve existing token-backed minimum widths rather than introduce a new product contract.
- Native Tauri viewport inspection remains manual unless a supported repository command is added by separate, explicitly approved tooling work.

### Risks

- Two linked positions can share the same phase placement or role. User-facing descriptions must stay distinguishable without falling back to lane numbers or static position labels.
- Removing numeric pitch markers can make the IP/OOP relationship unclear unless focus, hover, selection, and accessible descriptions emphasize the linked counterpart together.
- A string add or removal can cross the combined/single-team fit threshold. The acted-on team and string header must remain visible and receive restored focus after the responsive mode changes.
- Multi-row team and string headers can lose programmatic context if group and column associations are only visual.
- Wider combined tables can recreate the original long eye path unless the sticky position column, assignment widths, score proximity, team separators, and row focus treatment are validated with populated data.

## Walking skeleton

Commit 1 remains the feature walking skeleton: it replaced vertical workspace stacking with URL-backed, keyboard-operable Squad, Tactic, and Club setup tabs while preserving the existing three workspace components and their behavior. For the added scope, commit 4 establishes truthful linked-position language across the existing surfaces before commit 5 changes the matrix's team composition.

## Delivery plan

### PR 1 — Redesign Squad Planner workspace

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(planner): redesign squad planner workspace`

**Purpose:** Deliver the complete Planner information architecture, truthful tactical-position language, and adaptive three-team depth overview in one review surface without changing Planner domain behavior.

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

**Status:** Completed

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

#### Commit 4 — Present linked tactical positions

**Status:** Completed

**Provisional commit:** `feat(planner): present linked tactical positions`

**Work:** Replace user-facing lane names and numbers with current IP/OOP position-and-role descriptions across the tactic pitches, selected-position inspector, squad matrix, player picker, confirmations, validation, and accessible names. Add linked focus, hover, and selection emphasis between both pitches so the relationship remains clear without numeric markers. Update focused tests, smoke coverage, and the implemented terminology contract in `DESIGN.md`.

**Out of scope:**

- Internal `TacticLane`, `laneId`, stable ordering, Rust fields, persistence, DTOs, save payloads, assignments, optimizer logic, or rank semantics.
- Adaptive all-team matrix composition, team action placement, column geometry, or responsive thresholds from commit 5.
- User-defined tactical-position labels, role abbreviations, formations, drag-and-drop, or new tactic data.

**Implementation packet:**

- Owners and files: `src/features/planner/utils/tactic-editor.ts`; `planner-tactic-editor.tsx`; `planner-tactic-pitch.tsx`; `planner-tactic-inspector.tsx`; `planner-depth-table.tsx`; `planner-slot-fit-picker.tsx`; `src/app/routes/planner.test.tsx`; `e2e/smoke.spec.ts`; the Planner terminology and accessibility bullets in `.wiki/DESIGN.md`; the React Planner composition paragraph in `.wiki/ARCHITECTURE.md`; and this ledger for progress evidence.
- Existing patterns to verify: current `phasePosition` and `roleLabel` derivation; shared `selectedLaneId` and `aria-pressed` state across both pitches; stable `TACTIC_LANE_IDS` order; matrix cell accessible names; picker `slotLocation` and move-confirmation copy; truncation plus `title`; and selected-position draft updates before save.
- Constraints and invariants: user-facing descriptions come from the current draft in Tactic and the returned tactic in Squad; they update immediately after phase edits; stable lane IDs remain the only persisted and mutation identity; mirrored or duplicate placements receive deterministic human spatial context without numeric lane fallback; importance rank remains a separate 1–11 value; all visible controls and validation errors use position-focused labels; linked emphasis is not conveyed by color alone.
- Dependencies and ordering: commits 1 through 3 are complete. This semantic foundation lands before commit 5 reuses the same position descriptions as shared row headers in the adaptive matrix.

**Implementation profile:** Luna Max — the data and state seams are established, but one truthful descriptor must stay synchronized across draft editing, pitch linking, matrix locations, confirmations, and accessibility without changing internal contracts.

**Review profile:** Sol High — review must detect semantic ambiguity, stale draft-derived labels, inaccessible duplicate names, broken linked emphasis, and any accidental public or persisted lane-contract change across several Planner surfaces.

**Validation:**

- RED then GREEN: `./scripts/dev test src/app/routes/planner.test.tsx`
- Browser control paths: `./scripts/dev smoke`
- Commit gate: `./scripts/dev check`
- Manual populated evidence at 1280×800 and 1600×900: no normal Planner surface exposes lane numbers, static lane-ID position labels, or lane-focused instructions; editing a position updates pitch, inspector, and validation context immediately; focus, hover, and selection identify the linked counterpart in Both view; duplicate placements remain distinguishable by visible text and accessible name.

**Stop conditions:** Stop and replan if unique number-free descriptions require a persisted label, lane-ID or DTO change, or invented football semantics that conflict with the current phase data; if linked emphasis requires moving tactic draft ownership or adding global state; or if validation cannot name an invalid position without exposing an internal identifier.

**Review mandate:**

- Verify the supplied lane 9 case is described from its current AMC positions and roles and never as **Left winger**.
- Verify all pitch buttons, inspector headings and controls, matrix row headers and cells, picker locations, confirmations, and validation errors avoid user-facing lane terminology and numbers.
- Verify descriptions update from unsaved tactic drafts and reset at the established save boundary.
- Verify mirrored and duplicate phase placements remain distinguishable without relying on color or DOM order alone.
- Verify focus, hover, and selection keep the same linked identity emphasized across IP and OOP while keyboard operation and visible focus remain intact.
- Verify tactic payloads, rank values, mutations, query invalidation, and assignment locations still use unchanged internal lane IDs.
- Reject internal DTO/database renames, new persisted display data, generic naming frameworks, or unrelated copy cleanup.

#### Commit 5 — Show all teams when the matrix fits

**Status:** Completed

**Provisional commit:** `feat(planner): adapt squad matrix to available width`

**Work:** Make the Squad matrix show Senior, Reserves, and Youth as grouped columns in one semantic table when the Planner container can preserve readable position and assignment widths. Retain the selected-team tabs when the current strings do not fit, move Clear Squad into the affected team's group header, constrain scan distances, and preserve all existing string, assignment, mutation, warning, overflow, and focus behavior. Update route tests, wide and constrained smoke paths, and the implemented matrix contract in `DESIGN.md` and `ARCHITECTURE.md`.

**Out of scope:**

- Team, string, assignment, candidate, optimizer, scoring, persistence, IPC, cache, or Rust behavior.
- A manual All teams toggle, saved layout preference, custom breakpoints in the global app shell, table virtualization, column reordering, or string reordering.
- Tactic or Club setup changes beyond consuming the position descriptions completed in commit 4.

**Implementation packet:**

- Owners and files: `src/features/planner/components/planner-depth-matrix.tsx`; `planner-depth-table.tsx`; `planner-clear-team-control.tsx`; one Planner-local fit helper only if needed to keep measurement separate from mutations; `src/app/routes/planner.test.tsx`; `e2e/smoke.spec.ts`; `e2e/tauri-ipc-stub.ts` only for representative multi-team strings and assignments; the Squad matrix bullets in `.wiki/DESIGN.md`; the React Planner composition paragraph in `.wiki/ARCHITECTURE.md`; and this ledger for progress evidence.
- Existing patterns to verify: `PLANNER_TEAMS` order and roving team tabs; complete `PlannerDepth.teams` read model; token-backed string minimum widths; sticky header and first-column layering; string header menus and focus refs; Clear Squad confirmation and trigger restoration; latest action status; assignment-cell accessible names; ScoreBadge; and nav-rail width changes.
- Constraints and invariants: combined mode uses one table with team column groups and ordered string headers; constrained mode exposes only the selected team's group plus the existing tabs; the fit decision uses actual Planner container space and current string counts, not viewport width alone; no duplicate interactive matrix exists in the accessibility tree; one sticky position-and-role column stays bounded; scores remain close to names; team boundaries and row focus are visible without color-only meaning; adding or removing a string can change mode without losing the acted-on team or focus target; Optimize remains global; each Clear Squad trigger names and clears only its own team.
- Dependencies and ordering: commit 4 supplies the shared truthful position descriptions. Do not duplicate descriptor logic in the responsive table.

**Implementation profile:** Luna Max — the matrix data and mutations are established, but responsive fit, grouped semantic headers, mutation-driven mode changes, sticky layering, and focus restoration create a broad presentation integration surface.

**Review profile:** Sol High — deterministic tests cannot alone prove wide-table scan quality, responsive mode changes, WebView sticky behavior, or complex header accessibility, so review must combine code, interaction, and populated visual evidence.

**Validation:**

- RED then GREEN: `./scripts/dev test src/app/routes/planner.test.tsx`
- Browser control paths at constrained and explicit wide viewports: `./scripts/dev smoke`
- Commit gate: `./scripts/dev check`
- Manual populated evidence at 1280×800, 1600×900, and the supplied wide viewport: 1280 keeps a usable selected-team table; the wide common two-string case shows all teams without document overflow; the 1600 result follows actual available width with collapsed and expanded navigation; one-, two-, and three-string combinations keep readable headers, visible scroll cues, and stable focus; names, scores, warnings, menus, and team-specific Clear Squad actions remain usable.

**Stop conditions:** Stop and replan if combined mode requires changing Planner DTOs or app-shell width ownership; if readable minimum widths cannot show the common two-string case at the supplied wide viewport; if mode changes cannot preserve the acted-on team and focus after string mutations; if grouped headers cannot retain correct programmatic associations; or if sticky context fails in the supported WebView.

**Review mandate:**

- Verify the combined table renders Senior, Reserves, and Youth in stable order with each team's real strings and assignments under the correct group header.
- Verify the fit decision responds to matrix container width, navigation-rail expansion, and string-count changes without a manual preference or duplicated interactive table.
- Verify constrained mode retains roving team tabs and the selected team across resize or mode changes.
- Verify Add string, Remove string, assignment picker, move and clear confirmations, Optimize squads, and each team-specific Clear Squad target and focus-return path.
- Verify bounded vertical and horizontal overflow, sticky multi-row headers, sticky position context, team separators, row focus, long-name truncation, and score proximity at all required viewports.
- Verify table captions, row headers, team groups, string headers, and explicit cell accessible names provide complete context.
- Reject backend, query, cache, optimizer, new-data, global breakpoint, dependency, or unrelated workspace changes.

## Active work

**PR:** PR 1 — Redesign Squad Planner workspace (Ready for publication)

**Commit:** Commit 5 — Show all teams when the matrix fits (completed)

### RED proof

Focused route tests failed before implementation because the current UI had no matrix container measurement or combined-table behavior. The tests cover a wide matrix with Senior, Reserves, and Youth groups, real string counts, grouped headers, team-scoped actions, the fallback selected-team presentation, resize focus, and a string mutation that crosses the fit threshold.

### GREEN implementation

- `PlannerDepthMatrix` measures its own container with `ResizeObserver` plus a window-resize fallback and compares the current string count with the existing `min-w-52` geometry. It removes team tabs and the global Clear Squad control only in the combined mode; constrained mode keeps the tabs and all hidden team panels mounted so existing focus and interaction references survive selection changes.
- `PlannerDepthTable` renders one grouped semantic table with `colgroup` team headers, ordered string headers, explicit `headers` relationships, sticky two-row context, and team-specific Clear Squad controls. Internal string IDs, lane IDs, query keys, and mutation payloads remain unchanged.
- Clear Squad focus is tracked by team across responsive mode changes, including a focused combined-mode team that was not previously selected; the constrained view synchronizes that team before restoring focus.
- Route tests now pass 45/45, including combined, constrained, resize, grouped Clear Squad, and threshold-crossing Add string behavior. The browser smoke suite adds explicit constrained and wide viewport paths.

### Expected outcome

When the Planner matrix has enough available width, one semantic table groups Senior, Reserves, and Youth with each team's ordered strings beneath its header. When those readable minimum widths do not fit, the existing team tabs show one team at a time. Team boundaries, position context, scores, warnings, string menus, team-specific Clear Squad actions, and focus remain usable in both modes without changing internal lane IDs or mutation semantics.

### Explicit exclusions

- Do not change team, string, assignment, candidate, optimizer, scoring, query, cache, IPC, persistence, or Rust behavior.
- Do not add a manual All teams toggle, saved layout preference, global breakpoint, table virtualization, column reordering, or string reordering.
- Do not change the Tactic or Club setup workspaces beyond consuming the shared position descriptions from Commit 4.

## Discoveries and replanning

- Planning confirmed that PR #30 is merged, but the local `origin/main` ref is stale. Refresh `main` before creating the redesign branch; do not stack this feature on the merged feature branch.
- A prior uncommitted workspace-tab attempt used `clubs`, `tactic`, and `squad` and passed focused route tests, but it was not accepted because only an empty save was available. The populated screenshot supplied for this plan now confirms the vertical-stacking problem. Treat the old attempt as historical evidence, not source code.
- No current runtime command provides the former native UI-agent inspection. Keep native populated-state verification as explicit manual evidence and do not report browser smoke as native proof.
- After commits 1 through 3, populated inspection showed that `planner-depth-table.tsx` lets the sticky first column absorb much of the available width and separates player names from their scores. The remaining scan problem is column geometry and team switching, not workspace stacking.
- The old `laneLabel` mapping exposed stable persisted IDs as static position names, while the editable tactic owns the current IP/OOP positions and roles. Commit 4 removes that presentational mismatch with a Planner-local description helper; it does not require a Rust, DTO, persistence, or migration change.
- `PlannerDepth` already returns all teams against one shared tactic. An adaptive combined matrix is a React presentation change, but selected-team Clear Squad and string-header focus restoration must become explicit team-context interactions when all groups are visible.
- The readable matrix contract can reuse `min-w-52` (13rem) for the sticky position column and every string column. The fit helper converts that token-backed width using the root font size and counts the current strings across all three teams; it does not use a global viewport breakpoint or a persisted preference.
- Constrained mode keeps the three team panels mounted with `hidden` so only the selected table is exposed to assistive technology while existing cell and header references remain stable. Combined mode renders one table and puts each team's Clear Squad trigger in its group header.
- The 2026-08-04 replanning pass reopened unpublished PR 1 and added commits 4 and 5. It preserved the existing branch and completed commit history because no trunk or PR merge boundary has occurred.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Add Planner workspace navigation | `c5d6bce` | Added validated workspace search state, accessible Planner tabs, configured and first-use defaults, primary-club context, hidden mounted panels, route/smoke coverage, and current-state documentation. | Sol High approved after one fix round; focused route suite 37/37. | Native Tauri viewport evidence remains open because the former UI-agent runtime is unavailable; no scope deviations. |
| PR 1 | Commit 2 — Unify tactic lane editing | `133089d` | Replaced per-pitch lane controls with one selected-lane inspector for shared settings and visible IP/OOP phase controls; preserved tactic state, save lifecycle, and route boundaries; updated tests and current-state docs. | Sol High approved; focused route suite 38/38, full suite 155/155, repository gate, and browser smoke 12/12. | Native Tauri viewport evidence remains open because the former UI-agent runtime is unavailable; no scope deviations. |
| PR 1 | Commit 3 — Compact the squad depth workspace | `4d45d0f` | Grouped team selection and squad actions in one toolbar, compacted lane and assignment rows, added bounded two-axis matrix overflow with sticky context, consolidated latest success feedback, and updated tests and current-state docs. | Sol High accepted with no findings; focused route suite 40/40, full suite 157/157, application and repository gates, browser smoke 12/12, and staged secret scan passed. | Native Tauri viewport evidence remains open because the former UI-agent runtime is unavailable; no scope deviations. |
| PR 1 | Commit 4 — Present linked tactical positions | `Pending record` | Replaced static lane labels and numbers with current IP/OOP position-and-role descriptions across tactic, matrix, picker, confirmation, validation, and accessible surfaces; added linked pitch emphasis and duplicate-position qualifiers without changing stable lane IDs or payloads. | Sol High clean after two fix rounds; focused route suite 42/42, full suite 159/159, repository gate (211 Rust passed, 2 ignored), and browser smoke 12/12. | Native Tauri populated-state inspection remains open because the former UI-agent runtime is unavailable; no scope deviations. |
| PR 1 | Commit 5 — Show all teams when the matrix fits | `Pending record` | Added container-fit combined Senior, Reserves, and Youth table groups with explicit team/string associations, constrained selected-team fallback, team-specific Clear Squad headers, responsive mutation handling, and team-aware focus restoration; preserved existing Planner data and mutation contracts. | Sol High accepted after two fix rounds; focused route suite 45/45, full suite 162/162, repository gate (211 Rust passed, 2 ignored), browser smoke 13/13, and staged secret scan passed. | Native Tauri populated-state inspection remains open because the former UI-agent runtime is unavailable; no scope deviations. |

## Final validation

- `./scripts/dev test src/app/routes/planner.test.tsx`
- `./scripts/dev test`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Native Tauri inspection with a representative configured and populated save at 1280×800, 1600×900, and the supplied wide viewport. Capture evidence for Squad with one-, two-, and three-string team combinations; Tactic Both and single-phase views; and Club setup with the navigation rail collapsed and expanded.
- Keyboard-only pass: workspace tabs, phase controls, linked pitch positions, inspector fields, constrained team tabs, grouped team and string headers, matrix cells, string menus, picker, confirmations, Save tactic, Optimize, and each team-specific Clear Squad action.
- Confirm workspace changes preserve drafts and selections; active-save changes reset or refresh them at the existing boundaries.
- Confirm normal Planner copy, validation, and accessible names contain no lane numbers, static lane-ID position labels, or lane-focused instructions while current IP/OOP positions and roles remain truthful and distinguishable.
- Confirm wide mode shows all teams only when readable position and assignment widths fit; constrained mode retains the selected team; string mutations can change mode without losing the acted-on team or focus target.
- Confirm no document-level horizontal overflow, no content hidden behind the top bar, visible matrix scroll cues, sticky multi-row and position context, bounded scan distance, and visible focus throughout overflow.
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
