use rusqlite::Connection;
use serde_json;

use super::archetypes::MetricWeight;
use super::error::StorageError;

/// Maps human-readable metric names to ParsedPlayer field paths.
/// The field paths use dot notation: "category.field_name"
/// e.g., "attacking.goals_per_90" maps to player.attacking.goals_per_90
pub fn metric_key_for_name(name: &str) -> Option<&'static str> {
    match name {
        // Attacking & Finishing
        "Goals per 90" => Some("attacking.goals_per_90"),
        "xG per 90" => Some("attacking.xg_per_90"),
        "NPxG per 90" => Some("attacking.np_xg_per_90"),
        "Shots per 90" => Some("attacking.shots_per_90"),
        "Shots on Target Ratio" => Some("attacking.shots_on_target_per_90"),
        "Conversion Rate" => Some("attacking.goals_per_90"), // Approximation
        "Average Minutes per Goal" => Some("attacking.goals_per_90"), // Inverted approximation

        // Creativity & Chance Creation
        "Assists per 90" => Some("chance_creation.assists_per_90"),
        "xA per 90" => Some("chance_creation.xa_per_90"),
        "Chances Created per 90" => Some("chance_creation.chances_created_per_90"),
        "Key Passes per 90" => Some("chance_creation.key_passes_per_90"),
        "Progressive Passes per 90" => Some("chance_creation.progressive_passes_per_90"),
        "Crosses Completed per 90" => Some("chance_creation.crosses_completed"),
        "Open Play Crosses Completed per 90" => Some("chance_creation.open_play_crosses_completed"),
        "Pass Completion Ratio" => Some("chance_creation.pass_completion_rate"),
        "Passes Attempted per 90" => Some("chance_creation.passes_attempted"),
        "Passes Completed per 90" => Some("chance_creation.passes_completed"),

        // Transition & Ball Progression
        "Dribbles Made per 90" => Some("movement.dribbles_per_90"),
        "Distance Covered per 90" => Some("movement.distance_per_90"),
        "High Intensity Sprints per 90" => Some("movement.sprints_per_90"),
        "Possession Lost per 90" => Some("movement.possession_lost_per_90"),

        // Defensive Actions
        "Tackles per 90" => Some("defending.tackles_per_90"),
        "Tackle Completion Ratio" => Some("defending.tackle_completion_rate"),
        "Interceptions per 90" => Some("defending.interceptions_per_90"),
        "Pressures Completed per 90" => Some("defending.pressures_per_90"),
        "Pressure Success Ratio" => Some("defending.pressure_completion_rate"),
        "Possession Won per 90" => Some("defending.possession_won_per_90"),
        "Blocks per 90" => Some("defending.blocks"),
        "Clearances per 90" => Some("defending.clearances_per_90"),

        // Aerial Presence
        "Headers Won Ratio" => Some("aerial.aerial_challenge_rate"),
        "Aerial Challenges Attempted" => Some("aerial.aerial_duels_per_90"),
        "Key Headers per 90" => Some("aerial.key_headers_per_90"),

        // Goalkeeping
        "Expected Goals Prevented per 90" => Some("goalkeeping.expected_goals_prevented"),
        "Save Ratio" => Some("goalkeeping.expected_save_pct"),
        "Saves Held" => Some("goalkeeping.saves_held"),
        "Clean Sheets" => Some("goalkeeping.clean_sheets"),

        // Discipline
        "Fouls Made" => Some("discipline.fouls_made_per_90"),
        "Fouls Against" => Some("discipline.fouls_against_per_90"),
        "Offsides" => Some("discipline.offsides"),
        "Mistakes Leading to Goal" => Some("discipline.mistakes_leading_to_goal"),

        // Match Impact
        "Average Rating" => Some("match_outcome.average_rating"),

        _ => None,
    }
}

/// Returns all default archetypes with their metrics.
/// Roles use the seed data format: "GK", "CB", "FB", "DM", "WB", "CM", "W", "AM", "ST"
pub fn default_archetypes() -> Vec<DefaultArchetype> {
    vec![
        // === GOALKEEPER ===
        DefaultArchetype {
            name: "Traditional Goalkeeper".to_string(),
            role: "GK".to_string(),
            metrics: vec![
                // In Possession
                ("Pass Completion Ratio", 0.70, false),
                ("Passes Attempted per 90", 0.30, false),
            ],
        },
        DefaultArchetype {
            name: "Ball-Playing Goalkeeper".to_string(),
            role: "GK".to_string(),
            metrics: vec![
                ("Progressive Passes per 90", 0.40, false),
                ("Pass Completion Ratio", 0.30, false),
                ("Passes Attempted per 90", 0.20, false),
                ("xA per 90", 0.10, false),
            ],
        },
        DefaultArchetype {
            name: "Shot Stopper".to_string(),
            role: "GK".to_string(),
            metrics: vec![
                // Out of Possession
                ("Expected Goals Prevented per 90", 0.50, false),
                ("Save Ratio", 0.30, false),
                ("Saves Held", 0.20, false),
            ],
        },
        DefaultArchetype {
            name: "Sweeper Keeper".to_string(),
            role: "GK".to_string(),
            metrics: vec![
                ("Expected Goals Prevented per 90", 0.40, false),
                ("Interceptions per 90", 0.30, false),
                ("High Intensity Sprints per 90", 0.20, false),
                ("Distance Covered per 90", 0.10, false),
            ],
        },

        // === CENTER BACK ===
        DefaultArchetype {
            name: "Traditional Center Back".to_string(),
            role: "CB".to_string(),
            metrics: vec![
                // In Possession
                ("Pass Completion Ratio", 0.60, false),
                ("Passes Completed per 90", 0.40, false),
            ],
        },
        DefaultArchetype {
            name: "Ball-Playing Center Back".to_string(),
            role: "CB".to_string(),
            metrics: vec![
                ("Progressive Passes per 90", 0.45, false),
                ("Pass Completion Ratio", 0.25, false),
                ("Passes Attempted per 90", 0.20, false),
                ("Key Passes per 90", 0.10, false),
            ],
        },
        DefaultArchetype {
            name: "Defensive Center Back".to_string(),
            role: "CB".to_string(),
            metrics: vec![
                // Out of Possession
                ("Headers Won Ratio", 0.30, false),
                ("Interceptions per 90", 0.25, false),
                ("Possession Won per 90", 0.20, false),
                ("Blocks per 90", 0.15, false),
                ("Clearances per 90", 0.10, false),
            ],
        },

        // === FULL BACK ===
        DefaultArchetype {
            name: "Full Back".to_string(),
            role: "FB".to_string(),
            metrics: vec![
                // In Possession
                ("Pass Completion Ratio", 0.40, false),
                ("Passes Attempted per 90", 0.30, false),
                ("Progressive Passes per 90", 0.20, false),
                ("Distance Covered per 90", 0.10, false),
            ],
        },
        DefaultArchetype {
            name: "Offensive Full Back".to_string(),
            role: "FB".to_string(),
            metrics: vec![
                ("xA per 90", 0.35, false),
                ("Open Play Crosses Completed per 90", 0.25, false),
                ("Dribbles Made per 90", 0.20, false),
                ("Key Passes per 90", 0.20, false),
            ],
        },
        DefaultArchetype {
            name: "Pressing Full Back".to_string(),
            role: "FB".to_string(),
            metrics: vec![
                // Out of Possession
                ("Pressures Completed per 90", 0.35, false),
                ("Possession Won per 90", 0.25, false),
                ("High Intensity Sprints per 90", 0.25, false),
                ("Tackles per 90", 0.15, false),
            ],
        },
        DefaultArchetype {
            name: "Defensive Full Back".to_string(),
            role: "FB".to_string(),
            metrics: vec![
                ("Tackle Completion Ratio", 0.35, false),
                ("Interceptions per 90", 0.25, false),
                ("Headers Won Ratio", 0.20, false),
                ("Blocks per 90", 0.20, false),
            ],
        },

        // === DEFENSIVE MIDFIELDER ===
        DefaultArchetype {
            name: "Defensive Midfielder".to_string(),
            role: "DM".to_string(),
            metrics: vec![
                // In Possession
                ("Pass Completion Ratio", 0.45, false),
                ("Passes Completed per 90", 0.30, false),
                ("Progressive Passes per 90", 0.15, false),
                ("Possession Lost per 90", 0.10, true), // inverted
            ],
        },
        DefaultArchetype {
            name: "Playmaker".to_string(),
            role: "DM".to_string(),
            metrics: vec![
                ("Progressive Passes per 90", 0.40, false),
                ("xA per 90", 0.20, false),
                ("Passes Attempted per 90", 0.20, false),
                ("Key Passes per 90", 0.20, false),
            ],
        },
        DefaultArchetype {
            name: "Pressing Defensive Midfielder".to_string(),
            role: "DM".to_string(),
            metrics: vec![
                // Out of Possession
                ("Pressures Completed per 90", 0.35, false),
                ("Possession Won per 90", 0.30, false),
                ("Tackles per 90", 0.20, false),
                ("Interceptions per 90", 0.15, false),
            ],
        },
        DefaultArchetype {
            name: "Deep-Lying Playmaker".to_string(),
            role: "DM".to_string(),
            metrics: vec![
                ("Interceptions per 90", 0.45, false),
                ("Distance Covered per 90", 0.25, false),
                ("Fouls Made", 0.20, false),
                ("Mistakes Leading to Goal", 0.10, true), // inverted
            ],
        },

        // === WING BACK ===
        DefaultArchetype {
            name: "Wing Back".to_string(),
            role: "WB".to_string(),
            metrics: vec![
                // In Possession
                ("Crosses Completed per 90", 0.35, false),
                ("Passes Attempted per 90", 0.25, false),
                ("Progressive Passes per 90", 0.20, false),
                ("Distance Covered per 90", 0.20, false),
            ],
        },
        DefaultArchetype {
            name: "Offensive Wing Back".to_string(),
            role: "WB".to_string(),
            metrics: vec![
                ("xA per 90", 0.35, false),
                ("Dribbles Made per 90", 0.25, false),
                ("Chances Created per 90", 0.20, false),
                ("Open Play Crosses Completed per 90", 0.20, false),
            ],
        },
        DefaultArchetype {
            name: "Pressing Wing Back".to_string(),
            role: "WB".to_string(),
            metrics: vec![
                // Out of Possession
                ("Pressures Completed per 90", 0.40, false),
                ("Possession Won per 90", 0.30, false),
                ("High Intensity Sprints per 90", 0.20, false),
                ("Tackles per 90", 0.10, false),
            ],
        },
        DefaultArchetype {
            name: "Tracking Wing Back".to_string(),
            role: "WB".to_string(),
            metrics: vec![
                ("Distance Covered per 90", 0.40, false),
                ("Interceptions per 90", 0.30, false),
                ("Tackle Completion Ratio", 0.20, false),
                ("Blocks per 90", 0.10, false),
            ],
        },

        // === CENTRAL MIDFIELDER ===
        DefaultArchetype {
            name: "All-Rounder Midfielder".to_string(),
            role: "CM".to_string(),
            metrics: vec![
                // In Possession
                ("Pass Completion Ratio", 0.30, false),
                ("Passes Attempted per 90", 0.25, false),
                ("Progressive Passes per 90", 0.25, false),
                ("Distance Covered per 90", 0.20, false),
            ],
        },
        DefaultArchetype {
            name: "Box-to-Box Midfielder".to_string(),
            role: "CM".to_string(),
            metrics: vec![
                ("xG per 90", 0.30, false),
                ("Distance Covered per 90", 0.25, false),
                ("Shots per 90", 0.25, false),
                ("Progressive Passes per 90", 0.20, false),
            ],
        },
        DefaultArchetype {
            name: "Advanced Playmaker".to_string(),
            role: "CM".to_string(),
            metrics: vec![
                ("xA per 90", 0.35, false),
                ("Progressive Passes per 90", 0.30, false),
                ("Key Passes per 90", 0.20, false),
                ("Pass Completion Ratio", 0.15, false),
            ],
        },
        DefaultArchetype {
            name: "Covering Midfielder".to_string(),
            role: "CM".to_string(),
            metrics: vec![
                // Out of Possession
                ("Interceptions per 90", 0.40, false),
                ("Tackle Completion Ratio", 0.30, false),
                ("Blocks per 90", 0.20, false),
                ("Distance Covered per 90", 0.10, false),
            ],
        },

        // === WINGER ===
        DefaultArchetype {
            name: "Traditional Winger".to_string(),
            role: "W".to_string(),
            metrics: vec![
                // In Possession
                ("Crosses Completed per 90", 0.40, false),
                ("Dribbles Made per 90", 0.30, false),
                ("Key Passes per 90", 0.20, false),
                ("Pass Completion Ratio", 0.10, false),
            ],
        },
        DefaultArchetype {
            name: "Goalscoring Winger".to_string(),
            role: "W".to_string(),
            metrics: vec![
                ("NPxG per 90", 0.45, false),
                ("Shots on Target Ratio", 0.25, false),
                ("Conversion Rate", 0.20, false),
                ("Goals per 90", 0.10, false),
            ],
        },
        DefaultArchetype {
            name: "Inside Forward".to_string(),
            role: "W".to_string(),
            metrics: vec![
                ("xG per 90", 0.30, false),
                ("xA per 90", 0.25, false),
                ("Dribbles Made per 90", 0.25, false),
                ("Key Passes per 90", 0.20, false),
            ],
        },
        DefaultArchetype {
            name: "Offensive Winger".to_string(),
            role: "W".to_string(),
            metrics: vec![
                // Out of Possession
                ("High Intensity Sprints per 90", 0.40, false),
                ("xG per 90", 0.30, false),
                ("Fouls Against", 0.20, false),
                ("Offsides", 0.10, true), // inverted
            ],
        },
        DefaultArchetype {
            name: "Tracking Winger".to_string(),
            role: "W".to_string(),
            metrics: vec![
                ("Distance Covered per 90", 0.35, false),
                ("Pressures Completed per 90", 0.30, false),
                ("Tackles per 90", 0.20, false),
                ("Interceptions per 90", 0.15, false),
            ],
        },

        // === ATTACKING MIDFIELDER ===
        DefaultArchetype {
            name: "Running Attacking Midfielder".to_string(),
            role: "AM".to_string(),
            metrics: vec![
                // In Possession
                ("NPxG per 90", 0.40, false),
                ("Shots per 90", 0.25, false),
                ("Distance Covered per 90", 0.20, false),
                ("Goals per 90", 0.15, false),
            ],
        },
        DefaultArchetype {
            name: "Creative Attacking Midfielder".to_string(),
            role: "AM".to_string(),
            metrics: vec![
                ("xA per 90", 0.35, false),
                ("Key Passes per 90", 0.30, false),
                ("Progressive Passes per 90", 0.20, false),
                ("Pass Completion Ratio", 0.15, false),
            ],
        },
        DefaultArchetype {
            name: "Offensive Attacking Midfielder".to_string(),
            role: "AM".to_string(),
            metrics: vec![
                // Out of Possession
                ("High Intensity Sprints per 90", 0.40, false),
                ("NPxG per 90", 0.30, false),
                ("Fouls Against", 0.20, false),
                ("Offsides", 0.10, true), // inverted
            ],
        },
        DefaultArchetype {
            name: "Tracking Attacking Midfielder".to_string(),
            role: "AM".to_string(),
            metrics: vec![
                ("Pressures Completed per 90", 0.40, false),
                ("Distance Covered per 90", 0.30, false),
                ("Tackles per 90", 0.15, false),
                ("Interceptions per 90", 0.15, false),
            ],
        },

        // === STRIKER ===
        DefaultArchetype {
            name: "Creative Forward".to_string(),
            role: "ST".to_string(),
            metrics: vec![
                // In Possession
                ("xA per 90", 0.35, false),
                ("Key Passes per 90", 0.25, false),
                ("Progressive Passes per 90", 0.20, false),
                ("Pass Completion Ratio", 0.10, false),
                ("Dribbles Made per 90", 0.10, false),
            ],
        },
        DefaultArchetype {
            name: "Goalscoring Forward".to_string(),
            role: "ST".to_string(),
            metrics: vec![
                ("NPxG per 90", 0.45, false),
                ("xG per Shot", 0.25, false),
                ("Shots on Target Ratio", 0.15, false),
                ("Conversion Rate", 0.10, false),
                ("Average Minutes per Goal", 0.05, false),
            ],
        },
        DefaultArchetype {
            name: "Offensive Striker".to_string(),
            role: "ST".to_string(),
            metrics: vec![
                // Out of Possession
                ("High Intensity Sprints per 90", 0.40, false),
                ("NPxG per 90", 0.30, false),
                ("Fouls Against", 0.20, false),
                ("Offsides", 0.10, true), // inverted
            ],
        },
        DefaultArchetype {
            name: "Pressing Forward".to_string(),
            role: "ST".to_string(),
            metrics: vec![
                ("Pressures Completed per 90", 0.40, false),
                ("Possession Won per 90", 0.30, false),
                ("Distance Covered per 90", 0.20, false),
                ("Tackles per 90", 0.10, false),
            ],
        },
    ]
}

/// Helper struct for defining default archetypes.
#[derive(Debug, Clone)]
pub struct DefaultArchetype {
    pub name: String,
    pub role: String,
    pub metrics: Vec<(&'static str, f64, bool)>, // (metric_name, weight, inverted)
}

/// Seeds the database with default archetypes if the table is empty.
/// This function is idempotent - safe to call multiple times.
pub fn seed_default_archetypes(conn: &Connection) -> Result<(), StorageError> {
    // Check if archetypes table already has data
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM archetypes",
        [],
        |row| row.get(0),
    )?;

    // If table already has archetypes, skip seeding
    if count > 0 {
        return Ok(());
    }

    // Seed all default archetypes
    for default_arch in default_archetypes() {
        // Convert metric names to keys
        let mut metrics = Vec::new();
        for (metric_name, weight, inverted) in default_arch.metrics {
            if let Some(metric_key) = metric_key_for_name(metric_name) {
                metrics.push(MetricWeight {
                    metric_key: metric_key.to_string(),
                    weight,
                    inverted,
                });
            }
        }

        // Skip if no valid metrics were found
        if metrics.is_empty() {
            continue;
        }

        // Normalize weights to sum to 1.0
        let total: f64 = metrics.iter().map(|m| m.weight).sum();
        if total > 0.0 {
            for m in metrics.iter_mut() {
                m.weight /= total;
            }
        }

        // Serialize metrics to JSON
        let metrics_json = serde_json::to_string(&metrics)
            .map_err(|e| StorageError::Database(
                format!("Failed to serialize metrics: {}", e)
            ))?;

        // Insert the archetype
        conn.execute(
            "INSERT INTO archetypes (name, role, metrics_json, is_default) VALUES (?1, ?2, ?3, 1)",
            rusqlite::params![default_arch.name, default_arch.role, metrics_json],
        )?;
    }

    Ok(())
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::schema::init_schema;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn metric_key_mapping_known_metrics() {
        // Test a sample of known metric mappings
        assert_eq!(metric_key_for_name("Goals per 90"), Some("attacking.goals_per_90"));
        assert_eq!(metric_key_for_name("xA per 90"), Some("chance_creation.xa_per_90"));
        assert_eq!(metric_key_for_name("Tackles per 90"), Some("defending.tackles_per_90"));
        assert_eq!(metric_key_for_name("Pass Completion Ratio"), Some("chance_creation.pass_completion_rate"));
        assert_eq!(metric_key_for_name("Distance Covered per 90"), Some("movement.distance_per_90"));
    }

    #[test]
    fn metric_key_mapping_unknown_metric() {
        // Test that unknown metrics return None
        assert_eq!(metric_key_for_name("Unknown Metric"), None);
        assert_eq!(metric_key_for_name(""), None);
    }

    #[test]
    fn default_archetypes_have_valid_roles() {
        let archetypes = default_archetypes();
        let valid_roles = ["GK", "CB", "FB", "DM", "WB", "CM", "W", "AM", "ST"];

        for arch in &archetypes {
            assert!(
                valid_roles.contains(&arch.role.as_str()),
                "Invalid role '{}' for archetype '{}'",
                arch.role,
                arch.name
            );
        }
    }

    #[test]
    fn default_archetypes_weights_sum_to_one() {
        let archetypes = default_archetypes();

        for arch in &archetypes {
            let total: f64 = arch.metrics.iter().map(|(_, w, _)| w).sum();
            assert!(
                (total - 1.0).abs() < 0.01,
                "Weights for '{}' sum to {} (expected ~1.0)",
                arch.name,
                total
            );
        }
    }

    #[test]
    fn default_archetypes_have_metrics() {
        let archetypes = default_archetypes();

        for arch in &archetypes {
            assert!(
                !arch.metrics.is_empty(),
                "Archetype '{}' has no metrics",
                arch.name
            );
        }
    }

    #[test]
    fn seed_default_archetypes_creates_all() {
        let conn = setup_test_db();

        // Seed the archetypes
        seed_default_archetypes(&conn).unwrap();

        // Count the seeded archetypes
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM archetypes WHERE is_default = 1",
            [],
            |row| row.get(0),
        ).unwrap();

        // We should have seeded all default archetypes
        assert!(count > 0, "No archetypes were seeded");
    }

    #[test]
    fn seed_default_archetypes_is_idempotent() {
        let conn = setup_test_db();

        // Seed twice
        seed_default_archetypes(&conn).unwrap();
        seed_default_archetypes(&conn).unwrap();

        // Count - should still be the same (idempotent)
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM archetypes WHERE is_default = 1",
            [],
            |row| row.get(0),
        ).unwrap();

        // Should not have duplicated
        let expected_count = default_archetypes().len() as i64;
        assert_eq!(count, expected_count, "Seeding should be idempotent");
    }

    #[test]
    fn seed_default_archetypes_skips_if_data_exists() {
        let conn = setup_test_db();

        // Manually insert an archetype first
        conn.execute(
            "INSERT INTO archetypes (name, role, metrics_json, is_default) VALUES (?1, ?2, ?3, 0)",
            rusqlite::params!["Manual Arch", "ST", "[]"],
        ).unwrap();

        // Try to seed
        seed_default_archetypes(&conn).unwrap();

        // Count - should only have the manual one
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM archetypes",
            [],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(count, 1, "Should not seed when data already exists");
    }

    #[test]
    fn seeded_archetypes_have_valid_metrics() {
        let conn = setup_test_db();
        seed_default_archetypes(&conn).unwrap();

        // Fetch all seeded archetypes
        let mut stmt = conn.prepare(
            "SELECT id, name, metrics_json FROM archetypes WHERE is_default = 1"
        ).unwrap();

        let archetypes: Vec<(i64, String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        // Verify each has valid JSON metrics
        for (id, name, metrics_json) in archetypes {
            let metrics: Result<Vec<MetricWeight>, _> = serde_json::from_str(&metrics_json);
            assert!(
                metrics.is_ok(),
                "Archetype '{}' (id {}) has invalid metrics JSON",
                name,
                id
            );

            let metrics = metrics.unwrap();
            assert!(
                !metrics.is_empty(),
                "Archetype '{}' (id {}) has no metrics",
                name,
                id
            );

            // Verify weights sum to ~1.0
            let total: f64 = metrics.iter().map(|m| m.weight).sum();
            assert!(
                (total - 1.0).abs() < 0.01,
                "Archetype '{}' weights sum to {} (expected ~1.0)",
                name,
                total
            );
        }
    }
}
