use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::Deserialize;

use super::catalog::{is_lower_is_better_statistic_key, is_moneyball_statistic_key};
use crate::features::scoring::catalog::all_roles;

const BUILTIN_ROLE_DEFINITIONS_V1: &str = include_str!("builtin_role_definitions_v1.json");

pub(crate) const BUILTIN_ROLE_CATALOG_VERSION: u32 = 1;
const BUILTIN_ROLE_DEFINITION_COUNT: usize = 88;

const EXPECTED_FAMILY_COUNTS: [(&str, usize); 10] = [
    ("attacking_midfielder", 9),
    ("central_defender", 12),
    ("central_midfielder", 10),
    ("defensive_midfielder", 10),
    ("full_back", 8),
    ("goalkeeper", 6),
    ("striker", 10),
    ("wide_midfielder", 7),
    ("wing_back", 7),
    ("winger", 9),
];

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RolePhase {
    InPossession,
    OutOfPossession,
}

impl RolePhase {
    pub(crate) fn id_suffix(&self) -> &'static str {
        match self {
            Self::InPossession => "ip",
            Self::OutOfPossession => "oop",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct RoleDefinition {
    pub(crate) id: String,
    pub(crate) display_name: String,
    pub(crate) phase: RolePhase,
    pub(crate) position_family: String,
    pub(crate) position_tags: Vec<String>,
    pub(crate) attribute_role_id: Option<String>,
    pub(crate) metrics: Vec<RoleMetric>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct RoleMetric {
    #[serde(default)]
    pub(crate) source_label: Option<String>,
    pub(crate) key: String,
    pub(crate) weight: f64,
    pub(crate) inverted: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct BuiltinRoleCatalog {
    pub(crate) version: u32,
    pub(crate) definitions: Vec<RoleDefinition>,
}

static BUILTIN_CATALOG: OnceLock<Result<BuiltinRoleCatalog, String>> = OnceLock::new();

pub(crate) fn builtin_catalog() -> Result<&'static BuiltinRoleCatalog, String> {
    BUILTIN_CATALOG
        .get_or_init(|| {
            let catalog =
                serde_json::from_str::<BuiltinRoleCatalog>(BUILTIN_ROLE_DEFINITIONS_V1)
                    .map_err(|error| format!("invalid built-in role catalog JSON: {error}"))?;
            validate_builtin_catalog(&catalog)?;
            Ok(catalog)
        })
        .as_ref()
        .map_err(Clone::clone)
}

pub(crate) fn validate_role_definition(definition: &RoleDefinition) -> Result<f64, String> {
    if definition.id.trim().is_empty() {
        return Err("role definition id must not be empty".to_owned());
    }

    if definition.display_name.trim().is_empty() {
        return Err(format!(
            "role definition {} must have a display name",
            definition.id
        ));
    }

    let mut metric_keys = HashSet::new();
    let mut total_weight = 0.0;

    for metric in &definition.metrics {
        if !is_moneyball_statistic_key(&metric.key) {
            return Err(format!(
                "role definition {} uses unknown metric {}",
                definition.id, metric.key
            ));
        }
        if !metric_keys.insert(metric.key.as_str()) {
            return Err(format!(
                "role definition {} repeats metric {}",
                definition.id, metric.key
            ));
        }
        if !metric.weight.is_finite() || metric.weight < 0.0 {
            return Err(format!(
                "role definition {} has invalid weight for {}",
                definition.id, metric.key
            ));
        }
        if metric.inverted != is_lower_is_better_statistic_key(&metric.key) {
            return Err(format!(
                "role definition {} has the wrong direction for {}",
                definition.id, metric.key
            ));
        }
        total_weight += metric.weight;
    }

    if !total_weight.is_finite() || total_weight <= 0.0 {
        return Err(format!(
            "role definition {} must have a positive finite weight total",
            definition.id
        ));
    }

    Ok(total_weight)
}

fn validate_builtin_catalog(catalog: &BuiltinRoleCatalog) -> Result<(), String> {
    if catalog.version != BUILTIN_ROLE_CATALOG_VERSION {
        return Err(format!(
            "unsupported built-in role catalog version {}",
            catalog.version
        ));
    }
    if catalog.definitions.len() != BUILTIN_ROLE_DEFINITION_COUNT {
        return Err(format!(
            "built-in role catalog has {} definitions, expected {}",
            catalog.definitions.len(),
            BUILTIN_ROLE_DEFINITION_COUNT
        ));
    }

    let expected_counts: HashMap<_, _> = EXPECTED_FAMILY_COUNTS.into_iter().collect();
    let mut actual_counts = HashMap::new();
    let mut role_ids = HashSet::new();
    let known_attribute_roles: HashSet<_> = all_roles().iter().map(|role| role.role_id).collect();

    for definition in &catalog.definitions {
        validate_role_definition(definition)?;
        if definition.metrics.len() != 5 {
            return Err(format!(
                "built-in role {} must have exactly five metrics",
                definition.id
            ));
        }
        if definition
            .metrics
            .iter()
            .any(|metric| metric.source_label.as_deref().map_or(true, str::is_empty))
        {
            return Err(format!(
                "built-in role {} must preserve source metric labels",
                definition.id
            ));
        }
        if !role_ids.insert(definition.id.as_str()) {
            return Err(format!("duplicate built-in role id {}", definition.id));
        }

        let expected_tags =
            expected_position_tags(&definition.position_family).ok_or_else(|| {
                format!(
                    "built-in role {} uses unknown position family {}",
                    definition.id, definition.position_family
                )
            })?;
        let actual_tags = definition
            .position_tags
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        if actual_tags != expected_tags {
            return Err(format!(
                "built-in role {} has unexpected position tags",
                definition.id
            ));
        }

        let expected_prefix = expected_position_prefix(&definition.position_family)
            .expect("position family tags and prefix must stay in sync");
        if !definition.id.starts_with(&format!("{expected_prefix}_"))
            || !definition.id.ends_with(definition.phase.id_suffix())
        {
            return Err(format!(
                "built-in role {} does not encode its family and phase",
                definition.id
            ));
        }

        if definition
            .attribute_role_id
            .as_deref()
            .is_some_and(|role_id| !known_attribute_roles.contains(role_id))
        {
            return Err(format!(
                "built-in role {} maps to unknown attribute role",
                definition.id
            ));
        }

        *actual_counts
            .entry(definition.position_family.as_str())
            .or_insert(0usize) += 1;
    }

    if actual_counts != expected_counts {
        return Err(format!(
            "unexpected built-in role family counts: {actual_counts:?}"
        ));
    }

    Ok(())
}

fn expected_position_prefix(family: &str) -> Option<&'static str> {
    Some(match family {
        "attacking_midfielder" => "amc",
        "central_defender" => "dc",
        "central_midfielder" => "mc",
        "defensive_midfielder" => "dm",
        "full_back" => "dl_dr",
        "goalkeeper" => "gk",
        "striker" => "st",
        "wide_midfielder" => "ml_mr",
        "wing_back" => "wbl_wbr",
        "winger" => "aml_amr",
        _ => return None,
    })
}

fn expected_position_tags(family: &str) -> Option<&'static [&'static str]> {
    Some(match family {
        "attacking_midfielder" => &["AMC"],
        "central_defender" => &["DC"],
        "central_midfielder" => &["MC"],
        "defensive_midfielder" => &["DM"],
        "full_back" => &["DL", "DR"],
        "goalkeeper" => &["GK"],
        "striker" => &["ST"],
        "wide_midfielder" => &["ML", "MR"],
        "wing_back" => &["WBL", "WBR"],
        "winger" => &["AML", "AMR"],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::builtin_catalog;
    use crate::features::scoring::catalog::all_roles;

    #[test]
    fn built_in_catalog_is_versioned_and_complete() {
        let catalog = builtin_catalog().expect("built-in catalog should load");

        assert_eq!(catalog.version, 1);
        assert_eq!(catalog.definitions.len(), 88);

        let expected_family_counts = [
            ("attacking_midfielder", 9),
            ("central_defender", 12),
            ("central_midfielder", 10),
            ("defensive_midfielder", 10),
            ("full_back", 8),
            ("goalkeeper", 6),
            ("striker", 10),
            ("wide_midfielder", 7),
            ("wing_back", 7),
            ("winger", 9),
        ];

        for (family, expected_count) in expected_family_counts {
            assert_eq!(
                catalog
                    .definitions
                    .iter()
                    .filter(|definition| definition.position_family == family)
                    .count(),
                expected_count,
                "unexpected count for {family}"
            );
        }
    }

    #[test]
    fn keeps_position_family_variants_distinct() {
        let catalog = builtin_catalog().expect("built-in catalog should load");

        let full_back_ids: Vec<_> = catalog
            .definitions
            .iter()
            .filter(|definition| definition.position_family == "full_back")
            .map(|definition| &definition.id)
            .collect();
        let wing_back_ids: Vec<_> = catalog
            .definitions
            .iter()
            .filter(|definition| definition.position_family == "wing_back")
            .map(|definition| &definition.id)
            .collect();
        let wide_midfielder_ids: Vec<_> = catalog
            .definitions
            .iter()
            .filter(|definition| definition.position_family == "wide_midfielder")
            .map(|definition| &definition.id)
            .collect();
        let winger_ids: Vec<_> = catalog
            .definitions
            .iter()
            .filter(|definition| definition.position_family == "winger")
            .map(|definition| &definition.id)
            .collect();

        assert!(full_back_ids.iter().all(|id| !wing_back_ids.contains(id)));
        assert!(wide_midfielder_ids
            .iter()
            .all(|id| !winger_ids.contains(id)));
    }

    #[test]
    fn maps_only_known_attribute_roles_and_preserves_unmapped_generic_roles() {
        let catalog = builtin_catalog().expect("built-in catalog should load");
        let known_attribute_ids: std::collections::HashSet<_> =
            all_roles().iter().map(|role| role.role_id).collect();
        let expected_unmapped = [
            "amc_attacking_midfielder_oop",
            "dc_centre_back_oop",
            "dc_wide_centre_back_oop",
            "mc_central_midfielder_oop",
            "dm_defensive_midfielder_oop",
            "dl_dr_full_back_oop",
            "gk_traditional_goalkeeper_oop",
            "st_centre_forward_oop",
            "ml_mr_wide_midfielder_oop",
            "wbl_wbr_wing_back_oop",
            "aml_amr_winger_oop",
        ]
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
        let actual_unmapped = catalog
            .definitions
            .iter()
            .filter(|definition| definition.attribute_role_id.is_none())
            .map(|definition| definition.id.as_str())
            .collect::<std::collections::HashSet<_>>();

        assert_eq!(
            catalog
                .definitions
                .iter()
                .filter(|definition| definition.attribute_role_id.is_some())
                .count(),
            77
        );
        assert_eq!(
            catalog
                .definitions
                .iter()
                .filter(|definition| definition.attribute_role_id.is_none())
                .count(),
            11
        );
        assert_eq!(actual_unmapped, expected_unmapped);
        assert!(catalog.definitions.iter().all(|definition| {
            definition
                .attribute_role_id
                .as_deref()
                .map_or(true, |role_id| known_attribute_ids.contains(role_id))
        }));
    }

    #[test]
    fn preserves_the_pinned_source_label_for_the_noncanonical_shots_alias() {
        let catalog = builtin_catalog().expect("built-in catalog should load");
        let poacher = catalog
            .definitions
            .iter()
            .find(|definition| definition.id == "st_poacher_ip")
            .expect("Poacher should be in the built-in catalog");
        let shots_on_target = poacher
            .metrics
            .iter()
            .find(|metric| metric.source_label.as_deref() == Some("Shots on Target Ratio"))
            .expect("Poacher should preserve the pinned source label");

        assert_eq!(shots_on_target.key, "shots_on_target_per_90");
    }

    #[test]
    fn tactic_compound_key_is_unique_and_covers_104_of_129_with_25_uncovered() {
        let catalog = builtin_catalog().expect("built-in catalog should load");
        // Unique (attribute_role_id, position_tag) among mapped definitions
        let mut seen = std::collections::HashSet::new();
        for def in &catalog.definitions {
            if let Some(attr) = def.attribute_role_id.as_deref() {
                for tag in &def.position_tags {
                    let key = (attr, tag.as_str());
                    assert!(seen.insert(key), "duplicate compound key {attr} + {tag}");
                }
            }
        }
        // General combos vs Moneyball coverage
        let general_roles = all_roles();
        let mut total = 0usize;
        let mut mapped = 0usize;
        let mut uncovered = Vec::new();
        for role in general_roles {
            for tag in role.position_tags {
                total += 1;
                let has = catalog.definitions.iter().any(|def| {
                    def.attribute_role_id.as_deref() == Some(role.role_id)
                        && def.position_tags.contains(&tag.to_string())
                });
                if has {
                    mapped += 1;
                } else {
                    uncovered.push((role.role_id, *tag));
                }
            }
        }
        assert_eq!(total, 129, "General (role, position) count");
        assert_eq!(mapped, 104, "mapped combos");
        assert_eq!(uncovered.len(), 25, "uncovered count");
        let expected: std::collections::HashSet<(&str, &str)> = [
            ("holding_wing_back_oop", "DL"),
            ("holding_wing_back_oop", "DR"),
            ("pressing_wing_back_oop", "DL"),
            ("pressing_wing_back_oop", "DR"),
            ("box_to_box_midfielder_ip", "MC"),
            ("box_to_box_playmaker_ip", "MC"),
            ("deep_lying_playmaker_ip", "MC"),
            ("second_striker_ip", "ST"),
            // Interim Commit 3: 11 generic OOP roles added before the
            // Moneyball mapping fills their presentation rows (Commit 4).
            ("goalkeeper_oop", "GK"),
            ("centre_back_oop", "DC"),
            ("wide_centre_back_oop", "DC"),
            ("full_back_oop", "DL"),
            ("full_back_oop", "DR"),
            ("wing_back_oop", "DL"),
            ("wing_back_oop", "DR"),
            ("wing_back_oop", "WBL"),
            ("wing_back_oop", "WBR"),
            ("defensive_midfielder_oop", "DM"),
            ("central_midfielder_oop", "MC"),
            ("wide_midfielder_oop", "ML"),
            ("wide_midfielder_oop", "MR"),
            ("attacking_midfielder_oop", "AMC"),
            ("winger_oop", "AML"),
            ("winger_oop", "AMR"),
            ("centre_forward_oop", "ST"),
        ]
        .into_iter()
        .collect();
        let actual: std::collections::HashSet<(&str, &str)> = uncovered.into_iter().collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn uses_distinct_canonical_metric_keys_and_matching_directions() {
        let catalog = builtin_catalog().expect("built-in catalog should load");
        let mut seen = std::collections::HashSet::new();

        for definition in &catalog.definitions {
            assert_eq!(definition.metrics.len(), 5);
            for metric in &definition.metrics {
                assert!(
                    crate::features::moneyball::is_moneyball_statistic_key(&metric.key),
                    "unknown metric key {}",
                    metric.key
                );
                assert!(seen.insert((&definition.id, &metric.key)));
                assert_eq!(
                    metric.inverted,
                    crate::features::moneyball::catalog::is_lower_is_better_statistic_key(
                        &metric.key
                    )
                );
            }
        }
    }
}
