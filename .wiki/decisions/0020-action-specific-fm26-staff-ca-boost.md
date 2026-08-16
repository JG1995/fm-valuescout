# 0020 — Action-specific FM26 staff CA boost

## Status

Accepted

## Context

The Staff workspace needs one development action: increase one staff member's current ability by 10, capped at potential ability. Before this decision, FM ValueScout read staff from FM26 but did not expose staff in the UI or mutate them.

[ADR-0017](./0017-action-specific-fm26-player-boosts.md) permits only two closed player operations and requires another product and architecture decision before adding an edit action. Staff use different live-object candidates and CA offsets from players, so accepting a staff UID through a player operation would blur the validated target boundary.

## Decision

Add one closed bridge operation, **Boost Staff CA**. Keep the C# BepInEx bridge as the only process-memory writer. Do not expose addresses, field names, arbitrary values, custom increments, or a general staff editor. [ADR-0021](./0021-sequential-club-family-staff-ca-boost.md) later permits Rust to orchestrate this unchanged one-staff operation across the configured club family without adding a bridge batch payload.

Rust accepts only a staff UID, resolves that UID against the effective current snapshot, and derives `min(current CA + 10, PA, 200)`. The action is unavailable when CA is equal to or greater than PA. Rust binds the request to the bridge request ID that produced the snapshot and supplies the expected CA and PA; it never accepts a target value or increment from React.

The bridge maintains a staff mutation index from one successful live dump, separate from its player candidates. On an exact build with completed live proof, it revalidates the staff UID and expected values, writes through the staff CA offset, reads the value back, and rolls back when verification fails. Staff and player operations share one serialized mutation gate.

After verified FM success, Rust updates only the matching staff row in the effective current snapshot. Staff role scores do not change because they use staff attributes rather than CA. Any unproven outcome or SQLite reconciliation failure marks the snapshot as recovery-required and blocks later player and staff boosts until Load Data establishes a new effective snapshot.

## Alternatives considered

### Reuse the player CA operation

Route staff UIDs through the existing player action. Players and staff have different candidate discovery and value offsets, and a person may occupy both roles. A distinct operation keeps target type validation explicit. Rejected.

### General staff editor

Expose a field and arbitrary value so later staff actions are cheaper to add. This widens the memory-write surface beyond the requested fixed action. Rejected.

### Let React supply the increment or target

Pass `10` or the computed target from the Staff table. A modified WebView could then request another value. Rejected; Rust and the bridge own the invariant.

### Boost the configured club family as a batch

Apply the action to all staff in My Staff. The original request was for a per-staff overview action, and batch writes added partial-success and orchestration complexity without a product need. Rejected at the time; ADR-0021 supersedes this rejection after the product requirement changed.

## Consequences

### Positive

- The requested staff action does not create a general memory editor.
- Staff identity, source snapshot, expected values, fixed increment, PA ceiling, readback, and recovery behavior are validated outside the WebView.
- One shared gate prevents overlapping player and staff mutations.
- Successful reconciliation changes every local staff view without a full scan.

### Negative

- The bridge gains another write path whose assembled behavior requires a controlled supported-build Windows session.
- A verified FM write can still precede a failed SQLite update; Load Data remains the recovery path.
- Every supported FM build must validate the staff candidate and offset before advertising the capability.

### Follow-up

- Preserve deterministic and live validation evidence in the [Staff Workspace feature record](../features/active/staff-workspace.md).
- Require another decision for arbitrary staff values or another staff field. ADR-0021 owns family-wide orchestration.

## Related work

- Feature plan: [Staff Workspace](../features/active/staff-workspace.md)
- Existing write boundary: [ADR-0017](./0017-action-specific-fm26-player-boosts.md)
- Existing sequential mutation policy: [ADR-0018](./0018-squad-wide-player-boosts.md)
- Commits: `1932350`, `48e5c5a`, `18c28ce`
- Supersedes: none; extends ADR-0017 with one staff-specific operation
