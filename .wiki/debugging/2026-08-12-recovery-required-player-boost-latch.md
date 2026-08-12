# Recovery-required player boost latch

## Status

Resolved

## Scope

- First confirmed: 2026-08-12
- Affected components: Profile and Squad player boosts, snapshot persistence
- Affected environment or versions: Desktop app

## Symptom

After a Squad boost returned a recovery-required result, a later boost could begin from the same snapshot and reach another player.

## Confirmed root cause

The terminal recovery state existed only in the returned DTO and React feedback. The shared gate serialized in-flight work, but it released when the command returned. The unchanged snapshot still passed the immutable-context check, so a later profile or Squad command could prepare a new bridge request.

## Resolution

Snapshots persist a `player_boost_recovery_required` flag. Any non-local terminal boost error sets that flag after the bridge/reconciliation path ends. Both profile and Squad commands reject a latched current snapshot before a bridge request. A new effective current snapshot from Load Data begins unlatched.

## Prevention and regression coverage

- Tests: `squad_recovery_requires_load_data_before_later_squad_or_profile_boosts`, `profile_recovery_requires_load_data_before_a_later_squad_boost`, and `migrates_v19_snapshots_to_unlatched_player_boost_recovery`
- Validation or guard: Rust owns the snapshot latch; the Squad action also remains disabled after it reports recovery-required.
- Commit or feature record: Squad Workspace Commit 5

## Future diagnosis

If a player boost adds a terminal outcome, verify it sets the snapshot recovery requirement unless the bridge proves that no write occurred.

## Related records

- ADR: [0018 — Squad-wide action-specific player boosts](../decisions/0018-squad-wide-player-boosts.md)
- Feature record: [Squad Workspace](../features/completed/squad-workspace.md)
- Supersedes: none
