# Task 05 — Field Parsers

## Overview

Implement individual field parsing functions: identity fields (uid, name, nationality, club, age), physical fields (height, footedness), financial fields (transfer value, wage), date/playing time fields (contract expiry, appearances, minutes), ability fields (CA, PA), and stat fields (80+ columns). Each function takes a raw string and returns the typed value, optionally producing warnings.

## Files to Create/Modify

- Modify: `src-tauri/src/parser/fields.rs`

## Steps

- [ ] **Step 1: Write field parser module with tests**

In `src-tauri/src/parser/fields.rs`:

```rust
use crate::parser::countries::lookup_country;
use crate::parser::positions::parse_positions;
use crate::parser::types::{
    Footedness, Nationality, ParseWarning, Position, Wage, TransferValue,
};
use crate::parser::headers::HeaderMap;

// ── Helper ─────────────────────────────────────────────────────────────

/// Parse a string field from the CSV record, returning None if index out of bounds
/// or the field is empty.
pub fn get_field(record: &csv::StringRecord, index: usize) -> Option<String> {
    record.get(index).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Parse an optional f64 from a CSV field. Returns None + warning on failure.
pub fn parse_f64(raw: &str, allow_negative: bool) -> Result<Option<f64>, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    match raw.parse::<f64>() {
        Ok(v) => {
            if !allow_negative && v < 0.0 {
                Err(format!("Negative value {} not allowed", v))
            } else {
                Ok(Some(v))
            }
        }
        Err(_) => Err(format!("Cannot parse '{}' as number", raw)),
    }
}

/// Parse a string as f64, stripping a known suffix (e.g., "km", " cm").
pub fn parse_f64_strip_suffix(raw: &str, suffix: &str, allow_negative: bool) -> Result<Option<f64>, String> {
    let raw = raw.trim().to_lowercase();
    let stripped = raw.strip_suffix(suffix).unwrap_or(&raw).trim();
    parse_f64(stripped, allow_negative)
}

// ── Identity fields ────────────────────────────────────────────────────

/// Parse UID. Returns Err if missing or non-integer (row must be skipped).
pub fn parse_uid(record: &csv::StringRecord, index: usize) -> Result<u32, String> {
    let raw = get_field(record, index).ok_or("Missing UID")?;
    raw.parse::<u32>().map_err(|_| format!("Invalid UID: '{}'", raw))
}

/// Parse player name. Returns Err if empty (row must be skipped).
pub fn parse_name(record: &csv::StringRecord, index: usize) -> Result<String, String> {
    get_field(record, index).ok_or("Missing player name".to_string())
}

/// Parse nationality from 3-letter code.
pub fn parse_nationality(raw: &str) -> Option<Nationality> {
    let code = raw.trim().to_uppercase();
    if code.is_empty() {
        return None;
    }
    let name = lookup_country(&code)
        .unwrap_or(&code)
        .to_string();
    Some(Nationality {
        code: Some(code),
        name,
    })
}

/// Parse second nationality (full name, not code).
pub fn parse_second_nationality(raw: &str) -> Option<Nationality> {
    let name = raw.trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(Nationality { code: None, name })
    }
}

/// Parse club name.
pub fn parse_club(raw: &str) -> Option<String> {
    let s = raw.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// Parse age as u16.
pub fn parse_age(raw: &str) -> Option<u16> {
    raw.trim().parse::<u16>().ok().filter(|&a| a > 0)
}

// ── Position ───────────────────────────────────────────────────────────

/// Parse position string. Returns Err if unparseable (row must be skipped).
pub fn parse_position_field(raw: &str) -> Result<Vec<Position>, String> {
    parse_positions(raw)
}

// ── Physical fields ────────────────────────────────────────────────────

/// Parse height, stripping " cm" suffix.
pub fn parse_height(raw: &str) -> Option<u16> {
    let cleaned = raw.to_lowercase().replace(" cm", "").trim().to_string();
    cleaned.parse::<u16>().ok()
}

/// Footedness label → score mapping.
const FEETEDNESS_MAP: &[(&str, u8)] = &[
    ("very strong", 5),
    ("strong", 4),
    ("fairly strong", 3),
    ("reasonable", 2),
    ("weak", 1),
];

/// Parse footedness label into Footedness struct.
/// Unrecognized label → score 3 + returns warning message.
pub fn parse_footedness(raw: &str) -> (Footedness, Option<String>) {
    let label = raw.trim().to_string();
    let lower = label.to_lowercase();
    for &(known, score) in FEETEDNESS_MAP {
        if lower == known {
            return (Footedness { label, score }, None);
        }
    }
    // Unknown label → default 3 with warning
    let warning = format!("Unrecognized footedness label '{}', defaulting to score 3", label);
    (Footedness { label, score: 3 }, Some(warning))
}

// ── Ability fields (optional columns) ─────────────────────────────────

/// Parse CA/PA. These columns may be entirely absent from the CSV.
pub fn parse_ability(raw: &str) -> Option<u16> {
    raw.trim().parse::<u16>().ok()
}

// ── Financial fields ───────────────────────────────────────────────────

/// Parse transfer value string like "€62M - €94M", "€57M", "€500".
/// Returns TransferValue. Unparseable → all None + warning message.
pub fn parse_transfer_value(raw: &str) -> (TransferValue, Option<String>) {
    let original = raw.trim().to_string();
    if original.is_empty() {
        return (
            TransferValue {
                currency_symbol: None,
                low: None,
                high: None,
                raw: None,
            },
            None,
        );
    }

    // Extract currency symbol (leading non-digit non-space characters)
    let (currency, numeric_part) = extract_currency_symbol(&original);

    // Handle range: split on " - "
    let parts: Vec<&str> = numeric_part.split(" - ").collect();

    match parts.len() {
        1 => {
            // Single value
            match parse_money_value(parts[0].trim()) {
                Some(val) => (
                    TransferValue {
                        currency_symbol: currency,
                        low: Some(val),
                        high: Some(val),
                        raw: Some(original),
                    },
                    None,
                ),
                None => (
                    TransferValue {
                        currency_symbol: currency,
                        low: None,
                        high: None,
                        raw: Some(original),
                    },
                    Some(format!("Cannot parse transfer value: '{}'", original)),
                ),
            }
        }
        2 => {
            let low_val = parse_money_value(parts[0].trim());
            let high_val = parse_money_value(parts[1].trim());
            match (low_val, high_val) {
                (Some(l), Some(h)) => (
                    TransferValue {
                        currency_symbol: currency,
                        low: Some(l),
                        high: Some(h),
                        raw: Some(original),
                    },
                    None,
                ),
                _ => (
                    TransferValue {
                        currency_symbol: currency,
                        low: None,
                        high: None,
                        raw: Some(original),
                    },
                    Some(format!("Cannot parse transfer value range: '{}'", original)),
                ),
            }
        }
        _ => (
            TransferValue {
                currency_symbol: currency,
                low: None,
                high: None,
                raw: Some(original),
            },
            Some(format!("Cannot parse transfer value: '{}'", original)),
        ),
    }
}

/// Extract leading currency symbol from a monetary string.
/// Returns (currency_symbol, remaining_string).
fn extract_currency_symbol(s: &str) -> (Option<String>, &str) {
    let trimmed = s.trim_start();
    let prefix: String = trimmed
        .chars()
        .take_while(|c| !c.is_ascii_digit() && *c != '.' && *c != '-')
        .collect();
    if prefix.is_empty() {
        (None, trimmed)
    } else {
        let remaining = &trimmed[prefix.len()..];
        (Some(prefix.trim().to_string()), remaining.trim())
    }
}

/// Parse a money value with optional K/M suffix.
/// "62M" → 62000000.0, "38.5K" → 38500.0, "500" → 500.0
fn parse_money_value(s: &str) -> Option<f64> {
    if s.is_empty() {
        return None;
    }
    let s = s.trim();
    let (num_str, multiplier) = if s.ends_with('M') || s.ends_with('m') {
        (&s[..s.len() - 1], 1_000_000.0)
    } else if s.ends_with('K') || s.ends_with('k') {
        (&s[..s.len() - 1], 1_000.0)
    } else {
        (s, 1.0)
    };
    num_str.trim().parse::<f64>().ok().map(|v| v * multiplier)
}

/// Parse wage string like "€74K p/w", "€1.2M p/a", "€500 p/m".
/// Returns Wage struct with normalized per-week value.
pub fn parse_wage(raw: &str) -> (Wage, Option<String>) {
    let original = raw.trim().to_string();
    if original.is_empty() {
        return (
            Wage {
                currency_symbol: None,
                raw_value: None,
                wage_per_week: None,
                denomination: None,
                raw: None,
            },
            None,
        );
    }

    let (currency, remaining) = extract_currency_symbol(&original);

    // Extract denomination: p/w, p/m, p/a
    let denomination = if remaining.contains("p/w") {
        Some("p/w".to_string())
    } else if remaining.contains("p/m") {
        Some("p/m".to_string())
    } else if remaining.contains("p/a") {
        Some("p/a".to_string())
    } else {
        None
    };

    // Strip denomination to get numeric part
    let numeric_part = remaining
        .replace("p/w", "")
        .replace("p/m", "")
        .replace("p/a", "")
        .trim()
        .to_string();

    let raw_value = parse_money_value(&numeric_part);

    let wage_per_week = match (raw_value, &denomination) {
        (Some(val), Some(d)) => {
            let per_week = match d.as_str() {
                "p/w" => val,
                "p/m" => val / 4.33,
                "p/a" => val / 52.0,
                _ => val,
            };
            Some(per_week)
        }
        (Some(val), None) => Some(val), // No denomination, assume raw value
        _ => None,
    };

    let warning = if raw_value.is_none() && !original.is_empty() {
        Some(format!("Cannot parse wage: '{}'", original))
    } else {
        None
    };

    (
        Wage {
            currency_symbol: currency,
            raw_value,
            wage_per_week,
            denomination,
            raw: Some(original),
        },
        warning,
    )
}

// ── Date and playing time fields ───────────────────────────────────────

/// Parse contract expiry date as dd/mm/yyyy with dd/mm/yy fallback.
/// Returns ISO date string (yyyy-mm-dd) or None.
pub fn parse_date(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    // Try dd/mm/yyyy first
    if let Ok(d) = chrono::NaiveDate::parse_from_str(raw, "%d/%m/%Y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }
    // Fall back to dd/mm/yy
    if let Ok(d) = chrono::NaiveDate::parse_from_str(raw, "%d/%m/%y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }
    None
}

/// Parse appearances "N (M)" → (started, sub). "N" → (N, 0).
pub fn parse_appearances(raw: &str) -> (Option<u16>, Option<u16>) {
    let raw = raw.trim();
    if raw.is_empty() {
        return (None, None);
    }
    // Check for "N (M)" pattern
    if let Some(paren_start) = raw.find('(') {
        let started_str = &raw[..paren_start].trim();
        let sub_str = raw[paren_start + 1..].trim_end_matches(')').trim();
        let started = started_str.parse::<u16>().ok();
        let sub = sub_str.parse::<u16>().ok();
        (started, sub)
    } else {
        (raw.parse::<u16>().ok(), Some(0))
    }
}

/// Parse minutes as u16.
pub fn parse_minutes(raw: &str) -> Option<u16> {
    raw.trim().parse::<u16>().ok()
}

// ── Stat field helpers ─────────────────────────────────────────────────

/// Parse a stat field as f64. Some stats allow negative values (xG-OP, xGP).
pub fn parse_stat(raw: &str, allow_negative: bool) -> Option<f64> {
    match parse_f64(raw, allow_negative) {
        Ok(Some(v)) => Some(v),
        _ => None,
    }
}

/// Parse a distance field (strip "km" suffix).
pub fn parse_distance(raw: &str) -> Option<f64> {
    parse_f64_strip_suffix(raw, "km", false).ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Identity fields ────────────────────────────────────────────────

    #[test]
    fn parse_uid_valid() {
        let rec = csv::StringRecord::from(vec!["71101334", "Trubin"]);
        assert_eq!(parse_uid(&rec, 0), Ok(71101334));
    }

    #[test]
    fn parse_uid_invalid() {
        let rec = csv::StringRecord::from(vec!["abc"]);
        assert!(parse_uid(&rec, 0).is_err());
    }

    #[test]
    fn parse_uid_missing() {
        let rec = csv::StringRecord::from(vec![""]);
        assert!(parse_uid(&rec, 0).is_err());
    }

    #[test]
    fn parse_name_valid() {
        let rec = csv::StringRecord::from(vec!["Anatolii Trubin"]);
        assert_eq!(parse_name(&rec, 0), Ok("Anatolii Trubin".to_string()));
    }

    #[test]
    fn parse_nationality_known() {
        let nat = parse_nationality("UKR").unwrap();
        assert_eq!(nat.code, Some("UKR".to_string()));
        assert_eq!(nat.name, "Ukraine");
    }

    #[test]
    fn parse_nationality_unknown() {
        let nat = parse_nationality("XXX").unwrap();
        assert_eq!(nat.code, Some("XXX".to_string()));
        assert_eq!(nat.name, "XXX"); // Fallback: code as name
    }

    #[test]
    fn parse_second_nationality() {
        let nat = parse_second_nationality("Ireland").unwrap();
        assert_eq!(nat.code, None);
        assert_eq!(nat.name, "Ireland");
    }

    #[test]
    fn parse_age_valid() {
        assert_eq!(parse_age("24"), Some(24));
    }

    #[test]
    fn parse_age_invalid() {
        assert_eq!(parse_age("abc"), None);
    }

    // ── Physical fields ────────────────────────────────────────────────

    #[test]
    fn parse_height_valid() {
        assert_eq!(parse_height("199 cm"), Some(199));
        assert_eq!(parse_height("175 cm"), Some(175));
    }

    #[test]
    fn parse_footedness_all_levels() {
        let (f, _) = parse_footedness("Very Strong");
        assert_eq!(f.score, 5);
        let (f, _) = parse_footedness("Strong");
        assert_eq!(f.score, 4);
        let (f, _) = parse_footedness("Fairly Strong");
        assert_eq!(f.score, 3);
        let (f, _) = parse_footedness("Reasonable");
        assert_eq!(f.score, 2);
        let (f, _) = parse_footedness("Weak");
        assert_eq!(f.score, 1);
    }

    #[test]
    fn parse_footedness_unknown_defaults_3() {
        let (f, warning) = parse_footedness("Extremely Good");
        assert_eq!(f.score, 3);
        assert!(warning.is_some());
    }

    // ── Financial fields ───────────────────────────────────────────────

    #[test]
    fn parse_transfer_value_range() {
        let (tv, warning) = parse_transfer_value("€62M - €94M");
        assert!(warning.is_none());
        assert_eq!(tv.currency_symbol, Some("€".to_string()));
        assert_eq!(tv.low, Some(62_000_000.0));
        assert_eq!(tv.high, Some(94_000_000.0));
    }

    #[test]
    fn parse_transfer_value_single() {
        let (tv, _) = parse_transfer_value("€57M");
        assert_eq!(tv.low, Some(57_000_000.0));
        assert_eq!(tv.high, Some(57_000_000.0));
    }

    #[test]
    fn parse_transfer_value_thousands() {
        let (tv, _) = parse_transfer_value("€38.5K");
        assert_eq!(tv.low, Some(38_500.0));
        assert_eq!(tv.high, Some(38_500.0));
    }

    #[test]
    fn parse_transfer_value_plain_number() {
        let (tv, _) = parse_transfer_value("€500");
        assert_eq!(tv.low, Some(500.0));
        assert_eq!(tv.high, Some(500.0));
    }

    #[test]
    fn parse_wage_per_week() {
        let (w, _) = parse_wage("€74K p/w");
        assert_eq!(w.raw_value, Some(74_000.0));
        assert_eq!(w.wage_per_week, Some(74_000.0));
        assert_eq!(w.denomination, Some("p/w".to_string()));
    }

    #[test]
    fn parse_wage_per_month() {
        let (w, _) = parse_wage("€100K p/m");
        let expected = 100_000.0 / 4.33;
        assert!((w.wage_per_week.unwrap() - expected).abs() < 1.0);
    }

    #[test]
    fn parse_wage_per_year() {
        let (w, _) = parse_wage("€1.2M p/a");
        let expected = 1_200_000.0 / 52.0;
        assert!((w.wage_per_week.unwrap() - expected).abs() < 1.0);
    }

    // ── Date fields ────────────────────────────────────────────────────

    #[test]
    fn parse_date_full_year() {
        assert_eq!(parse_date("30/6/2028"), Some("2028-06-30".to_string()));
    }

    #[test]
    fn parse_date_two_digit_year() {
        assert_eq!(parse_date("30/6/28"), Some("2028-06-30".to_string()));
    }

    #[test]
    fn parse_date_invalid() {
        assert_eq!(parse_date("not-a-date"), None);
    }

    // ── Appearances ────────────────────────────────────────────────────

    #[test]
    fn parse_appearances_with_sub() {
        let (started, sub) = parse_appearances("46 (9)");
        assert_eq!(started, Some(46));
        assert_eq!(sub, Some(9));
    }

    #[test]
    fn parse_appearances_no_sub() {
        let (started, sub) = parse_appearances("51");
        assert_eq!(started, Some(51));
        assert_eq!(sub, Some(0));
    }

    #[test]
    fn parse_appearances_empty() {
        let (started, sub) = parse_appearances("");
        assert_eq!(started, None);
        assert_eq!(sub, None);
    }

    // ── Stat fields ────────────────────────────────────────────────────

    #[test]
    fn parse_stat_valid() {
        assert_eq!(parse_stat("23", false), Some(23.0));
    }

    #[test]
    fn parse_stat_negative_rejected() {
        assert_eq!(parse_stat("-5", false), None);
    }

    #[test]
    fn parse_stat_negative_allowed() {
        assert_eq!(parse_stat("-2.94", true), Some(-2.94));
    }

    #[test]
    fn parse_distance() {
        assert_eq!(parse_distance("312.7km"), Some(312.7));
        assert_eq!(parse_distance("66.2km"), Some(66.2));
    }

    #[test]
    fn parse_minutes_valid() {
        assert_eq!(parse_minutes("4470"), Some(4470));
    }

    #[test]
    fn parse_minutes_zero() {
        assert_eq!(parse_minutes("0"), Some(0));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd src-tauri && cargo test --lib parser::fields`
Expected: All 30 tests PASS.

## Dependencies

- Task 01 (types: Footedness, Nationality, Position, Wage, TransferValue, ParseWarning)
- Task 02 (countries::lookup_country)
- Task 03 (positions::parse_positions)
- Task 04 (headers::HeaderMap, get_field)

## Success Criteria

- All 30 field parsing tests pass.
- Identity, physical, financial, date, appearance, and stat parsing all work.
- Transfer value ranges and single values both handled.
- Wage normalization to per-week works for all denominations.
- Unrecognized footedness defaults to 3 with warning.
- Unknown nationality codes use code as name fallback.

## Tests

### Tests cover: UID, name, nationality (known/unknown), second nationality, age, height, footedness (all levels + unknown), transfer value (range/single/K/M/plain), wage (p/w, p/m, p/a), date (full year, 2-digit), appearances (with/without sub), stat (valid/negative rejected/allowed), distance, minutes.
**Feasibility:** ✅ All can be tested
