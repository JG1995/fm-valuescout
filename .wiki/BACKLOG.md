# Backlog

Ideas, improvements, and tasks worth remembering but not scheduled for the near term. When something moves from "we should do this eventually" to "we plan to do this soon", it graduates to [TODO.md](./TODO.md).

This is the parking lot — aspirational work, deferred features, technical debt, investigations, and anything else that would be useful but has no committed delivery window.

---

## High

[Feasible and valuable, but not yet planned. These would move to TODO.md Next if priorities shift or capacity opens up.]

## Medium

- **In-app BepInEx / FM bridge install and remove** — Let the Tauri app install or remove the memory-read plugin DLL (and optionally ensure BepInEx is present) for Steam FM26, instead of manual copy into `BepInEx/plugins`. **Deferred from** [FM26 memory read](./features/active/fm26-memory-read.md) MVP (manual install is enough). **Trigger:** sharing the app beyond the author, or install friction blocking Load Data. **Constraints:** avoid brittle privilege/AV fights; do not remove BepInEx when other plugins may use it; keep Steam path detection Windows-only until other stores matter. **Completion:** reversible install from the app, clear failure states, documented security/AV expectations.
- **Real WebView e2e (tauri-driver)** — Add [tauri-driver](https://v2.tauri.app/develop/tests/webdriver/) + WebdriverIO or Selenium when Playwright stub smoke hides real WebView integration bugs (IPC wiring, capabilities, platform WebView behaviour). **Trigger:** repeated regressions that Vitest + `cargo test` miss but manual `pnpm tauri dev` catches. **Not for template scaffold** — smoke scope is documented in [ARCHITECTURE.md](./ARCHITECTURE.md) §6.4; ponytail in `tauri.md` matches this trigger. **Completion:** optional CI job or documented fork path; template gate stays stub-smoke unless a product needs real-app automation.

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
