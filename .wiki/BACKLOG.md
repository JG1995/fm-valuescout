# Backlog

Ideas, improvements, and tasks worth remembering but not scheduled for the near term. When something moves from "we should do this eventually" to "we plan to do this soon", it graduates to [TODO.md](./TODO.md).

This is the parking lot — aspirational work, deferred features, technical debt, investigations, and anything else that would be useful but has no committed delivery window.

---

## High

- **Bridge scan performance (full player dump)** — First live Cap A dump on FM 26.3.2 scanned ~4.1 GB / ~7.5M vtable hits and took ~3m 47s for ~184k accepted rows, longer than the app’s 120s request wait. Temporary early-stop cap (`PersonScanner.DefaultMaxAccepted`) keeps manual testing usable. **Trigger:** need complete dumps for ingest, or Load Data must finish a full DB inside a bounded UI wait. **Directions:** tighter region filters, better person/player discrimination before CA/PA reads, streaming dump write, progress in `status.json`, longer/adaptive wait, request-driven `maxPlayers`. **Completion:** full-DB dump finishes reliably within an agreed budget; remove or raise the testing cap; document expected scan time on a reference save.

## Medium

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
