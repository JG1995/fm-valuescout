
// ── Types ──────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

use super::error::StorageError;

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
///
/// Role uses COARSE system: "GK", "D", "WB", "DM", "M", "AM", "ST"
/// This aligns with parser::types::Role enum and validate_role().
/// Side (R/L/C) is not stored — used only for player-to-archetype matching.
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

/// Valid FM position roles
const VALID_ROLES: &[&str] = &["GK", "D", "WB", "DM", "M", "AM", "ST"];

/// Validate archetype role: must be a valid FM position role.
pub fn validate_role(role: &str) -> Result<String, StorageError> {
    let trimmed = role.trim().to_string();
    if !VALID_ROLES.contains(&trimmed.as_str()) {
        return Err(StorageError::Validation(
            format!("Role must be one of: {}", VALID_ROLES.join(", ")),
        ));
    }
    Ok(trimmed)
}

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
    // Check for duplicate metric keys
    let mut seen_keys = std::collections::HashSet::new();
    for m in metrics {
        if !seen_keys.insert(&m.metric_key) {
            return Err(StorageError::Validation(
                "Duplicate metric keys are not allowed.".to_string(),
            ));
        }
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
// ── CRUD Operations ───────────────────────────────────────────────────

use rusqlite::Connection;

/// Helper to convert a database row to an Archetype struct.
fn row_to_archetype(row: &rusqlite::Row) -> rusqlite::Result<Archetype> {
    let metrics_json: String = row.get(3)?;
    let metrics: Vec<MetricWeight> = serde_json::from_str(&metrics_json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
        ))?;

    Ok(Archetype {
        id: row.get(0)?,
        name: row.get(1)?,
        role: row.get(2)?,
        metrics,
        is_default: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

/// Create a new archetype.
/// Validates name and metrics, normalizes weights, and persists to database.
/// Returns the created Archetype with its assigned id.
pub fn create_archetype(
    conn: &Connection,
    name: &str,
    role: &str,
    metrics: &[MetricWeight],
) -> Result<Archetype, StorageError> {
    // Validate inputs
    let clean_name = validate_archetype_name(name)?;
    let clean_role = validate_role(role)?;
    validate_metrics(metrics)?;

    // Normalize weights
    let mut normalized_metrics = metrics.to_vec();
    normalize_weights(&mut normalized_metrics);

    // Serialize metrics to JSON
    let metrics_json = serde_json::to_string(&normalized_metrics)
        .map_err(|e| StorageError::Database(
            format!("Failed to serialize metrics: {}", e)
        ))?;

    // Insert into database
    conn.execute(
        "INSERT INTO archetypes (name, role, metrics_json, is_default) VALUES (?1, ?2, ?3, 0)",
        rusqlite::params![clean_name, clean_role, metrics_json],
    )?;

    // Fetch the created archetype
    let id = conn.last_insert_rowid();
    get_archetype(conn, id)
}

/// List all archetypes for a given role.
pub fn list_archetypes(conn: &Connection, role: &str) -> Result<Vec<Archetype>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, role, metrics_json, is_default, created_at, updated_at
         FROM archetypes WHERE role = ?1 ORDER BY name"
    )?;

    let archetypes: Vec<Archetype> = stmt
        .query_map(rusqlite::params![role], row_to_archetype)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(archetypes)
}

/// List all archetypes regardless of role.
pub fn list_all_archetypes(conn: &Connection) -> Result<Vec<Archetype>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, role, metrics_json, is_default, created_at, updated_at
         FROM archetypes ORDER BY role, name"
    )?;

    let archetypes: Vec<Archetype> = stmt
        .query_map([], row_to_archetype)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(archetypes)
}

/// Get a single archetype by its id.
pub fn get_archetype(conn: &Connection, id: i64) -> Result<Archetype, StorageError> {
    let archetype = conn.query_row(
        "SELECT id, name, role, metrics_json, is_default, created_at, updated_at
         FROM archetypes WHERE id = ?1",
        rusqlite::params![id],
        row_to_archetype,
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            StorageError::NotFound(format!("Archetype with id {} not found.", id))
        }
        other => StorageError::Database(other.to_string()),
    })?;

    Ok(archetype)
}

/// Update an existing archetype's name and metrics.
/// Preserves the is_default flag. Validates and normalizes like create.
pub fn update_archetype(
    conn: &Connection,
    id: i64,
    name: &str,
    metrics: &[MetricWeight],
) -> Result<Archetype, StorageError> {
    // Validate inputs
    let clean_name = validate_archetype_name(name)?;
    validate_metrics(metrics)?;

    // Normalize weights
    let mut normalized_metrics = metrics.to_vec();
    normalize_weights(&mut normalized_metrics);

    // Serialize metrics to JSON
    let metrics_json = serde_json::to_string(&normalized_metrics)
        .map_err(|e| StorageError::Database(
            format!("Failed to serialize metrics: {}", e)
        ))?;

    // Update the database
    let rows_affected = conn.execute(
        "UPDATE archetypes SET name = ?1, metrics_json = ?2, updated_at = datetime('now')
         WHERE id = ?3",
        rusqlite::params![clean_name, metrics_json, id],
    )?;

    if rows_affected == 0 {
        return Err(StorageError::NotFound(
            format!("Archetype with id {} not found.", id)
        ));
    }

    get_archetype(conn, id)
}

/// Delete an archetype by its id.
/// Returns an error if deleting would leave a role with zero archetypes.
pub fn delete_archetype(conn: &Connection, id: i64) -> Result<(), StorageError> {
    // First, check if this archetype exists and get its role
    let role: String = conn.query_row(
        "SELECT role FROM archetypes WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            StorageError::NotFound(format!("Archetype with id {} not found.", id))
        }
        other => StorageError::Database(other.to_string()),
    })?;

    // Check how many archetypes exist for this role
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM archetypes WHERE role = ?1",
        rusqlite::params![role],
        |row| row.get(0),
    )?;

    // Cannot delete the last archetype for a role
    if count <= 1 {
        return Err(StorageError::Validation(
            format!("Cannot delete the last archetype for role '{}'. At least one archetype must exist for each role.", role)
        ));
    }

    // Perform the deletion
    let rows_affected = conn.execute(
        "DELETE FROM archetypes WHERE id = ?1",
        rusqlite::params![id],
    )?;

    if rows_affected == 0 {
        return Err(StorageError::NotFound(
            format!("Archetype with id {} not found.", id)
        ));
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────

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

    fn sample_metrics() -> Vec<MetricWeight> {
        vec![
            MetricWeight {
                metric_key: "attacking.goals_per_90".to_string(),
                weight: 0.6,
                inverted: false,
            },
            MetricWeight {
                metric_key: "chance_creation.assists_per_90".to_string(),
                weight: 0.4,
                inverted: false,
            },
        ]
    }

    #[test]
    fn create_archetype_basic() {
        let conn = setup_test_db();
        let metrics = sample_metrics();
        let archetype = create_archetype(&conn, "Goal Poacher", "ST", &metrics).unwrap();

        assert!(archetype.id > 0);
        assert_eq!(archetype.name, "Goal Poacher");
        assert_eq!(archetype.role, "ST");
        assert_eq!(archetype.metrics.len(), 2);
        assert!(!archetype.is_default);
        // Verify weights were normalized to sum to 1.0
        let sum: f64 = archetype.metrics.iter().map(|m| m.weight).sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn create_archetype_normalizes_weights() {
        let conn = setup_test_db();
        let metrics = vec![
            MetricWeight {
                metric_key: "a".to_string(),
                weight: 0.3,
                inverted: false,
            },
            MetricWeight {
                metric_key: "b".to_string(),
                weight: 0.2,
                inverted: false,
            },
        ];
        let archetype = create_archetype(&conn, "Test", "DM", &metrics).unwrap();

        assert!((archetype.metrics[0].weight - 0.6).abs() < 1e-9);
        assert!((archetype.metrics[1].weight - 0.4).abs() < 1e-9);
    }

    #[test]
    fn create_archetype_empty_name_rejected() {
        let conn = setup_test_db();
        let result = create_archetype(&conn, "", "ST", &sample_metrics());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn create_archetype_empty_metrics_rejected() {
        let conn = setup_test_db();
        let result = create_archetype(&conn, "Test", "ST", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one metric"));
    }

    #[test]
    fn create_archetype_duplicate_name_role_rejected() {
        let conn = setup_test_db();
        create_archetype(&conn, "Test Arch", "GK", &sample_metrics()).unwrap();
        let result = create_archetype(&conn, "Test Arch", "GK", &sample_metrics());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UNIQUE constraint failed"));
    }

    #[test]
    fn create_archetype_same_name_different_role_ok() {
        let conn = setup_test_db();
        create_archetype(&conn, "Complete", "D", &sample_metrics()).unwrap();
        let archetype2 = create_archetype(&conn, "Complete", "DM", &sample_metrics()).unwrap();

        assert_eq!(archetype2.name, "Complete");
        assert_eq!(archetype2.role, "DM");
    }

    #[test]
    fn create_archetype_invalid_role_rejected() {
        let conn = setup_test_db();
        let result = create_archetype(&conn, "Test Arch", "XYZ", &sample_metrics());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Role must be one of"));
    }


    #[test]
    fn validate_role_valid() {
        // Test all COARSE roles (aligns with parser::types::Role)
        assert!(validate_role("GK").is_ok());
        assert!(validate_role("D").is_ok());
        assert!(validate_role("WB").is_ok());
        assert!(validate_role("DM").is_ok());
        assert!(validate_role("M").is_ok());
        assert!(validate_role("AM").is_ok());
        assert!(validate_role("ST").is_ok());
        // Trimmed whitespace
        assert!(validate_role(" GK ").is_ok());
    }

    #[test]
    fn validate_role_invalid() {
        // Fine-grained roles should be rejected (they're not in VALID_ROLES)
        assert!(validate_role("CB").is_err());
        assert!(validate_role("FB").is_err());
        assert!(validate_role("CM").is_err());
        assert!(validate_role("W").is_err());
        // Other invalid
        assert!(validate_role("XYZ").is_err());
        assert!(validate_role("").is_err());
        assert!(validate_role("   ").is_err());
        assert!(validate_role("goalkeeper").is_err());
    }

    #[test]
    fn validate_metrics_duplicate_keys_rejected() {
        let metrics = vec![
            MetricWeight {
                metric_key: "goals_per_90".to_string(),
                weight: 0.5,
                inverted: false,
            },
            MetricWeight {
                metric_key: "goals_per_90".to_string(),
                weight: 0.5,
                inverted: false,
            },
        ];
        let result = validate_metrics(&metrics);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn validate_metrics_duplicate_keys_three_entries() {
        let metrics = vec![
            MetricWeight {
                metric_key: "goals_per_90".to_string(),
                weight: 0.3,
                inverted: false,
            },
            MetricWeight {
                metric_key: "assists_per_90".to_string(),
                weight: 0.3,
                inverted: false,
            },
            MetricWeight {
                metric_key: "goals_per_90".to_string(),
                weight: 0.4,
                inverted: false,
            },
        ];
        let result = validate_metrics(&metrics);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }
    #[test]
    fn list_archetypes_empty() {
        let conn = setup_test_db();
        let archetypes = list_archetypes(&conn, "ST").unwrap();
        assert!(archetypes.is_empty());
    }

    #[test]
    fn list_archetypes_filters_by_role() {
        let conn = setup_test_db();
        create_archetype(&conn, "ST Arch 1", "ST", &sample_metrics()).unwrap();
        create_archetype(&conn, "ST Arch 2", "ST", &sample_metrics()).unwrap();
        create_archetype(&conn, "D Arch", "D", &sample_metrics()).unwrap();

        let st_archetypes = list_archetypes(&conn, "ST").unwrap();
        assert_eq!(st_archetypes.len(), 2);
        assert!(st_archetypes.iter().all(|a| a.role == "ST"));

        let d_archetypes = list_archetypes(&conn, "D").unwrap();
        assert_eq!(d_archetypes.len(), 1);
        assert_eq!(d_archetypes[0].name, "D Arch");
    }

    #[test]
    fn list_all_archetypes_returns_all() {
        let conn = setup_test_db();
        create_archetype(&conn, "Arch 1", "ST", &sample_metrics()).unwrap();
        create_archetype(&conn, "Arch 2", "D", &sample_metrics()).unwrap();
        create_archetype(&conn, "Arch 3", "GK", &sample_metrics()).unwrap();

        let all = list_all_archetypes(&conn).unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn get_archetype_by_id_found() {
        let conn = setup_test_db();
        let created = create_archetype(&conn, "Target Arch", "AM", &sample_metrics()).unwrap();
        let fetched = get_archetype(&conn, created.id).unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, "Target Arch");
    }

    #[test]
    fn get_archetype_by_id_not_found() {
        let conn = setup_test_db();
        let result = get_archetype(&conn, 9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn update_archetype_name_and_metrics() {
        let conn = setup_test_db();
        let created = create_archetype(&conn, "Old Name", "ST", &sample_metrics()).unwrap();

        let new_metrics = vec![
            MetricWeight {
                metric_key: "defensive.tackles_per_90".to_string(),
                weight: 1.0,
                inverted: false,
            },
        ];
        let updated = update_archetype(&conn, created.id, "New Name", &new_metrics).unwrap();

        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.metrics.len(), 1);
        assert_eq!(updated.metrics[0].metric_key, "defensive.tackles_per_90");
    }

    #[test]
    fn update_archetype_not_found() {
        let conn = setup_test_db();
        let result = update_archetype(&conn, 9999, "Test", &sample_metrics());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn update_archetype_duplicate_name_rejected() {
        let conn = setup_test_db();
        create_archetype(&conn, "Arch A", "ST", &sample_metrics()).unwrap();
        let arch_b = create_archetype(&conn, "Arch B", "ST", &sample_metrics()).unwrap();

        let result = update_archetype(&conn, arch_b.id, "Arch A", &sample_metrics());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UNIQUE constraint failed"));
    }

    #[test]
    fn update_archetype_preserves_is_default() {
        let conn = setup_test_db();
        let created = create_archetype(&conn, "Default Test", "GK", &sample_metrics()).unwrap();
        // Manually set is_default to true in the database
        conn.execute(
            "UPDATE archetypes SET is_default = 1 WHERE id = ?1",
            rusqlite::params![created.id],
        ).unwrap();

        let updated = update_archetype(&conn, created.id, "Updated Name", &sample_metrics()).unwrap();
        assert!(updated.is_default);
    }

    #[test]
    fn delete_archetype_basic() {
        let conn = setup_test_db();
        create_archetype(&conn, "To Delete 1", "ST", &sample_metrics()).unwrap();
        let to_delete = create_archetype(&conn, "To Delete 2", "ST", &sample_metrics()).unwrap();

        delete_archetype(&conn, to_delete.id).unwrap();

        let remaining = list_archetypes(&conn, "ST").unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].name, "To Delete 1");
    }

    #[test]
    fn delete_archetype_not_found() {
        let conn = setup_test_db();
        let result = delete_archetype(&conn, 9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn delete_last_archetype_for_role_rejected() {
        let conn = setup_test_db();
        let archetype = create_archetype(&conn, "Last One", "GK", &sample_metrics()).unwrap();

        let result = delete_archetype(&conn, archetype.id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot delete the last archetype"));

        // Verify it still exists
        let remaining = list_archetypes(&conn, "GK").unwrap();
        assert_eq!(remaining.len(), 1);
    }


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
