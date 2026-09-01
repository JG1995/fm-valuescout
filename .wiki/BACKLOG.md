# Backlog

Ideas, improvements, and tasks worth remembering but not scheduled for the near term. When something moves from "we should do this eventually" to "we plan to do this soon", it graduates to [TODO.md](./TODO.md).

This is the parking lot — aspirational work, deferred features, technical debt, investigations, and anything else that would be useful but has no committed delivery window.

---

## High

_None._

## Medium

- **Historical Moneyball seasons** — CSV enrichment now keeps one latest Moneyball row per snapshot and player, while Youth career values remain save-scoped; Moneyball still has no season identity. **Trigger:** need to compare exported seasons, import waves, or trends. **Completion:** define an explicit season key and amendment rules, retain historical canonical statistics without changing current snapshot-row semantics, and add the required read model and UI. See the [snapshot history record](./features/completed/snapshot-history.md) for the current ownership boundary.

- **Individual Player Profile timeline** — Snapshot history retains raw player facts, while current-snapshot compact metrics intentionally keep Search, Profile, Planner, and Academy on the effective latest snapshot. **Trigger:** need to inspect one player's earlier state or chart development over time. **Completion:** add a Player Profile timeline that reads that player's retained raw facts across snapshots and recalculates projected attributes and role scores on demand with the then-current projection and scoring models. Keep latest-only default reads, do not persist historical projected attributes or role-score matrices, and do not imply that the current compact-metrics feature implements this UI.

- **In-app BepInEx bootstrap** — Install BepInEx 6 IL2CPP into the Steam FM26 folder from the app, not only `FmDataBridge.dll`. DLL-only install is delivered ([bridge-plugin-install](./features/completed/bridge-plugin-install.md)); users without BepInEx still need a manual setup step. **Trigger:** onboarding friction or support burden from missing BepInEx. **Completion:** app bootstraps BepInEx (or equivalent guided install) on default Steam path; documented in bridge README; does not replace DLL-only remove/update semantics.

- **In-app bridge DLL build-before-copy** — Install/Update plugin copies a pre-bundled `src-tauri/resources/FmDataBridge.dll`, not a fresh `dotnet build` from `bridge/`. Source changes (e.g. scan cap) do not reach the game folder until a developer rebuilds the bridge, copies into `resources/`, and restarts or rebuilds Tauri — or runs `./scripts/dev bridge-install`. **Trigger:** stale-plugin confusion during dev; release builds accidentally shipping an outdated bundled DLL. **Directions:** run `dotnet build` (or copy a CI-built artifact) immediately before install copy; version/hash the output; surface build failures in the install UI; keep `bridge-install` as the direct dev path. **Completion:** Install/Update always deploys a DLL built from current sources (dev) or the release artifact (prod); placeholder bundled DLL no longer required for real in-app testing.

- **Real WebView e2e (tauri-driver)** — Add [tauri-driver](https://v2.tauri.app/develop/tests/webdriver/) + WebdriverIO or Selenium when Playwright stub smoke hides real WebView integration bugs (IPC wiring, capabilities, platform WebView behaviour). **Trigger:** repeated regressions that Vitest + `cargo test` miss but manual `pnpm tauri dev` catches. **Not for template scaffold** — smoke scope is documented in [ARCHITECTURE.md](./ARCHITECTURE.md) §6.4. **Completion:** optional CI job or documented fork path; template gate stays stub-smoke unless a product needs real-app automation.

## Low / Icebox

[Interesting ideas, speculative improvements, or items blocked on external factors. Review periodically — most will stay here.]

## Technical Debt

[Architecture, tooling, or code-quality issues that add friction. These are not scheduled but should be tracked so the cost is visible.]

Each debt entry should capture the current evidence, the target state, the risk of leaving it, and the completion criteria. See [ARCHITECTURE.md](./ARCHITECTURE.md) for the current system description.

---

## Maintenance

- **Promote** entries to TODO.md when they become planned work — move the item text, don't duplicate it.
- **Demote** entries from TODO.md here when planned work gets deprioritised.
- **Prune** entries that are no longer relevant. The backlog is not a landfill.
- **Review** the backlog every few cycles to promote, prune, or re-prioritise.
