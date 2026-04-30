# Task 05 - Add Default Derives to Stat Structs

## Overview

The `ParsedPlayer::empty()` constructor in `types.rs` manually sets every `Option<f64>` field to `None` across 8 stat structs — ~140 lines of pure boilerplate. Add `Default` derives to the stat structs and use `..Default::default()` struct update syntax to eliminate it.

## Files to Create/Modify

- Modify: `src-tauri/src/parser/types.rs` — add `Default` derives, simplify `ParsedPlayer::empty()`

## Steps

- [ ] **Step 1: Add Default derives to all stat structs**

In `src-tauri/src/parser/types.rs`, add `Default` to the derive list for each stat struct. Every field in these structs is `Option<f64>`, which defaults to `None`:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttackingStats { ... }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChanceCreationStats { ... }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MovementStats { ... }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefendingStats { ... }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AerialStats { ... }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoalkeepingStats { ... }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisciplineStats { ... }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatchOutcomeStats { ... }
```

Note: `TransferValue` and `Wage` should already have `Default` derives from Task 03. Verify they are present before proceeding.

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransferValue { ... }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Wage { ... }
```

- [ ] **Step 2: Simplify ParsedPlayer::empty()**

Replace the entire `ParsedPlayer::empty()` method body (lines 270-406) with:

```rust
impl ParsedPlayer {
    /// Create a ParsedPlayer with all optional fields set to None/empty.
    /// Used as a builder base during row parsing.
    pub fn empty(uid: u32, name: String, positions: Vec<Position>) -> Self {
        Self {
            uid,
            name,
            positions,
            nationality: None,
            second_nationality: None,
            club: None,
            age: None,
            height: None,
            left_foot: None,
            right_foot: None,
            ca: None,
            pa: None,
            contract_expires: None,
            appearances_started: None,
            appearances_sub: None,
            minutes: None,
            ..Default::default()
        }
    }
}
```

For this to work, `ParsedPlayer` itself needs `Default`. Add `Default` derive to `ParsedPlayer`:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParsedPlayer { ... }
```

This works because every field is either an `Option` (defaults to `None`), a `Vec` (defaults to empty), a `TransferValue`/`Wage` (now has Default), or one of the stat structs (now has Default). The `u32` uid and `String` name will be explicitly set in the constructor.

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors.

- [ ] **Step 4: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All 113+ tests pass. `ParsedPlayer::empty()` still produces the same result — all fields `None` except uid, name, positions.

## Dependencies

- Task 04 should be completed first. Task 04 modifies `mod.rs` but doesn't change `types.rs`. This task modifies `types.rs` only. They can technically run in parallel, but completing Task 04 first means any issues with the macro are caught before this simplification.

## Success Criteria

- All 10 structs (8 stat structs + TransferValue + Wage) have `Default` derives.
- `ParsedPlayer` has a `Default` derive.
- `ParsedPlayer::empty()` is under 20 lines (down from ~140).
- `types.rs` drops below 350 lines (from 466).
- All existing tests pass.

## Tests

### Test 1: Default ParsedPlayer matches empty()

**What to test:** `ParsedPlayer::default()` produces the same field values as `ParsedPlayer::empty(0, String::new(), vec![])` for all stat fields.

**Feasibility:** ✅ Can be tested — the existing `empty_player_has_required_fields` test already validates `empty()` behavior. Running all tests confirms no regression.
