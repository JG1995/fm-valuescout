# 0017 — Action-specific FM26 player boosts

## Status

Accepted

## Context

FM ValueScout currently reads FM26 through an in-process C# BepInEx bridge and treats the running game as its source of truth. The product now needs two player-profile actions that change four known scalar values: a fixed CA boost and a bounded mentality boost across Ambition, Professionalism, and Determination.

The offsets and encodings already exist in the reader, but adding writes changes the trust and failure boundary. A general memory-write API could corrupt unrelated game state. A successful FM write can also be followed by a failed local SQLite update, and a snapshot can refer to another app save or an earlier plugin session.

## Decision

Keep the C# BepInEx bridge as the only process-memory writer. Expose only two action-specific operations: **Boost CA** and **Wonderkid Mentality**. Do not expose addresses, field names, arbitrary values, or a generic write command to Rust or the WebView.

Rust owns snapshot-derived policy and persistence. Its Tauri commands accept only a player UID, derive the age rule and eligible snapshot values, and bind the request to the bridge request ID that produced the active snapshot. A CA boost targets `min(current CA + age-based increment, PA, 200)` and is unavailable when age or PA is unknown or CA is equal to or greater than PA. The bridge keeps the corresponding player locations in memory only, revalidates the live UID and expected values, including PA for a CA boost, writes through typed scalar operations, reads the result back, and reports the verified values.

Keep dump schema v6 and file protocol version 1. Extend request and status documents with optional action-specific fields and a boost-capability signal. Old consumers ignore the new status fields; the new app refuses boosts when the plugin does not advertise support.

After a verified FM write, Rust updates only the matching current snapshot in one transaction and recomputes affected current role scores. It does not run a complete Load Data cycle. If FM succeeds but snapshot reconciliation fails, the result states that FM changed and requires Load Data.

Only exact FM builds with a completed live write proof may advertise write support. Scans and mutations remain serialized, PSS snapshots remain read-only, and process addresses never leave the plugin.

## Alternatives considered

### General player editor

Expose the four numeric fields or a reusable address-and-value command. This would make future fields easy to add, but it creates a broader and harder-to-review mutation surface than the two requested behaviors. Rejected.

### Rust external-process writer

Open FM from Rust and write memory outside the existing plugin. This would duplicate version, address, process, and safety ownership across two languages and contradict ADR-0016. Rejected.

### React-owned targets and randomness

Let the WebView calculate the CA value and random mentality targets. This would move authoritative rules into the untrusted presentation layer and allow arbitrary payloads at IPC. Rejected.

### Full Load Data after every boost

Treat the next complete dump and ingest as the only local reconciliation. This reuses existing code but imposes a full-world scan and multi-gigabyte ingest after each small action and makes using both actions in sequence unnecessarily slow. Rejected in favor of a verified targeted transaction.

### Query-cache-only reconciliation

Patch the open profile while leaving SQLite, Search, Planner, and Academy data unchanged until the next Load Data. This would display conflicting values across the app. Rejected.

## Consequences

### Positive

- The user gets the requested actions without a general editor or arbitrary memory API.
- Existing layout and candidate discovery work supplies the field locations; no new raw-heap research path is required.
- Source-request binding, expected values, live readback, and exact-build gating reduce stale-player and patch-drift risk.
- The CA ceiling is enforced from snapshot policy and checked again against live PA before the bridge writes.
- A targeted SQLite transaction keeps profile, search, role scores, Planner, Academy, and sanity views consistent without a full refresh.
- Dump schema v6 and existing full-dump consumers remain compatible.

### Negative

- The bridge gains write and rollback paths that cannot be proved against FM in CI; one controlled Windows session remains mandatory.
- Snapshot migration and request provenance add persistence work to a four-value feature.
- FM can change successfully before local reconciliation fails. The app must expose this partial outcome and require Load Data.
- Exact-build write support requires stricter maintenance than the current major/minor read-layout lookup.

### Follow-up

- Delivered and feature-reviewed in [Player Development Boosts](../features/completed/player-development-boosts.md). Final deterministic suites passed; fresh integrated live-FM validation was explicitly skipped and accepted, and the final PR remains unpublished and unmerged.
- Update ADR-0016 and current-state documents only where final implementation changes their stated boundaries.
- Keep any additional edit action behind a new product decision and review; do not treat this ADR as authorization for a general editor.

## Related work

- Feature record: [Player Development Boosts](../features/completed/player-development-boosts.md)
- Existing bridge decision: [0016 — C# BepInEx bridge for FM26 memory read](./0016-csharp-bepinex-fm26-bridge.md)
- Prior bridge PR: [PR 37](https://github.com/JG1995/fm-valuescout/pull/37), merge ref `1f4c57754de3585fe71cfc1830963601a8da296c`
- Final PR content commits: `9f0b5983d9c2b2abc960f066f264d248148f4c96`, `bc5678c906f4f7d429ae38114911450a3cd0c40c`, `77998deeae6f44ace64c425ba6e88bf68211eaa8`, `62a23afd85a22b2592ad953ffbfb775d36ffce1c`
- Reviewed correction: `8052c90880ab4bd3354e5294d370591905e6f26d`
- Supersedes: none; narrows and extends ADR-0016 for these two actions only
