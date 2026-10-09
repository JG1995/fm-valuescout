---
name: FM ValueScout
colors:
    # Foundation — background & surface elevation layers (Signal Night Slate)
    background: "oklch(0.194 0.019 255.7)"
    on-background: "oklch(0.964 0.003 264.5)"
    surface-dim: "oklch(0.169 0.011 268)"
    surface: "oklch(0.226 0.013 264.3)"
    surface-bright: "oklch(0.375 0.013 267.2)"
    surface-container-lowest: "oklch(0.169 0.011 268)"
    surface-container-low: "oklch(0.194 0.019 255.7)"
    surface-container: "oklch(0.226 0.013 264.3)"
    surface-container-high: "oklch(0.289 0.012 264.4)"
    surface-container-highest: "oklch(0.375 0.013 267.2)"
    on-surface: "oklch(0.964 0.003 264.5)"
    on-surface-variant: "oklch(0.76 0.007 255.5)"
    inverse-surface: "oklch(0.964 0.003 264.5)"
    inverse-on-surface: "oklch(0.226 0.013 264.3)"
    # Borders & outlines
    outline: "oklch(0.66 0.01 264.5)"
    outline-variant: "oklch(0.375 0.013 267.2)"
    surface-tint: "oklch(0.766 0.157 157.1)"
    # Primary — main interactive and brand colour (Signal Green)
    primary: "oklch(0.766 0.157 157.1)"
    on-primary: "oklch(0.169 0.011 268)"
    primary-container: "oklch(0.456 0.106 157)"
    on-primary-container: "oklch(0.909 0.098 156.9)"
    inverse-primary: "oklch(0.523 0.121 157.4)"
    # Filled-button states — the Button spec's 8% mixes, resolved in oklab
    primary-hover: "oklch(0.78 0.144 157.1)"
    primary-active: "oklch(0.718 0.144 157.5)"
    # Semantic — status indicators
    success: "oklch(0.746 0.119 179)"
    on-success: "oklch(0.16 0.03 179)"
    success-container: "oklch(0.34 0.085 179)"
    on-success-container: "oklch(0.92 0.07 179)"
    warning: "oklch(0.76 0.165 55)"
    on-warning: "oklch(0.16 0.03 55)"
    warning-container: "oklch(0.34 0.085 55)"
    on-warning-container: "oklch(0.92 0.04 55)"
    error: "oklch(0.77 0.13 18)"
    on-error: "oklch(0.16 0.03 18)"
    error-container: "oklch(0.34 0.115 18)"
    on-error-container: "oklch(0.92 0.035 18)"
    info: "oklch(0.72 0.11 245)"
    on-info: "oklch(0.16 0.02 245)"
    info-container: "oklch(0.34 0.08 245)"
    on-info-container: "oklch(0.92 0.035 245)"
    # FM-style data ramp — red, grey, amber, green
    score-1: "oklch(0.77 0.13 18)"
    score-2: "oklch(0.75 0.008 264)"
    score-3: "oklch(0.8 0.145 75)"
    score-4: "oklch(0.746 0.119 179)"
    # Chart series — subject, two comparisons, one reference line
    chart-1: "oklch(0.766 0.157 157.1)"
    chart-2: "oklch(0.72 0.11 245)"
    chart-3: "oklch(0.68 0.18 340)"
    chart-4: "oklch(0.62 0.01 264)"
typography:
    headline-lg:
        {
            fontFamily: "Archivo Variable",
            fontSize: 28px,
            fontWeight: "800",
            lineHeight: "1.25",
            letterSpacing: -0.01em,
        }
    headline-md:
        {
            fontFamily: "Archivo Variable",
            fontSize: 22px,
            fontWeight: "700",
            lineHeight: "1.3",
        }
    headline-sm:
        {
            fontFamily: "Archivo Variable",
            fontSize: 18px,
            fontWeight: "600",
            lineHeight: "1.35",
        }
    body-lg:
        {
            fontFamily: "Archivo Variable",
            fontSize: 16px,
            fontWeight: "400",
            lineHeight: "1.6",
        }
    body-md:
        {
            fontFamily: "Archivo Variable",
            fontSize: 14px,
            fontWeight: "400",
            lineHeight: "1.5",
        }
    body-sm:
        {
            fontFamily: "Archivo Variable",
            fontSize: 13px,
            fontWeight: "400",
            lineHeight: "1.4",
        }
    label-lg:
        {
            fontFamily: "Archivo Variable",
            fontSize: 14px,
            fontWeight: "700",
            lineHeight: "1.2",
            letterSpacing: 0.02em,
        }
    label-md:
        {
            fontFamily: "Archivo Variable",
            fontSize: 12px,
            fontWeight: "700",
            lineHeight: "1.2",
            letterSpacing: 0.05em,
        }
    label-sm:
        {
            fontFamily: "Archivo Variable",
            fontSize: 11px,
            fontWeight: "600",
            lineHeight: "1.2",
            letterSpacing: 0.08em,
        }
    # Monospace roles for numeric/tabular data
    mono-xl:
        {
            fontFamily: "JetBrains Mono Variable",
            fontSize: 36px,
            fontWeight: "600",
            lineHeight: "1.2",
        }
    mono-lg:
        {
            fontFamily: "JetBrains Mono Variable",
            fontSize: 24px,
            fontWeight: "600",
            lineHeight: "1.2",
        }
    mono-md:
        {
            fontFamily: "JetBrains Mono Variable",
            fontSize: 14px,
            fontWeight: "500",
            lineHeight: "1.4",
        }
    mono-sm:
        {
            fontFamily: "JetBrains Mono Variable",
            fontSize: 12px,
            fontWeight: "500",
            lineHeight: "1.4",
        }
rounded:
    none: 0
    sm: 0.25rem
    DEFAULT: 0.375rem
    md: 0.375rem
    lg: 0.5rem
    xl: 0.75rem
    full: 9999px
spacing:
    unit: 4px
    # Product-specific named dimensions
    table-row-height: 36px
    table-row-height-two-line: 40px
    table-header-height: 32px
    header-height: 56px
    inspector-width: 320px
    gutter: 16px
    stack-xs: 4px
    stack-sm: 8px
    stack-md: 16px
    stack-lg: 24px
    stack-xl: 32px
    content-max-width: none
    min-window-width: 1280px
    min-window-height: 800px
---

# Design System: FM ValueScout

> **Authority:** This document owns the visual language, design tokens, and UI decisions. It does not own product purpose ([CONCEPT.md](./CONCEPT.md)) or implemented system shape ([ARCHITECTURE.md](./ARCHITECTURE.md)).

> **Status:** Tokens, shared primitives (`src/components/ui/`, including **Modal** and **ScoreBadge**), the app shell, Settings management, My Club managed-club tools, format-specific CSV enrichment, Dashboard, Player Search with General and Moneyball views plus integrated shortlist upload and filtering, Staff Search with integrated shortlist and assignment controls, Staff and Player profiles, and the three-workspace My Club surface are implemented. `src/styles/global.css` bridges the full token set into Tailwind `@theme` ([ADR-0007](./decisions/0007-tailwind-css-v4.md)).

The Signal palette, bundled typefaces, and launcher icon are integrated; that does not mean every design principle or accessibility target below is satisfied.
[REDESIGN.md](./REDESIGN.md) tracks the redesign inventory and completed decisions.
The [shared framing rules](#shared-framing-rules) are implemented: compact navigation, purpose-based radii, workspace headings, table framing, local feedback, and raw-value typography.
The frontmatter and component descriptions below match those runtime changes, including corrected shared contrast tokens.
Moneyball Profile proportions, smaller-desktop Tactic composition, and feature-specific emphasis remain redesign work; native Windows verification belongs to step 5.

## Shared framing rules

**Status: implemented and verified in Chromium.** Step 1 settled these six rules; step 2 applied them to the shell, common primitives, and their consumers.
See [implementation evidence and remaining scope](#implementation-evidence-and-remaining-scope) for proof and limits.

### Navigation treatment

- Keep the **56px utility bar** and all its global controls. Use a **48px destination band**, including its bottom border, below it; no second caption row.
- Keep all ten direct destinations, existing group membership, labels, URLs, history behavior, and route-state transitions. Use **16px icons beside sentence-case 12px labels**, with 6px icon/label gaps and 36px-high links. Do not replace direct destinations with menus.
- Keep fine group separators. Put the Players, Staff, and Club captions inline before their links; use label-sm (11px, weight 600) in readable muted text without reduced opacity. Home and Settings need no duplicate caption.
- Active links use a **10% primary tint mixed over the panel surface in OKLab**, on-surface text, a primary icon, a reinforced label, and a persistent **2px bottom indicator**. Reserve strong solid green fills for primary actions. Keep the indicator present without hover; focus is a separate visible 2px primary outline, contained within the link so it cannot be clipped by the band.
- Retain exactly one `aria-current="page"` for direct destinations. Profiles keep only their Players or Staff group at `aria-current="location"`; unknown routes keep neither. Group context remains visibly reinforced without pretending a child destination is selected.
- The utility bar uses the tracked, finished outline symbol at 36px with a green off-center blip and accessible product name. The saturated launcher tile remains an OS icon, not utility chrome.

### Heading hierarchy and spacing

- Use one workspace header: destination title or equivalent visible destination context, optional secondary facts, and feature-owned actions. Maintain **one semantic h1** per workspace. If the active destination already supplies sufficient visible context, as in Staff Search, the h1 may be visually hidden; do not add a title-only row merely for uniformity.
- Visible workspace titles use **headline-md: 22px, weight 700**. A profile's player or staff name may retain **headline-lg: 28px, weight 800** as the persistent identity heading. Modal titles remain headline-md.
- Meaningful panel sections use **h2, headline-sm: 18px, weight 600**. Nested sections use **h3, 14px sans, weight 600**. Keep heading levels in order; choose semantics independently from visual size.
- Remove generic Results headings and repeated Staff, Graphics, or Graduates headings when destination/section context already names that content. Keep meaningful headings, accessible table captions, and dialog labels. An action row must survive removal of its generic panel title.
- Use **8px between related controls**, **16px between workspace regions**, and **16px page padding**. Within a dataset host, use one feature-action row, its local feedback region when needed, then the dataset toolbar and table. Do not accumulate header margins, panel padding, and an extra generic-title gap for the same separation.
- Keep controls at 32–36px and retain all table/profile row heights. Use tracked capitals only for short structural/group labels; headings, ordinary controls, names, and explanatory sentences stay sentence case.

### Radius roles

Shared radius tokens and consumers use purpose-based rectangles rather than pill-first controls.

| Purpose | Radius |
| --- | --- |
| Panels, cards, table hosts, independent scrolling regions | **8px** |
| Ordinary text/icon buttons, fields, search inputs, navigation links, segmented-control group | **6px** |
| Segmented items, checkboxes, compact rectangular value tags | **4px** |
| Dialogs and floating overlays | **12px** |
| Filter tags, status chips, genuine circular scores and position pickers | **Full** |
| Table rows | **None** |

- Use the central scale: sm 4px, DEFAULT/md 6px, lg 8px, xl 12px, with none/full unchanged. Do not scatter local pixel overrides.
- Apply the ordinary-button role across primary, secondary, ghost, and destructive variants without changing their action meaning, pending-width reservation, or 32/36px height.
- Focus geometry follows the component shape. Keep at least 44px position-selection targets and meaningful circles; a tactic marker with a wrapped role label need not become circular. Flush tables share their host's outside corners rather than adding a second rounded card.
- Geometry does not waive contrast requirements. The shared muted, outline, and low-tier score tokens now pass their overlay pairings; evaluate new component states separately.

### Dataset toolbar

- Retain the existing ownership boundary: `TableToolbar` accepts only summary, filter chips, Clear all/Edit filters, Columns, and caller-owned dataset toggles. Upload, Add Tactic, boosts, configuration, and optimization stay in feature/workspace action rows. Do not add a generic page-action slot or feature imports.
- First row: **count and committed sort context on the left; filter state/Edit filters, Columns, and dataset toggles on the right**, in that order. Squad and My Staff omit inapplicable filter controls instead of reserving empty slots. Use one shared alignment pattern for Search, Moneyball, Squad, Staff Search, and My Staff.
- Active rules occupy a full-width second row below those controls, with the AND/OR context, removable chips, and Clear all. Keep all rules reachable; wrap rather than collapse them into an unexplained count. Use neutral, quiet chips with explicit removal names.
- Use **8px vertical padding, 16px horizontal padding, and 8px control/chip gaps**. The toolbar has no independent rounded card; one hairline separates it from the recessed table headers.
- With no rules, use concise **No filters** copy beside Edit filters; retain OR context when that setting is active. Setup/no-shortlist/no-import states retain their specific next-step guidance, rather than losing instructions indiscriminately.
- Let a long sort summary wrap in its own flexible slot and controls wrap as a group when required. Never clip controls or focus rings, hide filter/remove actions, or introduce page-level horizontal overflow. Staff's conditional Preferred Job and Only unemployed controls belong with dataset toggles and may occupy another wrapped control row.
- Keep committed versus requested sort feedback, current rows during replacement, table captions, sticky headers/identity, saved widths, and virtual paging unchanged. Typography or framing must not change query ownership or displayed facts.

### Feedback footprint

- **Local mutation feedback:** retain one mounted, programmatically focusable region below the feature action row, outside the dataset toolbar. Reserve a **24px minimum**, with a 4px preceding gap, instead of a permanent 64px empty slot. Staff's existing 24px region is the starting analogue, not a region to delete.
- Keep the action header's position and button widths stable. One-line pending/success copy fits the slot; longer messages expand below it, reducing the data workspace rather than moving actions or causing document overflow. Do not promise that the table's top edge never moves.
- Keep the latest truthful outcome, processed/updated/skipped/failed counts, phase names, and error/recovery instructions. Do not replace required inline outcomes with transient toasts or silently dismiss them.
- Keep the essential error/recovery summary and next action visible outside any clipped detail area. Longer supplementary outcomes may use a **96px-max detail scroller** with a visible scrollbar, accessible name, and keyboard access. Never impose that cap on the only copy of a safety instruction.
- Retain live-region behavior without duplicate nested announcements, recovery focus destinations, Modal focus restoration, context-bound suppression, recovery locks, and draft retention. Squad progress stays in its confirmation Modal; import feedback stays in its import Modal. Expanded Staff recommendations remain a distinct data region, not feedback squeezed into the status slot.
- **Global Load Data feedback:** retain the mounted polite region with **zero idle footprint**, below the unchanged utility controls. Pending/outcome copy uses a compact 32–48px row when it fits, but wraps or expands for truthful longer content. Keep the native progress element, detailed outcome access, dismissal, and save/snapshot guards; do not force it into the local reserved-slot rule.
- Setup/readiness guidance is not empty status space. Keep required next steps and `aria-describedby` explanations visible. Implement the smaller reservation without weakening the existing Staff optimizer and Squad recovery contracts.

### Numeric typography

- **Raw table values:** use **Archivo 13px, weight 500, tabular figures, right alignment** for CA/PA, counts, money, heights, and raw performance metrics. Numeric age columns use the same alignment and tabular treatment. Names, labels, and prose remain sans at their existing text sizes.
- **Scores and compact scored/attribute units:** retain **JetBrains Mono 12px, weight 500** for table scores, percentile badges, and Current → Potential attribute pairs. Standalone score circles retain their existing appropriate mono role and centered alignment.
- **Summary metrics:** retain JetBrains Mono at the applicable mono-md/lg/xl scale; prominence follows the metric's role, not a green fill or automatic display weight.
- **Literal strings:** retain JetBrains Mono for paths, versions, identifiers, and diagnostics. Ordinary dates and facts in prose need not become display metrics; keep tabular figures where alignment matters.
- In mixed raw-value/percentile cells, apply the raw sans and score mono roles separately; do not inherit one family over both. Keep decimal precision, units, missing `—`, concealed information, score tiers, accessible score names, sorting, and persisted column widths unchanged.
- Shared cells and relevant callers implement this split without changing the mono token or ScoreBadge globally. Do not shrink columns on the assumption that sans is always narrower.

### Implementation evidence and remaining scope

Step 2 compared the same **19 canonical populated routes at 1280×800 and 1600×900**, before at `7169c53` and after the shared-framing implementation.
The 56px utility bar and 48px destination band retain all ten 36px-high links, readable inline group context, a persistent active indicator, and inset keyboard focus without page overflow.
Search's first data row now begins around y=284px rather than y=362px; Squad's begins around y=380px rather than y=444px.
Nominal 40px two-line rows remain unchanged; measured outer row height is 41px including the hairline.
With no applied filters or Load Data feedback, General Search's table-owned viewport is **68.63% at 1280×800 and 72.21% at 1600×900**; Moneyball Search is **67.90% and 71.56%**.
These measurements include sticky table headers, not the dataset toolbar; General Search exposes about 500px/600px of data-row height after its 64px headers.
Applied chips and truthful feedback consume additional height rather than hiding controls or shrinking rows.

Runtime checks covered normal and shortlist Staff controls, eight long mixed-script filters at both sizes, removal/Clear all, filter-modal focus return, long save names with a separate visible snapshot date, recovery focus/locks, and expanding error copy.
The full unit, smoke, and repository gates passed; existing routing, persistence, virtualization, concealment, and mutation tests remain in place.
The selected active tint gives 13.06:1 text and 7.36:1 icon/indicator contrast; the updated muted group captions give 7.95:1 against the panel.
Composited navigation, raw-cell, and scored-unit foreground samples passed 4.5:1; token-pair checks also cover raised/overlay text, control outlines, and filled-button states.
This is bounded evidence, not a blanket accessibility claim or proof of native FM operations.

Earlier typography specimens showed that sans is not universally narrower: `200` occupied 21px in both roles, `12.34` 35px mono versus 32px sans, and `€12.5M` 42px mono versus 44px sans.
Keep persisted widths and choose fonts by information role, not an assumed width saving.
Screenshots are disposable evidence, not runtime assets.
Moneyball Profile height/pitch proportions, smaller-desktop Tactic reflow, profile-summary regrouping, Academy metric/list treatment, Planner refinements, and Settings reading widths remain later work.
Native Windows font, scaling, scrollbar, and focus verification remains pending in step 5; Chromium with synthetic Tauri IPC does not prove it.

## Brand & Style

**The central concept is a blip on the night pitch: a dark instrument field, and exactly one lit green mark.**

FM ValueScout is an instrument, not a destination. The user already has Football Manager open on the same machine, and probably a second monitor. They alt-tab in with a question — *who fills this role best right now?* — and they want the answer in the first second of looking. The app is a quiet dark surface holding a lot of numbers, with a controlled green accent for location, selection, and primary actions. The mark and wordmark are the Signal identity: a top-down pitch — ring, halfway line, one blip off the line — beside **VALUESCOUT** in wide tracked Archivo capitals. The blip is never on the centerline (the player is found *off* it), and the pitch is never filled in (the mark is a line, not a surface).

The intended qualities are **quiet, precise, and alert**. Cool near-black surfaces, hairline separations, dense rows, and one green accent vocabulary support sustained scanning rather than a friendly pastel dashboard. The tension to hold is **dense but not cramped**: shared analysis tables use 40px two-line rows, while the 36px single-line token remains available for other tables. Text and raw numeric cells use 13px sans; compact scored units use 12px monospace. Spacing and hairlines separate the data without turning every value into a card.

The brand kit demonstrates the identity, not a desktop layout specification. Its light presentation pages, 64px display headings, and hosted-font examples do not override the app's dark-only, compact, offline rules. Keep logo proportions, clear space, and the optically adjusted small symbol; never stretch, recolour arbitrarily, or add an outline to the finished wordmark. Use an appropriate light-on-dark variant on dark ground and a one-colour dark variant on light or green ground. Signal Green alone does not provide 3:1 on a light ground. The utility bar displays the finished outline symbol at 36px; the launcher tile is retained separately for OS app-icon contexts.

Hard stances:

- **Dark only.** There is no light theme and no `prefers-color-scheme` branch. FM runs full-screen and dark; a bright companion window beside it is hostile.
- **One lit green.** This is the identity motif and accent discipline, not a limit of one green element per screen. Signal Green is the brand accent for active nav, primary actions, focus, selection, and the identity mark. It also defines the subject chart-series token. Numeric data uses the score ramp; semantic status, informational annotation, and IP/OOP phase graphics retain their separate hues. Success and the top score tier use hue 179 (teal-green), distinct from the hue-157 brand green. Passive metrics should not acquire brand colour merely as decoration; the existing Academy icon treatment remains a redesign item.
- **Desktop only.** Minimum window 1280×800, designed at 1600×900. No mobile or narrow breakpoints. ([CONCEPT.md](./CONCEPT.md) excludes mobile and web clients.)
- **Offline by construction.** No network requests for fonts, icons, images, or analytics. Bundled application assets and exact-UID user-selected local FM graphics are permitted; the app never downloads, copies, or manages graphics packs. This follows the offline-first principle in [CONCEPT.md](./CONCEPT.md), and it is a design constraint, not only an infrastructure one.
- **Text-first identity.** Club and player names remain the identity facts. Optional local FM graphics can enhance Player Profile, Search, Squad, and My Club when an exact UID mapping exists; they never replace readable text. Player portraits are the only person images, and club logos are decorative beside club text. Fixed portrait and logo slots retain initials or shield fallbacks while images are pending, missing, invalid, or unavailable. Player tables may render a nationality string as a bundled SVG flag only after an explicit FM-name mapping; unknown values stay visible as text, and the app never guesses a flag. This remains a data constraint, not a style preference.
- **No decorative imagery.** No hero art, no illustration, no stock photography.

## Colors

The palette is one saturated chrome accent on a cool near-neutral base, plus four semantic status colours and one multi-hue data ramp. Elevation is carried by **tonal layering plus hairline borders**, not by shadows. Dark surfaces swallow shadows, and the app stacks a lot of panels; a tonal step reads reliably at any brightness setting where a drop shadow does not. Shadows appear at one level only — floating overlays.

The canvas anchors the palette in **Night Slate `#0F151D`**, represented by `background` at `oklch(0.194 0.019 255.7)`. The elevation surfaces run from `#0D0F14` through `#191C22` and `#282B31` to `#3E4148`, with hue ~264–268 and chroma 0.011–0.013. Text is `#F2F3F5` or muted `#AEB1B5`. These cool near-neutrals support **Signal Green `#45D08A`**, rather than competing with it.

**Primary — Signal Green (hue 157):** `primary` marks **chrome state**: the active nav item, the primary button, the focus ring, the selected row indicator, checked controls, and the subject series in a chart. It answers "where am I, and what is the main action here?" Green also carries the product idea — the blip in the mark is a player found, and a lit green mark on a dark field is what *found* looks like.

**Steel (hue 245):** `info` is the single cool counterpoint. It carries neutral factual annotation that is neither good nor bad: transfer-status tags, "U-21" style qualifiers, informational banners, and the first comparison series in a chart. There is no separate `secondary` token; steel does that work.

**Score ramp:** `score-1` through `score-4` colour role fit and player-profile attributes with Football Manager's familiar red, grey, amber, and green progression. The number and tier label remain the facts; colour makes the four broad bands faster to scan.

| Tier      | Score  | Label     | oklch                    | Meaning                         |
| --------- | ------ | --------- | ------------------------ | ------------------------------- |
| `score-1` | 0–40   | Weak      | `oklch(0.77 0.13 18)`    | Does not suit this role         |
| `score-2` | 41–60  | Average   | `oklch(0.75 0.008 264)`  | Emergency or fringe cover       |
| `score-3` | 61–80  | Good      | `oklch(0.8 0.145 75)`    | Viable squad or starting option |
| `score-4` | 81–100 | Excellent | `oklch(0.746 0.119 179)` | High-confidence role fit        |

Player- and staff-profile attributes use the same colours with FM-scale bands: 1–5 Weak, 6–10 Average, 11–15 Good, and 16–20 Excellent. The raw value remains visible, and the colour never replaces it. **`primary` never appears inside a data cell, and the score ramp never appears on chrome.**

**Semantic Colours:** Four fixed roles for status indicators.

| Semantic  | oklch                    | Role                                                                        |
| --------- | ------------------------ | --------------------------------------------------------------------------- |
| `success` | `oklch(0.746 0.119 179)` | Load Data completed, bridge plugin installed and current, snapshot is fresh  |
| `warning` | `oklch(0.76 0.165 55)`   | Snapshot truncated at the scan cap, snapshot is stale, plugin update pending |
| `error`   | `oklch(0.77 0.13 18)`    | Scan failed, ingest failed, FM not running, destructive confirmation         |
| `info`    | `oklch(0.72 0.11 245)`   | Neutral annotation and explanatory banners                                   |

Success and `score-4` share one hue-179 teal-green: *good* reads as green, but it is deliberately not the brand green (hue 157). Signal Green identifies brand/chrome emphasis or the subject chart series; teal-green identifies a successful status or an excellent score. Warning sits at hue 55 (orange) rather than amber (hue 75) so it never reads as the score ramp's good tier. The data ramp uses separate token names even where its red, grey, and green reuse established system colours; component code still states whether colour carries status or a score band.

`primary-hover` and `primary-active` are the Button spec's hover and active mixes resolved once, in oklab, rather than recomputed per component. Both stay in sRGB gamut and hold an `on-primary` label above 8:1. Unfilled variants have no mix — they press to `surface-container-highest`, one tonal step above their hover fill.

The template's `tertiary` role and the fixed tonal pairs (`primary-fixed`, `secondary-fixed`, `tertiary-fixed`, and their `on-*` partners) are removed on purpose. Nothing in this design needs a third accent or a tint that stays constant across themes, and there is only one theme. Do not restore them without a component that requires them.

Borders come in two roles with different rules:

- `outline` bounds **interactive** components — inputs, selects, secondary buttons, checkboxes. It must clear 3:1 against whatever surface it sits on, because the border is the only thing that shows the control exists.
- `outline-variant` is the **decorative** hairline for table row separators, card edges, and section rules. It is subtle (1.67:1 against `surface-container`) so it contains the data without competing with it. The 3:1 boundary requirement does not apply to purely decorative rules; do not use this token as the sole meaningful control boundary. Its value equals `surface-container-highest`, so it provides no contrast against an overlay of that colour.

### Accessibility of Colour

**Colour is never the sole indicator of meaning.**

- **Score badges** always render the number. The tier colour is redundant encoding that speeds scanning; the number is the fact. A tier label is available in the badge `title` and in the accessible name.
- **Status chips and banners** pair colour with an icon and a text label — never a bare coloured dot.
- **Active nav item** pairs a 10% primary tint, a persistent 2px primary indicator, a primary icon, a reinforced label, and `aria-current="page"`; the same Lucide component uses `strokeWidth` 2 when active and 1.5 when inactive.
- **Selected table row** pairs the tint with a 2px primary left indicator and `aria-selected`.
- **Chart series** differ by colour *and* stroke pattern — solid, dashed, dotted — plus a direct label or legend entry. A radar chart with three overlaid players is unreadable by colour alone at any palette.
- **Trend arrows** carry direction as shape (up, down, flat), with colour as reinforcement.

**Contrast targets:** WCAG 2.2 AA: at least 4.5:1 for normal text and 3:1 for meaningful control boundaries and graphical objects. Normal body text clears AAA (7:1) on the surface stack, but secondary text does not. The shared token pairings below pass their applicable targets; this is not a blanket compliance claim for every rendered component.

Ratios below use the WCAG relative-luminance formula after converting the frontmatter's OKLCH tokens to sRGB. Hex values are rounded display equivalents; ratios use unrounded channels and are shown to two decimals. Alpha fills, disabled opacity, and composited states require separate calculations.

| Role | Foreground | Background | Ratio / result |
| --- | --- | --- | --- |
| Body text | `on-surface` (`#f2f3f5`) | `background` (`#0f151d`) | 16.50:1 (AAA) |
| Body text | `on-surface` (`#f2f3f5`) | `surface-container` (`#191c22`) | 15.37:1 (AAA) |
| Body text on overlay | `on-surface` (`#f2f3f5`) | `surface-container-highest` (`#3e4148`) | 9.21:1 (AAA) |
| Secondary text | `on-surface-variant` (`#aeb1b5`) | `surface-container` (`#191c22`) | 7.95:1 (AAA) |
| Secondary text on raised surface | `on-surface-variant` (`#aeb1b5`) | `surface-container-high` (`#282b31`) | 6.60:1 (AA) |
| Secondary text on overlay | `on-surface-variant` (`#aeb1b5`) | `surface-container-highest` (`#3e4148`) | 4.76:1 (AA) |
| Accent text and icons | `primary` (`#45d08a`) | `surface-container` (`#191c22`) | 8.66:1 (AAA) |
| Primary button label | `on-primary` (`#0d0f14`) | `primary` (`#45d08a`) | 9.72:1 (AAA) |
| Primary button label on hover | `on-primary` (`#0d0f14`) | `primary-hover` (`#5ad392`) | 10.15:1 (AAA) |
| Primary button label on press | `on-primary` (`#0d0f14`) | `primary-active` (`#41be80`) | 8.15:1 (AAA) |
| Destructive button label | `on-error` (`#180808`) | `error` (`#fc9095`) | 8.90:1 (AAA) |
| Score tier 1 (weakest) | `score-1` (`#fc9095`) | `surface-container` (`#191c22`) | 7.79:1 (AAA) |
| Score tier 1 on hover | `score-1` (`#fc9095`) | `surface-container-high` (`#282b31`) | 6.46:1 (AA) |
| Score tier 1 on overlay | `score-1` (`#fc9095`) | `surface-container-highest` (`#3e4148`) | 4.67:1 (AA) |
| Score tier 2 | `score-2` (`#abaeb3`) | `surface-container` (`#191c22`) | 7.67:1 (AAA) |
| Score tier 2 on overlay | `score-2` (`#abaeb3`) | `surface-container-highest` (`#3e4148`) | 4.60:1 (AA) |
| Score tier 3 | `score-3` (`#f4af41`) | `surface-container` (`#191c22`) | 8.96:1 (AAA) |
| Score tier 4 (strongest) | `score-4` (`#3fc5ae`) | `surface-container` (`#191c22`) | 7.97:1 (AAA) |
| Error text | `error` (`#fc9095`) | `surface-container` (`#191c22`) | 7.79:1 (AAA) |
| Warning text | `warning` (`#ff9138`) | `surface-container` (`#191c22`) | 7.58:1 (AAA) |
| Success text | `success` (`#3fc5ae`) | `surface-container` (`#191c22`) | 7.97:1 (AAA) |
| Banner text | `on-error-container` (`#fbdcdc`) | `error-container` (`#661420`) | 9.68:1 (AAA) |
| Control border against panel | `outline` (`#8f9299`) | `surface-container` (`#191c22`) | 5.48:1 (3:1 UI) |
| Control border against field fill | `outline` (`#8f9299`) | `surface-container-high` (`#282b31`) | 4.55:1 (3:1 UI) |
| Control border on overlay | `outline` (`#8f9299`) | `surface-container-highest` (`#3e4148`) | 3.29:1 (3:1 UI) |

**Correction and scope:** shared secondary text, tier-1/error text, tier-2 text, and meaningful outlines now pass their raised/overlay pairings without lowering the targets. Check both the inner fill and surrounding ground when evaluating a control. The overlay's decorative `outline-variant` edge still has 1:1 contrast with its fill and must not be the only meaningful boundary. Do not round a failing value up to the threshold.

All score tiers pass normal-text AA on the default, raised, and overlay surfaces. When a token, opacity, or pairing changes, calculate the actual result before use. The table is token-pair evidence, not a rendered audit of every component.

## Typography

**Archivo** leads the brand, readable UI, and raw numeric table values. **JetBrains Mono** carries scored units, summary metrics, and literal strings. **IBM Plex Sans** is the bundled fallback for glyphs Archivo does not cover, especially Cyrillic and Greek.

- **Archivo (variable):** UI, headings, labels, player names, and prose. Headline roles use 800/700/600; body roles use 400; label roles use 700/700/600. The identity wordmark uses uppercase ExtraBold 800 with 0.06em tracking. UI headings keep their smaller product sizes and token tracking rather than copying that display treatment.
- **IBM Plex Sans (weights 400/500/600):** the second family in the sans stack. Archivo's bundled subsets cover Latin, Latin Extended, and Vietnamese, but not Cyrillic or Greek. Plex provides those scripts without a network request; the browser selects fallback glyphs through the font stack and each face's `unicode-range`.
- **JetBrains Mono (variable):** scored analysis cells, percentiles, attribute pairs, and table ScoreBadges use `font-mono text-mono-sm` (12px, weight 500). Larger score badges, headline metrics, version strings, paths, and diagnostics use the appropriate mono role. Names and prose stay sans. The installed package includes Latin, Latin Extended, Cyrillic, Cyrillic Extended, Greek, and Vietnamese subsets.

**Scale principle:** keep display emphasis limited. Shared text cells use `body-sm` (13px); raw numeric cells add sans weight 500 and tabular figures, while scored units use `mono-sm` (12px). Ordinary prose and controls use `body-md` (14px) or the applicable label role. Short structural labels can use tracked capitals; applying a `label-*` token does not itself uppercase the text. The 11px micro-label above a 13–14px value remains a compact alternative to extra boxes. The implemented raw-value sans/score mono split is documented under [numeric typography](#numeric-typography).

Numeric rules:

- Apply `font-variant-numeric: tabular-nums` to every numeric table column, score, and metric so digits align in a column and do not jitter when values update.
- Never set body copy in all-caps. Uppercase is for `label-*` roles only, at 11–14px, always with the letterspacing from the token.
- Use `text-wrap: pretty` on prose blocks. Truncate names in fixed-width cells with an ellipsis and a `title` attribute. Text must never wrap inside a table cell — the two-line table variant stacks two separate elements at a fixed row height, which is not the same thing as letting a value wrap.

**Loading:** self-host everything in the bundle via `@fontsource-variable/archivo`, `@fontsource-variable/jetbrains-mono`, and `@fontsource/ibm-plex-sans` (per-weight `400.css`, `500.css`, `600.css` entrypoints). No Google Fonts link, no CDN — the app must render identically with no network, per the offline stance above.

Bundle Latin, Latin Extended, Cyrillic, Cyrillic Extended, Greek, and Vietnamese support. Names such as `Magalhães`, `Håland`, `Şahin`, `Дзюба`, `Παυλίδης`, and `Phạm` are ordinary data. Archivo covers the Latin/Vietnamese range; Plex supplies the Cyrillic/Greek fallback. The variable families' `index.css` and the per-weight Plex entrypoints declare `unicode-range` per subset. Use those entrypoints rather than combining per-subset Plex files that omit range declarations. Test mixed-script names offline; font-stack declarations alone do not prove the rendering of every glyph.

Font stacks:

```css
--font-sans: "Archivo Variable", "IBM Plex Sans", system-ui, sans-serif;
--font-mono: "JetBrains Mono Variable", ui-monospace, monospace;
```

### Value & Number Formatting

The app is mostly formatted numbers, so formatting is a design decision, not a per-component choice. Implement these once in a shared formatter module and use it everywhere.

**Money** — euro prefix, no space, abbreviated by magnitude:

| Value          | Renders as | Rule                                       |
| -------------- | ---------- | ------------------------------------------ |
| 750            | `€750`     | Below 1,000: exact                         |
| 900,000        | `€900k`    | Below 1M: thousands, no decimals           |
| 12,500,000     | `€12.5M`   | 1M to 100M: millions, one decimal if not whole |
| 120,000,000    | `€120M`    | Above 100M: millions, no decimals          |
| Range          | `€12M – €18M` | En dash with spaces                     |

**Other values:**

- **Role and position scores:** integer 0–100, no unit, no percent sign. Shared table badges use `font-mono text-mono-sm`; card and hero variants use `mono-md` and `mono-lg`. All use tabular figures.
- **Linear position order:** list positions from the goalkeeper band toward the striker band and from the player's right to left within each band. Familiarity-ranked lists keep the strongest value first and use this pitch order for ties. Tactical XI rows use the IP position as their primary order. The normalized tactic canvas uses portrait attack-up geometry below 2100px and landscape attack-right geometry at or above 2100px; DOM and tab order follow the current visual order in either orientation.
- **FM attributes:** integer 1–20. **CA and PA:** integer 1–200. Both as raw integers — never rescaled to 0–100, because the user knows the FM scale.
- **Age:** integer. Where both are shown, birth date first and age in parentheses: `21/03/2001 (25)`.
- **Snapshot timestamps:** relative in the UI (`4 min ago`, `2 hours ago`, `yesterday`), absolute ISO-like in the `title` attribute (`2026-07-29 20:14 UTC`). Relative age is what tells the user whether to reload.
- **In-game dates:** use the canonical stored value in data-management views. Beside the active-save selector, a nonshrinking `<time>` formats the matching snapshot date as an English ordinal, such as `11th June 2027`. A long save name may truncate in the native select; its full name remains in `title`, and it cannot displace the date.
- **Percentages:** one decimal maximum, `%` suffix, no space: `62.5%`.
- **Missing values:** an em dash `—` in `on-surface-variant`. Never `null`, `N/A`, `0`, or an empty cell. Absent data and zero are different facts.
- **Truncated counts:** never show a total from a truncated scan without the cap. Render `1,247 players (scan capped)` with the warning chip, not a bare count.
- **Alignment:** numeric columns right-aligned, text columns left-aligned, single-glyph columns centred.

## Design Principles

Seven governing constraints. These are design requirements, not a claim that every current screen passes. Existing framing and proportion gaps are recorded in [REDESIGN.md](./REDESIGN.md).

1. **Data outranks chrome.** Decorative framing must not displace useful data. At 1600×900 with the Search filter editor closed, aim for the table-owned scroll viewport to cover at least 70% of the window area. The inspected idle General and Moneyball views reach 72.21% and 71.56%; this is not a guarantee with applied chips or refresh feedback. Measure the scroller, including its sticky headers, not the containing panel or toolbar; report visible data-row height separately. Record filter-chip and Load Data feedback state with the measurement.
2. **Separate with hairlines, not redundant boxes.** Prefer a 1px `outline-variant` rule or a tonal step between rows, fields, and sections. Independent scrollers, interactive assignment cells, and overlays can need their own boundaries; they are not permission for repeated decorative card layers. *Review:* every inner container has a distinct purpose, and analysis tables have no decorative vertical rules between columns.
3. **Snapshot provenance is always visible.** Every screen that shows player data states which save is active and its current snapshot's in-game date, without scrolling. *Test:* screenshot any data view and you can name the save and in-game date from the image alone. This follows the explicit-refresh principle in [CONCEPT.md](./CONCEPT.md) — the user must never mistake one save's data for another.
4. **Brightness carries value; the number carries the fact.** Score meaning comes from the ramp, and the number is always present. *Test:* convert a screenshot to greyscale — the ranking still reads.
5. **Every mutation reports its phase.** Long operations name what they are doing and which stage failed. Load Data distinguishes a scan failure from an ingest failure, because the fixes differ: start FM versus retry the ingest. *Test:* every mutation has a pending label, a success state, and a phase-specific error message.
6. **Keyboard reaches everything; hover reveals nothing.** Hover may only change colour. Any action or information available on hover is also available from the keyboard and visible without a pointer. *Test:* complete a full search-to-profile pass with the keyboard alone.
7. **Nothing loads from the network.** Fonts, icons, and images ship in the bundle. *Test:* run the app with networking disabled and no glyph, icon, or layout changes.

## Layout & Spacing

The app is a **single window with a utility bar and grouped top navigation** — a desktop tool, not a set of pages. Minimum 1280×800; the top navigation fit is proven at 1280×800. Content fills the window width; `content-max-width` is `none` because a clamped column wastes the space a 20-column player table needs.

Regions, in visual order:

1. **Utility bar** (`header-height` 56px). It contains the app logo, Back and Forward, a fixed-width global player search, the active save selector with its current in-game date, and **Load Data**. It stays first so global controls remain separate from destination navigation.
2. **Top navigation** uses a 48px destination band with 36px inline-icon links: Home (Dashboard), Players (Search and Moneyball), Staff (Staff Search and My Staff), Club (Squad, Planner, Tactic, and Youth), and Settings. Fine separators and inline captions retain group context. Active state combines a subtle tint, primary icon, reinforced label, and persistent 2px indicator.
3. **Workspace header** (inside the content area). One semantic h1 uses `headline-md`, with optional secondary context and feature-owned controls. Profile identity keeps `headline-lg`; Staff Search retains a visually hidden h1 because active navigation already names it. Use `stack-md` between regions, not an extra generic Results title.
4. **Content area.** Panels on `surface-container` with `gutter` 16px between them and 16px page padding.
5. **Inspector** (right, `inspector-width` 320px, optional and dismissible). Comparison and detail controls on a profile. Slides over the content edge; never squeezes the table below its usable width. **Search does not use the inspector for filters** — filters use the compact strip and editor modal below.

Spacing rhythm, all multiples of the 4px `unit`:

- `stack-xs` (4px) — between a micro-label and its value; inside a chip.
- `stack-sm` (8px) — between related controls in a row; cell padding in a dense table.
- `stack-md` (16px) — default panel padding, gutter between panels, gap between form fields.
- `stack-lg` (24px) — between distinct sections inside one panel.
- `stack-xl` (32px) — above a major screen division. Rare.

Hard dimensions: shared analysis tables use fixed 40px two-line rows and two 32px header rows. A table picks one row height for all its rows — ragged row heights make a long list unreadable and break virtualization. Other semantic tables may use the 36px single-line token where their content requires it. Player cards in grid view are minimum 260px wide in an auto-fill grid.

## Elevation & Depth

Depth is tonal. The recessed surface is darker than the canvas; subsequent levels become lighter. The canvas-to-panel step is 1.07:1, panel-to-raised is 1.20:1, and raised-to-overlay is 1.39:1. Hairlines support grouping without heavy boxes, but they are not proof of an accessible boundary. In particular, `outline-variant` equals the overlay fill and cannot separate content inside that surface; meaningful boundaries must be evaluated against their actual adjacent colours.

- **Level 0 (Canvas):** `background` — the window itself. Nothing sits directly on it except panels.
- **Level 1 (Recessed):** `surface-container-lowest` — sticky table headers, diagnostic and log wells. Darker than the canvas, so it reads as behind it.
- **Level 2 (Panel):** `surface-container` — the default. Cards, tables, panels, the top bar.
- **Level 3 (Raised):** `surface-container-high` — hovered table rows, input fields, nested blocks inside a panel, the inactive half of a segmented control.
- **Level 4 (Overlay):** `surface-container-highest` — dropdowns, context menus, popovers, modals, toasts. This is the only level with a shadow: `0 8px 24px oklch(0 0 0 / 0.6)`, plus the standard hairline border. Modals also dim the content behind them with `oklch(0 0 0 / 0.6)`.

### Z-Index Scale

Layers are separated by steps of 10 to leave room for future insertion. No element uses an arbitrary value.

| Layer         | Value  | Usage                                   |
| ------------- | ------ | --------------------------------------- |
| Base          | `z-0`  | Content area, tables, cards             |
| Sticky        | `z-10` | Sticky table headers, utility bar, top navigation |
| Dropdown      | `z-20` | Select dropdowns, autocomplete panels   |
| Context Menu  | `z-30` | Right-click context menus               |
| Overlay       | `z-40` | Modal backdrops, toast container region |
| Modal Content | `z-50` | Modal dialogs, individual toasts        |

Every component that creates a stacking context declares its `z-index` from this scale. Components at the same layer must not overlap in normal use; where they can, rely on source-order stacking within the layer.

## Shapes

The shape language is **Rounded-Instrument** with purpose-based geometry: quiet data hosts, rectangular controls, and meaningful circles/chips. [Radius roles](#radius-roles) own the shared scale.

- **Panels, cards, table hosts:** `lg` (0.5rem / 8px). Flush tables share the host's corners, without a second decorative frame.
- **Modals, inspector panel, floating overlays:** `xl` (0.75rem / 12px).
- **Buttons, fields, global search, top-navigation links, segmented groups:** `md` (0.375rem / 6px). `DEFAULT` uses the same value.
- **Segmented items and compact value tags:** `sm` (0.25rem / 4px).
- **Filter tags, status chips, circular scores, position pickers:** `full`; ordinary text buttons and search fields are not pills.
- **Square score badges and small tags inside a cell:** `sm` (0.25rem / 4px). Circular score badges use `full`.
- **Table rows:** `none`. Rows are separated by hairlines, not individually rounded; rounding is on the table container only.
- **Focus rings:** 2px solid `primary`, matching the element's own radius. Use a 2px outward offset where space permits, or an inset offset in bounded navigation and feedback regions to avoid clipping. `:focus-visible` only, never `:focus`. Never removed and never replaced by a colour change alone.

## Components

Each spec below is the contract for that component. Reference tokens by name; never put a raw colour value in a component.

### Button

The action primitive. One primary action per screen region.

- **Container:** `md` radius, `stack-sm` vertical and 16px horizontal padding, 32px height (36px for the top-bar Load Data button), `label-lg` text. Icon-only variant is a 32×32 square with `md` radius.
- **States:** hover takes `primary-hover` on a filled variant and fills an unfilled variant with `surface-container-high`; active takes `primary-active` on a filled variant and `surface-container-highest` on an unfilled one; `:focus-visible` adds the 2px primary ring; disabled drops opacity to 45% and sets `cursor: not-allowed`; loading disables the button, swaps the label for a phase-specific pending label ("Scanning…", "Saving…"), and shows a spinner in the leading icon slot. Transition `background-color 150ms ease-out`. Width never changes between states — reserve the loading label's width, and keep the inactive label `aria-hidden` so it stays out of the accessible name.
- **Variants:** `primary` — `primary` fill, `on-primary` label; the one main action. `secondary` — transparent fill, `outline` border, `on-surface` label. `ghost` — no fill or border, `on-surface-variant` label, hover fills `surface-container-high`; for toolbar and icon actions. `destructive` — `error` fill, `on-error` label; requires a confirmation modal before it executes. Snapshot and save deletion use this variant only after the target-specific destructive Modal confirms the cascade.
- **Content / Anatomy:** optional 16px leading icon, label in `label-lg`, optional trailing chevron for menu buttons. Never icon-plus-text in the icon-only variant.
- **Behaviour:** always a `<button>` with an explicit `type`. Icon-only buttons carry `aria-label` and a tooltip — the props type requires both an icon and an `aria-label` for that size, so an unlabelled icon button does not compile. A button that opens a menu sets `aria-expanded` and `aria-haspopup`.

### Top Navigation

Primary navigation between the app's main destinations.

- **Container:** `surface-container`, a 48px centered band below the utility bar, with a bottom `outline-variant` border. Links are 36px high with 16px inline icons and 12px labels; fine vertical separators and inline 11px captions retain group context. All ten destinations fit the supported 1280×800 minimum.
- **States:** hover changes colour only. Active links use a 10% primary tint, a primary icon, a reinforced label, and a persistent 2px primary bottom indicator. `:focus-visible` shows a separate 2px primary ring inset within the link bounds.
- **Content / Anatomy:** **Home** contains Dashboard; **Players** contains Search and Moneyball; **Staff** contains Staff Search and My Staff; **Club** contains Squad, Planner, Tactic, and Youth; **Settings** contains Settings. Search and Moneyball select `/search?view=general|moneyball`; Staff links select `/staff?view=search|my-staff`; Club links select `/my-club?view=squad|planner|tactic`; Youth selects `/academy`.
- **Behaviour:** a `<nav>` contains router links. Each supported direct destination sets exactly one link to `aria-current="page"`. Unknown and not-found routes set no destination current. Player and staff profile routes set only the Players or Staff group caption to `aria-current="location"`; no child destination is current. Top-navigation Club destination changes use normal Link navigation, add a browser-history entry, and let browser Back return to the prior destination. Same-route Club changes retain `squadSort` and `squadDir`; route-local sort controls use replace navigation. Search view changes retain only the route's existing shortlist/combine state. Profile analysis tabs and Youth tabs remain local.

### Utility Bar

Global search, save and snapshot context, and the Load Data action.

- **Container:** `surface-container`, `header-height` 56px, 1px `outline-variant` bottom border, 16px horizontal padding, sticky at `z-10`.
- **States:** static. Its children carry their own states.
- **Variants:** none.
- **Content / Anatomy:** 36px outline symbol with its lit green blip, **Back** and **Forward** icon buttons, 480px rectangular global search field, right-aligned active-save selector plus a separate nonshrinking matching snapshot date, and Load Data (`primary` button).
- **Behaviour:** the search field takes focus on `Ctrl+K` from anywhere. Back and Forward use only the current TanStack Router session history. They are disabled at the reachable history boundaries, and a new navigation after Back removes Forward availability. They do not persist history, own separate scroll state, or add custom scroll restoration. TanStack Router's existing scroll restoration remains authoritative. Switching saves swaps all snapshot-scoped views and suppresses any stale Load Data progress or outcome from a prior invocation. Load Data is an async Tauri command with command-scoped best-effort `Channel`; the frontend hook captures the active save ID/token and `api/load-data.ts` constructs the `Channel`; Rust verifies the supplied `saveId`/`contextToken` before scan and again before publication and echoes them on every progress event, rejecting stale publication while still allowing the active-save switch to succeed concurrently. The top-bar banner keeps a stable polite live-region text and an adjacent native `<progress>` (indeterminate for scan, determinate only when counts are truthful). Phases are `scan` → `preparing` → `scoring` → `saving` → `finalizing`; success shows the detailed `scanMs`, `prepareMs`, `scoringMs`, `saveMs`, `finalizeMs`, and `totalMs` timings, while the result retains undisplayed compatibility `ingestMs = saveMs + finalizeMs`. Search and Squad remain mounted during the command and on failure; a successful matching current replacement cancels/removes the exact Search/Squad roots under a continuation guard, then schedules current-owner invalidations; mutation settlement does not await those refetches, while suppressing stale updates. The final command result and phase-specific error remain authoritative if a progress message is missed.

### Settings management

The `/settings` route is one vertical page with **Preferences**, **All boosts**, **Graphics**, **Save data**, and **Bridge** sections. Preferences contains the labelled **Default player analysis view** select. It sets one app-local General or Moneyball default for Player Search and Player Profile when their URL does not specify a view. All boosts contains one confirmed **Apply all boosts** action with phase progress and a combined outcome. A recovery result disables the action until Load Data replaces the current snapshot; a save or snapshot change removes feedback from the prior context. Each section keeps its own loading and error boundary where it loads data, so one failure does not blank the page. The managed-club selector lives in the My Club header; `/settings#managed-club` is a replace redirect for old bookmarks. The top bar remains the only save switcher and the only location for **Load Data**. Dashboard contains only its heading and `Placeholder.`

- **Snapshot history panel:** show one semantic table for the active save, ordered by valid in-game date descending, then load time and snapshot ID. Dated rows always precede undated rows. Each row shows the custom name when present, the in-game date as a separate line, player count, relative load age with an absolute UTC `title`, a visible **Current** marker, and separate Edit date, Rename, and Delete actions. Names organize rows but never replace date metadata or change order. Edit date opens a focused form Modal with a bounded canonical `YYYY-MM-DD` field. The Modal reports empty, malformed, or impossible dates locally, retains the input on errors, prevents duplicate submission, and restores focus to the row action on close. On success, history refreshes; a winner change refreshes current-only views, an unchanged current edit updates only the matching cached summary date, and a non-current edit refreshes history without sibling refetches.
- **Empty and loading states:** when a save has no snapshots, show `No snapshots stored` and direct the user to **Load Data**. The history panel keeps its existing panel geometry and horizontal overflow behavior.
- **Rename:** open a focused form Modal with a bounded snapshot name. Blank input clears the custom name and restores the in-game date label. Successful rename invalidates only the active-save history query; the row remains in the same position.
- **Snapshot deletion:** open a destructive Modal whose title includes the exact snapshot date or custom name plus its internal snapshot identifier. Explain that players, staff, role scores, bridge provenance, and Moneyball data are removed, while Planner, Academy, and Youth data remain. Disable duplicate submission, keep errors inside the dialog, and return focus to the history panel. Deleting the current row refreshes every current-only view; deleting a non-current row does not.
- **Save management:** list save names and the active marker below the save rename/create forms. Every save delete Modal names the save and states that all snapshots, player data, Moneyball data, Planner settings, Academy records, and Youth enrichment will be removed. State whether the active save stays unchanged, another save becomes active, or a blank `Default save` replaces the final save. Bind the confirmation to the immutable target context so a save switch or row-ID reuse cannot retarget the action.
- **Context feedback:** Load Data success copy remains bound to the save that started the scan. If the final save is deleted and a new `Default save` receives the same numeric ID, stale feedback is cleared. Current/save-changing success invalidates Search, Player, Staff, Planner, and Academy state from the route composition layer.
- **Accessibility:** destructive dialogs use `role="dialog"`, `aria-modal`, target-specific headings, keyboard focus trapping, Escape/Cancel protection while pending, and focus restoration. Duplicate targets include visible and assistive identifiers so two rows with the same date or name remain distinguishable.

### Data Table

The core surface. Player and staff search results, squad lists, and comparison sets.

- **Container:** the host owns the `surface-container`, `lg` radius, and 1px `outline-variant` border; the table is full-bleed with no inner padding or second rounded/bordered scroller. Shared analysis tables use fixed 40px two-line rows, bounded pixel column widths, and one table-owned scroller for both horizontal and vertical overflow. At 1280×800, minimum widths produce table-local horizontal overflow while the sticky identity and both header rows remain visible; at 3440×1440, the table reveals more columns without stretching cells without bound. Search and Squad panels are `flex` columns with `min-h-0`; their route roots use `h-full` so the document does not grow with the virtual spacer. The two 32px header rows use `surface-container-lowest` and remain sticky at `z-10` (64px total). Body rows carry a 1px `outline-variant` bottom border. Cell padding is `stack-sm` horizontal.
- **States:** row hover fills `surface-container-high`; row `:focus-visible` shows the primary ring inset; selected row fills `primary-container` with a 2px `primary` left indicator and `aria-selected`; sorted leaf headers show `primary` label text plus a direction caret. Row height never changes on any state.
- **Identity:** every shared analysis table has a required, caller-owned, non-removable sticky identity region before analysis columns. Player identity uses the name with available club and division context and reserves stable portrait and club-logo slots for optional exact-UID local graphics; staff supplies its own identity content without person images. Identity is separate from configurable analysis columns, so Club and Division are not configurable identity metrics and cannot be duplicated.
- **Content / Anatomy:** grouped headers use the same group metadata as the keyboard-operable **Columns** control. Header cells use `label-md` uppercase `on-surface-variant`; text cells use `body-sm` `on-surface`; secondary identity lines use 11px regular `on-surface-variant`. Compact tactic headers show the placement identifier with smaller role context; the full tactic definition is the accessible name and a visible disclosure on keyboard focus, never hover-only. Raw numeric cells are right-aligned with `font-sans text-body-sm font-medium tabular-nums`; scored units retain `font-mono text-mono-sm`. Score cells use the current unfilled `ScoreBadge` table variant and tier ramp; missing values use a neutral `—`.
- **Behaviour:** a real `<table>` with `<caption class="sr-only">`, `<thead>`, and `<th scope="col">`. Sortable leaf headers set `aria-sort`. Header menus expose column movement, grouped analysis-column management, and bounded keyboard or pointer resizing; identity has resize only. A table-associated toolbar owns the dataset summary, filters, grouped **Columns**, and view-specific dataset controls. Page actions remain in the page header or feature-owned controls. Up and Down arrows move row focus across bounded 50-row virtual pages, Enter or a row click opens the player, and the sticky header never covers the focused row. No Previous or Next controls or unbounded client collection exist. Empty, loading, and error states replace the body with the states below — never blank space.

### Score Badge

A role or position fit score. The most repeated element in the app.

- **Container:** in a table, no fill and no border — the number sits directly on the row in its tier colour. Elsewhere, a 28px circle with `full` radius, `surface-container-high` fill, and a 1px border in the tier colour at 40% alpha. The table variant is unfilled on purpose: a filled badge would match the hovered row background and vanish, and 500 filled chips in a column is exactly the boxing that principle 2 forbids.
- **States:** static inside a row. All tiers pass normal-text AA on default, raised/hovered, and overlay surfaces; see the contrast table. In an interactive context — a clickable role chip — hover raises the surrounding fill, never the number's colour. Check the actual pairing before adding another badge context.
- **Variants:** `table` (`mono-sm`, unfilled, right-aligned), `card` (28px filled circle, `mono-md`), `hero` (48px, `mono-lg`, unfilled, used for the current and potential best-role summaries on a player profile). A `muted` variant renders the number in `on-surface-variant` instead of a tier colour, for roles outside the player's positional familiarity — the score is still shown, but it does not compete for attention.
- **Content / Anatomy:** the integer score, nothing else. No unit, no percent sign, no trailing zero. Colour comes from the `score-1` to `score-4` ramp by the tier table in Colors.
- **Behaviour:** the accessible name is the full statement — `"Deep-lying playmaker: 82, Excellent"` — not just the digits. The tier label also appears in `title`. Missing scores render the neutral `—` instead of a badge. Never render a badge without its number.

### Status Chip

Compact non-interactive state: transfer status, bridge state, and qualifiers.

- **Container:** `full` radius, 20px tall, `stack-xs` vertical and `stack-sm` horizontal padding, `label-md` text. Fill is the matching `*-container` token with a 1px border in the semantic tone at 40% alpha, because container fills sit only 1.4:1 above the panel and need the edge to read.
- **States:** static. A chip is never a button; if it needs a click, it is a Filter Tag or a Button.
- **Variants:** `success`, `warning`, `error`, `info`, and `neutral` (`surface-container-high` fill, `on-surface-variant` text).
- **Content / Anatomy:** 12px leading icon, then the label. The icon is required — the chip must not rely on fill colour.
- **Behaviour:** decorative chips that duplicate adjacent text take `aria-hidden`. Chips carrying unique information stay in the accessible tree.

### Text Input, Select, and Search Field

Form controls, including the global and local search fields.

- **Container:** 32px tall, `surface-container-high` fill, 1px `outline` border, `md` radius for all field variants, `stack-sm` horizontal padding, `body-md` text. Select adds a 16px trailing chevron.
- **States:** hover brightens the border to `on-surface-variant`; focus replaces the border with a 2px `primary` border and no offset ring, so the field does not shift; invalid shows an `error` border plus a `body-sm` `error` message below, tied to the field with `aria-describedby`; disabled drops to 45% opacity. Placeholder text is `on-surface-variant`.
- **Variants:** `text`, `number` (tabular figures, right-aligned), `select`, `search` (rectangular field with a 16px leading magnifier and a clear button once non-empty).
- **Content / Anatomy:** every field has a visible `<label>` in `label-md` above it, or an `aria-label` when the icon and context make the purpose obvious — as in the global search. Placeholder text is never the only label.
- **Behaviour:** search fields debounce at 200ms and never block typing on a query. Escape clears a non-empty search field before it closes any surrounding panel. Selects are native `<select>` unless multi-select or option-rendering requires a custom listbox, in which case implement full arrow-key and type-ahead support.

### Panel

The default container for a titled block of content.

- **Container:** `surface-container`, `lg` radius, 1px `outline-variant` border, `stack-md` padding.
- **States:** static. A panel is not interactive.
- **Variants:** `default` and `flush` (no padding, for a panel whose only child is a full-bleed table).
- **Content / Anatomy:** optional header row with an h2 `headline-sm` title, or h3 14px/600 for a nested section, and wrapping actions on the right. Actions render even without a title. Titled flush content follows at `stack-md`; an actions-only flush header uses a 4px gap before local feedback/content. Avoid a nested Panel used only as decoration; prefer spacing and a hairline rule. A distinct interactive or scrolling region can require its own boundary. Existing nested feature containers are subject to the framing review in REDESIGN.md.
- **Behaviour:** the panel title is the section heading and must keep the document's heading order correct.

### Compact Filter Strip, Filter Tag, and Filter Editor

Progressive filtering on the Search screen — Genie Scout / FM-style operator rules, not an inspector of sliders.

- **Compact strip (above results):** the dataset toolbar puts count/sort left and concise idle filter state, **Edit filters**, **Columns**, and dataset toggles right. Applied tags, AND|OR context when relevant, and **Clear all** occupy a wrapping second row. Staff Search adds Preferred Job and Only unemployed only while shortlist filtering is on. Feature actions — **Add Tactic**, **Upload Shortlist**, **Configure staffing needs**, and **Optimize assignments** — remain outside this toolbar. Tags retain a meaningful `full` radius with neutral raised fill, on-surface text, and explicit remove names; remove hover uses `surface-container-highest`.
- **Filter tag label:** field label, operator word, and value (e.g. `CA > 150`, `Role · Deep-Lying Playmaker (IP) > 70`). Incomplete draft rules do not appear as tags.
- **Filter editor modal:** `form` modal variant. Lists rules as field + operator + value rows with add/remove. Single AND|OR toggle for the flat rule list (no nested groups). Field selection uses the categorized, searchable metric picker, grouped by identity, club and contract, ability and reputation, visible attributes, hidden attributes, personality, position suitability, current role scores, and potential role scores. The editor owns a local draft copied from the applied URL state whenever it opens. **Done** applies one complete draft; incomplete rules disable Done. Cancel, the close control, Escape, and backdrop dismissal discard the draft without changing the URL, starting a query, or materializing potential scores.
- **Operators by field kind:** strings — contains / does not contain / is / is not; integers (CA, attributes, role scores, suitability) — greater than / less than / equals / does not equal; booleans and closed enums — is / is not.
- **Behaviour:** Done applies the complete rule set and combine mode in one URL update; typing and other draft edits are query-silent. Active filters, combine mode, and sort live in the URL search params so the view survives a reload. Removing the last tag restores the unfiltered result set. Applying a filter adds its metric column once when it is not already visible, but filters and columns can then change independently; removing a column does not remove the filter.

### Search tactic columns

Search General and Moneyball show two tactic controls in the route-owned page header: **Add Tactic (Current)** and **Add Tactic (Potential)**. Each is an accessible toggle with `aria-pressed`; active uses the primary container fill while inactive uses the secondary/ghost treatment. Disabled applies when no tactic context exists, during loading or error, or on cached refresh read-only, and no layout change occurs until data is ready. Interleaving is deterministic: one active group renders 11 lanes in `TACTIC_POSITION_ORDER` at the far right; both active interleave as `current` then `potential` per lane. Each header shows a compact placement identifier as its visible primary text with smaller role context; its full `"{IP Position} ({IP Role}) / {OOP Position} ({OOP Role})"` definition from `TacticOptions` remains the accessible name and appears in a visible disclosure on keyboard focus. Cells are right-aligned, width 112 (clamped 72–360) with joint `replaceLayout` persistence, show `ScoreBadge` for numeric scores and `—` in `on-surface-variant` for unavailable, and use header-X to deactivate the group, re-compact the survivor without gaps, and fall back from a removed tactic sort. Sort keeps unavailable last.

### Nationality flags

Shared nationality cells remove later exact duplicate names while preserving first-occurrence order. The first unique nationality is the primary flag at the standard size. Later unique nationalities render as smaller, muted secondary flags when the source supplies them. Known FM names use the bundled `country-flag-icons` SVG package, including the four UK home nations; `Zanzibar` uses the checked-in public-domain SVG because the package has no matching flag. Every flag exposes the full stored name through its accessible name and title. An unmapped future value stays as text with the same primary or secondary hierarchy, and an empty nationality list renders `—`. This treatment applies to shared player and Staff tables and the player profile summary. The app performs no runtime network request and never substitutes a guessed country or a league flag.

### Modal

Focused decisions and destructive confirmations.

- **Container:** `surface-container-highest`, `xl` radius, `stack-lg` padding, maximum 560px wide (720px for a two-column picker), overlay shadow, `z-50` over a `z-40` backdrop of `oklch(0 0 0 / 0.6)`.
- **States:** entrance fades the backdrop and lifts the panel 8px over 200ms ease-out; exit reverses it at 150ms. Both are suppressed under `prefers-reduced-motion`.
- **Variants:** `informational` (single dismiss), `form` (Cancel plus a `primary` submit), `destructive` (Cancel plus a `destructive` confirm, with the affected object named in the body — "Delete save *Braga 2029*?" — never a bare "Are you sure?").
- **Content / Anatomy:** `headline-md` title, `body-md` body, right-aligned footer actions with the primary action last. A close button sits top-right on informational modals only; a destructive modal requires an explicit choice.
- **Behaviour:** traps focus, returns focus to the trigger on close, closes on Escape and on backdrop click — except destructive variants, which ignore backdrop clicks. Uses `role="dialog"` with `aria-modal="true"` and `aria-labelledby` on the title. Player and staff shortlist import modals use the context-bound native picker, report stored, total, and skipped rows, preserve the old list after a zero-match error, and discard late results after a save or snapshot change.

### Toast

Transient confirmation for a completed background action.

- **Container:** `surface-container-highest`, `lg` radius, `stack-md` padding, 360px wide, overlay shadow, bottom-right stack with `stack-sm` between items, `z-50`.
- **States:** slides in 8px over 200ms; auto-dismisses after 5 seconds; the timer pauses on hover and on keyboard focus anywhere inside the toast. Manual dismiss is always available.
- **Variants:** `success`, `warning`, `error`, `info`. Error toasts do not auto-dismiss.
- **Content / Anatomy:** semantic leading icon, `body-md` message, optional single action link, dismiss button. One line of message where possible; two at most.
- **Behaviour:** the container is an `aria-live="polite"` region, or `assertive` for errors. Toasts confirm outcomes; they never carry information the user must act on later. Anything that needs a decision is a modal or an inline banner. Maximum three toasts at once — collapse the rest into a count.

### Empty, Loading, and Error States

Every data view defines all three. A blank region is a bug.

- **Container:** centred block inside the host panel or table body, minimum 160px tall, `stack-lg` padding.
- **States:** these *are* states. Loading uses skeleton rows at the real row height in `surface-container-high` with a 1.5s pulse, suppressed under `prefers-reduced-motion` in favour of a static block plus a "Loading…" label. Never an unlabelled spinner.
- **Variants:** **No snapshot** — "No data loaded for this save", naming Load Data as the next step; this is the app's true first-run state and it must point at the one useful next step. It names the top bar's button rather than repeating it, because Load Data is already on screen and a second copy would be the only emphasized element competing with it. **No results** — a player or staff-specific no-match message, with "Clear filters". A shortlist filter with no saved or current matching rows uses setup/no-shortlist guidance and keeps **Upload Shortlist** available; the removed shortlist tabs and workspace have no separate empty-state copy. **Error** — a phase-specific message as the title, the underlying reason as the explanation line, and a Retry button. **Truncated** — results render normally with a persistent `warning` banner naming the cap.
- **Content / Anatomy:** 24px icon in `on-surface-variant`, `headline-sm` title, one line of `body-md` explanation, one action. No illustration.
- **Behaviour:** an error state names the phase in the user's terms — scan errors mention FM and the bridge, ingest errors mention the database — and never shows a raw error string as the headline. Loading skeletons match the final layout so nothing jumps when data arrives.

### Charts

Radar for role and attribute profiles; line or area for value and score trends.

- **Container:** inside a Panel, `surface-container` background, no chart border. Axis lines and grid rings in `outline-variant`; axis labels in `label-sm` `on-surface-variant`.
- **States:** hover on a series or point raises a tooltip on `surface-container-highest` at `z-20` showing the label and exact value. Keyboard focus steps through series and exposes the same values as text.
- **Variants:** `radar` (subject plus up to two comparison players, plus one reference), `trend` (line with an optional 20%-alpha area fill), `bar` (single-series comparison).
- **Content / Anatomy:** series colours in order — `chart-1` Signal Green for the subject, `chart-2` steel and `chart-3` magenta for comparisons, `chart-4` neutral dashed for a league or squad average. Series fills use 20% alpha so overlaps stay readable. Every series also gets a distinct stroke pattern.
- **Behaviour:** one subject plus two comparisons, which is why only three player series tokens exist; beyond three overlaid shapes a radar stops informing. A chart is never the only representation of its data — the same values appear in an adjacent table or an `sr-only` table, because a radar chart is not accessible on its own.

### Scrollbar, Icons, and Motion

Cross-cutting rules rather than components.

- **Scrollbars:** 10px, transparent track, `outline-variant` thumb with `full` radius, brightening to `outline` on hover. Applied via `scrollbar-color` and `scrollbar-width`. Never hide a scrollbar on a scrollable region — a dense table needs a visible position cue.
- **Icons:** [Lucide](https://lucide.dev) via `lucide-react`, bundled. Shared Button icons are 16px, status-chip icons are 12px, top-navigation icons are 16px, and empty-state icons are 24px. Other feature icons use the appropriate 16/20/24px size. Use `strokeWidth` 1.5 by default and 2 for active top-navigation links; use `currentColor` always, so an icon inherits its context. One icon set only, and no emoji as an icon anywhere.
- **Motion:** 150ms ease-out for colour and opacity on hover, focus, and active. 200ms ease-out for overlay entrance; 150ms for exit. Nothing animates longer than 200ms, and layout, size, and position never animate on hover. Under `prefers-reduced-motion: reduce`, drop every transform and entrance animation and keep colour changes instant.

### Player profile layout

Dedicated route `/players/$uid` (not an inspector overlay). Comparison inspector remains unused until a later compare feature.

- **Section switch:** the profile uses four canonical sections: **Overview**, **Attributes**, **Role Fit**, and **Moneyball**. A valid `section=overview|attributes|role-fit|moneyball` URL value wins; legacy `view=moneyball` remains compatible. When the section is absent, the saved General preference maps to Overview and the saved Moneyball preference maps to Moneyball. The General attribute `tab` remains compatible within Attributes.
- **Profile workspace:** at the 1280×800 minimum, the persistent identity rail stays visible while the selected section owns the remaining bounded workspace. Overview owns Ability and same-role Current → Potential best-current IP/OOP summaries. Attributes owns dense grouped attribute evidence. Role Fit owns the pitch and role filter. Moneyball owns contextual metrics and current-only tactical summaries. The layout remains contained at 1280×800 and bounded through 3440×1440; a Load Data outcome reduces available panel height without creating page scrolling.
- **Overview cards:** five separate surfaces show Current Ability, concealment-gated Potential Ability, Market Value, and the best current IP/OOP roles. Neutral headline values use `mono-xl`; role names wrap above aligned Current → Potential pairs. Labels and captions use body typography rather than small, tracked control labels.
- **Identity rail:** player name, club and division, nationality flags, age, date of birth, height, preferred foot, and market value stay visible across sections in vertically aligned rows. A larger neutral stage reserves portrait and crest slots for optional exact-UID local images, with fixed initials and shield fallbacks. Transfer flags appear only when true. Overview's Ability region owns CA and concealment-gated PA; neither appears in the rail. The rail's bottom action area keeps **Modify Player** and the separate hidden-information control available across all four sections.
- **Information visibility:** the separate hidden-information control reveals or conceals hidden information for the active save. In the concealed state, PA, projected values, potential role scores, Hidden and Personality values, and development actions are absent. Hidden and Personality subtabs show an explicit concealed state. Current values remain visible. Modify Player disclosure contains the development actions and preserves their guarded mutation flow.
- **Development actions:** when information is revealed, **Boost CA** and **Wonderkid Mentality** sit inside the identity rail's **Modify Player** disclosure, separate from the hidden-information control. The action slot keeps one control-row minimum height. Only the inline outcome uses a bounded vertical scroll region, so completed feedback remains reachable without moving the section switch. Hover or keyboard focus reveals each snapshot preview or disabled reason in an upward-opening tooltip that stays within the desktop viewport. One Modal confirms each action, preserves the existing guarded command contract, and restores focus to its trigger or the verified outcome. Results stay inline below the actions. There is no numeric input, arbitrary value control, or random-value selection.
- **Attribute tabs:** the Attributes panel uses four tabs. Outfield players see **Outfield** | **Goalkeeping** | **Hidden** | **Personality**, defaulting to Outfield. Their Outfield tab renders Technical, Mental, and Physical together, with Set Pieces below Physical to balance column height. Profile rows use a 32px minimum height and wrap labels when needed. At 1920×1009, all 36 outfield attributes fit in Overview without scrolling; smaller workspaces retain panel-owned scrolling. Players with recorded GK familiarity of 15 or higher see **Goalkeeping** | **Outfield** | **Hidden** | **Personality**, defaulting to Goalkeeping. Their Goalkeeping tab renders the alphabetized goalkeeper list, including First Touch, Passing, and Technique, with Mental and Physical; their Outfield tab contains only the remaining Technical attributes and Set Pieces. An explicit canonical `tab` URL search value wins; legacy `technical`, `mental`, and `physical` values normalize to Outfield. Missing or invalid values use the player-sensitive default. Arrow keys, Home, and End follow the visible order.
- **Attribute values:** when information is revealed, visible groups show text-accessible `Current → Potential` pairs of raw FM integers. Players aged 29 or older use the current value as potential because the projection model does not increase them further. When concealed, visible groups show Current values only, while Hidden and Personality show the explicit concealed state. Each known value uses the shared four-band data ramp with the documented 1–20 bands; the number remains the primary fact. Null renders `—`.
- **Role fit:** a narrow pitch offers the 14 supported role positions as 44px circular buttons and omits `SW`. The selected-position heading and familiarity sit above the table, outside its aligned Role, Phase, Current, and Potential header row. Role names wrap, and scores use centered 36px outlined circles in the shared score ramp. At the `lg` two-column breakpoint, its position-picker column owns vertical scrolling around the pitch's natural height; below that breakpoint, the stacked workspace grows naturally. The strongest positive recorded familiarity is selected first; if none exists, the highest current role supplies the fallback position. Known positive familiarity shows its raw 1–20 value and the same attribute tier colour; red-tier familiarity values 1–5 use muted surfaces to de-emphasize unusable positions without reducing text or focus contrast. Zero, unread, and legacy-missing values remain `—` and cannot become the best position. Selecting a position shows only roles whose catalog `positionTags` contain that exact position. **Current** descending is the default sort. When information is revealed, the **Current** and **Potential** column headers switch the score basis and toggle ascending or descending order. Players aged 29 or older show current role scores as potential. When concealed, Role fit exposes only Current scores. Unavailable scores stay last and catalog order breaks ties. Rows retain the role name and IP/OOP phase. Missing scores render `—`.
- **Moneyball workspace:** a scored current import shows contextual metrics and current-only tactical summaries. The raw panel shows asking price, starts, substitute appearances, and minutes as neutral context. For a player with natural positions, it also states the natural positions and the backend-provided unique comparison-player count. Eight metric tabs — Shooting, Creation, Possession, Defending, Aerial, Goalkeeping, Discipline, and Results — show raw values with a 0–100 ScoreBadge. The role-fit panel uses the position picker and one sortable current Moneyball score column; it does not invent a potential Moneyball score. Each role has a keyboard-reachable disclosure with its five metric contributions, direction, catalog version, and natural-position comparison basis. If an imported player has no natural position, raw context and metrics remain visible while explicit percentile and role-score unavailable states replace badges and the role-score table. Raw values remain visible and unavailable values remain `—`. Arrow keys, Home, and End move through the visible metric tabs. A player absent from the current import and a pre-score import show separate neutral recovery states that direct the user to Player Search's Moneyball upload.
- **States:** no snapshot → EmptyState pointing at Load Data; unknown UID → “Player not in this snapshot”; one loading skeleton mirrors the identity rail and selected section workspace.
- **Out of this layout for now:** radar charts, history/trend blocks, compare inspector, and combined IP/OOP weight controls. The pitch is a role filter and familiarity display, not a new suitability calculation.

### Staff workspace layout

The `/staff` route owns Staff Search and canonical **My Staff**; `/staff/$uid` owns Staff Profile. My Club owns **Squad**, **Planner**, and **Tactic** only. Staff Search and My Staff are separate top-navigation destinations, and the managed-club selector remains in the My Club header.

- **Staff Search:** the shared configurable virtual table and filter editor cover the current snapshot. The active top-navigation item identifies the destination, so the page does not repeat a **Staff Search** title. **Optimize assignments** is the one primary action; **Upload CSV** and **Configure staffing needs** are secondary actions in a full-width route-owned row outside the generic toolbar. Readiness and recovery guidance remain visible, and optimizer feedback occupies a stable full-width status region below the controls. The staffing dialog provides bounded step controls, direct numeric entry, a zero-excludes explanation, and valid-draft team and section totals. Results lead with filled, vacancy, current-staff, and recruit metrics plus supporting counts; consecutive Rust-provided names form groups without a Scope column. Vacancy rows use warning icon/text treatment, concise reasons, native detailed evidence disclosure, and **Adjust staffing needs** and **Review shortlist** recovery actions. Current staff keep steel/icon treatment, the bounded result scroller keeps sticky headers, and the result control visibly says **Collapse** or **Expand** without rerunning optimization. A URL-backed toggle beside the result count enables shortlist filtering. Preferred Job and Only unemployed appear only when it is on. Filtering off uses the `staff-search` layout and normal metrics. Filtering on retains the `staff-shortlist` layout and presentation: All jobs uses configurable CSV metadata; mapped jobs use fixed identity and metadata plus their mapped score and current sort; Coach shows six outfield coaching scores without automatic score sorting; unrecognized jobs use CA behavior. Upload, configuration, optimizer setup and result feedback, immutable context guards, Staff Profile navigation, and row navigation retain their existing behavior.
- **My Staff:** `/staff?view=my-staff` shows every current-snapshot staff member whose club exactly matches the saved managed club. It has selected-club context, an independent table layout and sort state, no search filters, and one confirmed **Boost all CA** action. When no managed club is configured, it links to Club setup at `/my-club#managed-club`; it does not render the selector.
- **Navigation and states:** activating any Staff Search or My Staff row opens `/staff/$uid`, and browser Back returns to the originating table URL. `/staff?view=shortlist` and `/my-club?view=staff-shortlist` replace-normalize to `/staff` with shortlist filtering on. `/my-club?view=staff&staffSort&staffDir` replace-redirects to `/staff?view=my-staff&myStaffSort&myStaffDir`, preserving valid sort values and applying existing validator defaults to invalid values. No snapshot, no shortlist, empty, unavailable-score, loading, and error states use the shared patterns without page-level nested scrolling.

### Club DNA definition

My Club places **Define DNA** beside **Save managed club**. The action is enabled only when the current save has a settled managed-club context and the definition query has loaded successfully. One accessible form Modal owns the full closed catalog, grouped by the existing FM attribute sections. It shows the selected summary and formula, requires at least one attribute, and uses one Modal for create, edit, and remove confirmation. Editing keeps the selected definition until the user confirms removal. Pending, unavailable, error, and stale-context states disable unsafe actions and retain truthful feedback. The metric is fixed at 0–100; unavailable values render `—`. Creating a definition appends `club_dna` once to Search and Squad layouts. Editing or removing it does not restore removed columns, change URL filters or sorts, or change layout ownership.

### Squad workspace layout

Dedicated route `/my-club`; `/planner` is a replace compatibility redirect to `/my-club?view=planner`. The My Club route contains the Squad overview, phase-aware tactic editor, and squad-depth board in three URL-backed workspaces: **Squad**, **Planner**, and **Tactic**. Squad is the default for every save. The active workspace is exposed while Planner and Tactic remain mounted with direct hidden props so local drafts, table layouts, and selections survive workspace changes. An explicit valid `view` search value wins. Top-navigation workspace links use normal Link navigation, so Club destination changes add browser-history entries and browser Back returns to the prior destination. Same-route changes preserve `squadSort` and `squadDir`; route-local sort controls use replace navigation. The route uses the active save and current snapshot shown in the utility bar.

- **Managed club:** My Club shows one explicitly saved selector at the stable `/my-club#managed-club` target when a current snapshot exists. Its options are exact current-club names from the latest snapshot. The picker and **Save managed club** action share one responsive control row that wraps at narrow widths, while warning and error feedback remains below the row. A saved selection remains visible with a warning when a later snapshot no longer contains it; users replace it explicitly. Squad, Planner, Academy, and managed-club Staff derive membership from this one save-scoped selection. The integrated Staff Search shortlist remains save-owned and is not managed-club filtered. No attached-club or fuzzy-name inference exists.
- **Page header:** `My Club` as `headline-md`, the compact managed-club selector, supporting club context, and the current Club workspace share one wrapping header. The Planner board shows all enabled squads at once. The Tactic command bar has **IP** | **OOP** | **Both** view controls.
- **Squad overview:** a configured managed club shows one full-height, filter-free virtual table of its current-snapshot players. The initial columns are **Name**, **Age**, **Nationality**, **Height**, **CA**, **PA**, **Value**, and **Suggested Training**; the shared metric menu can add, remove, resize, and move any sortable Search metric while keeping at least one visible column. Absent values render `—`. CA descending is the default and sort stays in the route URL; column order and widths persist in the Squad layout independently from Search. The shared table requests bounded 50-row pages as the user scrolls, has no Previous / Next controls, keeps arrow-key traversal across page boundaries, and activates the full row or its name link to `/players/$uid`. An empty managed club explains that no current-snapshot players match.
- **Squad development:** the overview header has a primary **Boost all CA** action, a secondary **Make all Wonderkids** action, and the secondary CSV uploads. The CA confirmation explains the fixed age rule: +5 at age 20 or younger, +10 from age 21 through 28, and no boost from age 29. The Wonderkid confirmation explains that known Ambition, Professionalism, and Determination values at 10 or below receive a random 11–20 value; unknown and higher values stay unchanged. Both actions run sequentially, prevent duplicate or overlapping submission, and report Rust-derived determinate progress as `processed / total` after the cohort is captured and after each terminal player outcome. The confirmation stays open while the command is pending and shows an indeterminate preparing state before the first progress payload. Final feedback appears in one reserved Squad overview region for the latest action, uses compact processed/updated/skipped/failed copy, and does not move the action header. If the app cannot verify a result or preserve the active context, it stops, disables both actions, focuses the shared feedback region, and tells the user to use Load Data before another boost. A newly current snapshot restores the actions and clears prior feedback. Neither action claims that skipped, failed, or recovery-stopped players changed.
- **CSV uploads:** Search keeps **Upload Moneyball CSV**. My Club Squad offers **Upload Squad CSV** and **Upload Youth Academy CSV**. Each opens the shared format-bound Modal with a clear drop zone and keyboard-reachable **Browse files** action; it accepts exactly one CSV through either path and states the selected format. Pending, success, mismatch, and context feedback stay inside that modal, use text plus status icons, and never display a local path. Moneyball copy says that an upload adds or updates matched players and that omitted enrichment remains; it does not imply whole-cohort replacement. A format mismatch names the required export; changing the save or current snapshot clears feedback and closes the modal, returning focus to its action.
- **Tactic editor:** one tactic per app save, shared by all teams. One command bar shows the phase view, save status when present, and **Save tactic** action. A single normalized canvas serves IP, OOP, and Both views. Wide visual order is the persistent Tactical XI panel on the left, the pitch in the middle, and the persistent **Selected Slot** inspector on the right at the 320px inspector width, keeping both phase controls available in every view. Below 2xl, the XI panel stacks above the pitch; below the overall `lg` workspace breakpoint, the tactic workspace stacks its major regions. The complete workspace fits the tested desktop viewports without horizontal overflow.
- **Pitch geometry:** the editor starts from a 4-3-3 DM In-Possession shape linked to a 4-1-4-1 DM Out-of-Possession shape, with compatible general-purpose roles already selected. The normalized canvas uses portrait attack-up orientation below 2100px and a real clockwise 90-degree landscape orientation with attack to the right at or above 2100px. The workspace portrait canvas is centered at a capped width and taller than wide so the vertical pitch reads at 1920×1080; landscape keeps the full-width canvas and the role-reference modal keeps its compact portrait canvas. Marker labels stay upright, and visual tab order matches the current orientation. In Both view, each lane has distinct IP and OOP markers: IP markers carry a steel (`chart-2`) border, OOP markers a magenta (`chart-3`) border that stays dashed, and the dual badge keeps its phase edge under the primary selected treatment while label text stays on-surface; a connector appears only when the canonical full qualified placement identity changes, and the selected lane shows its readable IP role → OOP role transition. Qualified placements (`DCR` / `DC` / `DCL`, `DMCR` / `DM` / `DMCL`, `MCR` / `MC` / `MCL`, `AMCR` / `AMC` / `AMCL`, and `STCR` / `STC` / `STCL`) remain unique within each phase. The tactic workspace alone is centered and capped at 1920px on ultrawide displays; the global `content-max-width: none` contract is unchanged.
- **Selected Slot:** the inspector stretches to the row height and groups its controls into hairline-separated **Phase Influence**, **General Settings**, **In Possession (IP)**, and **Out of Possession (OOP)** sections with a full-width IP/OOP weight slider. It contains IP/OOP weight, optional importance rank from 1 through 11, preferred foot (**Either**, **Left**, **Right**, or **Both**), **Preferred** or **Strict** mode, and both phase position and role controls. Either foot disables the mode control. Selecting an occupied qualified placement swaps the two lanes in the edited phase only. Lane identity, non-placement settings, and compatible roles stay with their lanes; incompatible roles clear for re-selection. Invalid or incomplete role-position pairs cannot be saved. Failed-save draft retention, validation, read-only behavior, and **Save tactic** stay unchanged. The Best role fit Modal remains portrait and single-phase, with its side-by-side table and a widened pitch column of about 330px inside the 720px dialog.
- **Squad depth board:** one simultaneous board renders every enabled squad under a sticky tactical-slot band. The band shows the IP and OOP role and position context for each of the 11 slots. Each squad uses its configured display name and ordered string names. The board owns horizontal overflow from the 1280×800 minimum through ultrawide displays; the tactical-slot band stays sticky, and fixed-width strings and cards do not stretch. Each occupied cell aligns the player name with a compact, accessible `Current → Potential` combined-score pair. Empty cells show an explicit **Assign** action. Unresolved and outside-pool assignments keep their occupied cell and show a warning; unknown scores render as `—`.
- **Best role fit reference:** the Planner toolbar opens a read-only **Best role fit reference** Modal with the tactic pitch on the left and independently ranked player results on the right. **In Possession** and **Out of Possession** controls select the tactic phase; **Current** and **Potential** controls select the ranking basis and show both score columns for the selected lane. The right-hand table has sortable **Name**, **Current**, and **Potential** headers, retains the selected tactic lane when a valid result exists, and keeps players without an eligible selected-basis score in a separate section with unavailable values shown as `—`. Opening defaults to In Possession, Current, the first lane, and Current descending; changing phase or basis requests the corresponding read-only reference and never changes assignments or tactic data.
- **Player assignment:** activating an empty cell opens a picker Modal. Candidates must match the exact managed club; the same club pool is available to every Planner team. Results show player name, current club, IP score, OOP score, current combined score, and any existing planner location. Rust sorts by current combined score descending. Selecting an unassigned player fills the cell. Selecting an assigned player opens a confirmation that names the old and new locations; confirming moves the player. Activating an occupied cell instead asks the user to clear it. Closing restores focus to the originating cell.
- **String management:** Manage teams owns string add, remove, rename, and reorder for each enabled squad. Every squad keeps at least one string. Populated string removal uses a target-specific destructive confirmation. Names remain configured on the board, and renames and reorders preserve assignments.
- **Squad actions:** **Optimize squads** is the primary current-score action and **Optimize by potential** is a secondary explicit action. Both assign eligible, unassigned players across available teams and strings through the same Rust allocation and transaction path, then report basis-specific pending, one latest success, or failure without masking blank gaps. Only the score basis changes; managed-club eligibility, ranking, matching, foot preferences, manual reservations, replacement, rollback, and assignment provenance stay shared. The optimizer processes the shared club pool in Senior, Reserves, Youth order, enforces one assignment per player, and keeps the existing age limits for Reserves and Youth. **Manage teams** opens a form Modal for the fixed Senior, Reserves, and Youth categories. It lets the user keep one to three categories and set unique display names. Removing a category with assignments opens a destructive confirmation that names each affected display name and count; successful removal updates the board and returns focus to **Manage teams**. **Clear all** is destructive and requires confirmation that names the available display names. One Rust transaction removes both manual and optimized assignments from every string in the active save while preserving strings, tactics, managed-club settings, scores, and other saves. The actions stay in the shared toolbar, prevent duplicate submission, retain errors, restore focus after close, and reconcile the displayed board and picker candidates after success.
- **States:** no snapshot uses the standard Load Data EmptyState. No managed club shows recovery guidance that links to the My Club selector. A saved club absent from the latest snapshot shows a warning without deleting configuration or assignments. Loading skeletons match the board or pitch geometry. Tactic-save and assignment failures stay inline with the affected control and preserve the user's draft or prior assignment.
- **Accessibility:** pitches expose each tactical position as a labelled button with phase, position, and role in its accessible name. Position selection and placement changes work without drag, and the linked counterpart is described to assistive technology. Board cells have row and column context in their accessible names, and empty cells expose an explicit Assign action. The sticky tactical-slot band preserves slot context during horizontal scrolling.
- **Data honesty:** Current combined scores use persisted IP and OOP role scores from the current snapshot. Potential combined scores use projected visible attributes and the same lane IP/OOP roles and weight. Missing phase or projected-required attributes produce `—`, not a partial or zero score. Imported FM team level remains nullable metadata and does not restrict Planner eligibility. Settings does not expose its diagnostic count because users cannot act on it. A player outside the managed club or absent from the snapshot remains assigned but carries a visible warning.

### Youth academy layout

Dedicated route `/academy`. The page tracks save-scoped youth cohorts against the managed club selected in My Club; it does not repeat club selection or import data from files.

- **Page header:** `Youth Academy` as `headline-md` and the managed club as secondary context. The Overview Classes panel owns the **Create class** primary action. The create Modal uses a labelled positive year input, prefilled from the current snapshot's in-game year when available, and previews `Class of YYYY` before submission.
- **Workspaces:** a compact, URL-backed control switches between **Overview**, **Graduates**, and an opened **Class** workspace. Class cards open the Class workspace instead of creating one tab per year, so a long-running save does not overflow the header. Invalid or deleted class selections return to Overview.
- **Overview:** a primary outcome group gives Graduates, Academy income, Released players, Goals, Assists, and International caps the largest type, generous card spacing, and restrained Lucide iconography. A quieter context group keeps Classes, Tracked players, and Reported senior visible without competing with outcomes. Reader-owned aggregates show `—` with an unavailable-memory explanation until their source fields exist. When details load successfully, manual sale income and released totals remain known zeroes when no outcome is recorded. A reported Senior squad count may include only resolved members whose current snapshot explicitly says `team_level = senior`; its label or help text must make that limitation clear. Class cards are ordered oldest year first by numeric class year and show the class year, tracked count, reported senior count, and only supported outcome highlights.
- **Graduates:** explain that one senior league appearance makes a player a graduate. While appearances are unavailable, render an intentional unavailable state rather than an empty-results claim or zero count. Once the typed field is populated, show only members with at least one appearance.
- **Class workspace:** show the class title, tracked count, **Add players**, and destructive **Delete class** action. A compact semantic table contains name, age, nationality, positions with familiarity 16 or higher ordered strongest-first, club, PA, determination, height, preferred foot, Apps, goals, assists, caps, Fee, and Actions. Familiarity 15 or lower, unread, and legacy-missing position slots are omitted from the label. The Actions cell keeps labelled **Sell**, **Release**, and **Remove** controls visible, with success, warning, and destructive treatments; Release becomes **Restore** for an already released player, and the Sell edit dialog can restore a recorded sale. Unsupported or unknown values render `—`; do not substitute stars for PA or synthesize a personality label. The table owns horizontal overflow and preserves a readable Name column at 1280×800.
- **Player assignment:** **Add players** opens a searchable Modal restricted by the Rust service to current-snapshot players at the exact managed-club name. Already classified players are absent. Rows show name, age, current club, and positions with familiarity 16 or higher in the same order as the class workspace. Selection adds the player to this class; removal is available from the roster. Closing restores focus to the originating control.
- **Persistence states:** members who leave the managed club or disappear from the current snapshot stay listed with their UID and last-known name. A visible text-and-icon warning distinguishes departed and unresolved records; neither condition deletes or reclassifies the player.
- **Empty and error states:** no snapshot uses the standard Load Data EmptyState. No managed club points to `/my-club#managed-club`. A configured Academy with no classes offers Add class; an empty class offers Add players. Mutation errors stay with the triggering Modal or row and retain recoverable input.
- **Accessibility and data honesty:** workspace controls use tab semantics, tables retain header associations, destructive dialogs name their target, and all actions work without drag. Null career values remain unavailable rather than becoming zero, no, sold, released, or graduated. Lucide supplies every icon; the reference tracker's emoji, fonts, and visual styling are not reused.

### Deferred specs

These surfaces are not specced because their features are not planned yet. Spec them in this document during `/skill:workflow-plan-feature` for the relevant feature, not before.

- **Optimizer extensions:** formation comparison, best-and-worst candidate highlighting, and gap recommendations.
- **Profile extensions:** position suitability map, attribute/role radar, comparison inspector, snapshot history on the profile.

---

## Pre-Delivery Checklist

Verify before delivering any UI code.

### Visual Quality

- [ ] No emojis used as icons — Lucide only; `strokeWidth` 1.5 by default and 2 for active top-navigation links; `currentColor`
- [ ] UI icons use Lucide at the component's documented size (12 / 16 / 20 / 24px); identity marks, graphics, and nationality flags follow their separate asset rules
- [ ] No raw colour values in components — every colour comes from a token
- [ ] Colour is never the sole indicator of meaning — paired with text, icon, or shape
- [ ] All text-on-background combinations meet the contrast minimum (verify against the table in Colors; compute new pairings before use)
- [ ] Score badges always render their number, and their accessible name includes the role and tier
- [ ] Grouping uses restrained tonal steps and hairlines; meaningful control and graphical boundaries meet 3:1 against their actual adjacent colours
- [ ] Signal Green marks brand/chrome emphasis or the subject chart series; numeric score tiers use the separate data ramp

### Typography & Numbers

- [ ] Numeric columns use `tabular-nums` and right alignment; standalone score circles can be centered; raw table cells use 13px/500 sans and compact scored units use `font-mono text-mono-sm`
- [ ] Money, scores, attributes, ages, and dates go through the shared formatter — no inline formatting
- [ ] Missing values render as `—`, never `null`, `N/A`, `0`, or an empty cell
- [ ] Names that can overflow use `truncate` plus a `title`; table cells never wrap
- [ ] No all-caps outside the `label-*` roles

### Interaction

- [ ] `cursor-pointer` on all clickable elements
- [ ] Hover changes colour or opacity only — no scale, margin, padding, size, or font-weight change
- [ ] Focus states visible only via `:focus-visible`, never `:focus`, and never removed
- [ ] Every mutation shows a phase-specific pending label, then success or error — no silent updates
- [ ] Load Data errors distinguish the scan phase from the ingest phase
- [ ] Destructive actions require explicit confirmation, and the modal names the affected object

### Accessibility

- [ ] Skip link present and functional on the first Tab press
- [ ] All interactive elements reachable by keyboard in logical Tab order
- [ ] Tables use `<caption>`, `<th scope>`, `aria-sort`, and arrow-key row navigation
- [ ] Modals trap focus, restore focus to the trigger, and dismiss on Escape
- [ ] `prefers-reduced-motion: reduce` respected — transforms and entrance animations disabled
- [ ] Charts have an equivalent table or `sr-only` data representation

### Z-Index & Layout

- [ ] All `z-index` values come from the scale (10 / 20 / 30 / 40 / 50) — no arbitrary values
- [ ] No content hidden behind the sticky top bar or sticky table header
- [ ] Layout holds at the 1280×800 minimum window with the filter editor closed (Search) or inspector open (profile)

### States & Data Honesty

- [ ] Loading, empty, and error states defined for every data view — never blank space
- [ ] Active save and current in-game date visible without scrolling on every data view
- [ ] Truncated snapshots carry a warning wherever their data appears, with the cap named
- [ ] Loading skeletons match the final layout at the real row height
- [ ] Toast auto-dismiss timers pause on hover and focus; error toasts do not auto-dismiss
- [ ] No network request for a font, icon, or image
