//! Per-role 0–100 score from primary/secondary attribute bands.

use std::collections::HashMap;

use super::catalog::RoleDefinition;

/// Computes a 0–100 role-fit score from visible attributes.
///
/// Within each band, attributes have equal weight. When `role.secondary` is
/// non-empty, blends `0.75 × primary_mean + 0.25 × secondary_mean`; otherwise
/// uses the primary mean alone. Scales FM 1–20 values via `/ 20 × 100` and
/// rounds to the nearest integer.
///
/// Returns `None` when any required attribute is missing or JSON-null.
pub fn score_role(attributes: &HashMap<String, Option<u8>>, role: &RoleDefinition) -> Option<u8> {
    let primary_mean = band_mean(role.primary, attributes)?;

    let blended = if role.secondary.is_empty() {
        primary_mean
    } else {
        let secondary_mean = band_mean(role.secondary, attributes)?;
        (0.75 * primary_mean) + (0.25 * secondary_mean)
    };

    Some(scale_to_hundred(blended))
}

fn band_mean(keys: &[&str], attributes: &HashMap<String, Option<u8>>) -> Option<f64> {
    if keys.is_empty() {
        return None;
    }

    let sum: u32 = keys.iter().try_fold(0u32, |acc, key| {
        let value = attributes.get(*key).copied().flatten()?;
        Some(acc + u32::from(value))
    })?;

    Some(f64::from(sum) / f64::from(keys.len() as u32))
}

fn scale_to_hundred(mean: f64) -> u8 {
    ((mean / 20.0) * 100.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::scoring::catalog::RoleDefinition;

    fn attrs(pairs: &[(&str, Option<u8>)]) -> HashMap<String, Option<u8>> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), *value))
            .collect()
    }

    const FIXTURE_ROLE: RoleDefinition = RoleDefinition {
        role_id: "fixture_striker_ip",
        display_name: "Fixture Striker",
        phase: super::super::catalog::RolePhase::InPossession,
        position_tags: &["ST"],
        primary: &["Finishing", "Composure"],
        secondary: &["OffTheBall", "Anticipation"],
    };

    const PRIMARY_ONLY_ROLE: RoleDefinition = RoleDefinition {
        role_id: "fixture_primary_only",
        display_name: "Fixture Primary Only",
        phase: super::super::catalog::RolePhase::InPossession,
        position_tags: &["ST"],
        primary: &["Finishing", "Composure"],
        secondary: &[],
    };

    #[test]
    fn equal_primary_and_secondary_means_use_seventy_five_twenty_five_blend() {
        let attributes = attrs(&[
            ("Finishing", Some(10)),
            ("Composure", Some(10)),
            ("OffTheBall", Some(20)),
            ("Anticipation", Some(20)),
        ]);

        assert_eq!(score_role(&attributes, &FIXTURE_ROLE), Some(63));
    }

    #[test]
    fn unequal_band_sizes_weight_within_band_means_only() {
        let role = RoleDefinition {
            role_id: "unequal_bands",
            display_name: "Unequal Bands",
            phase: super::super::catalog::RolePhase::InPossession,
            position_tags: &["ST"],
            primary: &["Finishing", "Composure"],
            secondary: &["OffTheBall"],
        };
        let attributes = attrs(&[
            ("Finishing", Some(20)),
            ("Composure", Some(20)),
            ("OffTheBall", Some(10)),
        ]);

        assert_eq!(score_role(&attributes, &role), Some(88));
    }

    #[test]
    fn empty_secondary_uses_primary_mean_only() {
        let attributes = attrs(&[("Finishing", Some(15)), ("Composure", Some(15))]);

        assert_eq!(score_role(&attributes, &PRIMARY_ONLY_ROLE), Some(75));
    }

    #[test]
    fn null_required_attribute_returns_none() {
        let attributes = attrs(&[
            ("Finishing", Some(15)),
            ("Composure", None),
            ("OffTheBall", Some(15)),
            ("Anticipation", Some(15)),
        ]);

        assert_eq!(score_role(&attributes, &FIXTURE_ROLE), None);
    }

    #[test]
    fn missing_required_attribute_returns_none() {
        let attributes = attrs(&[
            ("Finishing", Some(15)),
            ("OffTheBall", Some(15)),
            ("Anticipation", Some(15)),
        ]);

        assert_eq!(score_role(&attributes, &FIXTURE_ROLE), None);
    }

    #[test]
    fn perfect_attributes_score_one_hundred() {
        let attributes = attrs(&[
            ("Finishing", Some(20)),
            ("Composure", Some(20)),
            ("OffTheBall", Some(20)),
            ("Anticipation", Some(20)),
        ]);

        assert_eq!(score_role(&attributes, &FIXTURE_ROLE), Some(100));
    }

    #[test]
    fn band_mean_returns_none_for_null_or_missing_values() {
        let attributes = attrs(&[("Finishing", Some(10)), ("Composure", None)]);

        assert_eq!(band_mean(&["Finishing", "Composure"], &attributes), None);
    }
}
