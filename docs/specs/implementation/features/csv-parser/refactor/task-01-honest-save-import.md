# Task 01 - Make save_import Stub Honest

## Overview

The `save_import` function in `src-tauri/src/storage/mod.rs` silently returns `Ok(())` without persisting data. The user sees a success response but their data is discarded. Make the stub return an honest error so no one is misled.

## Files to Create/Modify

- Modify: `src-tauri/src/storage/mod.rs` (entire file, 34 lines)
- Test: `src-tauri/src/storage/mod.rs` (inline tests, lines 16-34)

## Steps

- [ ] **Step 1: Update the test to expect failure**

In `src-tauri/src/storage/mod.rs`, change the test `save_import_stub_accepts_players` (line 27) to assert that `save_import` returns an **error**, not success:

```rust
#[test]
fn save_import_stub_rejects_players() {
    let players = vec![ParsedPlayer::empty(1, "Test".to_string(), vec![Position {
        role: Role::ST,
        sides: vec![Side::C],
    }])];
    let result = save_import(players, "2026-01-01");
    assert!(result.is_err(), "save_import should return Err until storage is implemented");
    assert!(result.unwrap_err().contains("not yet implemented"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test storage::tests::save_import_stub_rejects_players`
Expected: FAIL — the current function returns `Ok(())`.

- [ ] **Step 3: Update save_import to return an error**

Replace the body of `save_import` in `src-tauri/src/storage/mod.rs`:

```rust
/// Persist imported players to the database.
/// Currently a stub — will be implemented with the persistence layer.
/// Returns an error so callers know data was NOT saved.
pub fn save_import(_players: Vec<ParsedPlayer>, _in_game_date: &str) -> Result<(), String> {
    Err("Storage is not yet implemented. Your data has not been saved.".to_string())
}
```

- [ ] **Step 4: Update the existing passing test**

The test `save_import_accepts_empty` (line 22) asserts `save_import(vec![], ...)` is `Ok`. Since the stub now always returns `Err`, update it:

```rust
#[test]
fn save_import_rejects_empty() {
    let result = save_import(vec![], "2026-01-01");
    assert!(result.is_err(), "save_import stub should always return Err");
}
```

- [ ] **Step 5: Run all storage tests**

Run: `cd src-tauri && cargo test storage`
Expected: All 2 tests PASS.

## Dependencies

- None.

## Success Criteria

- `save_import` returns `Err` with a clear message for any input.
- No code path exists where `save_import` returns `Ok(())`.
- All storage tests pass.

## Tests

### Test 1: save_import returns Err for non-empty input

**What to test:** Calling `save_import` with a non-empty player vec returns an error containing "not yet implemented".

**Feasibility:** ✅ Can be tested — inline test in `storage/mod.rs`.

### Test 2: save_import returns Err for empty input

**What to test:** Even with empty input, stub returns Err (not Ok).

**Feasibility:** ✅ Can be tested — inline test in `storage/mod.rs`.
