# 0018 — Squad-wide action-specific player boosts

## Status

Accepted

## Context

[ADR-0017](./0017-action-specific-fm26-player-boosts.md) limits FM26 writes to Boost CA and Wonderkid Mentality for one snapshot-bound player UID. The Squad workspace needs to apply those same two actions to every eligible player in the configured club family.

A squad-wide action can change several live FM players before a later player fails. The existing bridge protocol processes one player action at a time, and each verified result is reconciled into SQLite after the FM write. A new batch bridge payload could increase throughput, but it would widen the write boundary, complicate expected-value verification and rollback, and make partial outcomes harder to attribute.

## Decision

Keep the two action-specific bridge operations and their one-player request shape. Do not add a general batch memory-write command, arbitrary UID list, address, field, target value, or custom increment.

Rust owns squad-wide orchestration. The WebView invokes one closed squad action without supplying player identities or values. Rust captures the active save, effective current snapshot and immutable context, derives the distinct player cohort from the configured club-family source names, and processes the cohort sequentially through the existing prepare, bridge request, verified readback, and SQLite reconciliation path.

Use one shared player-boost gate for profile and squad actions. Ineligible players are skipped without a bridge request. A bridge-proven player-local rejection where no write occurred is counted as failed and does not stop later players. Each verified result commits locally before the next player begins.

Rust stops before another FM write when the result cannot prove that FM and SQLite remain synchronized. Stop conditions include an unconfirmed or timed-out result, uncertain rollback, verified FM success followed by reconciliation failure, active save or snapshot replacement, and other recovery-required bridge or protocol failures. The command returns the updated, skipped, and failed counts already observed plus a terminal recovery state. This preserves best-effort progress without authorizing another write after the app requires Load Data.

Correct the CA age eligibility for both profile and squad actions: age 20 or younger receives +5, age 21 through 28 receives +10, and age 29 or older is ineligible. Age-ineligible players do not reach the bridge. Eligible targets remain capped by PA and 200. Wonderkid Mentality keeps the existing field-level rule: only known Ambition, Professionalism, and Determination values at or below 10 receive bridge-selected inclusive random targets from 11 through 20.

The batch is best-effort, not atomic. The UI confirms that scope before starting, prevents overlapping actions, reports updated, skipped, and failed counts, and directs the user to Load Data when context or reconciliation uncertainty makes the local snapshot unreliable.

## Alternatives considered

### One bridge request containing every player

Send a list of UIDs, expected values, and operations to the C# plugin. This could reduce file-protocol round trips, but it creates a new batch write surface, makes per-player rollback and readback more complex, and duplicates orchestration that Rust can perform through the verified one-player path. Rejected.

### Parallel one-player requests

Run several current operations concurrently. The plugin already serializes scans and mutations, and parallel submission would add ordering, cancellation, and stale-context complexity without parallel execution in FM. Rejected.

### Stop on the first non-successful player

Abort after any ineligible or bridge-rejected player. This is simple, but one capped player or one proven no-write live-value rejection would prevent independent eligible players from receiving the requested action. Rejected in favor of best-effort continuation only when the result proves that no write occurred. Recovery-required uncertainty still stops the batch.

### Roll back every earlier player after a later failure

Attempt an all-or-nothing squad transaction across FM memory and SQLite. There is no durable transaction spanning the live process and SQLite, FM can evolve values between requests, and compensating writes could create more corruption risk than the original failure. Rejected.

### Let React enumerate the visible table rows

Send the currently loaded or selected UIDs from the WebView. This would trust a paginated presentation cache as the authoritative squad and allow a modified WebView to choose targets. Rejected; Rust derives the cohort from the current snapshot and club family.

## Consequences

### Positive

- The write surface remains limited to the two already approved actions and their validated one-player protocol.
- Every player keeps the existing expected-value checks, verified readback, rollback reporting, and targeted SQLite reconciliation.
- One proven no-write player-local rejection does not prevent unrelated eligible squad players from changing.
- Context loss stops new writes while preserving truthful results for work already completed.
- Any recovery-required uncertainty stops new writes before another player reaches the bridge.
- The WebView cannot select targets or values.

### Negative

- Large squads take one bridge round trip and one local reconciliation per eligible player.
- Partial success is expected and cannot be rolled back as one unit.
- FM can change before a local reconciliation failure; Load Data remains the recovery path.
- CI can prove orchestration and bridge behavior only with fakes. The assembled path still needs a supported live FM session for full integration evidence.

### Follow-up

- Implement and validate this decision through the [Squad Workspace](../features/active/squad-workspace.md) ledger.
- Keep any new edit action, arbitrary value, parallel mutation, or general batch protocol behind another product and architecture decision.
- Reassess progress reporting only if measured squad runtimes make one pending state insufficient.

## Related work

- Feature plan: [Squad Workspace](../features/active/squad-workspace.md)
- Existing action boundary: [ADR-0017](./0017-action-specific-fm26-player-boosts.md)
- Commits: Pending implementation
- Supersedes: none; extends ADR-0017 for sequential squad scope and corrects its CA age eligibility
