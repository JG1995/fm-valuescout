---
name: FM ValueScout
colors:
    # Foundation — background & surface elevation layers
    background: "oklch(0.15 0.008 264)"
    on-background: "oklch(0.95 0.004 264)"
    surface-dim: "oklch(0.1 0.008 264)"
    surface: "oklch(0.22 0.009 264)"
    surface-bright: "oklch(0.3 0.01 264)"
    surface-container-lowest: "oklch(0.1 0.008 264)"
    surface-container-low: "oklch(0.19 0.008 264)"
    surface-container: "oklch(0.22 0.009 264)"
    surface-container-high: "oklch(0.26 0.01 264)"
    surface-container-highest: "oklch(0.3 0.01 264)"
    on-surface: "oklch(0.95 0.004 264)"
    on-surface-variant: "oklch(0.74 0.008 264)"
    inverse-surface: "oklch(0.95 0.004 264)"
    inverse-on-surface: "oklch(0.22 0.009 264)"
    # Borders & outlines
    outline: "oklch(0.58 0.01 264)"
    outline-variant: "oklch(0.32 0.01 264)"
    surface-tint: "oklch(0.8 0.145 82)"
    # Primary — main interactive and brand colour (floodlight gold)
    primary: "oklch(0.8 0.145 82)"
    on-primary: "oklch(0.18 0.02 82)"
    primary-container: "oklch(0.34 0.07 82)"
    on-primary-container: "oklch(0.92 0.06 82)"
    inverse-primary: "oklch(0.52 0.1 82)"
    # Filled-button states — the Button spec's 8% mixes, resolved in oklab
    primary-hover: "oklch(0.812 0.133 82)"
    primary-active: "oklch(0.748 0.133 82)"
    # Semantic — status indicators
    success: "oklch(0.76 0.16 150)"
    on-success: "oklch(0.16 0.03 150)"
    success-container: "oklch(0.34 0.085 150)"
    on-success-container: "oklch(0.92 0.07 150)"
    warning: "oklch(0.76 0.165 55)"
    on-warning: "oklch(0.16 0.03 55)"
    warning-container: "oklch(0.34 0.085 55)"
    on-warning-container: "oklch(0.92 0.04 55)"
    error: "oklch(0.66 0.2 18)"
    on-error: "oklch(0.16 0.03 18)"
    error-container: "oklch(0.34 0.115 18)"
    on-error-container: "oklch(0.92 0.035 18)"
    info: "oklch(0.72 0.11 245)"
    on-info: "oklch(0.16 0.02 245)"
    info-container: "oklch(0.34 0.08 245)"
    on-info-container: "oklch(0.92 0.035 245)"
    # FM-style data ramp — red, grey, amber, and green
    score-1: "oklch(0.66 0.2 18)"
    score-2: "oklch(0.74 0.008 264)"
    score-3: "oklch(0.8 0.145 75)"
    score-4: "oklch(0.76 0.16 150)"
    # Chart series — subject, two comparisons, one reference line
    chart-1: "oklch(0.8 0.145 82)"
    chart-2: "oklch(0.72 0.11 245)"
    chart-3: "oklch(0.68 0.18 340)"
    chart-4: "oklch(0.62 0.01 264)"
typography:
    headline-lg:
        {
            fontFamily: "IBM Plex Sans",
            fontSize: 28px,
            fontWeight: "600",
            lineHeight: "1.25",
            letterSpacing: -0.01em,
        }
    headline-md:
        {
            fontFamily: "IBM Plex Sans",
            fontSize: 22px,
            fontWeight: "600",
            lineHeight: "1.3",
        }
    headline-sm:
        {
            fontFamily: "IBM Plex Sans",
            fontSize: 18px,
            fontWeight: "500",
            lineHeight: "1.35",
        }
    body-lg:
        {
            fontFamily: "IBM Plex Sans",
            fontSize: 16px,
            fontWeight: "400",
            lineHeight: "1.6",
        }
    body-md:
        {
            fontFamily: "IBM Plex Sans",
            fontSize: 14px,
            fontWeight: "400",
            lineHeight: "1.5",
        }
    body-sm:
        {
            fontFamily: "IBM Plex Sans",
            fontSize: 13px,
            fontWeight: "400",
            lineHeight: "1.4",
        }
    label-lg:
        {
            fontFamily: "IBM Plex Sans",
            fontSize: 14px,
            fontWeight: "600",
            lineHeight: "1.2",
            letterSpacing: 0.02em,
        }
    label-md:
        {
            fontFamily: "IBM Plex Sans",
            fontSize: 12px,
            fontWeight: "600",
            lineHeight: "1.2",
            letterSpacing: 0.05em,
        }
    label-sm:
        {
            fontFamily: "IBM Plex Sans",
            fontSize: 11px,
            fontWeight: "600",
            lineHeight: "1.2",
            letterSpacing: 0.08em,
        }
    # Monospace roles for numeric/tabular data
    mono-lg:
        {
            fontFamily: "IBM Plex Mono",
            fontSize: 24px,
            fontWeight: "600",
            lineHeight: "1.2",
        }
    mono-md:
        {
            fontFamily: "IBM Plex Mono",
            fontSize: 14px,
            fontWeight: "500",
            lineHeight: "1.4",
        }
    mono-sm:
        {
            fontFamily: "IBM Plex Mono",
            fontSize: 12px,
            fontWeight: "500",
            lineHeight: "1.4",
        }
rounded:
    none: 0
    sm: 0.25rem
    DEFAULT: 0.5rem
    md: 0.5rem
    lg: 0.75rem
    xl: 1rem
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

## Brand & Style

**The central concept is a floodlit pitch at night: a dark field, and gold light on the thing worth looking at.**

FM ValueScout is an instrument, not a destination. The user already has Football Manager open on the same machine, and probably a second monitor. They alt-tab in with a question — *who fills this role best right now?* — and they want the answer in the first second of looking. Every pixel that is not an answer is in the way. The app is a quiet dark surface holding a lot of numbers, with gold reserved for two jobs: marking where you are, and marking what is good.

The mood is a night-shift control room. Cool near-black surfaces, hairline separations, dense rows, and one warm accent. This is deliberately not the friendly pastel dashboard look: the primary user is a single expert reading their own data for an hour at a time, so the design optimizes for sustained scanning over first-run charm. The tension to hold is **dense but not cramped** — 36px rows and 13px text are tight, so the spacing scale and hairline borders must do the separating work that whitespace usually does.

Hard stances:

- **Dark only.** There is no light theme and no `prefers-color-scheme` branch. FM runs full-screen and dark; a bright companion window beside it is hostile.
- **Desktop only.** Minimum window 1280×800, designed at 1600×900. No mobile or narrow breakpoints. ([CONCEPT.md](./CONCEPT.md) excludes mobile and web clients.)
- **Offline by construction.** No webfont CDN, no icon CDN, no remote image, no analytics beacon. Every asset ships in the bundle. This follows the offline-first principle in [CONCEPT.md](./CONCEPT.md), and it is a design constraint, not only an infrastructure one.
- **Text-first identity.** The FM26 dump supplies clubs and nationalities as strings and no crests or portraits (`bridge/DUMP_SCHEMA.md`). Player tables may render a nationality string as a bundled SVG flag only after an explicit FM-name mapping; unknown values stay visible as text, and the app never guesses a flag. Where a reference design would place an avatar, use an initials monogram on `surface-container-high`, or omit the slot and give the name more room. This remains a data constraint, not a style preference.
- **No decorative imagery.** No hero art, no illustration, no stock photography.

## Colors

The palette is one warm chrome accent on a cool near-neutral base, plus four semantic status colours and one multi-hue data ramp. Elevation is carried by **tonal layering plus hairline borders**, not by shadows. Dark surfaces swallow shadows, and the app stacks a lot of panels; a tonal step reads reliably at any brightness setting where a drop shadow does not. Shadows appear at one level only — floating overlays.

The neutrals carry a whisper of blue (hue 264, chroma 0.008–0.010). That is barely perceptible on its own, but it keeps the greys from looking dead and it sets up the complementary tension with the gold accent.

**Primary — floodlight gold (hue 82):** `primary` marks **chrome state**: the active nav item, the primary button, the focus ring, the selected row indicator, checked controls, and the subject series in a chart. It answers "where am I, and what is the main action here?" Gold also carries the product idea — ValueScout is about spotting value, and gold is what value looks like.

**Steel (hue 245):** `info` is the single cool counterpoint. It carries neutral factual annotation that is neither good nor bad: transfer-status tags, "U-21" style qualifiers, informational banners, and the first comparison series in a chart. There is no separate `secondary` token; steel does that work.

**Score ramp:** `score-1` through `score-4` colour role fit and player-profile attributes with Football Manager's familiar red, grey, amber, and green progression. The number and tier label remain the facts; colour makes the four broad bands faster to scan.

| Tier      | Score  | Label     | oklch                    | Meaning                         |
| --------- | ------ | --------- | ------------------------ | ------------------------------- |
| `score-1` | 0–40   | Weak      | `oklch(0.66 0.2 18)`     | Does not suit this role         |
| `score-2` | 41–60  | Average   | `oklch(0.74 0.008 264)`  | Emergency or fringe cover       |
| `score-3` | 61–80  | Good      | `oklch(0.8 0.145 75)`    | Viable squad or starting option |
| `score-4` | 81–100 | Excellent | `oklch(0.76 0.16 150)`   | High-confidence role fit        |

Player- and staff-profile attributes use the same colours with FM-scale bands: 1–5 Weak, 6–10 Average, 11–15 Good, and 16–20 Excellent. The raw value remains visible, and the colour never replaces it. **`primary` never appears inside a data cell, and the score ramp never appears on chrome.**

**Semantic Colours:** Four fixed roles for status indicators.

| Semantic  | oklch                    | Role                                                                        |
| --------- | ------------------------ | --------------------------------------------------------------------------- |
| `success` | `oklch(0.76 0.16 150)`   | Load Data completed, bridge plugin installed and current, snapshot is fresh  |
| `warning` | `oklch(0.76 0.165 55)`   | Snapshot truncated at the scan cap, snapshot is stale, plugin update pending |
| `error`   | `oklch(0.66 0.2 18)`     | Scan failed, ingest failed, FM not running, destructive confirmation         |
| `info`    | `oklch(0.72 0.11 245)`   | Neutral annotation and explanatory banners                                   |

Warning sits at hue 55 (orange) rather than amber so it never reads as the gold accent. The data ramp uses separate token names even where its red, grey, and green reuse established system colours; component code still states whether colour carries status or a score band.

`primary-hover` and `primary-active` are the Button spec's hover and active mixes resolved once, in oklab, rather than recomputed per component. Both stay in sRGB gamut and hold an `on-primary` label above 8:1. Unfilled variants have no mix — they press to `surface-container-highest`, one tonal step above their hover fill.

The template's `tertiary` role and the fixed tonal pairs (`primary-fixed`, `secondary-fixed`, `tertiary-fixed`, and their `on-*` partners) are removed on purpose. Nothing in this design needs a third accent or a tint that stays constant across themes, and there is only one theme. Do not restore them without a component that requires them.

Borders come in two roles with different rules:

- `outline` bounds **interactive** components — inputs, selects, secondary buttons, checkboxes. It must clear 3:1 against whatever surface it sits on, because the border is the only thing that shows the control exists.
- `outline-variant` is the **decorative** hairline for table row separators, card edges, and section rules. It is deliberately near-invisible (1.36:1) so it contains the data without competing with it. It is exempt from the 3:1 rule because it never carries meaning on its own.

### Accessibility of Colour

**Colour is never the sole indicator of meaning.**

- **Score badges** always render the number. The tier colour is redundant encoding that speeds scanning; the number is the fact. A tier label is available in the badge `title` and in the accessible name.
- **Status chips and banners** pair colour with an icon and a text label — never a bare coloured dot.
- **Active nav item** pairs `primary-container`, a `primary` icon, a reinforced label, and `aria-current="page"`; the same Lucide component uses `strokeWidth` 2 when active and 1.5 when inactive.
- **Selected table row** pairs the tint with a 2px gold left indicator and `aria-selected`.
- **Chart series** differ by colour *and* stroke pattern — solid, dashed, dotted — plus a direct label or legend entry. A radar chart with three overlaid players is unreadable by colour alone at any palette.
- **Trend arrows** carry direction as shape (up, down, flat), with colour as reinforcement.

**Contrast compliance:** target WCAG 2.2 AA for all text (4.5:1 minimum) and 3:1 for interactive component boundaries and graphical objects. Body and secondary text clear AAA (7:1) on every surface in the stack. Ratios below are computed from the token values in the frontmatter against sRGB.

| Text Role                 | Foreground                                  | Background                                       | Ratio    |
| ------------------------- | ------------------------------------------- | ------------------------------------------------ | -------- |
| Body text                 | `on-surface` (`#edeef1`)                    | `background` (`#090b0f`)                         | 17.0:1 (AAA) |
| Body text                 | `on-surface` (`#edeef1`)                    | `surface-container` (`#181b1f`)                  | 15.0:1 (AAA) |
| Body text on overlay      | `on-surface` (`#edeef1`)                    | `surface-container-highest` (`#2b2e33`)          | 11.8:1 (AAA) |
| Secondary text            | `on-surface-variant` (`#a8abb0`)            | `surface-container` (`#181b1f`)                  | 7.5:1 (AAA) |
| Secondary text on overlay | `on-surface-variant` (`#a8abb0`)            | `surface-container-highest` (`#2b2e33`)          | 5.9:1 (AA) |
| Accent text and icons     | `primary` (`#ecb33c`)                       | `surface-container` (`#181b1f`)                  | 9.2:1 (AAA) |
| Primary button label      | `on-primary` (`#161107`)                    | `primary` (`#ecb33c`)                            | 10.0:1 (AAA) |
| Destructive button label  | `on-error` (`#180808`)                      | `error` (`#f44f62`)                              | 5.7:1 (AA) |
| Score tier 1 (weakest)    | `score-1` (`#f44f62`)                       | `surface-container` (`#181b1f`)                  | 5.1:1 (AA) |
| Score tier 1 on hover     | `score-1` (`#f44f62`)                       | `surface-container-high` (`#222429`)             | 4.5:1 (AA) |
| Score tier 2              | `score-2` (`#a8abb0`)                       | `surface-container` (`#181b1f`)                  | 7.5:1 (AAA) |
| Score tier 3              | `score-3` (`#f4af41`)                       | `surface-container` (`#181b1f`)                  | 9.1:1 (AAA) |
| Score tier 4 (strongest)  | `score-4` (`#58cd78`)                       | `surface-container` (`#181b1f`)                  | 8.6:1 (AAA) |
| Error text                | `error` (`#f44f62`)                         | `surface-container` (`#181b1f`)                  | 5.1:1 (AA) |
| Warning text              | `warning` (`#ff9138`)                       | `surface-container` (`#181b1f`)                  | 7.7:1 (AAA) |
| Success text              | `success` (`#58cd78`)                       | `surface-container` (`#181b1f`)                  | 8.6:1 (AAA) |
| Banner text               | `on-error-container` (`#fbdcdc`)            | `error-container` (`#661420`)                    | 9.7:1 (AAA) |
| Control border            | `outline` (`#777a80`)                       | `surface-container` (`#181b1f`)                  | 4.0:1 (3:1 UI) |
| Control border on overlay | `outline` (`#777a80`)                       | `surface-container-highest` (`#2b2e33`)          | 3.2:1 (3:1 UI) |

Every score tier clears 4.5:1 on both the default and the hovered row background, so a score is legible as text at every tier. When a new token or pairing is added, verify it before use — do not assume a value passes because a neighbouring one does.

## Typography

Two families, one superfamily. **IBM Plex Sans** does everything a human reads as language; **IBM Plex Mono** does the numbers that need to line up or be read character by character.

- **IBM Plex Sans (grotesque, weights 400/500/600):** UI, headings, labels, player names, prose. Chosen because it was designed for dense interface use, holds its shape at 11–13px, and ships a metrically related mono, so the two families never fight. Its low-contrast, slightly technical letterforms suit an instrument. It is not a friendly geometric sans, and that is the choice: this is a tool for one expert user, not a landing page.
- **IBM Plex Mono (weights 500/600):** score badges, hero metrics, game version strings, file paths, and bridge diagnostics. Monospace digits are wide, so mono is *not* used for in-table figures — it is reserved for values read as single units or as literal text.

**Scale principle:** headlines are rare — a page title and at most one section title per screen. Most of the app is `body-sm` (13px) in table cells, `body-md` (14px) in prose and controls, and the three `label-*` roles for uppercase micro-labels above values. The micro-label pattern is the workhorse: an 11px uppercase letterspaced label in `on-surface-variant` sitting above a 13–14px value in `on-surface`. It packs a labelled field into two tight lines without a colon or a box.

Numeric rules:

- Apply `font-variant-numeric: tabular-nums` to every numeric table column, score, and metric so digits align in a column and do not jitter when values update.
- Never set body copy in all-caps. Uppercase is for `label-*` roles only, at 11–14px, always with the letterspacing from the token.
- Use `text-wrap: pretty` on prose blocks. Truncate names in fixed-width cells with an ellipsis and a `title` attribute. Text must never wrap inside a table cell — the two-line table variant stacks two separate elements at a fixed row height, which is not the same thing as letting a value wrap.

**Loading:** self-host both families in the bundle via `@fontsource/ibm-plex-sans` and `@fontsource/ibm-plex-mono`, importing the per-weight entrypoints (`400.css`, `500.css`, `600.css` for Sans; `500.css`, `600.css` for Mono). No Google Fonts link, no CDN — the app must render identically with no network, per the offline stance above.

Ship **every latin, cyrillic, greek, and vietnamese subset**, not a latin-only cut. FM's playable leagues include Russia, Ukraine, Greece, Serbia, and Bulgaria, so names outside latin are ordinary data, and a missing glyph in a scouting database is a data error the user cannot distinguish from a bug (`Magalhães`, `Håland`, `Şahin`, `Дзюба`, `Παυλίδης`). Bundle weight is not a constraint for a local desktop app. Use the per-weight entrypoints rather than the per-subset ones (`latin-400.css`): only the per-weight files carry `unicode-range`, so combining per-subset files leaves two identical `@font-face` descriptors and the browser silently keeps one.

Font stacks:

```css
--font-sans: "IBM Plex Sans", system-ui, sans-serif;
--font-mono: "IBM Plex Mono", ui-monospace, monospace;
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

- **Role and position scores:** integer 0–100, no unit, no percent sign. `mono-md` in a badge, tabular sans in a table column.
- **Linear position order:** list positions from the goalkeeper band toward the striker band and from the player's right to left within each band. Familiarity-ranked lists keep the strongest value first and use this pitch order for ties. Linked Planner rows use the IP position as their primary order. Visual pitch grids keep their attack-up geometry and left-to-right DOM order.
- **FM attributes:** integer 1–20. **CA and PA:** integer 1–200. Both as raw integers — never rescaled to 0–100, because the user knows the FM scale.
- **Age:** integer. Where both are shown, birth date first and age in parentheses: `21/03/2001 (25)`.
- **Snapshot timestamps:** relative in the UI (`4 min ago`, `2 hours ago`, `yesterday`), absolute ISO-like in the `title` attribute (`2026-07-29 20:14 UTC`). Relative age is what tells the user whether to reload.
- **In-game dates:** exactly as FM reports them. Do not reformat or localize game dates.
- **Percentages:** one decimal maximum, `%` suffix, no space: `62.5%`.
- **Missing values:** an em dash `—` in `on-surface-variant`. Never `null`, `N/A`, `0`, or an empty cell. Absent data and zero are different facts.
- **Truncated counts:** never show a total from a truncated scan without the cap. Render `1,247 players (scan capped)` with the warning chip, not a bare count.
- **Alignment:** numeric columns right-aligned, text columns left-aligned, single-glyph columns centred.

## Design Principles

Seven constraints. Every component and screen satisfies all of them.

1. **Data outranks chrome.** No decorative element may take space a data column could use. *Test:* on the player search screen at 1600×900 with the filter editor closed, the results table covers at least 70% of the window area.
2. **Separate with hairlines, not boxes.** Rows, fields, and sections are divided by a 1px `outline-variant` rule or a tonal step. *Test:* no nested card inside a card, and no vertical rules between table columns.
3. **Snapshot provenance is always visible.** Every screen that shows player data states which save is active and how old the snapshot is, without scrolling. Truncated and stale snapshots carry a warning wherever their data appears. *Test:* screenshot any data view and you can name the save and the snapshot age from the image alone. This follows the explicit-refresh principle in [CONCEPT.md](./CONCEPT.md) — the user must never mistake old data for current data.
4. **Brightness carries value; the number carries the fact.** Score meaning comes from the ramp, and the number is always present. *Test:* convert a screenshot to greyscale — the ranking still reads.
5. **Every mutation reports its phase.** Long operations name what they are doing and which stage failed. Load Data distinguishes a scan failure from an ingest failure, because the fixes differ: start FM versus retry the ingest. *Test:* every mutation has a pending label, a success state, and a phase-specific error message.
6. **Keyboard reaches everything; hover reveals nothing.** Hover may only change colour. Any action or information available on hover is also available from the keyboard and visible without a pointer. *Test:* complete a full search-to-profile pass with the keyboard alone.
7. **Nothing loads from the network.** Fonts, icons, and images ship in the bundle. *Test:* run the app with networking disabled and no glyph, icon, or layout changes.

## Layout & Spacing

The app is a **single window with a utility bar and grouped top navigation** — a desktop tool, not a set of pages. Minimum 1280×800; the top navigation fit is proven at 1280×800. Content fills the window width; `content-max-width` is `none` because a clamped column wastes the space a 20-column player table needs.

Regions, in visual order:

1. **Utility bar** (`header-height` 56px). It contains Back and Forward, global player search, the active save selector, snapshot freshness, optional **Cap players** controls, and **Load Data**. It stays first so global controls remain separate from destination navigation.
2. **Top navigation** follows the utility bar. It groups icon-plus-label links as Home (Dashboard), Players (Search and Moneyball), Staff (Staff Search and My Staff), Club (Squad, Planner, Tactic, and Youth), and Settings. Fine vertical separators divide groups. Each group has a low-emphasis caption under its links. The active link uses a contained state with a reinforced label.
3. **Page header** (inside the content area). Page title in `headline-lg`, then local controls on the right. One row, `stack-md` below it.
4. **Content area.** Panels on `surface-container` with `gutter` 16px between them and 16px page padding.
5. **Inspector** (right, `inspector-width` 320px, optional and dismissible). Comparison and detail controls on a profile. Slides over the content edge; never squeezes the table below its usable width. **Search does not use the inspector for filters** — filters use the compact strip and editor modal below.

Spacing rhythm, all multiples of the 4px `unit`:

- `stack-xs` (4px) — between a micro-label and its value; inside a chip.
- `stack-sm` (8px) — between related controls in a row; cell padding in a dense table.
- `stack-md` (16px) — default panel padding, gutter between panels, gap between form fields.
- `stack-lg` (24px) — between distinct sections inside one panel.
- `stack-xl` (32px) — above a major screen division. Rare.

Hard dimensions: headers are `table-header-height` 32px. Single-line rows are `table-row-height` 36px; rows with a stacked secondary line (name over birth date) are `table-row-height-two-line` 40px, which is what two lines of 13px and 11px text plus padding actually need. A table picks one row height for all its rows — ragged row heights make a long list unreadable and break virtualization. Player cards in grid view are minimum 260px wide in an auto-fill grid.

## Elevation & Depth

Depth is tonal. Each level is a lighter surface than the one below it, and **every level boundary that matters also carries a 1px `outline-variant` border**. This second part is not optional: adjacent steps in the dark end of the ramp are only 1.05:1 to 1.14:1 apart, which is a real but subtle difference. The tonal step gives the impression of depth; the hairline makes the boundary unambiguous.

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

The shape language is **Rounded-Instrument**: containers are softly rounded rectangles, and chrome controls are full pills. The split is the rule — if it holds data, it is a rounded rectangle; if you click it to change state, it is a pill.

- **Panels, cards, tables:** `lg` (0.75rem / 12px).
- **Modals, inspector panel, overlays:** `xl` (1rem / 16px).
- **Inputs, selects, secondary buttons, menu items, checkboxes:** `md` (0.5rem / 8px). `DEFAULT` is set to the same 0.5rem so a bare `rounded` cannot land off-scale.
- **Pills — global search, segmented toggles, chips, filter tags, primary action buttons:** `full`.
- **Square score badges and small tags inside a cell:** `sm` (0.25rem / 4px). Circular score badges use `full`.
- **Table rows:** `none`. Rows are separated by hairlines, not individually rounded; rounding is on the table container only.
- **Focus rings:** 2px solid `primary` at 2px offset, matching the element's own radius. `:focus-visible` only, never `:focus`. Never removed and never replaced by a colour change alone.

## Components

Each spec below is the contract for that component. Reference tokens by name; never put a raw colour value in a component.

### Button

The action primitive. One primary action per screen region.

- **Container:** `full` radius, `stack-sm` vertical and 16px horizontal padding, 32px height (36px for the top-bar Load Data button), `label-lg` text. Icon-only variant is a 32×32 square with `md` radius.
- **States:** hover takes `primary-hover` on a filled variant and fills an unfilled variant with `surface-container-high`; active takes `primary-active` on a filled variant and `surface-container-highest` on an unfilled one; `:focus-visible` adds the 2px gold ring; disabled drops opacity to 45% and sets `cursor: not-allowed`; loading disables the button, swaps the label for a phase-specific pending label ("Scanning…", "Saving…"), and shows a spinner in the leading icon slot. Transition `background-color 150ms ease-out`. Width never changes between states — reserve the loading label's width, and keep the inactive label `aria-hidden` so it stays out of the accessible name.
- **Variants:** `primary` — `primary` fill, `on-primary` label; the one main action. `secondary` — transparent fill, `outline` border, `on-surface` label. `ghost` — no fill or border, `on-surface-variant` label, hover fills `surface-container-high`; for toolbar and icon actions. `destructive` — `error` fill, `on-error` label; requires a confirmation modal before it executes. Snapshot and save deletion use this variant only after the target-specific destructive Modal confirms the cascade.
- **Content / Anatomy:** optional 16px leading icon, label in `label-lg`, optional trailing chevron for menu buttons. Never icon-plus-text in the icon-only variant.
- **Behaviour:** always a `<button>` with an explicit `type`. Icon-only buttons carry `aria-label` and a tooltip — the props type requires both an icon and an `aria-label` for that size, so an unlabelled icon button does not compile. A button that opens a menu sets `aria-expanded` and `aria-haspopup`.

### Top Navigation

Primary navigation between the app's main destinations.

- **Container:** `surface-container`, one row below the utility bar, with a bottom `outline-variant` border. Groups use fine vertical separators and low-emphasis captions below their links. The layout fits all destinations at the supported 1280×800 minimum window.
- **States:** links use Lucide icons and ValueScout tokens. Hover changes colour only. The active link uses `primary-container`, a `primary` icon, and a reinforced label. `:focus-visible` shows the gold ring inside the link bounds.
- **Content / Anatomy:** **Home** contains Dashboard; **Players** contains Search and Moneyball; **Staff** contains Staff Search and My Staff; **Club** contains Squad, Planner, Tactic, and Youth; **Settings** contains Settings. Search and Moneyball select `/search?view=general|moneyball`; Staff links select `/staff?view=search|my-staff`; Club links select `/my-club?view=squad|planner|tactic`; Youth selects `/academy`.
- **Behaviour:** a `<nav>` contains router links. Each supported direct destination sets exactly one link to `aria-current="page"`. Unknown and not-found routes set no destination current. Player and staff profile routes set only the Players or Staff group caption to `aria-current="location"`; no child destination is current. Top-navigation Club destination changes use normal Link navigation, add a browser-history entry, and let browser Back return to the prior destination. Same-route Club changes retain `squadSort` and `squadDir`; route-local sort controls use replace navigation. Search view changes retain only the route's existing shortlist/combine state. Profile analysis tabs, Youth tabs, and Planner team tabs remain local.

### Utility Bar

Global search, save context, snapshot freshness, and the Load Data action.

- **Container:** `surface-container`, `header-height` 56px, 1px `outline-variant` bottom border, 16px horizontal padding, sticky at `z-10`.
- **States:** static. Its children carry their own states. When no snapshot exists for the active save, the freshness chip reads "No data loaded" in `on-surface-variant` and Load Data is the only emphasized element on screen.
- **Variants:** none.
- **Content / Anatomy:** **Back** and **Forward** icon buttons, global search field (pill, grows), save selector (`secondary` menu button showing the active save name), snapshot freshness chip (`label-md` relative age; `success` under 30 minutes, `on-surface-variant` under 6 hours, `warning` beyond that or when the scan was truncated), **Cap players** checkbox with numeric limit field (visible when on; default 500 when enabling), Load Data (`primary` button).
- **Behaviour:** the search field takes focus on `Ctrl+K` from anywhere. Back and Forward use only the current TanStack Router session history. They are disabled at the reachable history boundaries, and a new navigation after Back removes Forward availability. They do not persist history, own separate scroll state, or add custom scroll restoration. TanStack Router's existing scroll restoration remains authoritative. Switching saves swaps all snapshot-scoped views and suppresses any stale Load Data progress or outcome from a prior invocation. Load Data is an async Tauri command with command-scoped best-effort `Channel`; the frontend hook captures the active save ID/token and `api/load-data.ts` constructs the `Channel`; Rust verifies the supplied `saveId`/`contextToken` before scan and again before publication and echoes them on every progress event, rejecting stale publication while still allowing the active-save switch to succeed concurrently. The top-bar banner keeps a stable polite live-region text and an adjacent native `<progress>` (indeterminate for scan, determinate only when counts are truthful). Phases are `scan` → `preparing` → `scoring` → `saving` → `finalizing`; success shows the detailed `scanMs`, `prepareMs`, `scoringMs`, `saveMs`, `finalizeMs`, and `totalMs` timings, while the result retains undisplayed compatibility `ingestMs = saveMs + finalizeMs`. Search and Squad remain mounted during the command and on failure; a successful matching current replacement cancels/removes the exact Search/Squad roots under a continuation guard, then schedules current-owner invalidations; mutation settlement does not await those refetches, while suppressing stale updates. The final command result and phase-specific error remain authoritative if a progress message is missed.

### Settings management

The `/settings` route is one vertical page with **Preferences**, **Save data**, and **Bridge** sections. Preferences contains the labelled **Default player analysis view** select. It sets one app-local General or Moneyball default for Player Search and Player Profile when their URL does not specify a view. Each section keeps its own loading and error boundary so one failure does not blank the page. The managed-club selector lives in the My Club header; `/settings#managed-club` is a replace redirect for old bookmarks. The top bar remains the only save switcher and the only location for **Load Data**. Dashboard contains only its heading and `Placeholder.`

- **Snapshot history panel:** show one semantic table for the active save, ordered by valid in-game date descending, then load time and snapshot ID. Dated rows always precede undated rows. Each row shows the custom name when present, the in-game date as a separate line, player count, relative load age with an absolute UTC `title`, a visible **Current** marker, and Rename/Delete actions. Names organize rows but never replace date metadata or change order.
- **Empty and loading states:** when a save has no snapshots, show `No snapshots stored` and direct the user to **Load Data**. The history panel keeps its existing panel geometry and horizontal overflow behavior.
- **Rename:** open a focused form Modal with a bounded snapshot name. Blank input clears the custom name and restores the in-game date label. Successful rename invalidates only the active-save history query; the row remains in the same position.
- **Snapshot deletion:** open a destructive Modal whose title includes the exact snapshot date or custom name plus its internal snapshot identifier. Explain that players, staff, role scores, bridge provenance, and Moneyball data are removed, while Planner, Academy, and Youth data remain. Disable duplicate submission, keep errors inside the dialog, and return focus to the history panel. Deleting the current row refreshes every current-only view; deleting a non-current row does not.
- **Save management:** list save names and the active marker below the save rename/create forms. Every save delete Modal names the save and states that all snapshots, player data, Moneyball data, Planner settings, Academy records, and Youth enrichment will be removed. State whether the active save stays unchanged, another save becomes active, or a blank `Default save` replaces the final save. Bind the confirmation to the immutable target context so a save switch or row-ID reuse cannot retarget the action.
- **Context feedback:** Load Data success copy remains bound to the save that started the scan. If the final save is deleted and a new `Default save` receives the same numeric ID, stale feedback is cleared. Current/save-changing success invalidates Search, Player, Staff, Planner, and Academy state from the route composition layer.
- **Accessibility:** destructive dialogs use `role="dialog"`, `aria-modal`, target-specific headings, keyboard focus trapping, Escape/Cancel protection while pending, and focus restoration. Duplicate targets include visible and assistive identifiers so two rows with the same date or name remain distinguishable.

### Data Table

The core surface. Player and staff search results, squad lists, and comparison sets.

- **Container:** `surface-container` with `lg` radius and 1px `outline-variant` border; the table itself is full-bleed inside it with no inner padding. Search and Squad panels are `flex` columns with `min-h-0`; their route roots use `h-full` so the shared table owns the vertical scroll and the document does not grow with the virtual spacer. The table fills the available width until configured minimums require horizontal overflow. Header row is `surface-container-lowest`, `table-header-height` 32px, sticky at `z-10`. Body rows carry a 1px `outline-variant` bottom border. Cell padding is `stack-sm` horizontal.
- **States:** row hover fills `surface-container-high`; row `:focus-visible` shows the gold ring inset; selected row fills `primary-container` with a 2px `primary` left indicator and `aria-selected`; sorted column header shows `primary` label text plus a direction caret. Transition `background-color 150ms ease-out`. Row height never changes on any state.
- **Variants:** `compact` — single-line cells at `table-row-height` 36px. `two-line` — a primary value over an 11px `on-surface-variant` secondary at `table-row-height-two-line` 40px. A table picks one variant for all its rows.
- **Content / Anatomy:** header cells in `label-md` uppercase `on-surface-variant`. Text cells in `body-sm` `on-surface`. Search, Moneyball Search, and Squad use player name as the primary identity line and available club and division values joined by ` · ` as the secondary line. If both values are absent, the cell shows only the name. Club and Division remain sortable, configurable metrics but do not appear in those default layouts. Secondary lines are 11px regular in `on-surface-variant` — not a `label-*` role, because a date or player context is not a label and must not be uppercased or letterspaced. Numeric cells are right-aligned with tabular figures. Score cells hold a Score Badge. There is no visible ellipsis or manage column; each header button sorts on click and exposes the same actions through right-click, the Context Menu key, or Shift+F10.
- **Behaviour:** a real `<table>` with `<caption class="sr-only">`, `<thead>`, and `<th scope="col">`. Sortable headers are buttons that set `aria-sort`. A header action menu exposes **Move left**, **Move right**, **Add column**, and **Remove column**, with edge and last-column guards. A searchable metric picker in that menu uses the same grouped catalog as Search filters. Resize handles are visible keyboard-focusable separators: pointer movement uses capture, while ArrowLeft/ArrowRight/Home/End change a clamped width. Up and Down arrows move row focus across virtual page boundaries, Enter or a row click opens the player, and the sticky header never covers the focused row. Search and Squad use one full-height virtual table with fixed 40px rows and bounded 50-row IPC pages; no Previous or Next controls or unbounded client collection exist. Empty, loading, and error states replace the body with the states below — never blank space.

### Score Badge

A role or position fit score. The most repeated element in the app.

- **Container:** in a table, no fill and no border — the number sits directly on the row in its tier colour. Elsewhere, a 28px circle with `full` radius, `surface-container-high` fill, and a 1px border in the tier colour at 40% alpha. The table variant is unfilled on purpose: a filled badge would match the hovered row background and vanish, and 500 filled chips in a column is exactly the boxing that principle 2 forbids.
- **States:** static inside a row; the tier colour is verified against both the default and the hovered row background. In an interactive context — a clickable role chip — hover raises the surrounding fill, never the number's colour.
- **Variants:** `table` (`mono-sm`, unfilled, right-aligned), `card` (28px filled circle, `mono-md`), `hero` (48px, `mono-lg`, unfilled, used for the current and potential best-role summaries on a player profile). A `muted` variant renders the number in `on-surface-variant` instead of a tier colour, for roles outside the player's positional familiarity — the score is still shown, but it does not compete for attention.
- **Content / Anatomy:** the integer score, nothing else. No unit, no percent sign, no trailing zero. Colour comes from the `score-1` to `score-4` ramp by the tier table in Colors.
- **Behaviour:** the accessible name is the full statement — `"Deep-lying playmaker: 82, Excellent"` — not just the digits. The tier label also appears in `title`. Never render a badge without its number.

### Status Chip

Compact non-interactive state: snapshot freshness, transfer status, bridge state, qualifiers.

- **Container:** `full` radius, 20px tall, `stack-xs` vertical and `stack-sm` horizontal padding, `label-md` text. Fill is the matching `*-container` token with a 1px border in the semantic tone at 40% alpha, because container fills sit only 1.4:1 above the panel and need the edge to read.
- **States:** static. A chip is never a button; if it needs a click, it is a Filter Tag or a Button.
- **Variants:** `success`, `warning`, `error`, `info`, and `neutral` (`surface-container-high` fill, `on-surface-variant` text).
- **Content / Anatomy:** 12px leading icon, then the label. The icon is required — the chip must not rely on fill colour.
- **Behaviour:** decorative chips that duplicate adjacent text take `aria-hidden`. Chips carrying unique information stay in the accessible tree.

### Text Input, Select, and Search Field

Form controls, including the global and local search fields.

- **Container:** 32px tall, `surface-container-high` fill, 1px `outline` border, `md` radius (`full` for search fields), `stack-sm` horizontal padding, `body-md` text. Select adds a 16px trailing chevron.
- **States:** hover brightens the border to `on-surface-variant`; focus replaces the border with a 2px `primary` border and no offset ring, so the field does not shift; invalid shows an `error` border plus a `body-sm` `error` message below, tied to the field with `aria-describedby`; disabled drops to 45% opacity. Placeholder text is `on-surface-variant`.
- **Variants:** `text`, `number` (tabular figures, right-aligned), `select`, `search` (pill with a 16px leading magnifier and a clear button once non-empty).
- **Content / Anatomy:** every field has a visible `<label>` in `label-md` above it, or an `aria-label` when the icon and context make the purpose obvious — as in the global search. Placeholder text is never the only label.
- **Behaviour:** search fields debounce at 200ms and never block typing on a query. Escape clears a non-empty search field before it closes any surrounding panel. Selects are native `<select>` unless multi-select or option-rendering requires a custom listbox, in which case implement full arrow-key and type-ahead support.

### Panel

The default container for a titled block of content.

- **Container:** `surface-container`, `lg` radius, 1px `outline-variant` border, `stack-md` padding.
- **States:** static. A panel is not interactive.
- **Variants:** `default` and `flush` (no padding, for a panel whose only child is a full-bleed table).
- **Content / Anatomy:** optional header row with a `headline-sm` title on the left and actions on the right, then the content with `stack-md` above it. Do not nest a panel inside a panel — use a `stack-lg` gap and a hairline rule instead.
- **Behaviour:** the panel title is the section heading and must keep the document's heading order correct.

### Compact Filter Strip, Filter Tag, and Filter Editor

Progressive filtering on the Search screen — Genie Scout / FM-style operator rules, not an inspector of sliders.

- **Compact strip (above results):** horizontal row of active filter tags, an AND|OR mode indicator when more than one rule is set, **Clear all** when any rule is active, and **Edit filters** to open the modal. Player Search General places **Upload Shortlist** immediately after **Edit filters**; tactic controls keep their established position. Staff Search uses the same strip with an integrated action slot for **Upload Shortlist**, **Configure Club Staff**, and **Optimize assignments**. Preferred Job and Only unemployed appear only while staff shortlist filtering is on. Tags are `full` pills with `primary-container` fill, `on-primary-container` text, and a trailing remove button. Tag remove hover fills `surface-container-highest`.
- **Filter tag label:** field label, operator word, and value (e.g. `CA > 150`, `Role · Deep-Lying Playmaker (IP) > 70`). Incomplete draft rules do not appear as tags.
- **Filter editor modal:** `form` modal variant. Lists rules as field + operator + value rows with add/remove. Single AND|OR toggle for the flat rule list (no nested groups). Field selection uses the categorized, searchable metric picker, grouped by identity, club and contract, ability and reputation, visible attributes, hidden attributes, personality, position suitability, current role scores, and potential role scores. The editor owns a local draft copied from the applied URL state whenever it opens. **Done** applies one complete draft; incomplete rules disable Done. Cancel, the close control, Escape, and backdrop dismissal discard the draft without changing the URL, starting a query, or materializing potential scores.
- **Operators by field kind:** strings — contains / does not contain / is / is not; integers (CA, attributes, role scores, suitability) — greater than / less than / equals / does not equal; booleans and closed enums — is / is not.
- **Behaviour:** Done applies the complete rule set and combine mode in one URL update; typing and other draft edits are query-silent. Active filters, combine mode, and sort live in the URL search params so the view survives a reload. Removing the last tag restores the unfiltered result set. Applying a filter adds its metric column once when it is not already visible, but filters and columns can then change independently; removing a column does not remove the filter.

### Search tactic columns

Search General and Moneyball show two tactic controls in their established filter-strip position: **Add Tactic (Current)** and **Add Tactic (Potential)**. Each is an accessible toggle with `aria-pressed`; active uses the primary container fill while inactive uses the secondary/ghost treatment. Disabled applies when no tactic context exists, during loading or error, or on cached refresh read-only, and no layout change occurs until data is ready. Interleaving is deterministic: one active group renders 11 lanes in `TACTIC_POSITION_ORDER` at the far right; both active interleave as `current` then `potential` per lane. Header label is `"{IP Position} ({IP Role}) / {OOP Position} ({OOP Role})"` from `TacticOptions` display names. Cells are right-aligned, width 112 (clamped 72–360) with joint `replaceLayout` persistence, show `ScoreBadge` for numeric scores and `—` in `on-surface-variant` for unavailable, and use header-X to deactivate the group, re-compact the survivor without gaps, and fall back from a removed tactic sort. Sort keeps unavailable last.

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
- **Content / Anatomy:** series colours in order — `chart-1` gold for the subject, `chart-2` steel and `chart-3` magenta for comparisons, `chart-4` neutral dashed for a league or squad average. Series fills use 20% alpha so overlaps stay readable. Every series also gets a distinct stroke pattern.
- **Behaviour:** one subject plus two comparisons, which is why only three player series tokens exist; beyond three overlaid shapes a radar stops informing. A chart is never the only representation of its data — the same values appear in an adjacent table or an `sr-only` table, because a radar chart is not accessible on its own.

### Scrollbar, Icons, and Motion

Cross-cutting rules rather than components.

- **Scrollbars:** 10px, transparent track, `outline-variant` thumb with `full` radius, brightening to `outline` on hover. Applied via `scrollbar-color` and `scrollbar-width`. Never hide a scrollbar on a scrollable region — a dense table needs a visible position cue.
- **Icons:** [Lucide](https://lucide.dev) via `lucide-react`, bundled. 16px in tables and chips, 16px in top-navigation links, 20px in the utility bar, and 24px in empty states. Use `strokeWidth` 1.5 by default and 2 for active top-navigation links; use `currentColor` always, so an icon inherits its context. One icon set only, and no emoji as an icon anywhere.
- **Motion:** 150ms ease-out for colour and opacity on hover, focus, and active. 200ms ease-out for overlay entrance; 150ms for exit. Nothing animates longer than 200ms, and layout, size, and position never animate on hover. Under `prefers-reduced-motion: reduce`, drop every transform and entrance animation and keep colour changes instant.

### Player profile layout

Dedicated route `/players/$uid` (not an inspector overlay). Comparison inspector remains unused until a later compare feature.

- **Analysis view switch:** a compact **General** | **Moneyball** tab strip sits below the shared two-band profile header in the same DOM and layout position for both modes. General is the factory default, while Settings can make Moneyball the default when the profile URL has no view. A valid URL view wins. The outer switch preserves the inner General attribute `tab` URL value, so returning to General restores the selected attribute group and keeps focus on the activated analysis tab.
- **General workspace:** at the 1280×800 minimum, the General summary sits above side-by-side **Attributes** and **Role fit** panels. The two panels consume the shell main area's remaining height and own bounded internal overflow. A visible global Load Data outcome banner reduces that height; it must not create nested page scrolling. Narrower unsupported widths may stack the panels. Both General and Moneyball role-fit workspaces stack the position picker above scores below the `lg` content breakpoint. There is no page-level Overview / Attributes / Roles switch.
- **Summary:** player name, club and division, age/DOB, nationality flags, height, preferred foot, CA, and market value stay visible above the analysis workspace. Each nationality flag exposes its name on hover and to assistive technology. PA appears when information is revealed. The compact desktop summary uses two tight bands: player identity and actions share the first row, followed by one aligned three-column detail grid for player facts, role summaries, and CA/PA/value. General uses all four fixed role slots: **Current IP**, **Current OOP**, **Potential IP**, and **Potential OOP**. Moneyball keeps the action slot, the same role-grid footprint with **Moneyball IP** and **Moneyball OOP**, and its third detail column reserved at every profile breakpoint. Each summary selects the best non-null eligible role score for its phase; ties retain catalog order and missing values render `—`. Transfer flags appear only when true. There is no crest or portrait because the source has none.
- **Information visibility:** the summary control reveals or conceals hidden information for the active save. In the concealed state, PA, projected values, potential role scores, Hidden and Personality values, and development actions are absent. Hidden and Personality tabs show an explicit concealed state. Current values remain visible. The Potential IP and Potential OOP slots remain as concealed placeholders, so the summary does not shift when the state changes.
- **Development actions:** when information is revealed, **Boost CA** and **Wonderkid Mentality** are compact summary actions, right-aligned beside the player identity rather than placed in a separate panel. The shared action slot keeps one control-row minimum height and preserves overflow for downward-opening tooltips. Only the inline outcome uses a bounded vertical scroll region, so completed feedback remains reachable without moving the analysis toggle. Hover or keyboard focus reveals each snapshot preview or disabled reason in a downward-opening tooltip that stays within the desktop viewport. One Modal confirms each action, preserves the existing guarded command contract, and restores focus to its trigger or the verified outcome. Results stay inline below the actions. There is no numeric input, arbitrary value control, or random-value selection.
- **Attribute tabs:** the Attributes panel uses four tabs. Outfield players see **Outfield** | **Goalkeeping** | **Hidden** | **Personality**, defaulting to Outfield. Their Outfield tab renders Technical, Mental, and Physical together, with Set Pieces below Technical. Players with recorded GK familiarity of 15 or higher see **Goalkeeping** | **Outfield** | **Hidden** | **Personality**, defaulting to Goalkeeping. Their Goalkeeping tab renders the alphabetized goalkeeper list, including First Touch, Passing, and Technique, with Mental and Physical; their Outfield tab contains only the remaining Technical attributes and Set Pieces. An explicit canonical `tab` URL search value wins; legacy `technical`, `mental`, and `physical` values normalize to Outfield. Missing or invalid values use the player-sensitive default. Arrow keys, Home, and End follow the visible order.
- **Attribute values:** when information is revealed, visible groups show text-accessible `Current → Potential` pairs of raw FM integers. Players aged 29 or older use the current value as potential because the projection model does not increase them further. When concealed, visible groups show Current values only, while Hidden and Personality show the explicit concealed state. Each known value uses the shared four-band data ramp with the documented 1–20 bands; the number remains the primary fact. Null renders `—`.
- **Role fit:** a compact pitch offers the 14 supported role positions as 44px buttons and omits `SW`. At the `lg` two-column breakpoint, its position-picker column owns vertical scrolling around the pitch's natural height; below that breakpoint, the stacked workspace grows naturally. The strongest positive recorded familiarity is selected first; if none exists, the highest current role supplies the fallback position. Known positive familiarity shows its raw 1–20 value and the same attribute tier colour; red-tier familiarity values 1–5 use muted surfaces to de-emphasize unusable positions without reducing text or focus contrast. Zero, unread, and legacy-missing values remain `—` and cannot become the best position. Selecting a position shows only roles whose catalog `positionTags` contain that exact position. **Current** descending is the default sort. When information is revealed, the **Current** and **Potential** column headers switch the score basis and toggle ascending or descending order. Players aged 29 or older show current role scores as potential. When concealed, Role fit exposes only Current scores. Unavailable scores stay last and catalog order breaks ties. Rows retain the role name and IP/OOP phase. Missing scores render `—`.
- **Moneyball workspace:** a scored current import keeps the player identity visible above best IP and OOP Moneyball role summaries and two bounded panels: raw metrics and role fit. The raw panel shows asking price, starts, substitute appearances, and minutes as neutral context. For a player with natural positions, it also states the natural positions and the backend-provided unique comparison-player count. Eight metric tabs — Shooting, Creation, Possession, Defending, Aerial, Goalkeeping, Discipline, and Results — show raw values with a 0–100 ScoreBadge. The role-fit panel uses the position picker and one sortable Moneyball score column. Each role has a keyboard-reachable disclosure with its five metric contributions, direction, catalog version, and natural-position comparison basis. The badge uses the existing score ramp and names the metric or role score, value, and tier for assistive technology. If an imported player has no natural position, raw context and metrics remain visible while explicit percentile and role-score unavailable states replace badges and the role-score table. Raw values remain visible and unavailable values remain `—`. Arrow keys, Home, and End move through the visible metric tabs. A player absent from the current import and a pre-score import show separate neutral recovery states that direct the user to Player Search's Moneyball upload.
- **States:** no snapshot → EmptyState pointing at Load Data; unknown UID → “Player not in this snapshot”; one loading skeleton mirrors the summary and two-panel workspace.
- **Out of this layout for now:** radar charts, history/trend blocks, compare inspector, and combined IP/OOP weight controls. The pitch is a role filter and familiarity display, not a new suitability calculation.

### Staff workspace layout

The `/staff` route owns Staff Search and canonical **My Staff**; `/staff/$uid` owns Staff Profile. My Club owns **Squad**, **Planner**, and **Tactic** only. Staff Search and My Staff are separate top-navigation destinations, and the managed-club selector remains in the My Club header.

- **Staff Search:** the shared configurable virtual table and filter editor cover the current snapshot. **Upload Shortlist**, **Configure Club Staff**, and **Optimize assignments** stay visible in the filter header. A URL-backed toggle beside the result count enables shortlist filtering. Preferred Job and Only unemployed appear only when it is on. Filtering off uses the `staff-search` layout and normal metrics. Filtering on retains the `staff-shortlist` layout and presentation: All jobs uses configurable CSV metadata; mapped jobs use fixed identity and metadata plus their mapped score and current sort; Coach shows six outfield coaching scores without automatic score sorting; unrecognized jobs use CA behavior. Upload, configuration, optimizer setup and result feedback, immutable context guards, Staff Profile navigation, and row navigation retain their existing behavior.
- **My Staff:** `/staff?view=my-staff` shows every current-snapshot staff member whose club exactly matches the saved managed club. It has selected-club context, an independent table layout and sort state, no search filters, and one confirmed **Boost all CA** action. When no managed club is configured, it links to Club setup at `/my-club#managed-club`; it does not render the selector.
- **Navigation and states:** activating any Staff Search or My Staff row opens `/staff/$uid`, and browser Back returns to the originating table URL. `/staff?view=shortlist` and `/my-club?view=staff-shortlist` replace-normalize to `/staff` with shortlist filtering on. `/my-club?view=staff&staffSort&staffDir` replace-redirects to `/staff?view=my-staff&myStaffSort&myStaffDir`, preserving valid sort values and applying existing validator defaults to invalid values. No snapshot, no shortlist, empty, unavailable-score, loading, and error states use the shared patterns without page-level nested scrolling.

### Club DNA definition

My Club places **Define DNA** beside **Save managed club**. The action is enabled only when the current save has a settled managed-club context and the definition query has loaded successfully. One accessible form Modal owns the full closed catalog, grouped by the existing FM attribute sections. It shows the selected summary and formula, requires at least one attribute, and uses one Modal for create, edit, and remove confirmation. Editing keeps the selected definition until the user confirms removal. Pending, unavailable, error, and stale-context states disable unsafe actions and retain truthful feedback. The metric is fixed at 0–100; unavailable values render `—`. Creating a definition appends `club_dna` once to Search and Squad layouts. Editing or removing it does not restore removed columns, change URL filters or sorts, or change layout ownership.

### Squad workspace layout

Dedicated route `/my-club`; `/planner` is a replace compatibility redirect to `/my-club?view=planner`. The My Club route contains the Squad overview, shared dual-phase tactic editor, and selected team's depth chart in three URL-backed workspaces: **Squad**, **Planner**, and **Tactic**. Squad is the default for every save. The active workspace is exposed while Planner and Tactic remain mounted with direct hidden props so local drafts, table layouts, and selections survive workspace changes. An explicit valid `view` search value wins. Top-navigation workspace links use normal Link navigation, so Club destination changes add browser-history entries and browser Back returns to the prior destination. Same-route changes preserve `squadSort` and `squadDir`; route-local sort controls use replace navigation. The route uses the active save and current snapshot shown in the utility bar.

- **Managed club:** My Club shows one explicitly saved selector at the stable `/my-club#managed-club` target when a current snapshot exists. Its options are exact current-club names from the latest snapshot. The picker and **Save managed club** action share one responsive control row that wraps at narrow widths, while warning and error feedback remains below the row. A saved selection remains visible with a warning when a later snapshot no longer contains it; users replace it explicitly. Squad, Planner, Academy, and managed-club Staff derive membership from this one save-scoped selection. The integrated Staff Search shortlist remains save-owned and is not managed-club filtered. No attached-club or fuzzy-name inference exists.
- **Page header:** `My Club` as `headline-lg`, the compact managed-club selector, supporting club context, and the current Club workspace share one wrapping header. The depth matrix keeps **Senior** | **Reserves** | **Youth** team tabs. The Tactic command bar has **IP** | **OOP** | **Both** view controls.
- **Squad overview:** a configured managed club shows one full-height, filter-free virtual table of its current-snapshot players. The initial columns are **Name**, **Age / DOB**, **Nationality**, **CA**, **PA**, and **Value**; the shared metric menu can add, remove, resize, and move any sortable Search metric while keeping at least one visible column. Absent values render `—`. CA descending is the default and sort stays in the route URL; column order and widths persist in the Squad layout independently from Search. The shared table requests bounded 50-row pages as the user scrolls, has no Previous / Next controls, keeps arrow-key traversal across page boundaries, and activates the full row or its name link to `/players/$uid`. An empty managed club explains that no current-snapshot players match.
- **Squad development:** the overview header has a primary **Boost all CA** action, a secondary **Make all Wonderkids** action, and the secondary CSV uploads. The CA confirmation explains the fixed age rule: +5 at age 20 or younger, +10 from age 21 through 28, and no boost from age 29. The Wonderkid confirmation explains that known Ambition, Professionalism, and Determination values at 10 or below receive a random 11–20 value; unknown and higher values stay unchanged. Both actions run sequentially, prevent duplicate or overlapping submission, and report Rust-derived determinate progress as `processed / total` after the cohort is captured and after each terminal player outcome. The confirmation stays open while the command is pending and shows an indeterminate preparing state before the first progress payload. Final feedback appears in one reserved Squad overview region for the latest action, uses compact processed/updated/skipped/failed copy, and does not move the action header. If the app cannot verify a result or preserve the active context, it stops, disables both actions, focuses the shared feedback region, and tells the user to use Load Data before another boost. A newly current snapshot restores the actions and clears prior feedback. Neither action claims that skipped, failed, or recovery-stopped players changed.
- **CSV uploads:** Search keeps **Upload Moneyball CSV**. My Club Squad offers **Upload Squad CSV** and **Upload Youth Academy CSV**. Each opens the shared format-bound Modal with a clear drop zone and keyboard-reachable **Browse files** action; it accepts exactly one CSV through either path and states the selected format. Pending, success, mismatch, and context feedback stay inside that modal, use text plus status icons, and never display a local path. Moneyball copy says that an upload adds or updates matched players and that omitted enrichment remains; it does not imply whole-cohort replacement. A format mismatch names the required export; changing the save or current snapshot clears feedback and closes the modal, returning focus to its action.
- **Tactic editor:** one tactic per app save, shared by all teams. One command bar shows the phase view, save status when present, and **Save tactic** action. The pitches and selected-position settings shelf provide the current linked-position context without repeating it in the command bar. Both view places the IP and OOP pitches side by side above the shelf; IP and OOP views show only the chosen pitch and its phase controls. The complete command bar, pitch canvas, and shelf fit without document scrolling at 1600×900 and 1920×1080 with either navigation-rail width. At 1280×800, the shelf reflows while preserving both usable pitches, reachable controls, and document-level horizontal fit.
- **Pitch geometry:** the editor starts from a 4-3-3 DM In-Possession shape linked to a 4-1-4-1 DM Out-of-Possession shape, with compatible general-purpose roles already selected. Each phase uses an attack-up pitch with the striker band at the top and the goalkeeper band at the bottom. Both pitches derive one shared card width from the densest visible row across IP and OOP, clamped to three through five slots, so every tactical slot has the same width within one tactic while different tactics can use different widths. A light base-position group contains its dark individual slot cards. Central selectors expose explicit right, centre, and left placements: `DCR` / `DC` / `DCL`, `DMCR` / `DM` / `DMCL`, `MCR` / `MC` / `MCL`, `AMCR` / `AMC` / `AMCL`, and `STCR` / `STC` / `STCL`. Each qualified placement occupies its matching horizontal column and can appear only once per phase. Wide slots remain centred within their groups, and empty outer groups absorb unused width. Existing tactics with one to three repeated base positions retain their previous visible order when they first load. Keyboard traversal follows visible pitch order. Current IP/OOP positions and roles describe each selection; focus, hover, and selection emphasize the linked counterpart across both pitches without numeric markers.
- **Selected-position settings:** the shelf contains one IP/OOP weight control, one optional importance rank from 1 through 11, preferred foot (**Either**, **Left**, **Right**, or **Both**), **Preferred** or **Strict** mode, and the visible phase position and role controls. Both view does not duplicate shared state. Either foot disables the mode control. Controls use visible labels, native keyboard behavior, inline validation, and failed-save draft retention. Invalid or incomplete role-position pairs cannot be saved.
- **Squad matrix:** available team tabs share one compact toolbar with **Optimize squads**, **Optimize by potential**, **Manage teams**, and **Clear all** when the available width cannot hold every team. When the Planner matrix container can hold the current strings at their readable minimum widths, one semantic table groups the available teams with ordered strings under each display-name header. Narrower containers show only the selected team's group with keyboard-operable tabs. Tactical positions are compact two-line rows in stable pitch order. Ordered strings are columns labelled **1st string**, **2nd string**, and so on. The matrix owns bounded horizontal and vertical overflow; its header and left position column stay sticky, and strings keep a readable minimum width rather than shrinking. Each position summary keeps the IP and OOP position/role context. Each occupied cell aligns the player name with a compact, accessible `Current → Potential` combined-score pair; the arrow is a visible direction cue and each badge names its basis. Unresolved and outside-pool assignments keep their occupied cell and show a warning; unknown scores render as `—`.
- **Best role fit reference:** the Planner toolbar opens a read-only **Best role fit reference** Modal with the tactic pitch on the left and independently ranked player results on the right. **In Possession** and **Out of Possession** controls select the tactic phase; **Current** and **Potential** controls select the ranking basis and show both score columns for the selected lane. The right-hand table has sortable **Name**, **Current**, and **Potential** headers, retains the selected tactic lane when a valid result exists, and keeps players without an eligible selected-basis score in a separate section with unavailable values shown as `—`. Opening defaults to In Possession, Current, the first lane, and Current descending; changing phase or basis requests the corresponding read-only reference and never changes assignments or tactic data.
- **Player assignment:** activating an empty cell opens a picker Modal. Candidates must match the exact managed club; the same club pool is available to every Planner team. Results show player name, current club, IP score, OOP score, current combined score, and any existing planner location. Rust sorts by current combined score descending. Selecting an unassigned player fills the cell. Selecting an assigned player opens a confirmation that names the old and new locations; confirming moves the player. Activating an occupied cell instead asks the user to clear it. Closing restores focus to the originating cell.
- **String management:** every team starts with one string and keeps at least one. A visible header overflow button and right-click on the same header open identical actions: **Add string** and **Remove string**. The final remaining string disables Remove. Removing a populated string uses a destructive Modal. After removal, ordinal labels close the gap.
- **Squad actions:** **Optimize squads** is the primary current-score action and **Optimize by potential** is a secondary explicit action. Both assign eligible, unassigned players across available teams and strings through the same Rust allocation and transaction path, then report basis-specific pending, one latest success, or failure without masking blank gaps. Only the score basis changes; managed-club eligibility, ranking, matching, foot preferences, manual reservations, replacement, rollback, and assignment provenance stay shared. The optimizer processes the shared club pool in Senior, Reserves, Youth order, enforces one assignment per player, and keeps the existing age limits for Reserves and Youth. **Manage teams** opens a form Modal for the fixed Senior, Reserves, and Youth categories. It lets the user keep one to three categories and set unique display names. Removing a category with assignments opens a destructive confirmation that names each affected display name and count; successful removal updates the matrix and moves focus to a remaining tab or **Manage teams**. **Clear all** is destructive and requires confirmation that names the available display names. One Rust transaction removes both manual and optimized assignments from every string in the active save while preserving strings, tactics, managed-club settings, scores, and other saves. The actions stay in the shared toolbar in both matrix modes, prevent duplicate submission, retain errors, restore focus after close, and reconcile the displayed matrix and picker candidates after success.
- **States:** no snapshot uses the standard Load Data EmptyState. No managed club shows recovery guidance that links to the My Club selector. A saved club absent from the latest snapshot shows a warning without deleting configuration or assignments. Loading skeletons match the matrix or pitch geometry. Tactic-save and assignment failures stay inline with the affected control and preserve the user's draft or prior assignment.
- **Accessibility:** pitches expose each tactical position as a labelled button with phase, position, and role in its accessible name. Position selection and placement changes work without drag, and the linked counterpart is described to assistive technology. Team and phase controls use proper tab semantics. Matrix cells have row and column context in their accessible names. Escape returns focus when the string menu trigger or a menu item owns focus. A right-click on a non-focusable string header opens the menu without moving focus, so Escape has no supported focus-restoration path.
- **Data honesty:** Current combined scores use persisted IP and OOP role scores from the current snapshot. Potential combined scores use projected visible attributes and the same lane IP/OOP roles and weight. Missing phase or projected-required attributes produce `—`, not a partial or zero score. Imported FM team level remains nullable metadata and does not restrict Planner eligibility. Settings does not expose its diagnostic count because users cannot act on it. A player outside the managed club or absent from the snapshot remains assigned but carries a visible warning.

### Youth academy layout

Dedicated route `/academy`. The page tracks save-scoped youth cohorts against the managed club selected in My Club; it does not repeat club selection or import data from files.

- **Page header:** `Youth Academy` as `headline-lg` and the managed club as secondary context. The Overview Classes panel owns the **Create class** primary action. The create Modal uses a labelled positive year input, prefilled from the current snapshot's in-game year when available, and previews `Class of YYYY` before submission.
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
- [ ] All icons from Lucide at 16 / 20 / 24px; no mixed icon sets
- [ ] No raw colour values in components — every colour comes from a token
- [ ] Colour is never the sole indicator of meaning — paired with text, icon, or shape
- [ ] All text-on-background combinations meet the contrast minimum (verify against the table in Colors; compute new pairings before use)
- [ ] Score badges always render their number, and their accessible name includes the role and tier
- [ ] Every surface boundary has a tonal step *and* a hairline border
- [ ] `primary` appears only on chrome; the score ramp appears only in data

### Typography & Numbers

- [ ] Numeric columns, scores, and metrics use `tabular-nums` and are right-aligned
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
- [ ] Active save and snapshot age visible without scrolling on every data view
- [ ] Truncated snapshots carry a warning wherever their data appears, with the cap named
- [ ] Loading skeletons match the final layout at the real row height
- [ ] Toast auto-dismiss timers pause on hover and focus; error toasts do not auto-dismiss
- [ ] No network request for a font, icon, or image
