---
name: FM ValueScout
colors:
    # Foundation — background & surface elevation layers
    background: "oklch(0 0 0)"
    on-background: "oklch(0.92 0 0)"
    surface-dim: "oklch(0 0 0)"
    surface: "oklch(0.1 0 0)"
    surface-bright: "oklch(0.18 0 0)"
    surface-container-lowest: "oklch(0.04 0 0)"
    surface-container-low: "oklch(0.08 0 0)"
    surface-container: "oklch(0.1 0 0)"
    surface-container-high: "oklch(0.14 0 0)"
    surface-container-highest: "oklch(0.18 0 0)"
    on-surface: "oklch(0.92 0 0)"
    on-surface-variant: "oklch(0.65 0 0)"
    inverse-surface: "oklch(0.92 0 0)"
    inverse-on-surface: "oklch(0.18 0 0)"
    # Borders & outlines
    outline: "oklch(0.35 0 0)"
    outline-variant: "oklch(0.27 0 0)"
    surface-tint: "oklch(0.63 0.14 256)"
    # Primary — main interactive and brand colour
    primary: "oklch(0.63 0.14 256)"
    on-primary: "oklch(0.09 0 0)"
    primary-container: "oklch(0.2 0.08 256)"
    on-primary-container: "oklch(0.82 0.06 256)"
    inverse-primary: "oklch(0.3 0.16 256)"
    # Secondary — highlights, CTAs, emphasis
    secondary: "oklch(0.75 0.18 86)"
    on-secondary: "oklch(0.12 0 0)"
    secondary-container: "oklch(0.3 0.08 86)"
    on-secondary-container: "oklch(0.88 0.12 86)"
    # Tertiary — subtle accents
    tertiary: "oklch(0.68 0.16 306)"
    on-tertiary: "oklch(0.12 0.02 306)"
    tertiary-container: "oklch(0.25 0.1 306)"
    on-tertiary-container: "oklch(0.82 0.1 306)"
    # Semantic — status indicators
    success: "oklch(0.72 0.2 154)"
    on-success: "oklch(0.08 0.02 154)"
    success-container: "oklch(0.16 0.08 154)"
    on-success-container: "oklch(0.85 0.12 154)"
    warning: "oklch(0.82 0.16 86)"
    on-warning: "oklch(0.14 0.02 86)"
    warning-container: "oklch(0.28 0.1 86)"
    on-warning-container: "oklch(0.88 0.12 86)"
    error: "oklch(0.65 0.2 27)"
    on-error: "oklch(0.1 0.02 27)"
    error-container: "oklch(0.28 0.12 27)"
    on-error-container: "oklch(0.88 0.08 27)"
    info: "oklch(0.72 0.14 226)"
    on-info: "oklch(0.08 0.01 226)"
    info-container: "oklch(0.18 0.08 226)"
    on-info-container: "oklch(0.85 0.08 226)"
    # Fixed tonal pairs (optional — keep if the design uses fixed accent tints)
    primary-fixed: "oklch(0.85 0.06 256)"
    primary-fixed-dim: "oklch(0.63 0.14 256)"
    on-primary-fixed: "oklch(0.09 0 0)"
    on-primary-fixed-variant: "oklch(0.2 0.08 256)"
    secondary-fixed: "oklch(0.9 0.1 86)"
    secondary-fixed-dim: "oklch(0.75 0.18 86)"
    on-secondary-fixed: "oklch(0.12 0 0)"
    on-secondary-fixed-variant: "oklch(0.3 0.08 86)"
    tertiary-fixed: "oklch(0.88 0.08 306)"
    tertiary-fixed-dim: "oklch(0.68 0.16 306)"
    on-tertiary-fixed: "oklch(0.12 0.02 306)"
    on-tertiary-fixed-variant: "oklch(0.25 0.1 306)"
typography:
    headline-lg:
        {
            fontFamily: "[Display Font]",
            fontSize: 28px,
            fontWeight: "600",
            lineHeight: "1.3",
            letterSpacing: -0.01em,
        }
    headline-md:
        {
            fontFamily: "[Display Font]",
            fontSize: 22px,
            fontWeight: "600",
            lineHeight: "1.35",
        }
    headline-sm:
        {
            fontFamily: "[Display Font]",
            fontSize: 18px,
            fontWeight: "500",
            lineHeight: "1.4",
        }
    body-lg:
        {
            fontFamily: "[Body Font]",
            fontSize: 16px,
            fontWeight: "400",
            lineHeight: "1.6",
        }
    body-md:
        {
            fontFamily: "[Body Font]",
            fontSize: 14px,
            fontWeight: "400",
            lineHeight: "1.6",
        }
    body-sm:
        {
            fontFamily: "[Body Font]",
            fontSize: 13px,
            fontWeight: "400",
            lineHeight: "1.5",
        }
    label-lg:
        {
            fontFamily: "[Body Font]",
            fontSize: 14px,
            fontWeight: "600",
            lineHeight: "1.2",
            letterSpacing: 0.02em,
        }
    label-md:
        {
            fontFamily: "[Body Font]",
            fontSize: 12px,
            fontWeight: "600",
            lineHeight: "1.2",
            letterSpacing: 0.05em,
        }
    label-sm:
        {
            fontFamily: "[Body Font]",
            fontSize: 11px,
            fontWeight: "600",
            lineHeight: "1.2",
            letterSpacing: 0.08em,
        }
    # Monospace roles for numeric/tabular data
    mono-lg:
        {
            fontFamily: "[Mono Font]",
            fontSize: 24px,
            fontWeight: "600",
            lineHeight: "1.2",
        }
    mono-md:
        {
            fontFamily: "[Mono Font]",
            fontSize: 14px,
            fontWeight: "500",
            lineHeight: "1.4",
        }
    mono-sm:
        {
            fontFamily: "[Mono Font]",
            fontSize: 12px,
            fontWeight: "500",
            lineHeight: "1.4",
        }
rounded:
    none: 0
    sm: 0.25rem
    DEFAULT: 0.375rem
    md: 0.5rem
    lg: 0.75rem
    xl: 1rem
    full: 9999px
spacing:
    unit: 4px
    # Add product-specific named dimensions as rows below
    table-row-height: [px]
    header-height: [px]
    gutter: [px]
    stack-xs: 4px
    stack-sm: 8px
    stack-md: 16px
    stack-lg: 24px
    stack-xl: 32px
    content-max-width: [value or none]
---

## Brand & Style

[Name the central design concept or metaphor in 1 phrase, then 2-3 short paragraphs: what mood/aesthetic the product targets, who the user is and in what context they use it, and the guiding stylistic tension (e.g. dense vs. airy, light vs. dark, playful vs. precise). State any hard stances — e.g. "no light mode", "desktop-only", "OLED-first".]

## Colors

[Describe the palette strategy in a sentence or two. Explain how elevation is conveyed (tonal layering vs. shadows) and why, given the product's needs.]

**Primary ([hue]):** [What it signals and where it is used.]

**Secondary ([hue]):** [What it signals and where it is used.]

**Tertiary ([hue]):** [What it signals and where it is used.]

**Semantic Colours:** Four fixed roles for status indicators:

| Semantic  | oklch        | Role                       |
| --------- | ------------ | -------------------------- |
| `success` | `oklch(...)` | [When this colour appears] |
| `warning` | `oklch(...)` | [When this colour appears] |
| `error`   | `oklch(...)` | [When this colour appears] |
| `info`    | `oklch(...)` | [When this colour appears] |

[Borders (`outline` / `outline-variant`) are deliberately low-contrast — just visible enough to contain a table or card without drawing attention from the data inside them.]

### Accessibility of Colour

**Colour is never the sole indicator of meaning.** [State how each colour-coded element also carries text/icon/pattern — status badges pair colour + label, KPI cards pair accent border + icon, error states pair red border + icon + descriptive text.]

**Contrast compliance:** [State the target standard, e.g. WCAG 2.2 AA (4.5:1 minimum) for normal text and AAA (7:1) for body text on surface containers.] Verified pairings:

| Text Role | Foreground               | Background               | Ratio         |
| --------- | ------------------------ | ------------------------ | ------------- |
| [Role]    | `[token]` (`oklch(...)`) | `[token]` (`oklch(...)`) | ~X:1 (AA/AAA) |
| [Role]    | `[token]` (`oklch(...)`) | `[token]` (`oklch(...)`) | ~X:1 (AA/AAA) |

## Typography

[Describe the type strategy: chosen typeface(s) and why, and the pairing rationale.]

- **[Font 1] ([category]):** [Where used and why.]
- **[Font 2] ([category]):** [Where used and why.]

**Scale principle:** [How headline/body/label/mono roles are applied — e.g. headlines use the display font for a distinctive identity, body and labels use the body font for long-session comfort, monospace roles are reserved for numeric data.]

[State how fonts are loaded — bundled/self-hosted/google-fonts — and the rationale.]

### [Optional] Value & Number Formatting

[If the product displays formatted data, specify formatting rules: currency (prefixed, decimal places), counts (raw integer vs. abbreviated), ratings (glyph-based vs. numeric), tabular figures, abbreviation thresholds.]

## Design Principles

[Non-negotiable constraints every component and interaction must satisfy. Number them; each is a short imperative rule with a concrete test. Adapt to the product — the list below is a starting set.]

1. **[Principle]:** [Rule + how it's verified.]
2. **[Principle]:** [Rule + how it's verified.]
3. **[Principle]:** [Rule + how it's verified.]

## Layout & Spacing

[Describe the top-level layout model (single-window tabbed dashboard, multi-pane, responsive page, etc.), the target/minimum viewport width, and the primary visual regions in order (header bar, navigation, content area).]

[State the spacing rhythm — which named stack steps are used between related elements, sections, and major blocks. Mention any hard layout dimensions (table row height, card widths, header height) and whether content width is clamped or fills available space.]

## Elevation & Depth

[Describe how depth is expressed (tonal layering vs. drop shadows) and why this matters for the product's density needs. Enumerate elevation levels:]

- **Level 0 ([name]):** `[token]` — [what lives at this level].
- **Level 1 ([name]):** `[token]` — [what lives at this level].
- **Level 2 ([name]):** `[token]` — [what lives at this level].
- **Level 3 ([name]):** `[token]` — [what lives at this level.]

### Z-Index Scale

[State the fixed z-index scale — no element uses arbitrary large values. Layers are separated by steps of 10 to leave room for future insertion.]

| Layer         | Value  | Usage                                   |
| ------------- | ------ | --------------------------------------- |
| Base          | `z-0`  | Content area, tables, cards             |
| Sticky        | `z-10` | Sticky headers, sticky nav              |
| Dropdown      | `z-20` | Select dropdowns, autocomplete panels   |
| Context Menu  | `z-30` | Right-click context menus               |
| Overlay       | `z-40` | Modal backdrops, toast container region |
| Modal Content | `z-50` | Modal dialogs, individual toasts        |

[Every component that creates a new stacking context declares its `z-index` from this scale. Components at the same layer must not overlap in normal use; if they can, use source-order stacking within the layer.]

## Shapes

[Name the shape language — e.g. "Sharp-Industrial" or "Soft-Rounded" — and map radii to element classes.]

- **[Element class]:** `[radius token]` ([value]). [Rationale if non-obvious.]
- **Focus Rings:** [Width, colour, radius. Always `:focus-visible`, never `:focus`.]

## Components

Spec each reusable component with the skeleton below. Duplicate the block per component. Order from most-used to least. Reference design tokens by name (never raw values in prose).

### [Component Name]

[One-line purpose: what it is and where it is used.]

- **Container:** [Background token, border, radius, padding, sizing constraints.]
- **States:** [Default / hover / active / focus / disabled / loading — describe the visual change for each and the transition used. No layout-shifting hovers.]
- **Variants:** [List each variant and what differs (colour, size, role).]
- **Content / Anatomy:** [Internal structure — icon, label, value, subtext — with the token/type-role for each.]
- **Behaviour:** [Interaction, keyboard support, feedback, error/empty/loading handling relevant to this component.]

<!--
Common components to spec for most products (add/remove as needed):
- Buttons & Interaction (primary, secondary, ghost, destructive, icon variants)
- Input Fields (text, number, date, select, textarea)
- Data Tables / Lists
- Modals (informational, form, destructive confirmation)
- Context Menu
- Toast / Notification
- Tabs / Navigation
- Status Badges / Chips
- Empty States
- Loading States / Skeleton Screens
- Error Boundary
- Scrollbars (custom-styled)
- Focus Ring & Keyboard Navigation
- Window Chrome (if desktop app)
- Animations & Transitions
- Icon System
- Typography Overflow / Truncation
-->

---

## Pre-Delivery Checklist

Before delivering any UI code, verify. [Adapt items to the product; grouped by concern.]

### Visual Quality

- [ ] No emojis used as icons (use your chosen icon library instead)
- [ ] All icons from a consistent icon set (identical viewBox, stroke width, join style)
- [ ] Colour is never the sole indicator of meaning — paired with text, icon, or pattern
- [ ] All text-on-background combinations meet the contrast minimum (verify against the contrast table in the Colors section)

### Interaction

- [ ] `cursor-pointer` on all clickable elements
- [ ] Hover states use colour/opacity transitions only — no layout-shifting effects (scale, margin, padding, font-weight changes on hover)
- [ ] Focus states visible only via `:focus-visible` (not `:focus`)
- [ ] Every mutation shows loading → success/error feedback — no silent updates
- [ ] Destructive actions require explicit confirmation before executing

### Accessibility

- [ ] Skip link present and functional (first Tab press)
- [ ] All interactive elements reachable via keyboard in logical Tab order
- [ ] Modals trap focus and dismiss on Escape
- [ ] Context menus navigable with arrow keys, dismissable on Escape
- [ ] `prefers-reduced-motion: reduce` respected — non-essential animations disabled

### Z-Index & Layout

- [ ] All `z-index` values come from the defined scale (10/20/30/40/50) — no arbitrary values
- [ ] No content hidden behind sticky navigation regions (account for combined height)

### States

- [ ] Loading, empty, and error states defined for every data view — never blank space
- [ ] Optimistic updates (if used) roll back on failure with an error message
- [ ] Toast auto-dismiss timers pause on hover
