//! Position-sensitive visible-attribute projection for potential role scores.

use std::collections::HashMap;

use super::projection_profiles::{AttributeProfile, POSITION_PROFILES};

/// Projects visible attributes from current ability to potential ability.
///
/// The empirical profile values come from FMSuperScout commit `0f270d39`.
/// Physical and mental deltas use this application's documented age factors.
pub fn project_attributes<'a>(
    attributes: &HashMap<String, Option<u8>>,
    ca: i64,
    pa: i64,
    age: Option<i64>,
    natural_positions: impl IntoIterator<Item = &'a str>,
) -> HashMap<String, Option<u8>> {
    if pa <= ca {
        return attributes.clone();
    }

    let groups = position_groups(natural_positions);
    attributes
        .iter()
        .map(|(key, value)| {
            let projected = value.map(|current| {
                let delta = mean_positive_delta(key, ca, pa, &groups);
                let growth = if is_physical_attribute(key) {
                    physical_growth_factor(age)
                } else if is_mental_attribute(key) {
                    mental_growth_factor(age)
                } else {
                    1.0
                };
                let rounded = (f64::from(current) + (delta * growth)).round() as u8;

                rounded.min(20).max(current)
            });

            (key.clone(), projected)
        })
        .collect()
}

fn position_groups<'a>(natural_positions: impl IntoIterator<Item = &'a str>) -> Vec<&'static str> {
    let mut groups = Vec::new();
    for position in natural_positions {
        let Some(group) = position_group(position) else {
            continue;
        };
        if !groups.contains(&group) {
            groups.push(group);
        }
    }

    if groups.is_empty() {
        groups.push("ALL");
    }

    groups
}

fn position_group(position: &str) -> Option<&'static str> {
    match position {
        "GK" => Some("GK"),
        "SW" | "DC" => Some("DC"),
        "DL" | "DR" | "WBL" | "WBR" => Some("FB"),
        "DM" => Some("DM"),
        "MC" => Some("MC"),
        "ML" | "MR" | "AML" | "AMR" => Some("W"),
        "AMC" => Some("AMC"),
        "ST" => Some("ST"),
        _ => None,
    }
}

fn mean_positive_delta(key: &str, ca: i64, pa: i64, groups: &[&str]) -> f64 {
    let mut total = 0.0;
    let mut count = 0;

    for group in groups {
        let Some(profile) = profile_attribute(group, key) else {
            continue;
        };
        total += (profile_value(profile, pa) - profile_value(profile, ca)).max(0.0);
        count += 1;
    }

    if count == 0 {
        0.0
    } else {
        total / f64::from(count)
    }
}

fn profile_attribute(group: &str, key: &str) -> Option<&'static AttributeProfile> {
    POSITION_PROFILES
        .iter()
        .find(|profile| profile.group == group)?
        .attributes
        .iter()
        .find(|attribute| attribute.key == key)
}

fn profile_value(profile: &AttributeProfile, ca: i64) -> f64 {
    let anchors = profile.anchors;
    if ca <= 80 {
        return anchors[0] - ((80 - ca) as f64 * (anchors[1] - anchors[0]) / 30.0);
    }
    if ca >= 170 {
        return anchors[3] + ((ca - 170) as f64 * (anchors[3] - anchors[2]) / 30.0);
    }

    for index in 1..anchors.len() {
        let upper = 80 + (30 * index as i64);
        if ca <= upper {
            let lower = upper - 30;
            return anchors[index - 1]
                + ((anchors[index] - anchors[index - 1]) * (ca - lower) as f64 / 30.0);
        }
    }

    unreachable!("CA below the final anchor returns inside the interpolation loop")
}

fn physical_growth_factor(age: Option<i64>) -> f64 {
    match age {
        None | Some(..=23) => 1.0,
        Some(24..=26) => 0.55,
        Some(27..=29) => 0.30,
        Some(30..=32) => 0.12,
        Some(_) => 0.05,
    }
}

fn mental_growth_factor(age: Option<i64>) -> f64 {
    match age {
        Some(32..) => 1.25,
        Some(28..) => 1.15,
        _ => 1.0,
    }
}

fn is_physical_attribute(key: &str) -> bool {
    matches!(
        key,
        "Acceleration"
            | "Agility"
            | "Balance"
            | "JumpingReach"
            | "NaturalFitness"
            | "Pace"
            | "Stamina"
            | "Strength"
    )
}

fn is_mental_attribute(key: &str) -> bool {
    matches!(
        key,
        "Aggression"
            | "Anticipation"
            | "Bravery"
            | "Composure"
            | "Concentration"
            | "Decisions"
            | "Determination"
            | "Flair"
            | "Leadership"
            | "OffTheBall"
            | "Positioning"
            | "Teamwork"
            | "Vision"
            | "WorkRate"
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::projection_profiles::POSITION_PROFILES;
    use super::{
        mental_growth_factor, physical_growth_factor, profile_attribute, profile_value,
        project_attributes,
    };

    fn attributes(pairs: &[(&str, Option<u8>)]) -> HashMap<String, Option<u8>> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), *value))
            .collect()
    }

    #[test]
    fn young_striker_matches_the_pinned_upstream_profile() {
        let projected = project_attributes(
            &attributes(&[
                ("Finishing", Some(10)),
                ("OffTheBall", Some(10)),
                ("Pace", Some(10)),
                ("Heading", Some(10)),
            ]),
            110,
            140,
            Some(20),
            ["ST"],
        );

        assert_eq!(projected["Finishing"], Some(12));
        assert_eq!(projected["OffTheBall"], Some(11));
        assert_eq!(projected["Pace"], Some(11));
        assert_eq!(projected["Heading"], Some(11));
    }

    #[test]
    fn physical_growth_drops_at_age_twenty_four() {
        let attributes = attributes(&[("Pace", Some(10))]);

        let at_twenty_three = project_attributes(&attributes, 80, 170, Some(23), ["ST"]);
        let at_twenty_four = project_attributes(&attributes, 80, 170, Some(24), ["ST"]);

        assert_eq!(at_twenty_three["Pace"], Some(14));
        assert_eq!(at_twenty_four["Pace"], Some(12));
    }

    #[test]
    fn mental_growth_factor_applies_at_age_twenty_eight() {
        let projected = project_attributes(
            &attributes(&[("OffTheBall", Some(10))]),
            110,
            140,
            Some(28),
            ["ST"],
        );

        assert_eq!(projected["OffTheBall"], Some(12));
    }

    #[test]
    fn mental_growth_factor_increases_again_at_age_thirty_two() {
        let attributes = attributes(&[("OffTheBall", Some(1))]);
        let at_thirty_one = project_attributes(&attributes, 110, 140, Some(31), ["UNKNOWN"]);
        let at_thirty_two = project_attributes(&attributes, 110, 140, Some(32), ["UNKNOWN"]);

        assert_eq!(at_thirty_one["OffTheBall"], Some(2));
        assert_eq!(at_thirty_two["OffTheBall"], Some(3));
    }

    #[test]
    fn recognized_natural_positions_contribute_the_mean_delta() {
        let projected = project_attributes(
            &attributes(&[("OffTheBall", Some(10))]),
            110,
            140,
            Some(20),
            ["DM", "ST"],
        );

        assert_eq!(projected["OffTheBall"], Some(12));
    }

    #[test]
    fn unrecognized_positions_use_the_all_profile() {
        let projected = project_attributes(
            &attributes(&[("Passing", Some(10))]),
            110,
            140,
            Some(20),
            ["UNKNOWN"],
        );

        assert_eq!(projected["Passing"], Some(12));
    }

    #[test]
    fn empty_positions_use_the_all_profile() {
        let projected = project_attributes(
            &attributes(&[("Passing", Some(10))]),
            110,
            140,
            Some(20),
            std::iter::empty(),
        );

        assert_eq!(projected["Passing"], Some(12));
    }

    #[test]
    fn unknown_age_keeps_physical_and_mental_growth_neutral() {
        let projected = project_attributes(
            &attributes(&[("Pace", Some(10)), ("OffTheBall", Some(10))]),
            80,
            170,
            None,
            ["ST"],
        );

        assert_eq!(projected["Pace"], Some(14));
        assert_eq!(projected["OffTheBall"], Some(14));
    }

    #[test]
    fn age_growth_factors_change_at_each_documented_boundary() {
        assert_eq!(physical_growth_factor(None), 1.0);
        assert_eq!(physical_growth_factor(Some(23)), 1.0);
        assert_eq!(physical_growth_factor(Some(24)), 0.55);
        assert_eq!(physical_growth_factor(Some(26)), 0.55);
        assert_eq!(physical_growth_factor(Some(27)), 0.30);
        assert_eq!(physical_growth_factor(Some(29)), 0.30);
        assert_eq!(physical_growth_factor(Some(30)), 0.12);
        assert_eq!(physical_growth_factor(Some(32)), 0.12);
        assert_eq!(physical_growth_factor(Some(33)), 0.05);
        assert_eq!(mental_growth_factor(None), 1.0);
        assert_eq!(mental_growth_factor(Some(27)), 1.0);
        assert_eq!(mental_growth_factor(Some(28)), 1.15);
        assert_eq!(mental_growth_factor(Some(31)), 1.15);
        assert_eq!(mental_growth_factor(Some(32)), 1.25);
    }

    #[test]
    fn non_increasing_potential_keeps_every_attribute_unchanged() {
        let attributes = attributes(&[("Finishing", Some(14)), ("Pace", None)]);

        assert_eq!(
            project_attributes(&attributes, 130, 130, Some(19), ["ST"]),
            attributes
        );
    }

    #[test]
    fn null_attributes_remain_null() {
        let projected = project_attributes(
            &attributes(&[("Finishing", None), ("Pace", Some(10))]),
            110,
            140,
            None,
            ["ST"],
        );

        assert_eq!(projected["Finishing"], None);
        assert_eq!(projected["Pace"], Some(11));
    }

    #[test]
    fn growth_rounds_to_whole_attributes_and_caps_at_twenty() {
        let projected = project_attributes(
            &attributes(&[("Pace", Some(19)), ("Finishing", Some(20))]),
            80,
            170,
            Some(20),
            ["ST"],
        );

        assert_eq!(projected["Pace"], Some(20));
        assert_eq!(projected["Finishing"], Some(20));
    }

    #[test]
    fn profile_values_interpolate_and_extrapolate_at_each_anchor() {
        let pace = profile_attribute("ST", "Pace").expect("striker pace profile");
        const TOLERANCE: f64 = 0.000_001;

        assert!((profile_value(pace, 80) - 11.8).abs() < TOLERANCE);
        assert!((profile_value(pace, 110) - 12.7).abs() < TOLERANCE);
        assert!((profile_value(pace, 140) - 13.8).abs() < TOLERANCE);
        assert!((profile_value(pace, 170) - 15.6).abs() < TOLERANCE);
        assert!((profile_value(pace, 50) - 10.9).abs() < TOLERANCE);
        assert!((profile_value(pace, 95) - 12.25).abs() < TOLERANCE);
        assert!((profile_value(pace, 200) - 17.4).abs() < TOLERANCE);
    }

    #[test]
    fn a_declining_profile_delta_never_reduces_an_attribute() {
        let projected = project_attributes(
            &attributes(&[("Heading", Some(10))]),
            140,
            170,
            Some(20),
            ["ST"],
        );

        assert_eq!(projected["Heading"], Some(10));
    }

    #[test]
    fn profiles_cover_each_visible_attribute_for_every_position_group() {
        for profile in POSITION_PROFILES {
            assert_eq!(
                profile.attributes.len(),
                crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS.len()
            );
            for key in crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS {
                assert!(
                    profile_attribute(profile.group, key).is_some(),
                    "{} must define {key}",
                    profile.group
                );
            }
        }
    }

    #[test]
    fn profile_table_matches_the_pinned_source_fingerprint() {
        assert_eq!(profile_fingerprint(), 5_253_764_438_882_972_555);
    }

    fn profile_fingerprint() -> u64 {
        let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
        for profile in POSITION_PROFILES {
            update_fingerprint(&mut fingerprint, profile.group.as_bytes());
            update_fingerprint(&mut fingerprint, &[0xff]);
            for attribute in profile.attributes {
                update_fingerprint(&mut fingerprint, attribute.key.as_bytes());
                update_fingerprint(&mut fingerprint, &[0xff]);
                for anchor in attribute.anchors {
                    update_fingerprint(&mut fingerprint, &anchor.to_bits().to_le_bytes());
                }
            }
        }

        fingerprint
    }

    fn update_fingerprint(fingerprint: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *fingerprint ^= u64::from(*byte);
            *fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}
