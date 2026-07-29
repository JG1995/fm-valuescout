# Dump schema v5 (frozen)

Contract between the FM26 BepInEx bridge (`dump.json`) and snapshot ingest (feature 2). File protocol details: [README.md](./README.md). Architecture: [ADR-0016](../.wiki/decisions/0016-csharp-bepinex-fm26-bridge.md).

**Schema version:** `5` (constant `BridgeProtocol.DumpSchemaVersion` / Rust `DUMP_SCHEMA_VERSION`).

## Document shape

Top-level JSON object, camelCase keys, pretty-printed by the bridge.

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `schemaVersion` | number | yes | Must be `5` for this contract |
| `generatedAtUtc` | string | yes | ISO-8601 UTC timestamp |
| `gameVersion` | string | yes | FM build string from the running process |
| `supportedGameVersion` | string | yes | Layout key, e.g. `26.3` |
| `bridgeVersion` | string | yes | Plugin assembly version |
| `protocolVersion` | number | yes | File-protocol version (`1`) |
| `gameDate` | string \| null | no | In-game date `yyyy-MM-dd` when known |
| `gameDateSource` | string | yes | `memory` \| `derived` \| `unknown` |
| `emptySave` | boolean | no | Explicit marker when `playerCount` is `0` (bridge normally omits empty dumps) |
| `playerCount` | number | yes | Must equal `players.length` |
| `players` | array | yes | Player objects (see below) |

### Ingestibility rules (Rust `validate_dump_json`)

1. Valid JSON object with all required top-level keys.
2. `schemaVersion == 5` and `protocolVersion == 1`.
3. `playerCount` equals `players` array length.
4. Either `playerCount > 0` with at least one player object, **or** `emptySave: true` with `playerCount == 0` and an empty `players` array.

The bridge **never** replaces a prior good dump with zero players (`DumpWriter.TryWriteReplaceOnSuccess`). `emptySave` exists for tests and future explicit empty-save handling.

## Player object

| Field | Type | Null when |
| --- | --- | --- |
| `uid` | number | never (required) |
| `ca`, `pa` | number | never (required) |
| `name` | string | never (empty names are skipped at scan time) |
| `birthYear`, `birthDayOfYear` | number | never |
| `age` | number \| null | DOB or game date missing |
| `nationalities` | string[] | never (may be empty) |
| `heightCm` | number \| null | unread |
| `preferredFoot` | string | never (`left` / `right` / `either` / `""`) |
| `positions` | object | never (map of position → suitability 0–20) |
| `attributes` | object | never (PascalCase keys → `number` 1–20 or `null`) |
| `hiddenAttributes` | object | never (same encoding as visible) |
| `personality` | object | never (PascalCase keys → raw 1–20 or `null`) |
| `weeklyWageGbp` | number \| null | free agent, unread, or `0xFFFFFFFF` sentinel |
| `contractExpiryYear` | number \| null | free agent or unread |
| `contractExpiryDayOfYear` | number \| null | with `contractExpiryYear` |
| `transferListed`, `loanListed`, `notForSale`, `setForRelease` | boolean \| null | free agent or unread flags |
| `marketValueGbp` | number \| null | unread, unset, or FM unfixed `300000000` |
| `reputation` | object | `{ current, world }` each `number` \| `null` |
| `currentClub`, `parentClub` | string \| null | unresolved / free agent |
| `onLoan` | boolean \| null | when either club unresolved |
| `division` | string \| null | competition unread |
| `teamLevel` | string \| null | `senior` \| `reserve` \| `youth` or null |

Attribute keys are stable English PascalCase (e.g. `Acceleration`, `Consistency`, `Ambition`). JSON `null` means unread or out of range — **not** a real score of zero.

## Intentional gaps (not in v5)

Deferred to later features or derivable at ingest:

- Manager name, managed club, currency metadata
- Asking price / transfer estimate heuristics
- Staff or non-player records
- Explicit loaned-in vs loaned-out relative to the human manager (`onLoan` is parent ≠ current only)
- Clubs with no contracted players in the accepted scan set (squad walk is contract-seeded only)

## Related files

| File | Writer | Purpose |
| --- | --- | --- |
| `request.json` | Tauri | Scan request (`operation: "full-dump"`, 30s TTL) |
| `status.json` | Bridge | Idle / scanning / ready / failed |
| `dump.json` | Bridge | This schema |
| `diagnostics.txt` | Bridge | Scan diagnostics (not validated for ingest) |

Golden fixture for Rust tests: `src-tauri/src/features/memory_read/fixtures/golden_dump_v5.json`.
