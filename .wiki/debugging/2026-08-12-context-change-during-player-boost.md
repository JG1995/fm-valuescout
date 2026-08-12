# Context change during a player boost

## Status

Resolved

## Scope

- First confirmed: 2026-08-12
- Affected components: Squad player boosts and snapshot commands
- Affected environment or versions: Desktop app

## Symptom

An active-save or snapshot change could occur after a squad boost checked its context and before it sent the next bridge request.

## Confirmed root cause

The player boost gate protected profile and squad boost commands only. Snapshot commands used the database mutex directly, so they could change the active context while a boost waited for the bridge.

## Resolution

The player feature owns one shared boost gate. Profile and squad boosts, Load Data, active-save selection, and save or snapshot deletion acquire that gate before they use the database. The database mutex stays released while a bridge request waits.

## Prevention and regression coverage

- Tests: `squad_current_ability_boost_blocks_active_save_changes_during_the_bridge_request` and `squad_current_ability_boost_stops_when_snapshot_values_are_invalid`
- Validation or guard: Context-changing commands reject with `inProgress` while the shared boost gate is held.
- Commit or feature record: Squad Workspace Commit 5

## Future diagnosis

If a context-changing command is added, make it acquire `features::player::boost_gate` before it locks SQLite.

## Related records

- ADR: [0018 — Squad-wide action-specific player boosts](../decisions/0018-squad-wide-player-boosts.md)
- Feature record: [Squad Workspace](../features/completed/squad-workspace.md)
- Supersedes: none
