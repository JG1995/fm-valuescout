# Bridge scan performance

## Intent

Make **Load Data** fast enough to ingest a complete FM26 player database. Replace the bridge's syscall-heavy scalar heap walk with safe block scanning, enable unlimited production loads after live validation, then expose a UI toggle and optional numeric cap for faster diagnostic loads.

## Delivered behavior

- A 500-player diagnostic load completes in under 10 seconds on the reference machine (~6.7s measured on FM 26.3.2).
- A complete reference save (~181,000 players) finishes the bridge dump in ~26s and end-to-end Load Data within the 90s budget.
- Production **Load Data** defaults to an unlimited scan (`maxAccepted: null`, `scanTruncated: false`).
- The top bar exposes a **Cap players** toggle and numeric limit (default off; enabling defaults to 500). Preferences persist via `useLoadDataPreferences`.
- `load_data` returns `timings` (`scanMs`, `ingestMs`, `totalMs`); the success banner shows them for live budget evidence.
- Failed scans and failed ingests preserve the prior good snapshot. Unsupported FM builds still fail closed.

## Final architecture

```text
bridge/Scanning/PersonScanner
  → VirtualQuery regions; TryReadBlock (32 MiB DefaultScanBlockSize, 16-byte overlap)
  → local aligned-word inspection; in-buffer UID; per-scan vtable→class-offset cache
  → request-scoped maxAccepted (null = unlimited)

bridge/Extraction/
  → batched TryReadBlock for contiguous attrs, personality, positions, bounded strings
  → pointer-chain hops (name, nation, contract) stay scalar

bridge/DumpWriter
  → compact schema-v5 Utf8JsonWriter stream; per-player flush; atomic replace-on-success

Rust snapshot/ingest
  → parse_and_validate_dump (single parse) + one prepared player INSERT per transaction

IPC load_data(maxAccepted?)
  → scan without Db lock → ingest_dump_file_for_save → LoadDataResult + timings
```

- Phase diagnostics land in `diagnostics.txt` (`regionEnumerationMs`, `candidateDiscoveryMs`, `extractionMs`, `clubIndexingMs`, `dumpWritingMs`, `totalMs`, plus process-memory call and byte counts).
- `PersonScanner.DefaultMaxAccepted` (500) remains the diagnostic/test constant; production requests omit or pass `null` for unlimited.

## Important decisions

- Preserve safe `ReadProcessMemory`; optimize syscall count before narrowing region heuristics.
- Single-threaded block scanning met both capped and unlimited live budgets — parallel workers not added.
- Unlimited production default only after live full-save validation; capped loads remain for diagnostics.
- Public FMSuperScout block-scan history informed design; reimplemented independently per [ADR-0016](../../decisions/0016-csharp-bepinex-fm26-bridge.md) and `.wiki/notes/superscout-permission.md`.
- Generated 500,000-player ingest test is `#[ignore]` in the default gate — a scale check, not a claim of a comparable live save.

## Migration and operational implications

- Rebuild and install the current bridge DLL before live benchmarks; a stale bundled plugin invalidates timing evidence ([bridge/README.md](../../../bridge/README.md), [BACKLOG.md](../../BACKLOG.md) build-before-copy item).
- Bridge wait timeout stays at 120s default (comfortable vs ~26s measured unlimited bridge dump).
- `diagnostics.txt` is the authoritative scan-phase profile; `load_data` timings cover end-to-end scan + ingest for the UI.

## Validation

- **Post-remediation (2026-07-30):** `./scripts/dev format` pass; `./scripts/dev test` — 55 Vitest passed; `./scripts/dev check` pass — 79 Rust tests, 1 ignored 500k scale check, smoke via dispatcher.
- **Live (FM 26.3.2):** Pre-block-scan capped 500 `totalMs≈73171` / `candidateDiscoveryMs≈72958`; post-block-scan capped 500 `totalMs=6700` (~11× faster); unlimited ~181k `totalMs=26155`, `scanTruncated=false`.
- **Automated:** C# fake-reader tests (block reads, truncation, dedupe, batched extraction); Rust ingest rollback, `maxAccepted` plumbing, `load_data` timings; frontend tests for capped vs unlimited invoke args.

**Delivery commits:** `b2c8663`, `6f299da`, `648b81f`, `8316c43`, `a686189`, `862486a`, `d8b7b04`, `b67fc3b`; finish-feature remediation `142c7b4`.

## Follow-up

- **Next feature:** [Role scoring engine](../../TODO.md) (order 3).
- **BACKLOG:** snapshot history per save; in-app bridge DLL build-before-copy.
- **Repin:** FM patches may require layout updates; scan performance does not remove fail-closed version checks.
