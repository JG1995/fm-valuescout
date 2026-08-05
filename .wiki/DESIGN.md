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
    # Score ramp — role and position fit, tier 1 (weak) to tier 5 (elite)
    score-1: "oklch(0.65 0.015 80)"
    score-2: "oklch(0.72 0.06 82)"
    score-3: "oklch(0.78 0.1 85)"
    score-4: "oklch(0.84 0.14 88)"
    score-5: "oklch(0.9 0.155 96)"
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
    rail-width: 56px
    rail-width-expanded: 208px
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

> **Status:** Tokens, shared primitives (`src/components/ui/`, including **Modal** and **ScoreBadge**), the app shell (nav rail, top bar with **GlobalPlayerSearch**), the **Search** surface (compact filter strip, editor modal, virtualized results table), the **Player profile layout** (`/players/$uid` with Overview / Attributes / Roles tabs), and the **Squad Planner** club-family setup, dual-phase tactic editor, compact three-team depth matrix, Optimize control, and one confirmed Clear all action (`/planner`) are implemented. `src/styles/global.css` bridges the full token set into Tailwind `@theme` ([ADR-0007](./decisions/0007-tailwind-css-v4.md)).

## Brand & Style

**The central concept is a floodlit pitch at night: a dark field, and gold light on the thing worth looking at.**

FM ValueScout is an instrument, not a destination. The user already has Football Manager open on the same machine, and probably a second monitor. They alt-tab in with a question — *who fills this role best right now?* — and they want the answer in the first second of looking. Every pixel that is not an answer is in the way. The app is a quiet dark surface holding a lot of numbers, with gold reserved for two jobs: marking where you are, and marking what is good.

The mood is a night-shift control room. Cool near-black surfaces, hairline separations, dense rows, and one warm accent. This is deliberately not the friendly pastel dashboard look: the primary user is a single expert reading their own data for an hour at a time, so the design optimizes for sustained scanning over first-run charm. The tension to hold is **dense but not cramped** — 36px rows and 13px text are tight, so the spacing scale and hairline borders must do the separating work that whitespace usually does.

Hard stances:

- **Dark only.** There is no light theme and no `prefers-color-scheme` branch. FM runs full-screen and dark; a bright companion window beside it is hostile.
- **Desktop only.** Minimum window 1280×800, designed at 1600×900. No mobile or narrow breakpoints. ([CONCEPT.md](./CONCEPT.md) excludes mobile and web clients.)
- **Offline by construction.** No webfont CDN, no icon CDN, no remote image, no analytics beacon. Every asset ships in the bundle. This follows the offline-first principle in [CONCEPT.md](./CONCEPT.md), and it is a design constraint, not only an infrastructure one.
- **Text-only data, so text-only identity.** The FM26 dump supplies clubs and nationalities as strings and no images at all (`bridge/DUMP_SCHEMA.md`). There are no crests, portraits, or flags to render, and the app must not invent or bundle them. Where a reference design would place an avatar, use an initials monogram on `surface-container-high`, or omit the slot and give the name more room. This is a real divergence from crest-heavy scouting tools, and it is a data constraint, not a style preference.
- **No decorative imagery.** No hero art, no illustration, no stock photography.

## Colors

The palette is one warm accent on a cool near-neutral base, plus four semantic status colours and one data ramp. Total hue count is six: gold, steel, green, orange, red, and a magenta used only as a third chart series. Elevation is carried by **tonal layering plus hairline borders**, not by shadows. Dark surfaces swallow shadows, and the app stacks a lot of panels; a tonal step reads reliably at any brightness setting where a drop shadow does not. Shadows appear at one level only — floating overlays.

The neutrals carry a whisper of blue (hue 264, chroma 0.008–0.010). That is barely perceptible on its own, but it keeps the greys from looking dead and it sets up the complementary tension with the gold accent.

**Primary — floodlight gold (hue 82):** `primary` marks **chrome state**: the active nav item, the primary button, the focus ring, the selected row indicator, checked controls, and the subject series in a chart. It answers "where am I, and what is the main action here?" Gold also carries the product idea — ValueScout is about spotting value, and gold is what value looks like.

**Steel (hue 245):** `info` is the single cool counterpoint. It carries neutral factual annotation that is neither good nor bad: transfer-status tags, "U-21" style qualifiers, informational banners, and the first comparison series in a chart. There is no separate `secondary` token; steel does that work.

**Score ramp (hue 80 → 96):** `score-1` through `score-5` colour role and position fit. This is the only colour system that appears **inside data**, and it is the one the eye scans. The ramp climbs in lightness and chroma together, from a near-grey `score-1` to a vivid `score-5`. A weak fit recedes into the surface; an elite fit glows. Because both lightness and chroma rise monotonically, the ramp survives greyscale and every form of colour blindness — the ordering is carried by brightness, not hue.

| Tier      | Score  | Label     | oklch                     | Meaning                          |
| --------- | ------ | --------- | ------------------------- | -------------------------------- |
| `score-1` | 0–39   | Weak      | `oklch(0.65 0.015 80)`    | Does not play this role          |
| `score-2` | 40–54  | Fringe    | `oklch(0.72 0.06 82)`     | Emergency cover only             |
| `score-3` | 55–69  | Rotation  | `oklch(0.78 0.1 85)`      | Squad depth                      |
| `score-4` | 70–84  | Starter   | `oklch(0.84 0.14 88)`     | First-choice standard            |
| `score-5` | 85–100 | Elite     | `oklch(0.9 0.155 96)`     | Best available for this role     |

Score tier 5 and `primary` both sit in the gold band. That is intentional, and the rule that keeps it readable is a usage boundary: **`primary` never appears inside a data cell, and the score ramp never appears on chrome.** Gold on the frame means "interactive"; gold in the grid means "good".

**Semantic Colours:** Four fixed roles for status indicators.

| Semantic  | oklch                    | Role                                                                        |
| --------- | ------------------------ | --------------------------------------------------------------------------- |
| `success` | `oklch(0.76 0.16 150)`   | Load Data completed, bridge plugin installed and current, snapshot is fresh  |
| `warning` | `oklch(0.76 0.165 55)`   | Snapshot truncated at the scan cap, snapshot is stale, plugin update pending |
| `error`   | `oklch(0.66 0.2 18)`     | Scan failed, ingest failed, FM not running, destructive confirmation         |
| `info`    | `oklch(0.72 0.11 245)`   | Neutral annotation and explanatory banners                                   |

Warning sits at hue 55 (orange) rather than amber so it never reads as the gold accent. Success green appears only on status chrome, never as a score colour — the ramp owns "good" inside data.

`primary-hover` and `primary-active` are the Button spec's hover and active mixes resolved once, in oklab, rather than recomputed per component. Both stay in sRGB gamut and hold an `on-primary` label above 8:1. Unfilled variants have no mix — they press to `surface-container-highest`, one tonal step above their hover fill.

The template's `tertiary` role and the fixed tonal pairs (`primary-fixed`, `secondary-fixed`, `tertiary-fixed`, and their `on-*` partners) are removed on purpose. Nothing in this design needs a third accent or a tint that stays constant across themes, and there is only one theme. Do not restore them without a component that requires them.

Borders come in two roles with different rules:

- `outline` bounds **interactive** components — inputs, selects, secondary buttons, checkboxes. It must clear 3:1 against whatever surface it sits on, because the border is the only thing that shows the control exists.
- `outline-variant` is the **decorative** hairline for table row separators, card edges, and section rules. It is deliberately near-invisible (1.36:1) so it contains the data without competing with it. It is exempt from the 3:1 rule because it never carries meaning on its own.

### Accessibility of Colour

**Colour is never the sole indicator of meaning.**

- **Score badges** always render the number. The tier colour is redundant encoding that speeds scanning; the number is the fact. A tier label is available in the badge `title` and in the accessible name.
- **Status chips and banners** pair colour with an icon and a text label — never a bare coloured dot.
- **Active nav item** pairs the gold fill with `aria-current="page"` and a filled-versus-outline icon change.
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
| Score tier 1 (weakest)    | `score-1` (`#948e85`)                       | `surface-container` (`#181b1f`)                  | 5.4:1 (AA) |
| Score tier 1 on hover     | `score-1` (`#948e85`)                       | `surface-container-high` (`#222429`)             | 4.8:1 (AA) |
| Score tier 5 (strongest)  | `score-5` (`#fddd54`)                       | `surface-container` (`#181b1f`)                  | 12.9:1 (AAA) |
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

1. **Data outranks chrome.** No decorative element may take space a data column could use. *Test:* on the player search screen at 1600×900 with the rail collapsed and the filter editor closed, the results table covers at least 70% of the window area.
2. **Separate with hairlines, not boxes.** Rows, fields, and sections are divided by a 1px `outline-variant` rule or a tonal step. *Test:* no nested card inside a card, and no vertical rules between table columns.
3. **Snapshot provenance is always visible.** Every screen that shows player data states which save is active and how old the snapshot is, without scrolling. Truncated and stale snapshots carry a warning wherever their data appears. *Test:* screenshot any data view and you can name the save and the snapshot age from the image alone. This follows the explicit-refresh principle in [CONCEPT.md](./CONCEPT.md) — the user must never mistake old data for current data.
4. **Brightness carries value; the number carries the fact.** Score meaning comes from the ramp, and the number is always present. *Test:* convert a screenshot to greyscale — the ranking still reads.
5. **Every mutation reports its phase.** Long operations name what they are doing and which stage failed. Load Data distinguishes a scan failure from an ingest failure, because the fixes differ: start FM versus retry the ingest. *Test:* every mutation has a pending label, a success state, and a phase-specific error message.
6. **Keyboard reaches everything; hover reveals nothing.** Hover may only change colour. Any action or information available on hover is also available from the keyboard and visible without a pointer. *Test:* complete a full search-to-profile pass with the keyboard alone.
7. **Nothing loads from the network.** Fonts, icons, and images ship in the bundle. *Test:* run the app with networking disabled and no glyph, icon, or layout changes.

## Layout & Spacing

The app is a **single window with a persistent left rail and a top bar** — a desktop tool, not a set of pages. Minimum 1280×800; designed at 1600×900. Content fills the window width; `content-max-width` is `none` because a clamped column wastes the space a 20-column player table needs.

Regions, in visual order:

1. **Nav rail** (left, `rail-width` 56px, `rail-width-expanded` 208px). Icon-only by default with the label as a tooltip; expands to icon-plus-label. Sits on `surface-container-lowest` — the rail is the darkest region, which pushes the content forward. Collapsed state persists across launches.
2. **Top bar** (`header-height` 56px, spans the area right of the rail). Left to right: global player search (pill, grows to fill), active save selector, snapshot freshness chip, optional **Cap players** toggle with a numeric limit when on, **Load Data** primary button. Load Data lives here rather than on a page because it is the app's one recurring action and must be reachable from every screen. Cap off means unlimited scan; cap on sends a positive `maxAccepted` (default 500 when enabling).
3. **Page header** (inside the content area). Page title in `headline-lg`, then view-mode toggles and a local search or filter trigger on the right. One row, `stack-md` below it.
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
- **Level 1 (Recessed):** `surface-container-lowest` — nav rail, sticky table header, diagnostic and log wells. Darker than the canvas, so it reads as behind it.
- **Level 2 (Panel):** `surface-container` — the default. Cards, tables, panels, the top bar.
- **Level 3 (Raised):** `surface-container-high` — hovered table rows, input fields, nested blocks inside a panel, the inactive half of a segmented control.
- **Level 4 (Overlay):** `surface-container-highest` — dropdowns, context menus, popovers, modals, toasts. This is the only level with a shadow: `0 8px 24px oklch(0 0 0 / 0.6)`, plus the standard hairline border. Modals also dim the content behind them with `oklch(0 0 0 / 0.6)`.

### Z-Index Scale

Layers are separated by steps of 10 to leave room for future insertion. No element uses an arbitrary value.

| Layer         | Value  | Usage                                   |
| ------------- | ------ | --------------------------------------- |
| Base          | `z-0`  | Content area, tables, cards             |
| Sticky        | `z-10` | Sticky table headers, top bar, nav rail |
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
- **Variants:** `primary` — `primary` fill, `on-primary` label; the one main action. `secondary` — transparent fill, `outline` border, `on-surface` label. `ghost` — no fill or border, `on-surface-variant` label, hover fills `surface-container-high`; for toolbar and icon actions. `destructive` — `error` fill, `on-error` label; requires a confirmation modal before it executes. **Not implemented** — add it with the first destructive action, together with the Modal.
- **Content / Anatomy:** optional 16px leading icon, label in `label-lg`, optional trailing chevron for menu buttons. Never icon-plus-text in the icon-only variant.
- **Behaviour:** always a `<button>` with an explicit `type`. Icon-only buttons carry `aria-label` and a tooltip — the props type requires both an icon and an `aria-label` for that size, so an unlabelled icon button does not compile. A button that opens a menu sets `aria-expanded` and `aria-haspopup`.

### Nav Rail

Primary navigation between the app's main surfaces.

- **Container:** `surface-container-lowest`, full height, `rail-width` 56px collapsed or `rail-width-expanded` 208px expanded, 1px `outline-variant` right border. Items are 40px tall, `md` radius, `stack-xs` apart.
- **States:** default icon in `on-surface-variant`; hover fills `surface-container-high`; active item fills `primary-container` with a `primary` icon and label, plus a 2px `primary` left indicator; `:focus-visible` shows the gold ring inside the item bounds.
- **Variants:** collapsed (icon only, label as tooltip after 400ms) and expanded (20px icon plus `label-lg`). One collapse toggle pinned at the bottom.
- **Content / Anatomy:** app mark at top, then navigation items — Search, Profiles, Planner, Optimizer, Settings — then the collapse toggle. Four to six items maximum; new surfaces go inside an existing one, not beside it.
- **Behaviour:** a `<nav>` containing a list of router links. The active item sets `aria-current="page"`. The collapsed state persists in the layout store across launches.

### Top Bar

Global search, save context, snapshot freshness, and the Load Data action.

- **Container:** `surface-container`, `header-height` 56px, 1px `outline-variant` bottom border, 16px horizontal padding, sticky at `z-10`.
- **States:** static. Its children carry their own states. When no snapshot exists for the active save, the freshness chip reads "No data loaded" in `on-surface-variant` and Load Data is the only emphasized element on screen.
- **Variants:** none.
- **Content / Anatomy:** global search field (pill, grows), save selector (`secondary` menu button showing the active save name), snapshot freshness chip (`label-md` relative age; `success` under 30 minutes, `on-surface-variant` under 6 hours, `warning` beyond that or when the scan was truncated), **Cap players** checkbox with numeric limit field (visible when on; default 500 when enabling), Load Data (`primary` button).
- **Behaviour:** the search field takes focus on `Ctrl+K` from anywhere. Switching saves swaps all snapshot-scoped views and clears any stale result banner from a previous load. Load Data reports scan and ingest phases separately, both in its own pending label and in the resulting error message. On success, the result banner appends scan, ingest, and total durations from `load_data` timings.

### Data Table

The core surface. Player search results, squad lists, comparison sets.

- **Container:** `surface-container` with `lg` radius and 1px `outline-variant` border; the table itself is full-bleed inside it with no inner padding. Header row is `surface-container-lowest`, `table-header-height` 32px, sticky at `z-10`. Body rows carry a 1px `outline-variant` bottom border. Cell padding is `stack-sm` horizontal.
- **States:** row hover fills `surface-container-high`; row `:focus-visible` shows the gold ring inset; selected row fills `primary-container` with a 2px `primary` left indicator and `aria-selected`; sorted column header shows `primary` label text plus a direction caret. Transition `background-color 150ms ease-out`. Row height never changes on any state.
- **Variants:** `compact` — single-line cells at `table-row-height` 36px. `two-line` — a primary value over an 11px `on-surface-variant` secondary (name over birth date, club over league) at `table-row-height-two-line` 40px. A table picks one variant for all its rows.
- **Content / Anatomy:** header cells in `label-md` uppercase `on-surface-variant`. Text cells in `body-sm` `on-surface`. Secondary lines are 11px regular in `on-surface-variant` — not a `label-*` role, because a date is not a label and must not be uppercased or letterspaced. Numeric cells right-aligned with tabular figures. Score cells hold a Score Badge. A trailing overflow-menu cell is 40px wide and fixed.
- **Behaviour:** a real `<table>` with `<caption class="sr-only">`, `<thead>`, and `<th scope="col">`. Sortable headers are buttons that set `aria-sort`. Up and Down arrows move row focus, Enter opens the row, and the sticky header never covers the focused row. Empty, loading, and error states replace the body with the states below — never blank space. Virtualize above roughly 200 rows; keep the row height fixed so virtualization stays simple.

### Score Badge

A role or position fit score. The most repeated element in the app.

- **Container:** in a table, no fill and no border — the number sits directly on the row in its tier colour. Elsewhere, a 28px circle with `full` radius, `surface-container-high` fill, and a 1px border in the tier colour at 40% alpha. The table variant is unfilled on purpose: a filled badge would match the hovered row background and vanish, and 500 filled chips in a column is exactly the boxing that principle 2 forbids.
- **States:** static inside a row; the tier colour is verified against both the default and the hovered row background. In an interactive context — a clickable role chip — hover raises the surrounding fill, never the number's colour.
- **Variants:** `table` (`mono-sm`, unfilled, right-aligned), `card` (28px filled circle, `mono-md`), `hero` (48px, `mono-lg`, unfilled, used once on a player profile for the best-role score). A `muted` variant renders the number in `on-surface-variant` instead of a tier colour, for roles outside the player's positional familiarity — the score is still shown, but it does not compete for attention.
- **Content / Anatomy:** the integer score, nothing else. No unit, no percent sign, no trailing zero. Colour comes from the `score-1` to `score-5` ramp by the tier table in Colors.
- **Behaviour:** the accessible name is the full statement — `"Deep-lying playmaker: 82, Starter"` — not just the digits. The tier label also appears in `title`. Never render a badge without its number.

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

- **Compact strip (above results):** horizontal row of active filter tags, an AND|OR mode indicator when more than one rule is set, **Clear all** when any rule is active, and **Edit filters** to open the modal. Tags are `full` pills with `primary-container` fill, `on-primary-container` text, and a trailing remove button. Tag remove hover fills `surface-container-highest`.
- **Filter tag label:** field label, operator word, and value (e.g. `CA > 150`, `Role · Deep-Lying Playmaker (IP) > 70`). Incomplete draft rules do not appear as tags.
- **Filter editor modal:** `form` modal variant. Lists rules as field + operator + value rows with add/remove. Single AND|OR toggle for the flat rule list (no nested groups). Primary action is not required — changes apply immediately to the results query as the user edits; Cancel/close dismisses the dialog.
- **Operators by field kind:** strings — contains / does not contain / is / is not; integers (CA, attributes, role scores, suitability) — greater than / less than / equals / does not equal; booleans and closed enums — is / is not.
- **Behaviour:** filters apply immediately, with no Apply button. Active filters, combine mode, and sort live in the URL search params so the view survives a reload. Removing the last tag restores the unfiltered result set. Dynamic result columns appear for each active non-basic filter field (attributes, role scores, and other non-basic fields); removing the filter removes that column.

### Modal

Focused decisions and destructive confirmations.

- **Container:** `surface-container-highest`, `xl` radius, `stack-lg` padding, maximum 560px wide (720px for a two-column picker), overlay shadow, `z-50` over a `z-40` backdrop of `oklch(0 0 0 / 0.6)`.
- **States:** entrance fades the backdrop and lifts the panel 8px over 200ms ease-out; exit reverses it at 150ms. Both are suppressed under `prefers-reduced-motion`.
- **Variants:** `informational` (single dismiss), `form` (Cancel plus a `primary` submit), `destructive` (Cancel plus a `destructive` confirm, with the affected object named in the body — "Delete save *Braga 2029*?" — never a bare "Are you sure?").
- **Content / Anatomy:** `headline-md` title, `body-md` body, right-aligned footer actions with the primary action last. A close button sits top-right on informational modals only; a destructive modal requires an explicit choice.
- **Behaviour:** traps focus, returns focus to the trigger on close, closes on Escape and on backdrop click — except destructive variants, which ignore backdrop clicks. Uses `role="dialog"` with `aria-modal="true"` and `aria-labelledby` on the title.

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
- **Variants:** **No snapshot** — "No data loaded for this save", naming Load Data as the next step; this is the app's true first-run state and it must point at the one useful next step. It names the top bar's button rather than repeating it, because Load Data is already on screen and a second copy would be the only emphasized element competing with it. **No results** — "No players match these filters", with "Clear filters". **Error** — a phase-specific message as the title, the underlying reason as the explanation line, and a Retry button. **Truncated** — results render normally with a persistent `warning` banner naming the cap.
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
- **Icons:** [Lucide](https://lucide.dev) via `lucide-react`, bundled. 16px in tables and chips, 20px in the rail and top bar, 24px in empty states. `strokeWidth` 1.5 and `currentColor` always, so an icon inherits its context. One icon set only, and no emoji as an icon anywhere.
- **Motion:** 150ms ease-out for colour and opacity on hover, focus, and active. 200ms ease-out for overlay entrance; 150ms for exit. Nothing animates longer than 200ms, and layout, size, and position never animate on hover. Under `prefers-reduced-motion: reduce`, drop every transform and entrance animation and keep colour changes instant.

### Player profile layout

Dedicated route `/players/$uid` (not an inspector overlay). Comparison inspector remains unused until a later compare feature.

- **Page header:** player name as `headline-lg` title; no secondary nav-rail item — profiles are reached from Search and global suggest. Back uses normal history (Search URL state already holds filters).
- **Tabs:** segmented control under the header — **Overview** | **Attributes** | **Roles**. Active tab lives in the URL search param `tab` (`overview` | `attributes` | `roles`). One content panel below; do not stack all three sections on one scroll for MVP.
- **Overview:** identity block in a Panel — the same basics the Search default columns show (name, age/DOB, nationality, club, division, CA, PA, market value) plus height, preferred foot, and contract/transfer fields when present. One **hero** Score Badge for the best non-null role score (label + accessible name). Initials monogram optional; no crest or portrait (data constraint).
- **Attributes:** three (or four) sections inside one scrollable Panel, separated by hairlines or `stack-lg` — Visible attributes in FM-style groups (**Technical** / **Mental** / **Physical** / **Goalkeeping**), then **Hidden**, then **Personality**. Each attribute is a label + integer (1–20) with tabular figures; null → `—`. No radar chart in this feature.
- **Roles:** sections headed by **position family** (Goalkeeper → … → Striker). Every catalog role appears; do not mute or hide by positional familiarity. Each row: role display name, IP/OOP phase as secondary text or chip, **card** Score Badge. Null score → `—` without a fake badge number.
- **States:** no snapshot → EmptyState pointing at Load Data; unknown uid → “Player not in this snapshot”; loading skeletons match the active tab’s layout.
- **Out of this layout for now:** position suitability map, radar charts, history/trend blocks, compare inspector, combined IP/OOP weight controls.

### Squad planner layout

Dedicated route `/planner`. The planner keeps club-family setup, the shared dual-phase tactic editor, and the selected team's depth chart in one working page with three URL-backed workspaces: **Squad**, **Tactic**, and **Club setup**. Only the active workspace is visible, while all three remain mounted so local drafts and selections survive workspace changes. A configured save defaults to Squad; a loaded save without a primary club defaults to Club setup; an explicit valid `view` search value wins. Workspace changes replace the URL search state rather than adding browser-history entries. The route uses the active save and current snapshot shown in the global top bar.

- **Club family:** the page shows a setup panel when the active app save has no planner configuration. Choose a required primary club from a searchable snapshot-derived list. Every player at the primary club is eligible for Senior, Reserves, and Youth, regardless of the dump's `teamLevel`. Reserves and Youth each expose **Add associated club** for separately modeled B or youth teams; every player at an associated club joins that target team's pool. Do not suggest relationships from similar names. Missing configured clubs remain visible with a warning and a Replace action.
- **Page header:** `Squad Planner` as `headline-lg`, with the configured primary club shown as supporting context. The workspace tabs are **Squad** | **Tactic** | **Club setup**. The depth matrix adds **Senior** | **Reserves** | **Youth** team tabs. The tactic editor has **IP** | **OOP** | **Both** view tabs.
- **Tactic editor:** one tactic per app save, shared by all teams. The editor starts from a 4-3-3 DM In-Possession shape linked to a 4-1-4-1 DM Out-of-Possession shape, with compatible general-purpose roles already selected. Each phase uses an attack-up pitch with the striker band at the top and the goalkeeper band at the bottom, with 11 linked tactical positions arranged beside one selected-position inspector. Repeated base positions share a three-column band: one placement is centred; two use right then left; and three use right, centre, then left in stable tactic order. The same derived placement supplies spatial qualifiers in visible and accessible descriptions, regardless of role. Selecting a position opens its shared weight, rank, and foot settings plus the position and phase-compatible role controls for the visible phase or phases. Current IP/OOP positions and roles describe each selection; focus, hover, and selection emphasize the linked counterpart across both pitches without numeric markers. Both view places the two pitches side by side beside the inspector. Pointer dragging may move a position, but selecting the position and choosing a placement must provide the complete keyboard path. Invalid or incomplete role-position pairs cannot be saved.
- **IP/OOP weight:** each linked position has one compact control in the selected-position inspector outside the phase-specific controls, so **Both** view does not duplicate it. Every position defaults to `50 / 50`. Its control always shows both numeric weights and changes the combined score used throughout the Planner. It is position state, not a player or team override.
- **Position importance:** the selected-position inspector has one optional rank from 1 through 11. Ranks are unique, gaps are valid, and the control uses a visible label and native select behavior. The optimizer handles ranked positions in ascending order within the current team and string before it matches unranked positions.
- **Preferred foot:** the selected-position inspector has preferred foot (**Either**, **Left**, **Right**, or **Both**) and **Preferred** or **Strict** mode. Either means no foot restriction and disables the mode control. Controls use visible labels, native keyboard behavior, inline validation, and the existing failed-save draft retention.
- **Squad matrix:** the team tabs share one compact toolbar with **Optimize squads** and **Clear all** when the available width cannot hold every team. When the Planner matrix container can hold the current strings at their readable minimum widths, one semantic table groups **Senior**, **Reserves**, and **Youth** with ordered strings under each team header. Narrower containers show only the selected team's group with the existing keyboard-operable team tabs. Tactical positions are compact two-line rows in stable pitch order. Ordered strings are columns labelled **1st string**, **2nd string**, and so on. The matrix owns bounded horizontal and vertical overflow; its header and left position column stay sticky, and strings keep a readable minimum width rather than shrinking. Each position summary keeps the IP and OOP position/role context. Each occupied cell aligns the player name with one combined Score Badge. Unresolved and outside-pool assignments keep their occupied cell and show a warning; unknown scores render as `—`.
- **Player assignment:** activating an empty cell opens a picker Modal. Candidates are the union of sources assigned to the target team, so an attached B or youth club is included automatically. Results show player name, current club, IP score, OOP score, combined score, and any existing planner location. Rust sorts by combined score descending. Selecting an unassigned player fills the cell. Selecting an assigned player opens a confirmation that names the old and new locations; confirming moves the player. Activating an occupied cell instead asks the user to clear it. Closing restores focus to the originating cell.
- **String management:** every team starts with one string and keeps at least one. A visible header overflow button and right-click on the same header open identical actions: **Add string** and **Remove string**. The final remaining string disables Remove. Removing a populated string uses a destructive Modal. After removal, ordinal labels close the gap.
- **Squad actions:** **Optimize squads** is the primary action in the shared toolbar. It assigns eligible, unassigned players across all teams and strings using Rust-provided results, then reports pending, one latest success, or failure without masking blank gaps. **Clear all** is destructive and requires confirmation that names Senior, Reserves, and Youth. One Rust transaction removes both manual and optimized assignments from every string in the active save while preserving strings, tactics, club-family settings, scores, and other saves. The action stays in the shared toolbar in both matrix modes, prevents duplicate submission, retains errors, restores focus after close, and reconciles the displayed matrix and picker candidates after success.
- **States:** no snapshot uses the standard Load Data EmptyState. No planner configuration uses club-family setup. A configured source absent from the current snapshot shows a warning without deleting configuration or assignments. Loading skeletons match the matrix or pitch geometry. Tactic-save and assignment failures stay inline with the affected control and preserve the user's draft or prior assignment.
- **Accessibility:** pitches expose each tactical position as a labelled button with phase, position, and role in its accessible name. Position selection and placement changes work without drag, and the linked counterpart is described to assistive technology. Team and phase controls use proper tab semantics. Matrix cells have row and column context in their accessible names. Header menus support Escape and return focus to their trigger.
- **Data honesty:** combined scores use the persisted IP and OOP role scores from the current snapshot with the position's IP/OOP weight. Missing phase scores produce `—`, not a partial or zero score. A player outside the current club-family sources or absent from the snapshot remains assigned but carries a visible warning.

### Deferred specs

These surfaces are not specced because their features are not planned yet. Spec them in this document during `$workflow-plan-feature` for the relevant feature, not before.

- **Optimizer extensions:** formation comparison, best-and-worst candidate highlighting, and gap recommendations.
- **Profile extensions:** position suitability map, attribute/role radar, comparison inspector, snapshot history on the profile.

---

## Pre-Delivery Checklist

Verify before delivering any UI code.

### Visual Quality

- [ ] No emojis used as icons — Lucide only, `strokeWidth` 1.5, `currentColor`
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
