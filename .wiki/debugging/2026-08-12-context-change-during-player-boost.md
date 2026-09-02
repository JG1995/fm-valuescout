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

The player boost gate protected profile and squad boost commands only. Snapshot commands used the database mutex directly, so they could change the active context while a boost waited for the bridge. The report previously described this as one shared mutex acquired identically by Load Data and active-save selection; the implemented lease model is more specific: squad boosts acquire the boost operation lease before capturing the cohort (`player/commands.rs` `execute_squad_player_boost_with`), and Load Data acquires `LOAD_GATE` before checking the supplied save ID/token, so a concurrent load/boost conflict rejects as `inProgress` before stale-context `saveChanged`.

## Resolution

The player feature owns operation leases (`BOOST_GATE`, `LOAD_GATE`, `CONTEXT_GATE` in `features/player/boost_gate.rs`). A boost acquires all three gates, so boosts exclude loads and context switches. A Load Data acquisition holds only the load gate and is exclusive with boosts but may coexist with an active-save switch: load and context leases can be held together, with stale publication rejected at the pre-publication `is_active` re-verification. Context switches (active-save selection) hold only the context gate and are exclusive with boosts but not loads; save and snapshot deletion remain boost-exclusive. The database mutex stays released while a bridge request waits.

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
