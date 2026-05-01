# Task 02 - Rust Archetype Types

## Overview

Define the Rust types for archetypes: `Archetype`, `MetricWeight`, and a `ScoutingRole` enum. These types model the data stored in the `archetypes` table and flow between the storage layer and Tauri commands.

## Files to Create/Modify

- Create: `src-tauri/src/storage/archetypes.rs` — Archetype storage module (types + CRUD, types first)
- Modify: `src-tauri/src/storage/mod.rs` — Add `pub mod archetypes;` and re-exports

## Context

### Existing Type Patterns

Storage types are defined in `src-tauri/src/storage/types.rs`. They use `serde::{Serialize, Deserialize}` for Tauri IPC serialization. The pattern is:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeName {
    pub field: Type,
}
```

The `Role` enum from `src-tauri/src/parser/types.rs` represents FM positions: `GK, D, WB, DM, M, AM, ST`. We need a compatible string representation for database storage.

### Archetype Structure

An archetype contains:
- `id`: Database row ID
- `name`: Human-readable name (e.g., "Ball-Playing Goalkeeper")
- `role`: Position role this archetype applies to (stored as string in DB: "GK", "D", etc.)
- `metrics`: Array of `{ metric_key, weight, inverted }` objects
- `is_default`: Whether this is a built-in archetype
- `created_at`, `updated_at`: Timestamps

### MetricWeight Structure

Each metric in an archetype has:
- `metric_key`: String matching a field name on `ParsedPlayer` (e.g., `"goals_per_90"`, `"pass_completion_rate"`)
- `weight`: `f64` between 0.0 and 1.0 (weights across all metrics in an archetype should sum to ~1.0)
- `inverted`: Whether lower values are better (e.g., `"fouls_made_per_90"`)

## Steps

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/storage/archetypes.rs` with just the tests first:

```rust
use serde::{Deserialize, Serialize};

// ── Types (initially empty, will fill in Step 3) ─────────────────────

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_weight_serde_roundtrip() {
        let mw = MetricWeight {
            metric_key: "goals_per_90".to_string(),
            weight: 0.45,
            inverted: false,
        };
        let json = serde_json::to_string(&mw).unwrap();
        let back: MetricWeight = serde_json::from_str(&json).unwrap();
        assert_eq!(back.metric_key, "goals_per_90");
        assert!((back.weight - 0.45).abs() < f64::EPSILON);
        assert!(!back.inverted);
    }

    #[test]
    fn archetype_serde_roundtrip() {
        let arch = Archetype {
            id: 1,
            name: "Ball-Playing Goalkeeper".to_string(),
            role: "GK".to_string(),
            metrics: vec![
                MetricWeight {
                    metric_key: "progressive_passes_per_90".to_string(),
                    weight: 0.40,
                    inverted: false,
                },
                MetricWeight {
                    metric_key: "pass_completion_rate".to_string(),
                    weight: 0.30,
                    inverted: false,
                },
            ],
            is_default: true,
            created_at: "2026-05-01 12:00:00".to_string(),
            updated_at: "2026-05-01 12:00:00".to_string(),
        };
        let json = serde_json::to_string(&arch).unwrap();
        let back: Archetype = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, 1);
        assert_eq!(back.name, "Ball-Playing Goalkeeper");
        assert_eq!(back.role, "GK");
        assert_eq!(back.metrics.len(), 2);
        assert!(back.is_default);
    }

    #[test]
    fn metric_weight_inverted_roundtrip() {
        let mw = MetricWeight {
            metric_key: "fouls_made_per_90".to_string(),
            weight: 0.10,
            inverted: true,
        };
        let json = serde_json::to_string(&mw).unwrap();
        let back: MetricWeight = serde_json::from_str(&json).unwrap();
        assert!(back.inverted);
    }

    #[test]
    fn validate_metrics_empty_rejected() {
        let result = validate_metrics(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one metric"));
    }

    #[test]
    fn validate_metrics_negative_weight_rejected() {
        let metrics = vec![MetricWeight {
            metric_key: "goals_per_90".to_string(),
            weight: -0.1,
            inverted: false,
        }];
        let result = validate_metrics(&metrics);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("positive"));
    }

    #[test]
    fn validate_metrics_weight_above_one_rejected() {
        let metrics = vec![MetricWeight {
            metric_key: "goals_per_90".to_string(),
            weight: 1.5,
            inverted: false,
        }];
        let result = validate_metrics(&metrics);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1.0"));
    }

    #[test]
    fn validate_metrics_empty_key_rejected() {
        let metrics = vec![MetricWeight {
            metric_key: "".to_string(),
            weight: 1.0,
            inverted: false,
        }];
        let result = validate_metrics(&metrics);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-empty"));
    }

    #[test]
    fn validate_metrics_valid_passes() {
        let metrics = vec![
            MetricWeight {
                metric_key: "goals_per_90".to_string(),
                weight: 0.6,
                inverted: false,
            },
            MetricWeight {
                metric_key: "assists_per_90".to_string(),
                weight: 0.4,
                inverted: false,
            },
        ];
        assert!(validate_metrics(&metrics).is_ok());
    }

    #[test]
    fn normalize_weights_sums_to_one() {
        let mut metrics = vec![
            MetricWeight {
                metric_key: "goals_per_90".to_string(),
                weight: 0.3,
                inverted: false,
            },
            MetricWeight {
                metric_key: "assists_per_90".to_string(),
                weight: 0.2,
                inverted: false,
            },
        ];
        normalize_weights(&mut metrics);
        let sum: f64 = metrics.iter().map(|m| m.weight).sum();
        assert!((sum - 1.0).abs() < 1e-9);
        assert!((metrics[0].weight - 0.6).abs() < 1e-9);
        assert!((metrics[1].weight - 0.4).abs() < 1e-9);
    }

    #[test]
    fn normalize_weights_zero_sum_noop() {
        let mut metrics = vec![MetricWeight {
            metric_key: "goals_per_90".to_string(),
            weight: 0.0,
            inverted: false,
        }];
        normalize_weights(&mut metrics);
        assert!((metrics[0].weight).abs() < 1e-9);
    }

    #[test]
    fn validate_archetype_name_empty_rejected() {
        let result = validate_archetype_name("");
        assert!(result.is_err());
    }

    #[test]
    fn validate_archetype_name_whitespace_rejected() {
        let result = validate_archetype_name("   ");
        assert!(result.is_err());
    }

    #[test]
    fn validate_archetype_name_valid() {
        assert!(validate_archetype_name("Ball-Playing CB").is_ok());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test --lib storage::archetypes`
Expected: FAIL — types not defined yet.

- [ ] **Step 3: Implement the types and validation**

Add the type definitions and validation functions to `src-tauri/src/storage/archetypes.rs`, above the `#[cfg(test)]` block:

```rust
use serde::{Deserialize, Serialize};

use super::error::StorageError;

// ── Types ──────────────────────────────────────────────────────────────

/// A single metric entry within an archetype.
/// `metric_key` matches a field name on `ParsedPlayer` (e.g., "goals_per_90").
/// `weight` is a value between 0.0 and 1.0; all weights in an archetype
/// are normalized to sum to 1.0 before saving.
/// `inverted` means lower values are better (e.g., "fouls_made_per_90").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricWeight {
    pub metric_key: String,
    pub weight: f64,
    pub inverted: bool,
}

/// A scoring archetype for a position role.
/// Contains a set of weighted metrics used to score players against this profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archetype {
    pub id: i64,
    pub name: String,
    /// Position role string: "GK", "D", "WB", "DM", "M", "AM", "ST"
    pub role: String,
    pub metrics: Vec<MetricWeight>,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ── Validation ─────────────────────────────────────────────────────────

/// Validate that a set of metrics is well-formed.
/// - At least one metric
/// - Each metric has non-empty key
/// - Each weight is > 0.0 and <= 1.0
pub fn validate_metrics(metrics: &[MetricWeight]) -> Result<(), StorageError> {
    if metrics.is_empty() {
        return Err(StorageError::Validation(
            "Archetype must have at least one metric.".to_string(),
        ));
    }
    for m in metrics {
        if m.metric_key.trim().is_empty() {
            return Err(StorageError::Validation(
                "Metric key must be non-empty.".to_string(),
            ));
        }
        if m.weight <= 0.0 {
            return Err(StorageError::Validation(
                "Metric weight must be positive.".to_string(),
            ));
        }
        if m.weight > 1.0 {
            return Err(StorageError::Validation(
                "Metric weight must be 1.0 or less.".to_string(),
            ));
        }
    }
    Ok(())
}

/// Normalize weights so they sum to 1.0.
/// If total is 0, does nothing (prevents division by zero).
pub fn normalize_weights(metrics: &mut [MetricWeight]) {
    let total: f64 = metrics.iter().map(|m| m.weight).sum();
    if total <= 0.0 {
        return;
    }
    for m in metrics.iter_mut() {
        m.weight /= total;
    }
}

/// Validate archetype name: non-empty after trim, max 100 chars.
pub fn validate_archetype_name(name: &str) -> Result<String, StorageError> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(StorageError::Validation(
            "Archetype name cannot be empty.".to_string(),
        ));
    }
    if trimmed.len() > 100 {
        return Err(StorageError::Validation(
            "Archetype name must be 100 characters or fewer.".to_string(),
        ));
    }
    Ok(trimmed)
}
```

- [ ] **Step 4: Register the module in `mod.rs`**

In `src-tauri/src/storage/mod.rs`, add `pub mod archetypes;` to the module declarations (after line 7, the `mod retrieval;` line), and add re-exports:

```rust
pub mod archetypes;
```

Add to the re-exports section:

```rust
pub use archetypes::{Archetype, MetricWeight, validate_metrics, normalize_weights, validate_archetype_name};
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --lib storage::archetypes`
Expected: ALL PASS.

- [ ] **Step 6: Run full test suite**

Run: `cd src-tauri && cargo test --lib`
Expected: ALL PASS — no regressions.

## Dependencies

- Task 01 (schema migration) — `archetypes` table must exist for integration tests in later tasks.

## Success Criteria

- `Archetype` and `MetricWeight` structs serialize/deserialize correctly via serde
- `validate_metrics` rejects empty, negative, oversized weights, and empty keys
- `normalize_weights` scales weights to sum to 1.0
- `validate_archetype_name` rejects empty/whitespace names
- All existing tests still pass

## Tests

### Test 1: Serde roundtrip for MetricWeight

**What to test:** MetricWeight serializes to JSON and deserializes back correctly.
**Feasibility:** ✅ Can be tested — pure unit test.

### Test 2: Serde roundtrip for Archetype

**What to test:** Full Archetype struct with nested metrics serializes and deserializes.
**Feasibility:** ✅ Can be tested — pure unit test.

### Test 3: Metrics validation

**What to test:** Empty metrics, negative weights, weights > 1.0, empty keys are all rejected.
**Feasibility:** ✅ Can be tested — pure unit test.

### Test 4: Weight normalization

**What to test:** Weights that don't sum to 1.0 get normalized; zero-sum is a no-op.
**Feasibility:** ✅ Can be tested — pure unit test.

### Test 5: Archetype name validation

**What to test:** Empty, whitespace, and valid names are handled correctly.
**Feasibility:** ✅ Can be tested — pure unit test.
