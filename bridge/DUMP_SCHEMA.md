# Dump schema v6 (frozen)

Contract between the FM26 BepInEx bridge (`dump.json`) and Rust snapshot ingest. File protocol details: [README.md](./README.md). Architecture: [ADR-0016](../.wiki/decisions/0016-csharp-bepinex-fm26-bridge.md).

**Schema version:** `6` (`BridgeProtocol.DumpSchemaVersion` / Rust `DUMP_SCHEMA_VERSION`). Schema v5 dumps are rejected with an instruction to update the bridge plugin and rescan.

## Document shape

The bridge streams one compact, camelCase JSON object. Whitespace is not significant.

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `schemaVersion` | number | yes | Must be `6` |
| `generatedAtUtc` | string | yes | ISO-8601 UTC timestamp |
| `gameVersion`, `supportedGameVersion`, `bridgeVersion` | string | yes | Running build, layout key, and plugin version |
| `protocolVersion` | number | yes | File-protocol version `1` |
| `gameDate` | string \| null | no | In-game `yyyy-MM-dd` when known |
| `gameDateSource` | string | yes | `memory` \| `derived` \| `unknown` |
| `gameDateBasis` | string | yes | `next-fixture-consensus` \| `birth-cohort-and-system-date` \| `unknown` |
| `playerDatabaseScope` | string | yes | `men` \| `women` \| `both` |
| `scanTruncated` | boolean | yes | True only when the accepted-player cap caused an early stop |
| `maxAccepted` | number \| null | yes | Accepted-player cap; null when unlimited |
| `emptySave` | boolean | no | Explicit marker for an empty player and staff result |
| `playerCount`, `players` | number, array | yes | Count must equal array length |
| `staffCount`, `staff` | number, array | yes | Count must equal array length |
| `manager` | object \| null | yes | Human-manager metadata when resolved |

Unlimited production scans write `scanTruncated: false` and `maxAccepted: null`. A capped scan still records its positive cap. It sets `scanTruncated: true` only when the accepted-player cap stops the walk; that flag means discovery may be incomplete.

## Ingestibility rules

Rust accepts only a schema-v6, protocol-v1 object with valid count, enum, and field types. Player UIDs and staff UIDs must each be unique and cannot overlap. A non-null manager must identify one emitted staff record. An empty result requires `emptySave: true`, zero players, zero staff, and `manager: null`.

The bridge never replaces a prior good dump with an empty player result. `emptySave` supports explicit tests and future handling only.

## File-protocol player boosts

Dump schema v6 remains unchanged. The shared protocol-v1 request file also permits two optional closed operations after a successful live full dump:

| Operation | Required action fields | Bridge rule |
| --- | --- | --- |
| `boost-current-ability` | `sourceRequestId`, `playerUid`, expected CA/PA, `currentAbilityIncrement` | Increment must be `5` or `10`; bridge caps live CA at live PA and `200` |
| `wonderkid-mentality` | `sourceRequestId`, `playerUid`, expected CA/PA, nullable expected Ambition/Professionalism/Determination | At least one known field must be `1..10`; the bridge generates its `11..20` target itself |

For Wonderkid Mentality, a `null` expected field means the source snapshot did not supply a writable value: the bridge neither reads nor changes that field. A known value above `10` is revalidated and remains unchanged.

`sourceRequestId` binds the action to the plugin's in-memory live candidate index. The index is replaced only after a successful live dump and is absent after a snapshot-backed scan or plugin restart. Manual force scans receive a distinct source request ID each time. Requests never carry memory addresses, field selectors, or arbitrary target values.

`status.json` may add `playerBoostsSupported` and a `playerBoost` result object. The result reports only verified CA/PA and mentality values plus rollback state. It contains no player UID, address, raw bytes, or process path. Existing full-dump readers can ignore these optional fields.

## Player object

All v5 player fields remain unchanged. Schema v6 adds:

| Field | Type | Null when |
| --- | --- | --- |
| `nationUid` | number \| null | Nation unread |
| `gender` | string | Never; `unknown` \| `male` \| `female` |
| `clubReputation` | number \| null | Selected-team club unread |
| `teamType` | number \| null | Selected team unread |

Existing nullable fields preserve JSON `null` for unread or out-of-range values, never a sentinel zero. Attribute maps remain PascalCase keys with number or null values. `loanListed` uses SuperScout status bit1, pending live confirmation.

## Staff object

Each staff record is snapshot-owned and has this shape:

| Field | Type | Null when |
| --- | --- | --- |
| `uid` | number | never |
| `name` | string \| null | Unread |
| `birthYear`, `birthDayOfYear`, `age` | number \| null | DOB or date unread |
| `nationalities` | string[] | never; may be empty |
| `nationUid` | number \| null | Nation unread |
| `gender` | string | Never; `unknown` \| `male` \| `female` |
| `ca`, `pa` | number | never |
| `attributes` | object | never; exactly the 22 keys below, each integer 1-20 or null |
| `jobId`, `weeklyWageGbp` | number \| null | Unread or absent |
| `contractExpiryYear`, `contractExpiryDayOfYear` | number \| null | Unread or absent |
| `club`, `division` | string \| null | Unresolved or unread |

The fixed keys are `Attacking`, `Defending`, `Fitness`, `Possession`, `Technical`, `Tactical`, `SetPieces`, `Determination`, `ManManagement`, `Motivating`, `JudgingPlayerAbility`, `JudgingPlayerPotential`, `JudgingStaffAbility`, `Negotiating`, `TacticalKnowledge`, `Physiotherapy`, `SportsScience`, `DataAnalysis`, `WorkingWithYoungsters`, `GoalkeepingDistribution`, `GoalkeepingHandling`, and `GoalkeepingReflexes`.

Staff data is persisted for future features. It has no query API, UI, per-attribute SQL columns, or search indexes in this schema.

## Manager object

When present, `manager` contains `uid`, non-empty `name`, nullable `club`, and nullable `clubReputation`. Its UID must match an emitted staff record. The object contains no raw address or process detail.

## Related files

| File | Writer | Purpose |
| --- | --- | --- |
| `request.json` | Tauri | Full-dump request or one closed player boost |
| `status.json` | Bridge | Idle, scanning, ready, or failed; optional cap and boost result signals |
| `dump.json` | Bridge | This schema |
| `diagnostics.txt` | Bridge | Scan diagnostics, never ingested |

Golden v6 fixture: `src-tauri/src/features/memory_read/fixtures/golden_dump_v6.json`. The v5 fixture remains only to prove stale-dump rejection.
