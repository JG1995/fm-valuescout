# Planner Workspace Redesign

## Status

Active

## Intent

Make Squad Planner a focused desktop workspace instead of one long page of equally weighted setup, tactic, and squad panels. The redesign puts squad decisions first, describes linked tactical positions in football terms instead of internal lane terminology, uses available desktop width to show the three teams together when the matrix remains readable, and presents both tactic phases as correctly oriented football shapes.

## User-visible behavior

- `/planner` has three URL-backed workspaces: **Squad**, **Tactic**, and **Club setup**.
- A configured save opens the Squad workspace when the URL does not select a workspace. A loaded save without a primary club opens Club setup. A valid explicit workspace in the URL wins over either default.
- The page header identifies the configured primary club when one exists. Only the active workspace is visible and exposed to assistive technology.
- Arrow keys, Home, and End move between workspace tabs. Tab changes replace the current URL search state so the selected workspace survives reload without adding noisy browser-history entries.
- Switching workspaces preserves unsaved club-family and tactic drafts, selected tactic lane, and selected Planner team. Switching the active app save keeps the existing reset and refresh behavior.
- The Tactic workspace shows the IP, OOP, or Both pitch view beside one selected-position inspector. The inspector contains the linked position's score weight, importance rank, preferred-foot rule, and the visible phase position and role controls.
- In Both view, IP and OOP pitches remain side by side at the supported desktop widths. The editor has one clear primary action: **Save tactic**.
- Both tactic pitches place the goal at the bottom and the attacking end at the top: GK is the lowest band and ST is the highest.
- Repeated DC, DM, MC, AMC, or ST placements use a three-slot central band instead of stacking vertically. One player is centred; two occupy right and left; three occupy right, centre, and left according to stable tactic order. The user continues to choose only the base position.
- Spatial qualifiers derive from every lane at the same phase position, independent of role. IP and OOP therefore use the same deterministic right-centre-left placement and naming rule.
- The Squad workspace keeps Senior, Reserves, and Youth presentation controls with **Optimize squads** in one compact toolbar above the matrix.
- Squad matrix rows use a compact two-line position-and-role summary and align each player name with the combined score. The matrix owns overflow when its strings or rows exceed the available workspace instead of pushing unrelated workspaces down the page.
- Successful squad actions use one compact latest-status region. Errors remain visible near the affected control and retain their existing accessible alert behavior.
- User-facing Planner copy does not expose lane IDs, lane numbers, or static labels such as **Left winger**. Tactic pitches, the inspector, the squad matrix, player-assignment flows, confirmations, validation, and accessible names describe the current linked IP/OOP positions and roles, with a spatial qualifier only when it is needed to distinguish positions.
- Focusing, hovering, or selecting a tactical position on either pitch emphasizes its linked counterpart without relying on a shared number.
- When the Squad workspace can preserve readable position and assignment widths, one semantic matrix shows Senior, Reserves, and Youth as grouped columns with their ordered strings beneath them. When the available Planner width or string count cannot preserve those widths, the existing team tabs show one team at a time.
- The matrix keeps position context and player scores close to the data they describe. Team boundaries and row focus remain visible across the combined view.
- **Optimize squads** and one **Clear all** action remain in the compact toolbar in both matrix layouts. Clear all requires confirmation that names Senior, Reserves, and Youth before it removes every current squad assignment.

## Invariants

- Rust continues to own Planner persistence, validation, candidate scope, score calculation, optimizer allocation, mutation semantics, and returned DTOs. Commits 1 through 5, 7, and 8 change React presentation and route state; commit 6 replaces only the existing team-scoped clear mutation within the established Rust and Tauri boundary.
- The clear command changes from team-scoped to save-scoped. Other IPC command names, query keys, cache reconciliation, SQLite schema, migrations, tactic payloads, and Planner DTOs do not change.
- Senior, Reserves, and Youth remain the fixed teams. Strings remain ordered and unlimited, and each team keeps at least one string.
- One save-scoped tactic with eleven stable linked lanes remains shared by all teams.
- Stable lane IDs and lane order remain internal Planner identities. User-facing labels derive from the current tactic draft or returned tactic and do not redefine persistence identity.
- Central horizontal slots are derived presentation state. They do not add MCL, MCR, DCL, DCR, STC, or another persisted placement value, and they do not change tactic payloads, validation, migrations, or optimizer inputs.
- Manual and optimized assignment provenance, save-wide player uniqueness, and manual-assignment precedence remain unchanged. Clear all removes assignments of both provenance types from every Planner string in the active save and does not change strings, tactics, or club-family settings.
- Missing phase scores render as `—`. Outside-pool and unresolved assignments remain occupied and visibly warned.
- Tactic and club-family drafts survive workspace changes. Failed saves retain the draft. An active-save change cannot carry a draft into the next save.
- All current keyboard paths, focus restoration, labelled controls, tab semantics, modal behavior, and mutation feedback remain available.
- The global top bar remains the owner of active-save and snapshot-freshness context. The Planner does not duplicate Load Data.
- The existing dark visual system, token-backed Tailwind utilities, IBM Plex typography, and Lucide icons remain unchanged.

## Non-goals

- No optimizer algorithm, eligibility rule, score calculation, preferred-foot behavior, ranking behavior, or gap recommendation changes.
- No tactic library, formation naming, custom string names, string reordering, drag-and-drop interaction, or new Planner data.
- No backend, IPC, or persistence change beyond replacing the team-scoped clear operation with one confirmed, transactional save-scoped clear operation. No schema, migration, dependency, design-token, app-shell, or global-state change.
- No mobile or narrow-window design. The product remains desktop-only at a 1280×800 minimum and a 1600×900 design viewport.
- No generic shared tabs or workspace framework. Planner follows the existing profile-tab pattern within its feature boundary.
- No accordion fallback or permanent three-pane cockpit.
- No user-defined display labels, persisted position names, manual matrix-layout preference, or new tactic-link identifier.
- No rename of internal `TacticLane`, `laneId`, Rust fields, database columns, or optimizer terminology that does not appear in the UI.
- No new formation constraints, maximum-position validation, user-controlled horizontal-slot selector, drag positioning, pitch zoom, or persisted pitch coordinates. Existing tactics with more than three lanes at one base position remain representable and are not rejected by this presentation change.

## Current-state map

- Relevant components: `src/app/routes/planner.tsx` loads all Planner queries and owns URL-backed Squad, Tactic, and Club setup workspaces. It keeps `PlannerClubFamilyPanel`, `PlannerTacticEditor`, and `PlannerDepthMatrix` mounted in labelled hidden tab panels; `src/app/components/app-shell-layout.tsx` gives the main region page-level vertical scrolling.
- Tactic presentation: `src/features/planner/components/planner-tactic-editor.tsx` owns the draft, phase view, selected lane ID, linked highlight, validation, and save mutation. `planner-tactic-pitch.tsx` renders each pitch with current position-and-role buttons and linked counterpart emphasis, while `planner-tactic-inspector.tsx` renders one selected-position inspector. `planner-tactic-pitch.tsx` now places repeated positions in a shared three-column band and renders attacking bands above the defensive bands with GK last, while `src/features/planner/utils/tactic-editor.ts` derives the same stable right-centre-left placement for labels and pitch columns regardless of role.
- Squad presentation: `planner-depth-matrix.tsx` owns selected-team state, container-fit mode, mutations, picker and menu state, and one latest squad-action status. `planner-depth-table.tsx` renders one semantic grouped table when the current strings fit the matrix container and keeps hidden non-selected team panels mounted for the constrained tabbed mode. Both presentations keep sticky position and string headers, bounded two-axis overflow, compact rows, explicit team context, and current IP/OOP position-and-role descriptions. `planner-slot-fit-picker.tsx` receives the current tactic and options so assignment locations and confirmations use the same descriptions.
- Current clear path: `PlannerClearAllControl`, `clearPlannerDepth`, the `clear_planner_depth` Tauri command, and Rust `clear_all` service clear every assignment for the active save after confirmation. The shared toolbar owns the one trigger in both combined and constrained matrix modes. Rust uses one transaction and returns the reconciled complete `PlannerDepth` read model.
- Club-family presentation: `planner-club-family-panel.tsx` owns a local draft and invalidates the Planner query tree after save.
- Existing analogue: `/players/$uid` validates a `tab` search parameter, replaces URL search state on tab changes, and uses an accessible roving-tabindex `PlayerProfileTabs` component with hidden tab panels.
- Data model: save-scoped club settings, tactic lanes, strings, and assignments remain unchanged.
- Persistence and migrations: SQLite migrations v4 through v10 remain outside this feature. Commit 6 replaces only the existing Planner clear service and command; all other Planner Rust services remain unchanged.
- Existing behavioral assumptions: the tactic editor is keyed by active save to prevent cross-save draft leakage; Planner workspace components already own their transient interaction state; query invalidation after saves and mutations is established.
- Tests: `src/app/routes/planner.test.tsx` covers Planner route behavior and interactions but does not assert pitch orientation, horizontal central-slot geometry, or role-independent duplicate labels; `src-tauri/src/features/planner/depth_tests.rs` covers confirmed team-scoped clearing and preservation of other teams; `e2e/smoke.spec.ts` covers no-snapshot, first-use setup, tactic save, string management, and optimizer paths through stubbed IPC.
- Project validation commands: `./scripts/dev test [target...]`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` is unsupported and is not acceptance evidence.
- Primary risks: keeping number-free tactical-position names truthful and distinguishable; keeping visible, DOM, keyboard, and accessible central-slot order aligned; preserving linked pitch emphasis and accessible names; avoiding focus loss when a string mutation changes responsive matrix mode; keeping complex team/string headers understandable; preventing nested-scroll or sticky-header failures; and ensuring the destructive Clear all operation cannot partially clear or target the wrong save.

## Feature architecture

The existing `/planner` route remains the only Planner page and continues to load the current snapshot plus all Planner queries. It owns a validated `view` search parameter, effective first-use default selection, the page header, and three labelled tab panels. Planner-local workspace navigation follows the established player-profile tab behavior and does not introduce a shared abstraction.

All three workspace components stay mounted while their tab panels use the native `hidden` attribute. This keeps current component-owned draft and selection state without a new store or lifted state. The existing active-save key continues to reset the tactic editor at the save boundary.

`PlannerTacticEditor` remains the draft and mutation owner. Pitch rendering stays presentation-only, while one Planner-local selected-position inspector renders the shared settings and the controls for the phase or phases visible in the selected tactic view. A Planner-local display helper derives user-facing descriptions from the current IP/OOP positions and roles and adds a deterministic spatial qualifier when positions would otherwise be ambiguous. Stable lane IDs continue to link both pitches, assignments, and save payloads but do not appear in user-facing copy. Focus, hover, and selection use the existing linked identity to emphasize the same tactical position on both pitches.

Commits 7 and 8 keep this ownership and replace only the pitch's derived geometry. Each phase groups lanes by base position without considering role, maps repeated central placements into a three-slot band from stable tactic order, and uses the same mapping for visible and accessible qualifiers. One placement uses centre; two use right then left; three use right, centre, then left. Additional existing placements continue through the same three-column presentation without changing or discarding tactic data. Pitch row order renders the attacking bands first and GK last so both shapes attack upward. The position selector continues to expose the existing base placements only.

`PlannerDepthMatrix` remains the squad interaction coordinator. It composes the compact team/action/status area, derives whether all teams fit from the matrix's available width and current string counts, and gives `PlannerDepthTable` one active presentation mode. The table uses one semantic grouped-header model: wide mode shows team column groups with ordered strings beneath them, while constrained mode shows the selected team and its tabs. Both modes keep one sticky position column, string header menus, cell accessible names, and focus restoration. One Clear all trigger stays in the shared toolbar in both modes, so team headers contain only team context.

Clear all uses one frontend mutation and one Tauri command. Rust requires explicit confirmation, resolves the active save, and deletes all assignments for that save in one transaction before returning the reconciled complete depth model. The implementation removes the team-scoped command, service, frontend adapter, control, mocks, tests, and focus state rather than keeping a compatibility path or issuing three sequential mutations.

The implementation uses existing React, TanStack Router, Tailwind, Panel, Button, field, Modal, and ScoreBadge patterns plus the established Rust-owned transactional mutation boundary. It adds no dependency and does not require an ADR because it replaces one internal Planner command without changing schema or layer ownership.

## Uncertainty register

### Known

- The original populated Planner screenshot supplied on 2026-08-04 showed club setup, the full dual-pitch tactic editor, and the squad matrix stacked in one long document. Commits 1 through 3 replaced that composition with workspaces and a compact bounded matrix.
- A second populated screenshot supplied on 2026-08-04 shows the implemented Squad workspace at about 1920px wide. The first column expands across roughly half the matrix, role and score context sit far apart, and the static **Left winger** label conflicts with lane 9's current AMC position.
- A populated Tactic screenshot supplied on 2026-08-05 shows Both view at 1920×1080. GK appears at the top, ST appears at the bottom, and repeated DC, MC, and ST positions stack inside one central cell. OOP duplicates show left/right qualifiers while IP duplicates with different roles do not.
- The app targets a 1280×800 minimum and a 1600×900 design viewport.
- PR [#30](https://github.com/JG1995/fm-valuescout/pull/30) merged Planner optimizer preferences into `main` as `1c4ec088246d6563e1ff05636af8928b4f5a290f` on 2026-08-04.
- The feature branch is `feat/planner-workspace-redesign`. It is published at `origin/feat/planner-workspace-redesign`, remains unmerged, and has no PR ref recorded.
- Local commit `500c081` compacted the team-specific Clear Squad triggers after commit 5. It remains one commit ahead of the published branch and will become obsolete when commit 6 removes those triggers.
- The removed UI-agent command is unavailable. Browser smoke uses stubbed IPC and does not prove native WebView or live SQLite behavior.

### Assumptions

- Squad is the frequent-use workspace after initial configuration. Club setup is low-frequency maintenance, and tactic editing is an occasional supporting workflow.
- A URL search parameter is the correct owner for the visible workspace because the state is reload-relevant and the profile route already establishes the pattern.
- Keeping hidden workspaces mounted is acceptable for three already-loaded Planner components and is simpler than a new client store or draft-lifting boundary.
- Existing design tokens and primitives are sufficient. The problem is hierarchy and composition, not branding.
- Current phase positions, roles, and stable linked identity contain enough information to produce truthful user-facing descriptions without a persisted display label.
- Stable tactic order is sufficient to derive horizontal slots. The user does not need separate DCL/DCR, MCL/MCR, or equivalent placement choices.
- The common two-string-per-team case can show all three teams at the supplied wide viewport once the position and assignment columns stop absorbing unused width. Constrained widths and larger string counts should retain the single-team presentation.

### Decisions

- Deliver the complete redesign in PR 1 with eight planned atomic commits. Commits 7 and 8 extend the same unpublished feature branch after commit 6 because two presentation-only tactic commits do not justify another publication and merge boundary.
- Use `squad`, `tactic`, and `clubs` as the URL values and **Squad**, **Tactic**, and **Club setup** as the visible labels.
- Use `replace: true` for workspace changes, matching the player-profile tab route.
- Keep inactive workspaces mounted inside hidden tab panels to preserve transient state.
- Keep the tactic inspector visible beside the pitch area. Both view shows two pitch boards and one inspector, not one inspector per pitch.
- Let the squad matrix own horizontal overflow and bounded vertical overflow. Do not constrain the entire route with an arbitrary pixel height.
- Keep stable lane IDs and order as internal persistence and linking contracts. Remove the word **lane**, lane numbers, and static lane-ID labels from normal Planner copy. Use current IP/OOP positions and roles, plus a deterministic spatial qualifier only when needed to distinguish linked positions.
- Use linked focus, hover, and selection emphasis across the IP and OOP pitches instead of visible numeric correspondence.
- Show all teams only when the Planner matrix container can preserve the existing readable minimum widths for the current strings. Otherwise show the selected team through the existing tabs. Do not add a manual layout toggle or persisted layout preference.
- Replace every team-specific Clear Squad action with one global Clear all action beside Optimize squads. The action clears Senior, Reserves, and Youth assignments together after explicit confirmation.
- Implement Clear all as one Rust-owned save-scoped transaction. Remove the old team-scoped command and every frontend trigger and adapter for it; do not sequence three team clears or keep a compatibility path.
- Derive central slots from the lanes that share the same phase position, regardless of their roles. For one, two, and three placements, assign stable tactic order to centre; right then left; and right, centre, then left respectively.
- Use the derived slot for both physical placement and spatial qualifiers. Do not expose horizontal-slot controls or add persisted position variants.
- Render both phase boards with the attacking end at the top and GK at the bottom.
- Update the relevant current-state `DESIGN.md` and `ARCHITECTURE.md` text only in the implementation or reconciliation step that makes each statement true.
- Do not create an ADR. The feature uses accepted React, Router, Tailwind, and Planner boundaries.
- For this experiment, use Luna Max for every implementation profile in this ledger. Keep Sol High for every review profile, including the final feature-complete review. This overrides the repository defaults for this ledger only.

### Unknowns

- The exact fit threshold for combined-team mode needs populated visual inspection with representative one-, two-, and three-string teams at 1280×800, 1600×900, and the supplied wide viewport. This does not block the terminology commit because the threshold must preserve existing token-backed minimum widths rather than introduce a new product contract.
- The exact button density for three central placements needs populated visual inspection in Both and single-phase views at the supported viewports. Existing truncation and `title` behavior must preserve full role text without widening the document.
- Native Tauri viewport inspection remains manual unless a supported repository command is added by separate, explicitly approved tooling work.

### Risks

- Two linked positions can share the same phase placement or role. User-facing descriptions must stay distinguishable without falling back to lane numbers or static position labels.
- Removing numeric pitch markers can make the IP/OOP relationship unclear unless focus, hover, selection, and accessible descriptions emphasize the linked counterpart together.
- A string add or removal can cross the combined/single-team fit threshold. The acted-on team and string header must remain visible and receive restored focus after the responsive mode changes.
- Multi-row team and string headers can lose programmatic context if group and column associations are only visual.
- Wider combined tables can recreate the original long eye path unless the sticky position column, assignment widths, score proximity, team separators, and row focus treatment are validated with populated data.
- A frontend sequence of three team clears could succeed only in part. Clear all must cross the IPC boundary once and commit or roll back every assignment deletion together.
- Removing the team-scoped command can leave dead registration, mock, focus, or test paths that mask the old contract unless review traces the symbol and user-visible action end to end.
- A CSS-only visual reorder can disagree with DOM and keyboard order. The slot helper, rendered grid placement, focus sequence, and accessible qualifier must all use the same right-centre-left mapping.
- Three central buttons can become too narrow in Both view. The central band must use more horizontal room than the current single centre cell while keeping wide positions anchored and avoiding document overflow.
- Deriving qualifiers from roles recreates the current IP/OOP mismatch whenever two lanes use different roles at the same position. Position grouping must ignore role.

## Walking skeleton

Commit 1 remains the feature walking skeleton: it replaced vertical workspace stacking with URL-backed, keyboard-operable Squad, Tactic, and Club setup tabs while preserving the existing three workspace components and their behavior. For the added scope, commit 4 establishes truthful linked-position language, commit 5 changes the matrix's team composition, commit 6 replaces repeated team-scoped destructive actions with one atomic save-scoped action, and commit 7 proves role-independent derived slots and horizontal central placement without changing tactic data.

## Delivery plan

### PR 1 — Redesign Squad Planner workspace

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(planner): redesign squad planner workspace`

**Purpose:** Deliver the complete Planner information architecture, truthful tactical-position language, adaptive three-team depth overview, one atomic Clear all action, and corrected tactic-board geometry in one review surface.

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

#### Commit 6 — Replace team clears with one atomic Clear all

**Status:** Completed

**Provisional commit:** `feat(planner): replace team clears with clear all`

**Work:** Replace the three team-scoped Clear Squad operations with one global Clear all action in the shared Squad toolbar. Add one confirmed Rust-owned transaction that removes every assignment across Senior, Reserves, and Youth for the active save, returns the reconciled depth model, and preserves all strings, tactics, and club-family settings. Remove the existing team-scoped buttons, focus state, frontend adapter, Tauri command, Rust service, mocks, and contract tests rather than retaining split-operation compatibility paths. Update route and Rust coverage, browser paths, and the implemented clear-action contracts in `DESIGN.md` and `ARCHITECTURE.md`.

**Out of scope:**

- Optimizer allocation, assignment provenance, candidate ranking, scoring, team or string configuration, tactic behavior, club-family behavior, or active-save selection.
- SQLite schema or migration changes, undo or restore support, selective clearing, per-team clearing, or a compatibility alias for `clear_planner_team`.
- Matrix fit thresholds, table geometry, row content, team tabs, string management, assignment-picker behavior, app-shell layout, dependencies, or design tokens.

**Implementation packet:**

- Owners and files: `src-tauri/src/features/planner/depth.rs`, `depth_tests.rs`, and `commands.rs`; `src-tauri/src/lib.rs`; replace `src/features/planner/api/clear-planner-team.ts` with a save-scoped adapter; replace or remove `src/features/planner/components/planner-clear-team-control.tsx`; update `planner-depth-matrix.tsx` and `planner-depth-table.tsx`; update the Planner IPC mock and setup in `src/testing/`; update `src/app/routes/planner.test.tsx`; update `e2e/smoke.spec.ts` and `e2e/tauri-ipc-stub.ts` where the clear contract is represented; update the implemented Planner clear contracts in `.wiki/DESIGN.md` and `.wiki/ARCHITECTURE.md`; and update this ledger with progress evidence.
- Existing patterns to verify: Rust `clear_team` confirmation and transaction handling; active-save resolution in Planner commands; `optimize_planner_depth` as one global mutation returning complete `PlannerDepth`; TanStack Query depth replacement and slot-candidate invalidation; destructive Modal copy, pending state, duplicate-submit prevention, error retention, trigger focus restoration, and latest action status; shared Squad toolbar behavior in combined and constrained modes.
- Constraints and invariants: expose exactly one Clear all trigger beside Optimize squads in both matrix modes; use one frontend mutation and one `clear_planner_depth` IPC call without a team argument; require authoritative Rust confirmation; delete all manual and optimizer assignments for the active save in one transaction; commit all deletions or none; return and cache one reconciled complete depth model; invalidate slot candidates once; preserve strings, team order, tactic, club-family settings, scores, and all non-clear mutations; use confirmation copy that names Senior, Reserves, and Youth; restore focus to the one trigger after cancel, error recovery, or completion; remove `clear_planner_team`, `clear_team`, `clearPlannerTeam`, every per-team trigger, and their mock and focus paths with no compatibility wrapper.
- Dependencies and ordering: commits 1 through 5 are complete. Local style commit `500c081` only compacted controls that this commit removes. Keep the work in PR 1 because the branch is unpublished and unmerged, and the change shares the final Squad toolbar, matrix, mutation, test, and documentation review surface.

**Implementation profile:** Luna Max — the current clear path, transaction, mutation, modal, cache, and test seams are established, but the replacement crosses React, IPC, Rust, mocks, accessibility, and destructive-state handling.

**Review profile:** Sol High — this is a destructive save-scoped persistence operation, so review must rule out wrong-save targeting, partial clearing, stale compatibility paths, and misleading confirmation or cache state.

**Validation:**

- RED then GREEN: `./scripts/dev test src/app/routes/planner.test.tsx`
- Rust behavior and gate: `./scripts/dev check-rust`
- Full frontend regression suite: `./scripts/dev test`
- Browser control paths at constrained and explicit wide viewports: `./scripts/dev smoke`
- Commit gate: `./scripts/dev check`
- Manual populated evidence at 1280×800, 1600×900, and the supplied wide viewport: exactly one Clear all trigger stays in the shared toolbar; no team header or selected-team Clear Squad trigger remains; the confirmation names all three teams; cancel and failure preserve every assignment; success removes assignments from every team and leaves strings, tactic, and club-family settings intact; pending state prevents duplicate submission; focus returns to the trigger.

**Stop conditions:** Stop and replan if one transaction cannot clear every active-save assignment without a schema or migration change; if any supported consumer outside the bundled app still requires `clear_planner_team`; if correct behavior requires sequential team commands, compensation, or undo semantics; if the returned complete depth model cannot reconcile the existing cache boundary; or if confirmation and focus cannot remain correct with one toolbar trigger in both matrix modes.

**Review mandate:**

- Verify the new Rust service resolves the active save through the existing command path and deletes only that save's Planner assignments in one transaction.
- Verify confirmation is enforced in Rust, cancel and error paths preserve all assignments, and pending state prevents duplicate destructive requests.
- Verify both manual and optimizer assignments across Senior, Reserves, and Youth are removed while strings, order, tactic, club-family settings, and other saves remain unchanged.
- Verify React invokes one clear-all command, replaces the depth cache once, invalidates slot candidates once, reports one truthful status, and never sequences three team mutations.
- Verify exactly one accessible Clear all trigger appears beside Optimize squads in combined and constrained modes, with complete confirmation copy and reliable focus restoration.
- Verify `clear_planner_team`, Rust `clear_team`, `clearPlannerTeam`, per-team buttons, team-target state, responsive clear-focus handling, command registration, mocks, and tests that preserve the old contract are removed without dead compatibility code.
- Reject schema changes, undo systems, selective-clear options, new abstractions, unrelated matrix work, or any weakening of destructive confirmation coverage.

#### Commit 7 — Arrange central positions across the pitch

**Status:** Completed

**Provisional commit:** `fix(planner): arrange central positions across pitch`

**Work:** Replace vertical stacking for repeated DC, DM, MC, AMC, and ST placements with a derived three-slot central band. Assign stable tactic order as centre for one lane, right then left for two lanes, and right, centre, then left for three lanes. Derive visible and accessible spatial qualifiers from the same phase-position grouping, regardless of role, so IP and OOP remain consistent. Keep existing tactic choices, payloads, draft behavior, linking, and save behavior unchanged. Update focused route and browser coverage plus the implemented tactic-board contract in `DESIGN.md`.

**Out of scope:**

- Pitch vertical orientation; commit 8 owns GK-at-bottom and ST-at-top row order.
- New persisted placement values, user-selected horizontal slots, formation constraints, validation limits, migrations, DTOs, Rust services, optimizer behavior, or squad-matrix behavior.
- Rejecting or rewriting an existing tactic with more than three lanes at one base position. Additional lanes must remain present and operable through the three-column presentation.

**Implementation packet:**

- Owners and files: `src/features/planner/components/planner-tactic-pitch.tsx`; `src/features/planner/utils/tactic-editor.ts`; one small Planner-local presentation helper or focused utility test only if it keeps the shared geometry and qualifier rule explicit; `src/app/routes/planner.test.tsx`; `e2e/smoke.spec.ts`; `e2e/tauri-ipc-stub.ts` only if a three-player central fixture is required; the tactic-board section in `.wiki/DESIGN.md`; and this ledger for progress evidence.
- Existing patterns to verify: current `PITCH_ROWS`; `phasePosition`, `phasePositionLabel`, and `phaseDescription`; stable tactic array order; LaneButton truncation, `title`, `aria-label`, `aria-pressed`, linked highlighting, and visible focus; the token-backed three-column pitch grid; and Both versus single-phase layouts.
- Constraints and invariants: group duplicates by phase position only, never by role; use one shared derived slot mapping for placement and labels; preserve lane array order and lane IDs; keep the base position selector unchanged; keep wide positions in their established left or right areas; give the central band enough room for three usable buttons; maintain full role text through accessible names and `title`; preserve every lane when more than three share a position; add no persistence or backend change.
- Dependencies and ordering: commit 6 must be complete and green. Keep this work on the existing PR 1 branch. Commit 8 relies on this commit's finished horizontal geometry but changes only row orientation.

**Implementation profile:** Luna Max — the tactic contract and component seam are established, but one derived mapping must coordinate labels, grid placement, DOM order, keyboard focus, responsive density, and unusual duplicate counts without changing persisted state.

**Review profile:** Sol High — review must catch mismatches between visual and accessible order, role-dependent regressions, clipped three-player bands, lost lanes, and accidental changes to tactic or optimizer contracts.

**Validation:**

- RED then GREEN: `./scripts/dev test src/app/routes/planner.test.tsx`
- Full frontend regression suite: `./scripts/dev test`
- Browser control path: `./scripts/dev smoke`
- Commit gate: `./scripts/dev check`
- Manual populated evidence at 1280×800, 1600×900, and the supplied 1920×1080 viewport in Both, IP, and OOP views: one central player is centred; two are side by side with the earlier lane on the right and the later lane on the left; three occupy right, centre, and left; different roles do not suppress qualifiers; buttons remain readable, focusable, linked, and free of document overflow.

**Stop conditions:** Stop and replan if correct placement requires new persisted position variants, changing lane order, role-catalog changes, Rust validation, or a maximum-position constraint; if additional existing lanes would be hidden or discarded; if Both view cannot fit three usable central buttons at 1280×800 within the supported layout; or if visual placement cannot share one deterministic mapping with DOM, keyboard, and accessible order.

**Review mandate:**

- Verify one, two, and three same-position central lanes use the specified centre, right-left, and right-centre-left allocation in stable tactic order.
- Verify duplicate grouping ignores role, including the supplied case where IP uses different MC roles while OOP uses the same role.
- Verify the spatial qualifier on every button, inspector heading, validation message, matrix description, and assignment location agrees with the derived pitch slot.
- Verify wide positions remain anchored, every lane remains selectable, and configurations above three duplicates lose no data or controls.
- Verify DOM order, keyboard traversal, visible focus, linked IP/OOP emphasis, truncation, `title`, and accessible names remain coherent.
- Verify tactic drafts, save payloads, lane IDs and order, query invalidation, optimizer inputs, Rust services, schema, and migrations are unchanged.
- Reject a second slot state, user-facing MCL/MCR-style choices, arbitrary raw CSS values, new dependencies, or unrelated Tactic and Squad cleanup.

#### Commit 8 — Orient tactic pitches toward attack

**Status:** Completed

**Provisional commit:** `fix(planner): orient tactic pitches toward attack`

**Work:** Reverse the vertical football orientation of both phase boards so ST is the highest band and GK is the lowest band. Preserve the central-slot geometry from commit 7, phase switching, selection, linked highlighting, inspector state, and tactic data. Update focused route and browser coverage plus the implemented orientation contract in `DESIGN.md`.

**Out of scope:**

- Horizontal slot allocation, duplicate qualifier rules, button sizing, or other pitch geometry beyond vertical row order.
- Direction arrows, goals, pitch markings, animation, drag-and-drop, formation names, data changes, or squad-workspace changes.

**Implementation packet:**

- Owners and files: `src/features/planner/components/planner-tactic-pitch.tsx`; `src/app/routes/planner.test.tsx`; `e2e/smoke.spec.ts`; the tactic-board section in `.wiki/DESIGN.md`; and this ledger for progress evidence.
- Existing patterns to verify: the shared `PITCH_ROWS` definition used by IP and OOP; Both, IP, and OOP rendering; selected goalkeeper default; linked selection and highlight state; pitch fieldset semantics; and commit 7's derived central-slot mapping.
- Constraints and invariants: render attacking bands from top to bottom and GK last in both phases; keep phase data, lane IDs, selection, inspector content, and save payloads unchanged; do not use CSS visual ordering that disagrees with DOM or focus order; preserve all existing button states and accessible names.
- Dependencies and ordering: commit 7 must be complete and green. This commit changes only shared row orientation and its evidence.

**Implementation profile:** Luna Max — the code change is narrow and reversible, but it must preserve shared IP/OOP semantics, selected-lane state, keyboard order, and commit 7's denser central layout across every phase view.

**Review profile:** Sol High — visual orientation is easy to make superficially correct while leaving DOM, focus, or one phase reversed, so review must combine deterministic order assertions with populated viewport evidence.

**Validation:**

- RED then GREEN: `./scripts/dev test src/app/routes/planner.test.tsx`
- Full frontend regression suite: `./scripts/dev test`
- Browser control path: `./scripts/dev smoke`
- Commit gate: `./scripts/dev check`
- Manual populated evidence at 1280×800, 1600×900, and the supplied 1920×1080 viewport in Both, IP, and OOP views: ST is visually highest, GK is visually lowest, IP and OOP share the same orientation, and every position remains readable and keyboard operable without overflow.

**Stop conditions:** Stop and replan if the correct visual orientation would require CSS order that disagrees with DOM or keyboard order; if one phase requires a different direction; if selected-lane or inspector behavior depends on the old row order; or if the change expands into pitch decoration, formation semantics, or tactic data.

**Review mandate:**

- Verify ST is the top band and GK the bottom band in IP, OOP, and Both views.
- Verify DOM and keyboard order follow the rendered top-to-bottom orientation rather than a CSS-only reversal.
- Verify selecting, focusing, and highlighting GK, central positions, and ST still links the same lane across phases and updates the existing inspector.
- Verify commit 7's one-, two-, and three-player central geometry and qualifiers remain unchanged.
- Verify tactic drafts, save behavior, payloads, validation, squad descriptions, and optimizer behavior remain unchanged.
- Reject pitch decoration, direction controls, animation, persistence work, dependencies, or unrelated layout polish.

## Active work

**PR:** PR 1 — Redesign Squad Planner workspace (Ready for publication)

**Commit:** All planned commits completed; awaiting PR publication

### RED proof

Add focused route coverage for IP, OOP, and Both pitch views that checks the first rendered row is an attacking band and the last rendered row is the goalkeeper band. Before implementation, these proofs fail because the shared `PITCH_ROWS` definition renders GK before ST, so the DOM and keyboard order put the goalkeeper above the striker.

### Expected outcome

Both IP and OOP pitches render ST and the other attacking bands above the midfield and defensive bands, with GK in the lowest band. The row order, DOM order, keyboard traversal, selected lane, linked highlighting, inspector content, and tactic payload remain unchanged apart from this shared vertical orientation.

### Explicit exclusions

- Do not change Commit 7's horizontal slot allocation, qualifiers, button sizing, or other pitch geometry.
- Do not add direction arrows, goals, pitch markings, animation, drag-and-drop, formation names, persisted pitch coordinates, data changes, or squad-workspace behavior.
- Do not change tactic payloads, lane IDs, draft behavior, linking, role validation, save behavior, Rust services, schema, migrations, or optimizer behavior.

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
- The current `clear_planner_team` path already requires confirmation, deletes one team's assignments in a Rust transaction, and returns the complete depth model. Calling it three times from React would permit partial success, so commit 6 replaces it with one save-scoped transaction and removes the split operation end to end.
- Local commit `500c081` compacted the soon-to-be-removed team-specific triggers as an explicitly approved trivial polish change outside the ledger. Preserve it in branch history; commit 6 supersedes its visible effect without rewriting history.
- The 2026-08-05 replanning pass reopened unpublished PR 1 and added commit 6. No trunk or PR merge boundary has occurred, and the change shares the existing Squad toolbar, mutation, and final visual review surface, so a second PR would add no independent merge value.
- The 2026-08-05 Tactic screenshot exposed a separate geometry problem after the workspace redesign: `PITCH_ROWS` renders GK before ST and gives each base position one cell, so repeated central positions stack. The current qualifier helper also groups by position and role, which explains why equal-role OOP midfielders receive left/right labels while different-role IP midfielders do not.
- The developer chose to keep tactic-board geometry in PR 1 because the extension contains only two focused presentation commits. Commits 6, 7, and 8 are complete on the existing branch, and PR 1 is ready for publication.
- The accepted horizontal rule is presentation-only: group by base phase position, keep the user's existing position choices, and assign stable tactic order right-centre-left when positions repeat. No schema, Rust validation, optimizer, or tactic payload change is required.
- Commit 7 implements the horizontal rule with one derived position layout map shared by pitch placement and descriptions. The focused route suite is green at 47/47, including one-, two-, and three-placement central cases, role-independent grouping, accessible names, and configurations above three lanes; the smoke assertion now covers the default two-MC case. Commit 8 completes the planned shared pitch orientation change without altering that geometry.
- The first two Sol High review passes found that nesting the three slots inside the old centre cell made controls too narrow, overflow-row labels did not describe their vertical row, and singleton central positions bypassed the shared centre slot. Commit 7 now gives the central position cell three of five pitch columns, routes every central position through the derived grid, and labels later rows explicitly, such as `right row 2`, while preserving stable DOM order and all lane controls.
- Commit 6 implementation replaces the team-scoped clear path end to end: one `clear_planner_depth` command and transaction clear the active save, the toolbar owns one confirmed Clear all control, and old team-target state, controls, adapters, registration, mocks, and tests are removed. The focused route suite is green at 45/45 and the Rust planner gate is green at 211 passed with 2 ignored; browser smoke coverage now includes the confirmed Clear all path. The repository gate is green, and the Sol High review is clear with no blocking findings.
- Commit 8 RED coverage failed against the existing GK-first `PITCH_ROWS` order, then passed after the shared row source was reordered to ST-first and GK-last. The focused route assertion covers Both, IP, and OOP DOM order, while browser smoke checks both rendered pitches; no CSS-only visual reversal or tactic-data change was needed.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 8 — Orient tactic pitches toward attack | Pending record | Reordered the shared pitch rows so ST is highest and GK is lowest in IP, OOP, and Both views; preserved DOM and keyboard order, linked selection, central-slot geometry, and tactic contracts; added route and smoke orientation assertions and updated the implemented design contract. | Sol High approved after 0 fix rounds; focused route suite 48/48, full frontend 165/165, Rust 211 passed/2 ignored, repository check, elevated smoke 14/14, format, and cached diff checks passed. | No scope deviations. Native populated viewport evidence remains open. |
| PR 1 | Commit 7 — Arrange central positions across the pitch | Pending record | Derived one-, two-, three-, and overflow-row position layouts from stable tactic order; widened the central band to three of five pitch columns; aligned labels, accessible names, DOM order, and slot classes regardless of role; preserved wide positions, singleton centre slots, all lane controls, and tactic contracts. | Sol High approved after two fix rounds; focused route suite 47/47, full frontend 164/164, Rust 211 passed/2 ignored, repository check, elevated smoke 14/14, format, and cached diff checks passed. | No scope deviations. Native populated viewport evidence remains open. |
| PR 1 | Commit 6 — Replace team clears with one atomic Clear all | Pending record | One confirmed toolbar action now clears all Senior, Reserves, and Youth assignments for the active save in one Rust transaction; old team-scoped paths were removed and implemented Planner contracts were updated. | Sol High clear after 0 fix rounds; focused route suite 45/45, full frontend 162/162, Rust 211 passed/2 ignored, repository check, elevated smoke 14/14, and check-fast passed. | No scope deviations. |
| PR 1 | Commit 1 — Add Planner workspace navigation | `c5d6bce` | Added validated workspace search state, accessible Planner tabs, configured and first-use defaults, primary-club context, hidden mounted panels, route/smoke coverage, and current-state documentation. | Sol High approved after one fix round; focused route suite 37/37. | Native Tauri viewport evidence remains open because the former UI-agent runtime is unavailable; no scope deviations. |
| PR 1 | Commit 2 — Unify tactic lane editing | `133089d` | Replaced per-pitch lane controls with one selected-lane inspector for shared settings and visible IP/OOP phase controls; preserved tactic state, save lifecycle, and route boundaries; updated tests and current-state docs. | Sol High approved; focused route suite 38/38, full suite 155/155, repository gate, and browser smoke 12/12. | Native Tauri viewport evidence remains open because the former UI-agent runtime is unavailable; no scope deviations. |
| PR 1 | Commit 3 — Compact the squad depth workspace | `4d45d0f` | Grouped team selection and squad actions in one toolbar, compacted lane and assignment rows, added bounded two-axis matrix overflow with sticky context, consolidated latest success feedback, and updated tests and current-state docs. | Sol High accepted with no findings; focused route suite 40/40, full suite 157/157, application and repository gates, browser smoke 12/12, and staged secret scan passed. | Native Tauri viewport evidence remains open because the former UI-agent runtime is unavailable; no scope deviations. |
| PR 1 | Commit 4 — Present linked tactical positions | `31c3c7e` | Replaced static lane labels and numbers with current IP/OOP position-and-role descriptions across tactic, matrix, picker, confirmation, validation, and accessible surfaces; added linked pitch emphasis and duplicate-position qualifiers without changing stable lane IDs or payloads. | Sol High clean after two fix rounds; focused route suite 42/42, full suite 159/159, repository gate (211 Rust passed, 2 ignored), and browser smoke 12/12. | Native Tauri populated-state inspection remains open because the former UI-agent runtime is unavailable; no scope deviations. |
| PR 1 | Commit 5 — Show all teams when the matrix fits | `9dcc5a4` | Added container-fit combined Senior, Reserves, and Youth table groups with explicit team/string associations, constrained selected-team fallback, team-specific Clear Squad headers, responsive mutation handling, and team-aware focus restoration; preserved existing Planner data and mutation contracts. | Sol High accepted after two fix rounds; focused route suite 45/45, full suite 162/162, repository gate (211 Rust passed, 2 ignored), browser smoke 13/13, and staged secret scan passed. | Native Tauri populated-state inspection remains open because the former UI-agent runtime is unavailable; no scope deviations. |

## Final validation

- `./scripts/dev test src/app/routes/planner.test.tsx`
- `./scripts/dev test`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Native Tauri inspection with a representative configured and populated save at 1280×800, 1600×900, and the supplied wide viewport. Capture evidence for Squad with one-, two-, and three-string team combinations; Tactic Both and single-phase views; and Club setup with the navigation rail collapsed and expanded.
- Keyboard-only pass: workspace tabs, phase controls, linked pitch positions, inspector fields, constrained team tabs, grouped team and string headers, matrix cells, string menus, picker, confirmations, Save tactic, Optimize squads, and the one Clear all action.
- Confirm workspace changes preserve drafts and selections; active-save changes reset or refresh them at the existing boundaries.
- Confirm normal Planner copy, validation, and accessible names contain no lane numbers, static lane-ID position labels, or lane-focused instructions while current IP/OOP positions and roles remain truthful and distinguishable.
- Confirm IP and OOP group repeated positions independently of role and use the same stable slot rule: one centred, two right then left, and three right then centre then left. Confirm DC, DM, MC, AMC, and ST central bands render horizontally, all lanes remain operable when a configuration exceeds three duplicates, and visible, DOM, keyboard, and accessible order agree.
- Confirm both pitches attack upward in Both and single-phase views: ST is the highest band and GK is the lowest band.
- Confirm wide mode shows all teams only when readable position and assignment widths fit; constrained mode retains the selected team; string mutations can change mode without losing the acted-on team or focus target.
- Confirm exactly one Clear all trigger appears in the shared toolbar in both matrix modes; confirmation names Senior, Reserves, and Youth; cancel and failure preserve every assignment; one successful command clears both manual and optimizer assignments from all teams in the active save without changing another save or any strings, tactic, or club-family settings; no team-scoped clear command or UI path remains.
- Confirm no document-level horizontal overflow, no content hidden behind the top bar, visible matrix scroll cues, sticky multi-row and position context, bounded scan distance, and visible focus throughout overflow.
- Confirm active save and snapshot age stay visible in the global top bar and all score and warning states remain truthful.
- Fresh Sol High feature-complete review over the exact recorded implementation commits and final PR ref.
- `./scripts/dev mutate` remains unsupported and must not be reported as passed.

## Documentation impact

During implementation and reconciliation:

- Update `.wiki/DESIGN.md` as each new workspace, tactic, and squad layout becomes implemented.
- Update the Planner composition paragraphs in `.wiki/ARCHITECTURE.md` after the implemented route and component ownership change.
- Update the Planner clear-operation text in `.wiki/ARCHITECTURE.md` and the Squad action contract in `.wiki/DESIGN.md` when commit 6 makes the save-scoped Clear all behavior true.
- Update the Tactic board geometry and orientation contract in `.wiki/DESIGN.md` with commits 7 and 8. No `ARCHITECTURE.md` update is expected because pitch geometry remains inside the existing React presentation owner.
- Keep `.wiki/CONCEPT.md` unchanged because product purpose and scope do not change.
- Create no ADR unless implementation crosses an accepted routing, feature, or app-shell boundary.
- On completion, condense this ledger into `.wiki/features/completed/planner-workspace-redesign.md` and move the TODO entry to Completed.
