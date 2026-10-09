# Signal Product Redesign

## Status and authority

This document records the proposed product-design follow-up to the Signal brand integration.
It is a comprehensive change inventory, not an accepted delivery ledger or a description of implemented behavior.
Checked items record completed work; unchecked items are proposals or verification work.
Neither checklist state grants approval to implement, commit, or publish.

[DESIGN.md](DESIGN.md) remains the current design-system authority.
[CONCEPT.md](CONCEPT.md) owns product purpose and boundaries.
Selected targets belong in DESIGN.md, clearly separated from current behavior; update current tokens and component descriptions only when implementation makes them true.
Accepted multi-commit delivery belongs in a feature ledger.
Do not maintain competing token definitions here.

**Step 1: done — shared rules settled.** The developer selected a decisions-only checkpoint, followed by step-2 implementation.
**Step 2: done — shared framing implemented and verified in Chromium.** [DESIGN.md's shared framing rules](DESIGN.md#shared-framing-rules) describe current behavior and comparison evidence.
Checked preservation items describe the step-2 change, not permission to stop protecting them in later work.
Moneyball Profile and smaller-desktop Tactic composition, feature-specific refinements, and native Windows verification remain pending.
Native rendering belongs to step 5, not the step-2 completion gate; completion does not authorize publication.

- [x] Settle navigation treatment.
- [x] Settle heading hierarchy and shared spacing.
- [x] Settle radius roles.
- [x] Settle dataset-toolbar composition and ownership.
- [x] Settle feedback footprint and safety constraints.
- [x] Settle numeric typography.

## Goal

Make FM ValueScout feel like one coherent analytical instrument, rather than a collection of dashboard pages wearing the same palette.

The Signal identity is **a blip on the night pitch**: a dark instrument field, precise typography, wide tracked capitals in limited display roles, and one lit green accent vocabulary.
The intended qualities are quiet, precise, and alert.
Translate those qualities through layout, density, hierarchy, shapes, and interaction—not only through colour and fonts.

The governing direction is:

> Keep the analytical core. Simplify the framing around it. Recover useful workspace without compressing readable data or hiding essential controls.

“One lit green” means one controlled accent vocabulary, not literally one green element per screen.
Scores, warnings, errors, and phase distinctions still need their own truthful semantic treatment.

## Evidence and limits

The audit inspected the Signal kit under `.work/brand-identity/`, DESIGN.md, shared UI components, route composition, and relevant feature components.
Rendered evidence covered all 19 canonical inspection screens at 1600×900, plus Search, Tactic, and Moneyball Profile at 1280×800.
The original audit baseline was `36fa5c9`, following the Signal integration commit `e78a011`; its observations below describe that baseline unless updated explicitly.

Step 2 compared all 19 canonical populated routes before at `7169c53` and after implementation, at both 1280×800 and 1600×900 (38 matched pairs).
Shared framing recovers workspace without reducing row heights: Search's first row moves from approximately y=362px to y=284px, and Squad's from y=444px to y=380px.
Additional runtime checks covered normal/shortlist Staff controls, long mixed-script filters and save names, removal/Clear all, modal focus return, recovery locks, and expanding error copy; lower Settings sections were inspected separately.
The full unit suite passed 1,044 tests, smoke passed 65 tests, and `./scripts/dev check` passed, including 882 Rust tests.
The canonical token-pair and actual-state contrast evidence belongs in [DESIGN.md](DESIGN.md#accessibility-of-colour).

The captures used Chromium and synthetic Tauri IPC fixtures.
They do not prove native WebView2 rendering, live FM integration, or behavior with every production dataset.
Developer-tool widgets visible in those captures are not part of the intended product design.
Disposable captures were removed after inspection.

Distinguish three kinds of item:

- **Observed:** a condition seen in rendered evidence or confirmed in source.
- **Proposed:** a design judgement to validate in the real component system.
- **Verify:** an uninspected state or realistic boundary to check; not a confirmed defect.

The kit is temporary evidence, not a durable repository dependency.
Its presentation pages and large display typography are brand demonstrations, not desktop workspace specifications.
Do not copy their marketing-page scale, light grounds, or generous hero spacing into the app.

## Preserve

These choices already fit the product and Signal identity:

- [x] Preserve dark-only, offline-first, desktop-only operation and the supported 1280×800 minimum window.
- [x] Preserve 40px two-line rows in shared analysis tables, including name and club/division context; measured outer rows remain 41px including the hairline.
- [x] Preserve the 36px single-line table option where its content warrants it; do not force all table types to share one height.
- [x] Preserve 32px minimum profile attribute rows and wrapping where long labels need it.
- [x] Preserve the 13px table-text and 14px body/control scale, with the verified raw-number/score split.
- [x] Preserve 16px page padding and default gutters, with the existing 4px spacing unit.
- [x] Preserve 32–36px ordinary desktop controls and 44px pitch-selection targets; compact framing must not shrink usability.
- [x] Preserve full-width analysis workspaces, sticky identity, grouped headers, bounded configurable column widths, and table-owned scrolling.
- [x] Preserve fixed-width Planner strings instead of stretching sparse boards to fill the display.
- [x] Preserve numeric values, missing-value distinctions, score tiers, phase cues, snapshot provenance, and explicit mutation feedback.
- [x] Preserve text-first player and club identity, optional local graphics, and stable graphic fallback slots; image-stage proportions are unchanged.
- [x] Preserve existing URLs, navigation history, selection, drafts, column preferences, and guarded mutation behavior.

Do not add a permanent sidebar, global content-width clamp, decorative charts, or dashboard statistics solely to express the brand.

## Priority overview

| Priority | Work | Reason |
| --- | --- | --- |
| P1 | Settle shared rules — **Done** | Six rules are documented and implemented |
| P1 | Compact shared navigation and table framing — **Done** | Shared framing returns space to data without shrinking rows |
| P1 | Moneyball Profile proportions and minimum-height usability | Evidence gets squeezed or falls out of the initial view |
| P1 | Tactic composition at smaller desktop widths | The XI list displaces the pitch from the initial viewport |
| P1 | Reconcile stale DESIGN.md statements — **Done** | Current rules, geometry, and corrected contrast tokens match implementation |
| P2 | Simplify surfaces, shapes, and emphasis | Removes inconsistent dashboard-like framing |
| P2 | Refine profile summaries, Academy, Planner, and Settings | Applies shared rules to feature-specific composition |
| P3 | Inspect dialogs, long-operation states, and remaining boundaries | Extends verified coherence beyond initial populated screens |

P1 and P2 are recommended order, not delivery commitments.
Resolve shared rules before applying local visual adjustments.

## 1. Shared shell and navigation

### 1.1 Compact the destination navigation

**Observed:** the utility bar and destination navigation occupy approximately 129px before page content begins.
The destination band stacks 24px icons, labels, and group captions.
That is a prominent launcher-like treatment for a data-first companion.

- [x] Retain the existing 56px utility bar and its global controls as the selected rule.
- [x] Compare and implement the compact inline-icon destination band recorded in DESIGN.md.
- [x] Preserve direct access to the existing destinations and their logical grouping.
- [x] Retain enough group context to distinguish player, staff, and club destinations without adding another tall caption row.
- [x] Implement quieter active navigation with a subtle tint, persistent indicator, and reinforced label.
- [x] Preserve non-colour selected cues, `aria-current`, keyboard focus, and profile group-context behavior.
- [x] Verify that all ten destinations fit in the comparison candidate at 1280px with readable labels and usable targets.
- [x] Reverify destination fit, focus, and group-context behavior in the production implementation.
- [x] Do not reclaim vertical space by burying frequently used destinations in menus.

**Primary owner:** `src/app/components/app-nav-bar.tsx`.

### 1.2 Separate the in-app mark from the launcher icon

**Observed:** the utility bar imports the saturated green launcher tile from the Tauri icon set.
It creates a bright block where the identity mechanism calls for a restrained outline with a lit blip.

- [x] Use an appropriate dark-ground outline symbol with its green blip in the utility bar.
- [x] Keep the launcher tile for OS app-icon contexts.
- [x] Put the finished SVG in `src/assets/signal-symbol.svg`; do not import from `.work/`.
- [x] Use a finalized coloured SVG, not an unprocessed master whose `data-color` roles still need resolving.
- [x] Verify optical size, clear space, and rendering at the actual 36px header size.
- [x] Keep an accessible product name without adding a large wordmark that crowds global search or save context.

**Primary owner:** `src/app/components/app-top-bar.tsx`.

### 1.3 Preserve global-context usability

- [x] Verify long save names, dates, global search, and pending Load Data labels at the minimum width.
- [x] Keep the active save and matching current snapshot date visible on every data workspace; separate the nonshrinking date from the name-only native select.
- [x] Keep Back and Forward recognizable and keyboard-operable.
- [x] Verify the reduced workspace height while a Load Data outcome or progress region is visible.
- [x] Keep pending, failed, and successful refresh feedback understandable without turning it into a large permanent banner.

## 2. Workspace headers and table framing

### 2.1 Remove redundant heading layers

**Observed:** Search stacks navigation, Player Search, Results, a dataset toolbar, and two table-header rows before showing data.
At 1600×900, its first data row begins around y=362px; Squad begins around y=444px.
These are observations of the inspected captures, not proposed fixed coordinates.

- [x] Define one workspace-header and semantic heading hierarchy in DESIGN.md, including equivalent destination context and feature-owned action ownership.
- [x] Apply that hierarchy to workspace headers, preserving meaningful context and actions.
- [x] Remove generic headings such as Results when they add no information.
- [x] Remove redundant Staff, Squad overview, Graphics, Bridge, and Graduates titles while retaining meaningful section headings.
- [x] Do not remove meaningful section headings or accessible table captions merely to reduce visible text.
- [x] Keep feature actions outside the generic dataset toolbar, preserving its existing ownership boundary.
- [x] Define one spacing pattern from workspace header through feature actions/feedback to dataset toolbar/table, without independently accumulated margins.

### 2.2 Standardize dataset toolbars

- [x] Settle the shared toolbar's slot order, ownership, compact geometry, chip row, concise idle copy, and wrapping policy in DESIGN.md.
- [x] Compare longer sort labels, added chips, and Staff shortlist controls at both core viewports; this is decision evidence, not proof of production filter behavior.
- [x] Align count, sort context, filter state, Columns, and dataset toggles consistently across Search, Squad, and Staff.
- [x] Prefer concise state copy to persistent instructional sentences once the user has an obvious Edit filters control.
- [x] Preserve first-use guidance where it genuinely explains a required next step.
- [x] Keep active filter chips and clear/remove actions visible and understandable.
- [x] Test wrapping with many filters, longer sort labels, and Staff shortlist controls, including eight long mixed-script rules at both core sizes.
- [x] Keep toolbar geometry compact without clipping focus rings or making controls inaccessible.

**Primary owner:** `src/components/player-table/table-toolbar.tsx` and feature-owned table panels.

### 2.3 Reduce idle feedback reservations

**Observed:** `SquadFeedbackSlot` reserves a minimum 64px even when it contains no feedback.
Staff also shows substantial space between setup controls and the results panel; inspect its status ownership before changing it.

- [x] Select the local reserved-slot and global zero-idle-footprint rules, bounded supplementary details, and visible recovery-summary constraints in DESIGN.md.
- [x] Replace large idle reservations with the selected compact 24px local status treatment and 4px preceding gap.
- [x] Preserve stable action headers and truthful pending, success, error, and recovery feedback.
- [x] Let long essential outcomes expand in the deliberate feedback region rather than clipping safety copy or reserving permanent empty space.
- [x] Retain live-region behavior and focus destinations for recovery states.
- [x] Check existing status-region tests and documented mutation contracts before implementation; retain Staff's stable region and Squad's recovery focus/lock behavior.

**Primary owner:** `src/features/squad/components/squad-overview-panel.tsx`; Staff route and assignment-optimizer components.

## 3. Shapes, surfaces, and spacing

### 3.1 Reduce nested container furniture

**Observed:** several workspaces combine rounded outer panels, rounded toolbars, rounded inner tables or cards, and pill controls.
Tactic, Planner, and Academy show this most clearly.

- [x] Audit shared table hosts and remove the redundant inner rounded/bordered scroller; retain their meaningful outer boundary.
- [x] Integrate flush analysis and Graduates tables with their host, using hairline separation instead of a second card.
- [x] Keep useful boundaries around independent scrollers, dialogs, and interactive assignment targets.
- [x] Preserve the recessed table-header treatment and neutral data surfaces.
- [ ] Complete the feature-specific border/surface audit for Tactic, Planner, and Academy's inner regions.

### 3.2 Adopt the settled radius roles

**Decision and shared rollout complete.** [DESIGN.md](DESIGN.md#radius-roles) owns the implemented purpose-based scale.
It is a product rule derived from comparisons, not a requirement copied from the brand kit.

- [x] Compare current and target radii in populated Search, Profile, Tactic, and Academy screens using disposable browser overrides.
- [x] Select radius roles by component purpose rather than treating every clickable element as a pill.
- [x] Select a distinct dialog/overlay role after inspecting the Staff configuration dialog.
- [x] Apply these roles without changing target sizes, action meaning, or meaningful circles.
- [x] Update shared primitives and tokens first; avoid scattering local radius overrides.
- [x] Match focus geometry to the revised shape.
- [x] Retain circles where they carry meaning, particularly pitch-selection targets; do not remove them merely for stylistic uniformity.

### 3.3 Keep density contextual

- [x] Preserve row heights rather than using smaller rows to compensate for excessive chrome.
- [x] Apply 8px related-control gaps and 16px region spacing in shared workspace framing.
- [ ] Review 12px gaps and 20px column gaps individually; their existence is not itself a defect, but repeated patterns should be intentional.
- [x] Align shared header, toolbar, search, Columns, and segmented-control heights, baselines, and padding across consumers.
- [x] Keep horizontal overflow local to tables and boards.
- [x] Do not stretch bounded table columns to eliminate empty space when that would slow scanning or violate saved widths.

## 4. Emphasis, typography, and icon treatment

### 4.1 Reserve strong emphasis for useful state and actions

- [ ] Make passive Academy metric icons neutral rather than brand green.
- [ ] Make local section/category selection quieter than primary actions, while retaining an unmistakable selected state.
- [ ] Keep neutral metrics neutral; do not colour a statistic green simply because it is positive-looking data.
- [x] Preserve score-tier meanings and separate data-green/brand-green roles while correcting low-tier contrast.
- [x] Keep IP/OOP phase distinctions readable by label and stroke treatment, not just colour.
- [x] Use a quieter Clear all in the dataset filter row; clearing filters is not a destructive data mutation.
- [ ] Test lower-emphasis Delete class and repeated Remove triggers; keep strong destructive styling for confirmation.
- [ ] Preserve explicit labels and target-specific confirmations. Reduced visual emphasis must not disguise the consequences.
- [ ] Review screens with both global Load Data and a local primary action so they have distinct roles rather than competing as equal focal points.

### 4.2 Reconcile numeric typography

**Original observation:** shared numeric table cells used JetBrains Mono.
Step 2 implements the [raw-value/score split in DESIGN.md](DESIGN.md#numeric-typography), including right-aligned numeric ages.

- [x] Choose and document one intentional rule for raw table figures, scores, summary metrics, and literal strings.
- [x] Test Archivo tabular figures for ordinary numeric columns while retaining JetBrains Mono for scored units, summary metrics, paths, and diagnostics.
- [x] Compare readability, occupied width, and mixed raw-value/percentile cells, including missing-value specimens, before deciding.
- [x] Apply the chosen numeric roles through shared cells and relevant callers; do not change the mono token or ScoreBadge globally.
- [x] Preserve tabular alignment and numeric sorting; typography must not alter displayed facts.
- [x] Settle tracked capitals as short structural labels, not ordinary controls or explanatory sentences.
- [x] Settle the heading/weight hierarchy so workspace identity, sections, and ordinary controls do not all compete.

**Primary owner:** `src/components/player-table/table-cells.tsx`, shared score/attribute primitives, and `src/styles/global.css`.

## 5. Player Profile

### 5.1 Preserve the persistent identity rail

- [x] Keep name, club, nationality, age, and supporting identity facts available across sections.
- [x] Preserve stable portrait and crest slots and readable fallbacks.
- [ ] Verify long names, mixed scripts, missing images, and real local portraits before changing rail width or image-stage height.
- [ ] Keep Modify Player and hidden-information controls visually secondary but accessible.
- [x] Preserve concealed-information behavior and development-action safeguards.

### 5.2 Strengthen Overview hierarchy

**Observed:** five similarly framed summary cards give neutral facts and tactical conclusions nearly equal visual weight.

- [ ] Group Current Ability, Potential Ability, and Market Value more tightly as neutral facts.
- [ ] Give the two role-fit summaries a distinct but restrained composition so the useful conclusion is easy to find.
- [ ] Reduce unnecessary card height and empty space without squeezing long role names or misaligning Current → Potential pairs.
- [ ] Preserve all existing facts and concealment rules; regrouping is not permission to remove information.
- [ ] Verify the balance of summary area and analytical evidence at both supported minimum and larger desktop sizes.

**Primary owner:** `src/features/player-profile/components/player-overview-panel.tsx`.

### 5.3 Keep Attributes and General Role Fit as strong reference surfaces

- [x] Preserve the dense attribute groups and hairline row rhythm.
- [x] Keep General Role Fit's bounded pitch sizing and usable role-name area.
- [ ] Review whether score-circle treatment needs simplification only after shared shape rules are settled.
- [x] Preserve readable wrapped role names, aligned score columns, and accessible position selection.
- [ ] Verify the combined Overview at the minimum height, not only the dedicated Attributes and Role Fit sections.

### 5.4 Repair Moneyball workspace proportions

**Observed:** at 1600px, the Moneyball pitch occupies most of its right-hand panel and squeezes role names into narrow wrapped columns.
At 1280×800, the initial capture shows summary/context but no actual metric rows.
The latter observation does not establish that those rows are unreachable; their scrolling and focus behavior require verification.

- [ ] Reuse the General Role Fit approach to bounded pitch sizing instead of the current 240–360px allocation in a narrow panel.
- [ ] Protect useful role-name width alongside the score column.
- [ ] Revise stacked-height allocation so summary context does not consume the metric evidence workspace.
- [ ] Ensure metric categories and a useful set of metric rows remain visible or clearly reachable at 1280×800.
- [ ] Make horizontal category overflow apparent; verify access to all eight categories, including Results.
- [ ] Keep role-contribution disclosures readable when expanded and reachable with keyboard navigation.
- [ ] Preserve raw data, cohort context, unavailable-score explanations, and current-only Moneyball semantics.
- [ ] Avoid creating multiple tiny nested scrollers as a workaround for insufficient space.

**Primary owners:** `src/app/routes/players.$uid.tsx`, `moneyball-profile-panel.tsx`, and `moneyball-role-fit-panel.tsx`.

## 6. Staff workspaces

- [x] Apply the shared workspace-header and dataset-toolbar rules to Staff Search and My Staff, including a semantic hidden Staff Search h1.
- [x] Keep upload, staffing configuration, and optimization actions distinct from table filters and columns.
- [x] Retain the compact mounted optimizer-feedback region, readiness guidance, and expanded results contract.
- [x] Verify the normal Staff Search state separately from shortlist-on inspection fixtures.
- [ ] Check expanded assignment results, vacancies, long job titles, and evidence disclosures; these were not covered by the initial-page audit.
- [x] Apply shared section-heading, radius, and action primitives to Staff Profile without forcing Player Profile's rail layout onto different content.
- [ ] Refine Staff Profile's feature-specific summary composition and long-label treatment.
- [ ] Keep long staff attribute labels readable and role-score lists aligned.
- [ ] Keep boost actions secondary to analysis in the profile's visual hierarchy, preserving their confirmation and outcome behavior.

## 7. My Club context and Squad

- [ ] Make the saved managed-club context compact and distinguish it from an unsaved selector draft.
- [ ] Test an explicit editing treatment instead of permanently prominent Save managed club controls in every workspace.
- [ ] Preserve exact-club selection, missing-club warnings, explicit save semantics, and the stable managed-club setup target.
- [ ] Keep Define DNA discoverable without giving configuration equal prominence to daily analysis.
- [x] Apply the compact header, toolbar, and feedback treatment to Squad.
- [ ] Keep boosts and CSV uploads available, but order and emphasize them consistently rather than making every action equally loud.
- [x] Preserve all upload-format, confirmation, progress, recovery, and cache-refresh behavior.

## 8. Tactic

### 8.1 Keep the pitch central at smaller desktop sizes

**Observed:** at 1280×800, the Tactical XI stacks above the pitch and dominates the initial viewport beside the inspector.
The pitch falls below the fold.

- [ ] Recompose the smaller-desktop layout around pitch visibility.
- [ ] Test a bounded XI list, compact disclosure, or another deliberate selection surface instead of placing the entire list above the pitch.
- [ ] Keep selected-slot controls accessible without horizontal page overflow.
- [x] Preserve both phase controls, lane selection, draft retention, and Save tactic behavior through shared-framing changes.
- [ ] Verify every lane is reachable by keyboard and its focused marker stays visible.
- [ ] Check portrait and landscape orientation on either side of the 2100px switch.
- [x] Do not shrink marker targets or role text to force a full pitch into an unsuitable rectangle.

### 8.2 Simplify the editor's framing

- [ ] Reduce repeated borders around outer panel, command bar, XI list, pitch wrapper, and inspector where region meaning is already clear.
- [x] Keep command controls compact and aligned with shared control rules.
- [ ] Use hairline-separated inspector sections rather than additional card layers.
- [ ] Make selected lanes clearly identifiable without large competing green surfaces.
- [x] Preserve steel IP borders, dashed magenta OOP borders, labels, and meaningful phase connectors.
- [ ] Verify long roles, edited dense midfield shapes, and Both view at minimum and wide desktop sizes.

**Primary owners:** `planner-tactic-editor.tsx`, `planner-tactic-lane-list.tsx`, `planner-tactic-inspector.tsx`, and pitch components.

## 9. Planner

- [ ] Remove the redundant boxed toolbar layer within the Squad depth panel where a simple aligned action row will suffice.
- [ ] Reduce repeated box-inside-cell treatment, especially for empty assignment cells.
- [ ] Keep Assign visibly actionable; empty cells must not become ambiguous blank space.
- [ ] Keep occupied players, Current → Potential scores, and unresolved/outside-pool warnings easy to distinguish.
- [x] Preserve fixed-width strings, team grouping, tactical-slot context, and sticky headers.
- [x] Do not fill the unused right-hand space by stretching the board when there are only a few strings.
- [ ] Review row padding and two-line role context together; do not create irregular rows or clipped labels.
- [ ] Apply restrained destructive treatment to Clear all while retaining target-specific confirmation.
- [ ] Inspect multiple strings, renamed teams, long player names, and more populated boards before finalizing.

**Primary owners:** `planner-depth-matrix.tsx` and `planner-squad-board.tsx`.

## 10. Youth Academy

### 10.1 Simplify metric presentation

**Observed:** six repeated raised metric cards contain icon boxes, values, and explanatory paragraphs.
Several passive icons use brand green.
The Class workspace repeats this group above its roster.

- [ ] Keep outcome metrics more prominent than administrative context, but reduce unnecessary card and icon-box framing.
- [ ] Make passive icons neutral and smaller where the label already identifies the metric.
- [ ] Keep metric definitions available in quieter supporting text or an accessible disclosure where appropriate.
- [ ] Never hide an unavailable-data explanation or convert an unknown value to zero for visual neatness.
- [ ] Reduce the repeated summary's vertical footprint in the Class workspace so the roster remains useful.
- [ ] Preserve overview and class-specific meanings; do not accidentally show global outcomes as class outcomes.

### 10.2 Refine classes and roster actions

- [ ] Replace card-inside-panel class presentation with a simpler selectable list or restrained tile treatment where appropriate.
- [ ] Keep year, tracked count, supported outcomes, and the opening action clear.
- [ ] Reduce visual competition from repeated Sell, Release, and Remove controls.
- [ ] Preserve visible labelled actions and target-specific confirmation; do not hide all actions behind hover.
- [ ] Make empty Sold and Released groups compact while retaining their meaning.
- [ ] Verify roster overflow, readable identity, long club names, and larger cohorts at 1280×800.

**Primary owners:** `academy-statistics.tsx`, `academy-overview.tsx`, and `academy-class-workspace.tsx`.

## 11. Settings

**Observed:** explanatory sections and panels span much more width than their content needs, and section-title treatments are inconsistent.

- [ ] Use a bounded reading/form width for preferences, boost explanation, and graphics controls.
- [ ] Let genuinely tabular save/snapshot management retain enough width; do not apply one narrow clamp to everything.
- [x] Apply the shared section-heading and control vocabulary, including nested Save data/Bridge management headings and loading states.
- [x] Remove redundant Graphics and Bridge panel titles while retaining meaningful outer sections and install context.
- [ ] Keep dangerous actions separated from ordinary configuration and explain their scope.
- [x] Inspect the lower populated Save data and Bridge sections, not only the initial viewport.
- [x] Preserve save/snapshot identity, current-state markers, and deletion consequences.

**Primary owner:** `src/app/routes/settings.tsx` and its feature-owned sections.

## 12. Dashboard

**Observed:** Dashboard contains its heading and `Placeholder.`
It is unfinished product content, not a colour, spacing, or branding defect.

- [ ] Decide separately whether Dashboard has a useful supported purpose or whether a useful existing workspace should be the entry destination.
- [x] Do not invent metrics, charts, onboarding cards, or promotional content solely to fill the page.
- [x] Keep any dashboard product decision outside the visual-consistency work until explicitly accepted.

## 13. Dialogs, feedback, and interaction states

These are verification tasks and extensions of the proposed shared rules, not findings from the initial captures.

- [ ] Inspect filter editing, column configuration, shortlist/CSV upload, player assignment, staffing configuration, Club DNA, and team-management dialogs.
- [x] Apply shared Modal title, field, footer-action, and radius treatments without rebuilding interaction logic.
- [ ] Complete the dialog-specific section/layout audit alongside the remaining feature refinements.
- [x] Verify shared filter/save-label wrapping and expanding mutation error/recovery copy, not only successful short-copy states.
- [ ] Complete long-content verification in the remaining feature-specific dialogs and disclosures.
- [ ] Check destructive confirmations for exact target names, clear consequences, safe cancellation, and obvious final confirmation.
- [ ] Check loading, no snapshot, no results, no shortlist, unavailable import, departed player, and recovery-stopped states.
- [x] Preserve distinct disabled controls and unmistakable selected state in the revised shared primitives/navigation.
- [x] Preserve phase-specific mutation labels and stable button widths during pending states.
- [x] Preserve reduced-motion behavior and add no decorative entrance or hover animations.
- [ ] Keep scroll cues visible in constrained panels, category strips, and tables.

## 14. Accessibility and rendering checks

- [x] Reverify shared-framing keyboard behavior through retained shell, table, tab, pitch, disclosure, dialog, and recovery tests.
- [x] Keep visible focus indicators, appropriate accessible names, semantic headers, and focus restoration.
- [x] Preserve redundant encoding for scores, phases, selection, warnings, and destructive state.
- [x] Recalculate shared text, control-boundary, focus, and score contrast, including raised/overlay pairings and composited active/score fills.
- [x] Test long mixed-script save names and applied filter labels with the bundled Archivo/Plex fallback stack.
- [ ] Extend long-name/script checks to real profile and feature-specific datasets in step 5.
- [x] Verify revised shared header/toolbar/control labels are not clipped at the core viewports.
- [ ] Complete long-role and feature-specific fixed-height label checks in the constrained workspaces.
- [ ] Inspect real portraits and club graphics as well as fallbacks before changing image-stage proportions.
- [ ] Check the native Windows app separately for font rendering, scaling, scrollbars, focus, and available content height.
- [x] Compare all 19 canonical populated routes at 1280×800 and 1600×900 before and after shared framing.
- [ ] Extend visual inspection to 1920×1080, the 2100px tactic boundary, and 3440×1440 with the relevant feature-composition work.

## 15. Reconcile DESIGN.md and implementation

**Documentation reconciliation: current for steps 1–2.** DESIGN.md reflects the implemented Signal identity, shared framing, numeric roles, and corrected contrast tokens.
Feature-specific composition and native verification remain explicitly pending.

- [x] Replace old gold hexadecimal examples and stale contrast ratios with calculations from the current tokens.
- [x] Correct body-family frontmatter to Archivo, with IBM Plex Sans retained as the bundled fallback.
- [x] Remove the duplicated Scale principle paragraph.
- [x] Resolve numeric typography with the implemented raw-value sans/score mono split.
- [x] Correct navigation icons and geometry to the implemented compact 16px-icon band.
- [x] Align the no-nested-panels principle with concrete rules for tables, interactive assignment cells, independent scroll regions, and overlays.
- [x] Verify attribute-fit section descriptions against source and tests. Overview does contain the 36 outfield attributes at the tested desktop size, so retain that statement rather than incorrectly changing it to Attributes-only.
- [x] Recalculate decorative-border and elevation ratios and verify documented shared shape/icon geometry against implementation.
- [x] Measure the Search table-owned viewport separately from its panel/toolbar and data-row height; record idle filter/feedback state and bounded results against the 70% target without claiming universal compliance.
- [x] Keep unfinished proposals out of current-state DESIGN.md and distinguish brand-kit presentation from desktop UI rules.

### Remaining implementation and documentation work

- [x] Correct runtime muted, low-tier score/error, and outline pairings; verify raised/overlay and actual composited states without weakening accessibility targets.
- [x] Record the six shared rules and reconcile their verified rollout with current tokens and behavior in DESIGN.md.
- [x] Reconcile current shared navigation emphasis, radius, header, toolbar, feedback, numeric, and viewport descriptions.
- [ ] Reconcile feature-specific composition/emphasis descriptions when later steps implement them.

## Recommended implementation sequence

1. **Settle the shared rules — Done:** navigation treatment, heading hierarchy, radius roles, dataset toolbar, feedback footprint, and numeric typography; [DESIGN.md](DESIGN.md#shared-framing-rules) owns the rules.
2. **Implement the shared framing — Done:** shell, headers, table panels, and common primitives implemented; all 19 canonical populated routes compared before/after at both core sizes, with focused interaction checks and full validation.
3. **Repair the constrained workspaces:** Moneyball Profile and smaller-desktop Tactic composition before secondary polish.
4. **Apply feature refinements:** profile summaries, Staff, Squad, Planner, Academy, and Settings.
5. **Inspect interaction states and native rendering:** dialogs, errors, progress, keyboard flow, long content, graphics, and Windows scaling.
6. **Maintain documentation and validate:** initial DESIGN.md reconciliation is complete; update it alongside subsequent implementation that makes each new rule true.

Do not convert this sequence into commits or PR boundaries without an accepted delivery plan.

## Acceptance criteria

- Analysis screens expose more useful workspace by reducing framing, not by shrinking readable data.
- Shared destinations, workspace headers, toolbars, controls, and surfaces use consistent visual rules.
- The minimum supported window keeps the primary task usable: Search shows meaningful rows, Moneyball exposes metric evidence, and Tactic does not subordinate the pitch to an oversized list.
- Existing table row heights, bounded column behavior, virtual scrolling, identity context, and persisted layout preferences remain correct.
- Primary actions, passive metrics, selected state, and destructive actions have distinct and restrained emphasis.
- Long names, role labels, scripts, errors, and real graphics do not break the revised composition.
- No data is hidden, fabricated, relabelled, or rescaled merely to simplify the interface.
- Keyboard operation, focus visibility, contrast, mutation safeguards, and snapshot provenance remain intact.
- DESIGN.md agrees with implemented tokens and component behavior.

## Validation

For implementation, use the stable repository commands:

```sh
./scripts/dev inspect-ui <route> [width] [height]
./scripts/dev inspect-ui all
./scripts/dev test [target...]
./scripts/dev smoke
./scripts/dev check
```

Use the full inspection set for shared-shell or cross-page changes; use focused routes for isolated work.
Open the actual captures, inspect the relevant overflow and interactive states, and remove disposable evidence afterward.
Run affected behavior tests and the repository gate separately from visual inspection.
Do not report Chromium-stub evidence as native application proof or assume an initial-page capture validates hidden content.
