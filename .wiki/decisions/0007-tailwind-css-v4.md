# 0007 — Tailwind CSS v4

## Status

Accepted

## Context

The template ships [DESIGN.md](../DESIGN.md) with OKLCH design tokens for web UI. Developers need fast layout and spacing iteration without a heavy component library lock-in. Styling must work with Vite and stay maintainable for solo developers.

## Decision

Use **Tailwind CSS v4** with the **`@tailwindcss/vite`** plugin. Bridge [DESIGN.md](../DESIGN.md) tokens into CSS `@theme` so components use token-backed utilities instead of raw color values in JSX.

## Alternatives considered

### CSS modules + custom properties

CSS modules map directly to DESIGN.md tokens with no utility framework. Layout and responsive work are slower. Every spacing decision is manual CSS.

### Tailwind CSS v3

v3 is still widely used but v4 is the current major line (stable since January 2025; v4.3.x active July 2026). v4 integrates with Vite via a dedicated plugin and simplifies configuration.

### Component library (MUI, Chakra, etc.)

Full UI kits speed common widgets but lock visual language and bundle size. This template owns visual design through DESIGN.md and headless patterns.

### Styled-components / CSS-in-JS

Runtime CSS-in-JS adds bundle and runtime cost. Tailwind keeps styles static and aligns with utility-first iteration.

## Consequences

### Positive

- Fast UI iteration for solo developers.
- v4 Vite plugin is the recommended Tailwind integration for Vite projects.
- `@theme` can expose OKLCH tokens from DESIGN.md.

### Negative

- Utility-class markup is unfamiliar to some developers.
- Tailwind v4 targets modern browsers (Safari 16.4+, Chrome 111+, Firefox 128+) — not suitable for legacy browser forks without a stack change.
- v4 does not support Sass/Less as a preprocessor — use CSS-native tokens.

### Follow-up

- Done at scaffold (`fc11b5a`) — `src/styles/global.css` with `@import "tailwindcss"` and minimal `@theme` tokens.
- Done on `feat/design-system` — full [DESIGN.md](../DESIGN.md) token bridge, IBM Plex via `@fontsource`, Lucide icons, and shared UI primitives.
- ui-design skill and DESIGN.md remain source for semantic color roles.

## Related work

- Commits: `fc11b5a`
- Supersedes: none
