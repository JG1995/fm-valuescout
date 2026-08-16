# 0021 — Sequential club-family staff CA boost

## Status

Accepted

## Context

[ADR-0020](./0020-action-specific-fm26-staff-ca-boost.md) accepts one closed +10 staff CA operation and originally rejects family-wide orchestration because My Staff needed only row actions. The My Staff product requirement now replaces those row actions with one **Boost all CA** action for every staff member in the configured club family.

The action can update several live staff members before a later request fails. The bridge protocol still validates and processes one staff member at a time, so Rust must define cohort ownership, partial results, and recovery behavior without widening the process-memory write operation.

## Decision

Keep the single-staff bridge operation from ADR-0020. Do not add a bridge batch payload, arbitrary UID list, target, increment, field, or parallel mutation path.

Add one closed `boost_my_staff_current_ability` command. The WebView supplies only a progress channel. Rust captures the active save, effective current snapshot, immutable context, and distinct staff UIDs whose clubs match any configured Senior, Reserves, or Youth source. Rust then processes that frozen cohort sequentially through the existing fixed +10 prepare, bridge request, verified readback, and SQLite reconciliation path.

Staff already at PA or 200 are skipped without a bridge request. A bridge-proven staff-local live-value rejection counts as failed and does not stop unrelated staff. A global pre-write bridge failure returns an error immediately without latching recovery. An uncertain result, context change, or reconciliation failure stops the batch, returns the counts already observed with `recoveryRequired`, and latches the shared snapshot recovery state before another player or staff write can start.

The action is best-effort, not atomic. React confirms the configured-family scope, prevents overlapping activation, reports progress and aggregate updated, skipped, and failed counts, and clears a prior recovery result when Load Data establishes a different effective current snapshot. The individual Staff Profile action remains unchanged.

## Alternatives considered

### Keep per-row My Staff actions

Retain the original overview controls. This conflicts with the requested overview workflow and repeats the same action in every row. Rejected.

### Send the visible table rows from React

Build the cohort from loaded or visible rows. The table is paginated and configurable, and the WebView is not authoritative for mutation targets. Rejected.

### Add one bridge batch request

Send every UID and expected value to C#. This widens the memory-write protocol and complicates per-staff verification, rollback, and partial outcomes. Rejected.

### Run staff requests in parallel

Parallel submission conflicts with the shared mutation gate and adds ordering and stale-context risk without parallel FM execution. Rejected.

## Consequences

### Positive

- Rust owns the full configured-family cohort and the WebView cannot select staff identities or values.
- The bridge write surface remains the existing fixed one-staff operation.
- Capped staff do not block eligible staff, and staff-local no-write failures do not force recovery.
- Uncertain outcomes stop later writes and preserve one recovery rule across player and staff actions.

### Negative

- Large staff families require one bridge round trip and one local reconciliation per eligible staff member.
- Partial success is expected and cannot be rolled back as one transaction.
- CI cannot prove the assembled multi-staff FM path without a supported live Windows session.

### Follow-up

- Record deterministic orchestration, recovery, and browser evidence in the Staff Workspace feature ledger.
- Require another decision for new staff fields, custom values, a bridge batch payload, or parallel writes.

## Related work

- Feature plan: [Staff Workspace](../features/active/staff-workspace.md)
- Closed staff operation: [ADR-0020](./0020-action-specific-fm26-staff-ca-boost.md)
- Existing sequential player policy: [ADR-0018](./0018-squad-wide-player-boosts.md)
- Commits: not implemented
- Supersedes: ADR-0020's rejection and follow-up requirement for family-wide staff orchestration; preserves its single-staff bridge boundary
