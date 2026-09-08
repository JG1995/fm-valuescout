---
name: ui-inspection
description: Capture and inspect populated FM ValueScout pages in Playwright Chromium. Use for material UI, layout, styling, responsive, or screenshot-comparison work and when an agent must see the rendered application after an edit.
compatibility: Requires project dependencies and Playwright Chromium.
---

# UI inspection

## Purpose

Use the project-owned inspection command to see rendered UI with representative data. The command captures Chromium screenshots from the Vite application with the existing Tauri IPC stub.

Use this skill when work changes:

- layout, spacing, typography, color, or visual hierarchy;
- responsive behavior or viewport containment;
- page, panel, table, modal, or navigation presentation;
- UI that must be compared with a supplied mockup or screenshot.

Do not run the complete page set for a narrow component change. Do not use these screenshots as proof of native behavior.

## Prerequisites

Install project dependencies and Chromium once:

```bash
pnpm install
pnpm exec playwright install chromium
```

Use `./scripts/dev` for inspection and validation. Do not invoke the inspection spec directly unless you are diagnosing the command itself.

## Capture one route

Run:

```bash
./scripts/dev inspect-ui <route> [width] [height]
```

The default viewport is `1600×900`. The minimum accepted viewport is `1280×720`.

Examples:

```bash
./scripts/dev inspect-ui /search
./scripts/dev inspect-ui '/search?view=moneyball'
./scripts/dev inspect-ui '/players/42?section=role-fit' 1920 1080
./scripts/dev inspect-ui '/academy?view=class&classId=7' 1280 800
```

Quote a route when it contains `&` or other shell metacharacters. The command accepts only local routes that begin with one `/`; it rejects external and protocol-relative URLs.

## Capture the canonical page set

Use the complete set only when a change affects shared shell, navigation, global styles, shared tables, or several features:

```bash
./scripts/dev inspect-ui all
```

This captures:

1. Dashboard
2. Search
3. Moneyball Search
4. Player Shortlist
5. Staff Search
6. My Staff
7. Staff Shortlist
8. Squad
9. Planner
10. Tactic
11. Academy Overview
12. Academy Graduates
13. Academy Class
14. Settings
15. Player Profile Overview
16. Player Profile Attributes
17. Player Profile Role Fit
18. Player Profile Moneyball
19. Staff Profile

The canonical captures wait for their representative content before writing an image. The shared shell must also show the active save and current snapshot date. A capture fails on a page error, a visible busy state that does not settle, or missing canonical content.

## Inspect the result

The command writes ignored files under:

```text
.work/ui-inspection/
```

Each filename includes the page or route slug and viewport:

```text
.work/ui-inspection/search-1600x900.png
.work/ui-inspection/player-profile-role-fit-1920x1080.png
```

Open the exact path printed by the command. In Pi, call the image-capable `read` tool:

```text
read({ path: ".work/ui-inspection/search-1600x900.png" })
```

A human can open the file with an editor or image viewer. On Linux:

```bash
xdg-open .work/ui-inspection/search-1600x900.png
```

On WSL:

```bash
explorer.exe "$(wslpath -w .work/ui-inspection/search-1600x900.png)"
```

A later capture of the same route at the same viewport replaces the prior file.

## Iteration loop

For material UI work:

1. Read the governing design and component context.
2. Capture the affected route before editing when a visual baseline is useful.
3. Open and inspect the screenshot.
4. Make the smallest requested UI change.
5. Capture the same route and viewport again.
6. Inspect the new screenshot for hierarchy, readability, clipping, overflow, focus visibility, and consistency with the requested design.
7. Repeat only while a concrete visual problem remains.
8. Run affected tests and repository validation separately.
9. Remove disposable screenshots during cleanup.

Use the same viewport before and after an edit. Add another supported viewport only when responsive behavior is in scope.

## Populated data

The inspection spec combines existing browser fixtures and generated rows. It provides representative data for:

- active save and current snapshot metadata;
- snapshot history;
- player Search, Moneyball, Shortlist, and Squad tables;
- Player Profile attributes, role fit, and Moneyball data;
- Planner depth and Tactic configuration;
- Staff Search, My Staff, Staff Shortlist, profile, and assignment data;
- Youth Academy classes, members, graduates, and candidates.

The data is synthetic. Do not infer production database, migration, scoring, or integration behavior from it.

## Inspection versus validation

`inspect-ui` creates visual evidence for an agent or maintainer. It is not a quality gate and has no pixel-baseline comparison.

Run the normal browser suite separately:

```bash
./scripts/dev smoke
```

`smoke` runs only `e2e/smoke.spec.ts`. It does not generate inspection screenshots. `inspect-ui` runs only `e2e/ui-inspection.spec.ts`.

A green inspection or smoke run does not prove:

- the native Tauri WebView;
- Rust command execution;
- SQLite persistence or migrations;
- native dialogs or file access;
- the FM bridge or a live Football Manager process;
- platform-specific rendering.

Use the validation contract in `AGENTS.md` and the boundaries in `.wiki/ARCHITECTURE.md` for those concerns.

## Boundaries and cleanup

- Do not commit `.work/ui-inspection/` images.
- Do not add temporary screenshot calls to `e2e/smoke.spec.ts`.
- Do not weaken smoke assertions to make an inspection capture pass.
- Do not claim pixel-perfect comparison without an explicit comparison method.
- Do not run `all` when one route supplies the needed evidence.
- Do not treat hidden or off-screen content as inspected; capture the relevant route and viewport.

Remove inspection output after use:

```bash
rm -rf -- .work/ui-inspection
```

## Troubleshooting

### The command reports missing dependencies

Run `pnpm install`. Do not install a second browser library.

### Chromium is missing

Run:

```bash
pnpm exec playwright install chromium
```

### Port 5173 is already in use

Playwright reuses the existing Vite server during local runs. If another application owns that port or the screenshot does not match FM ValueScout, stop that process and rerun the command.

### A populated readiness check fails

Read the Playwright error context. Confirm whether the page failed to load, the fixture became stale, or the expected UI contract changed. Update canonical inspection data or readiness markers only when the application contract changed; do not replace them with arbitrary delays.

### The requested state requires interaction

The command captures the initial state of a route. Use an existing focused Playwright test when it already reaches the required modal or interaction state. Add a canonical inspection case only when that state will be inspected repeatedly; do not add one-off state machinery for a single screenshot.
