# FM ValueScout

A React + Tauri desktop application with a test-driven, AI-assisted development workflow in Cursor.

The template ships a React + Tauri v2 walking skeleton (thin frontend, thick Rust backend, SQLite via IPC) with TanStack Router/Query, Zustand, Tailwind v4, Vitest with `mockIPC`, Playwright smoke, Biome, and Rust `cargo fmt/clippy/test` in the gate — plus `./scripts/dev` wiring, durable documentation, and Cursor commands, agents, skills, and MCP configuration.

## Quick start

```bash
pnpm install
pnpm exec playwright install chromium
./scripts/dev check
./scripts/dev test
pnpm tauri dev
```

Install the [Rust toolchain](https://rustup.rs/) and Tauri Linux system packages before `pnpm tauri dev` on Linux/WSL (see [.wiki/ARCHITECTURE.md](.wiki/ARCHITECTURE.md) §11). WSLg or an X server is required for the native window on WSL.

`pnpm tauri dev` is the default dev loop — WebView + Rust IPC with real SQLite persistence. `pnpm dev` serves the frontend only; IPC calls fail unless stubbed (tests and Playwright smoke use stubs). `./scripts/dev check` runs Biome, TypeScript, secretlint, repository contract tests, Playwright smoke, and `cargo fmt/clippy/test`. Run `pnpm exec playwright install chromium` once after install so smoke can run.

## Forking this template

The walking skeleton is already implemented — IPC health demo, SQLite persistence, tests, and CI gates. You do not need `/stack` unless you change the default stack.

### New project checklist

1. Install prerequisites — Node 24, pnpm, Rust, and Linux/WSL system packages ([ARCHITECTURE §11](.wiki/ARCHITECTURE.md)).
2. `pnpm install` — installs Node packages and Husky hooks.
3. `pnpm exec playwright install chromium` — once, so smoke and `./scripts/dev check` can run.
4. `./scripts/dev check` and `./scripts/dev test` — confirm the gate is green before you build features.
5. Install recommended editor extensions from [.vscode/extensions.json](.vscode/extensions.json) — Biome, rust-analyzer, Even Better TOML.
6. ~~Rename the template identity~~ — done (`FM ValueScout` / `fm-valuescout`).
7. ~~Recallium `project_name` in `AGENTS.md`~~ — done (`fm-valuescout`).
8. Fill [.wiki/CONCEPT.md](.wiki/CONCEPT.md) — especially MVP scope and boundaries.
9. Run `/roadmap` when CONCEPT has real bullets — approve the sequence in [.wiki/TODO.md](.wiki/TODO.md).
10. Run `/plan-feature` on **Plan next** (or the first sequence row), then the build loop below.

Skip `/stack` when you keep this template's defaults. Run `/stack` only when you need to change stack choices and reconcile [.wiki/ARCHITECTURE.md](.wiki/ARCHITECTURE.md) §1.

Optional before `/roadmap`: add feature specs in `.wiki/features/planned/<slug>.md` when you can describe user-visible behavior — CONCEPT bullets alone suffice for a provisional sequence.

### Rename checklist (completed)

Identity set to **FM ValueScout** (`fm-valuescout`, Tauri identifier `app.fmvaluescout`). Reference for future renames:

| What | File(s) |
| --- | --- |
| npm package name | `package.json` → `name` |
| Desktop app identity | `src-tauri/tauri.conf.json` → `productName`, `identifier`, window `title` |
| Rust crate metadata | `src-tauri/Cargo.toml` → `description` (and `name` if you rename the crate) |
| Home page heading | `src/app/routes/index.tsx` |
| Tests that assert the title | `e2e/smoke.spec.ts`, `src/app/app-shell-routing.test.tsx` |
| Repo and architecture titles | `README.md`, [.wiki/ARCHITECTURE.md](.wiki/ARCHITECTURE.md) title and §1 |
| Product wiki names | [.wiki/CONCEPT.md](.wiki/CONCEPT.md), [.wiki/DESIGN.md](.wiki/DESIGN.md) → `name` / title fields |

## Development loop

The workflow turns product notes into atomic, reviewed commits on trunk. After the [forking checklist](#forking-this-template), each feature is planned with `/plan-feature`, built one commit at a time (`/build` → `/checkpoint`, with `/fix` when review blocks), and closed with `/finish-feature`. See [.cursor/README.md](.cursor/README.md) for the full cycle.

## Commands

- `pnpm tauri dev` — open the desktop app with WebView + Rust IPC (default dev loop)
- `pnpm tauri build` — build OS installers (unsigned by default)
- `pnpm dev` — frontend-only Vite dev server (IPC stub required for feature code)
- `./scripts/dev test [args...]` — run `vitest run` (full suite or forwarded file patterns and flags)
- `./scripts/dev check-fast` — fast pre-commit path (Biome, TypeScript, staged secretlint)
- `./scripts/dev check` — full gate including contract tests, smoke contract, and Rust (CI runs this)
- `./scripts/dev format [paths...]` — Biome lint/format fixes (`biome check --write`), then `cargo fmt` in `src-tauri/`; optional paths forward to Biome only; run before checkpoint, not in CI
- `./scripts/dev secrets [--staged]` — scan for secrets with secretlint (full tree or staged files only)
- `./scripts/dev smoke` — Playwright smoke (`e2e/smoke.spec.ts`; stub IPC in Chromium, not real WebView or SQLite — see ARCHITECTURE §6.4; run `pnpm exec playwright install chromium` once after install)
- `./scripts/dev mutate <target>` — scoped mutation tests (not configured yet)
- `/stack`, `/roadmap`, `/plan-feature`, `/build`, `/fix`, `/checkpoint`, `/review`, `/docs-review`, `/finish-feature` — core workflow (see `.cursor/commands/`)
- `/spike` — optional disposable experiment when a technical question needs runtime evidence (not in the main loop)
- `/security-audit` — optional read-only security audit before deploy or after auth, payments, or sensitive data (not in the main loop)
- Dispatch Reviewer or Documentation Steward via `.cursor/agents/`; use Cursor's built-in `explore` subagent for fast codebase search

## Cursor MCP

Project MCP servers in [.cursor/mcp.json](.cursor/mcp.json):

- **recallium** — persistent project memory at `http://10.189.1.195:8001/mcp`
- **context7** — current library documentation

Disable duplicate marketplace plugins in **Customize** if you prefer project-level servers only.

## Documents

- [Forking checklist](#forking-this-template) — prerequisites, rename table, CONCEPT → roadmap → plan-feature
- [Cursor workflow](.cursor/README.md) — MCP, hooks, per-feature planning, build, checkpoint, and finish-feature
- [Cursor MCP](.cursor/mcp.json) — Recallium and Context7
- [Contributing](CONTRIBUTING.md) — gate, commits, fork-merge workflow
- [Development contract](AGENTS.md) — how to make changes
- [Scripts](scripts/dev) — the stable command surface
- [Wiki](.wiki/INDEX.md) — durable project knowledge
- [Architecture](.wiki/ARCHITECTURE.md) — implemented stack, layout, and data flow
- [Architecture decision records](.wiki/decisions/README.md) — consequential choices
- [Planned work](.wiki/TODO.md) — development sequence and delivery intent

## License

MIT — see [LICENSE](LICENSE).
