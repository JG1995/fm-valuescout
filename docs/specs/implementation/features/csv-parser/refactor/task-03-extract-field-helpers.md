# Task 03 - Extract Field Extraction Helpers in parse_csv

## Overview

The `parse_csv` function in `src-tauri/src/parser/mod.rs` contains ~115 lines (248–362) of repetitive `if let Some(idx) = col_X { if let Some(raw) = record.get(idx) { ... } }` blocks for soft-field extraction. Extract two helper functions to collapse this boilerplate.

## Files to Create/Modify

- Modify: `src-tauri/src/parser/fields.rs` — add two new public helper functions
- Modify: `src-tauri/src/parser/mod.rs` — rewrite lines 248–362 to use the helpers

## Steps

- [ ] **Step 1: Write failing tests for the helpers**

In `src-tauri/src/parser/fields.rs`, add tests at the bottom of the existing `#[cfg(test)]` module (after the current tests):

```rust
#[test]
fn extract_field_some_returns_parsed_value() {
    let record = csv::StringRecord::from(vec!["42", "hello", ""]);
    // Index 0 has value, parser is |s| s.parse::<u32>().ok()
    let result = extract_field(&record, Some(0), |s| s.trim().parse::<u32>().ok());
    assert_eq!(result, Some(42u32));
}

#[test]
fn extract_field_none_index_returns_none() {
    let record = csv::StringRecord::from(vec!["42"]);
    let result: Option<u32> = extract_field(&record, None, |s| s.trim().parse().ok());
    assert!(result.is_none());
}

#[test]
fn extract_field_out_of_bounds_returns_none() {
    let record = csv::StringRecord::from(vec!["42"]);
    let result: Option<u32> = extract_field(&record, Some(5), |s| s.trim().parse().ok());
    assert!(result.is_none());
}

#[test]
fn extract_field_with_warning_returns_value_and_no_warning() {
    let record = csv::StringRecord::from(vec!["Very Strong"]);
    let mut warnings: Vec<crate::parser::types::ParseWarning> = vec![];
    let result = extract_field_with_warning(
        &record,
        Some(0),
        1,
        "Left Foot",
        crate::parser::fields::parse_footedness,
        &mut warnings,
    );
    assert!(result.is_some());
    assert!(warnings.is_empty());
}

#[test]
fn extract_field_with_warning_none_index_no_warning() {
    let record = csv::StringRecord::from(vec!["Very Strong"]);
    let mut warnings: Vec<crate::parser::types::ParseWarning> = vec![];
    let result = extract_field_with_warning(
        &record,
        None,
        1,
        "Left Foot",
        crate::parser::fields::parse_footedness,
        &mut warnings,
    );
    assert!(result.is_none());
    assert!(warnings.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test fields::tests::extract_field`
Expected: FAIL — `extract_field` and `extract_field_with_warning` are not defined yet.

- [ ] **Step 3: Implement the helper functions**

In `src-tauri/src/parser/fields.rs`, add these two functions after the existing helpers (after line 14, the `get_field` function):

```rust
/// Extract a field from a CSV record by optional column index, apply a parser.
/// Returns None if the column is missing (index is None), out of bounds, or the
/// field is empty.
pub fn extract_field<T>(
    record: &csv::StringRecord,
    col_index: Option<usize>,
    parse: impl Fn(&str) -> T,
) -> Option<T> {
    col_index.and_then(|i| record.get(i).map(parse))
}

/// Extract a field from a CSV record where the parser also returns an optional warning.
/// Pushes any warning into the warnings vector with the given row_number and field name.
pub fn extract_field_with_warning<T>(
    record: &csv::StringRecord,
    col_index: Option<usize>,
    row_number: usize,
    field_name: &str,
    parse: impl Fn(&str) -> (T, Option<String>),
    warnings: &mut Vec<crate::parser::types::ParseWarning>,
) -> Option<T> {
    let idx = match col_index {
        Some(i) => i,
        None => return None,
    };
    let raw = match record.get(idx) {
        Some(r) => r,
        None => return None,
    };
    let (value, warning) = parse(raw);
    if let Some(msg) = warning {
        warnings.push(crate::parser::types::ParseWarning {
            row_number,
            field: field_name.to_string(),
            message: msg,
        });
    }
    Some(value)
}
```

- [ ] **Step 4: Run the new tests to verify they pass**

Run: `cd src-tauri && cargo test fields::tests::extract_field`
Expected: All 5 new tests PASS.

- [ ] **Step 5: Rewrite soft-field extraction in parse_csv**

In `src-tauri/src/parser/mod.rs`, replace lines 248–362 (the entire soft-field block from "Soft fields: nationality" through "Date / playing time") with:

```rust
        // ── Soft fields ────────────────────────────────────────────────
        player.nationality = extract_field(&record, col_nation, parse_nationality);
        player.second_nationality = extract_field(&record, col_2nd_nat, parse_second_nationality);
        player.club = extract_field(&record, col_club, parse_club);
        player.age = extract_field(&record, col_age, parse_age);
        player.height = extract_field(&record, col_height, parse_height);

        // ── Soft fields with warnings ──────────────────────────────────
        player.left_foot = extract_field_with_warning(
            &record, col_left_foot, row_number, "Left Foot", parse_footedness, &mut warnings,
        );
        player.right_foot = extract_field_with_warning(
            &record, col_right_foot, row_number, "Right Foot", parse_footedness, &mut warnings,
        );

        // ── Optional columns ───────────────────────────────────────────
        player.ca = extract_field(&record, col_ca, parse_ability);
        player.pa = extract_field(&record, col_pa, parse_ability);

        // ── Financial fields ────────────────────────────────────────────
        player.transfer_value = extract_field_with_warning(
            &record, col_transfer_value, row_number, "Transfer Value",
            parse_transfer_value, &mut warnings,
        )
        .unwrap_or_default();
        player.wage = extract_field_with_warning(
            &record, col_wage, row_number, "Wage", parse_wage, &mut warnings,
        )
        .unwrap_or_default();

        // ── Date / playing time ─────────────────────────────────────────
        player.contract_expires = extract_field(&record, col_expires, parse_date);
        let (started, sub) = extract_field(&record, col_appearances, parse_appearances)
            .unwrap_or((None, None));
        player.appearances_started = started;
        player.appearances_sub = sub;
        player.minutes = extract_field(&record, col_minutes, parse_minutes);
```

Note: `extract_field_with_warning` returns `Option<T>`. For `transfer_value` and `wage`, we need the default (empty) struct when the column is missing. This requires `TransferValue` and `Wage` to implement `Default`. Add these derives in `types.rs`:

In `src-tauri/src/parser/types.rs`, add `Default` to the derive lists for `TransferValue` (line 200) and `Wage` (line 208):

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransferValue { ... }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Wage { ... }
```

- [ ] **Step 6: Remove the unused `get_field` import if needed**

The `use fields::*` import in `mod.rs` line 13 will pick up the new functions. No import changes needed.

- [ ] **Step 7: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All 113+ tests pass. The `parse_csv` function behavior is unchanged.

- [ ] **Step 8: Verify line count reduction**

Run: `wc -l src-tauri/src/parser/mod.rs`
Expected: ~460-480 lines (reduced from 574).

## Dependencies

- None (can run in parallel with Tasks 01 and 02 since it touches different files).

## Success Criteria

- `extract_field` and `extract_field_with_warning` are public functions in `fields.rs` with their own tests.
- The soft-field block in `parse_csv` is reduced from ~115 lines to ~25 lines.
- All existing tests pass without modification.
- `mod.rs` is under 500 lines.

## Tests

### Test 1: extract_field with valid index

**What to test:** Returns parsed value when index exists and field is present.

**Feasibility:** ✅ Can be tested — inline unit test.

### Test 2: extract_field with None index

**What to test:** Returns None without calling the parser.

**Feasibility:** ✅ Can be tested — inline unit test.

### Test 3: extract_field_with_warning propagates warning

**What to test:** Warning is pushed into the vec when parser returns one.

**Feasibility:** ✅ Can be tested — inline unit test.

### Test 4: All existing parse_csv tests still pass

**What to test:** The refactored parse_csv produces identical results to the original.

**Feasibility:** ✅ Can be tested — `cargo test` in src-tauri.
