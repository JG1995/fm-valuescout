use super::scoring::{all_staff_roles, StaffRoleDefinition};

pub const MAX_REQUESTED_FIELDS: usize = 256;

pub const STAFF_ATTRIBUTE_KEYS: &[&str] = &[
    "Attacking",
    "Defending",
    "Fitness",
    "Possession",
    "Technical",
    "Tactical",
    "SetPieces",
    "Determination",
    "ManManagement",
    "Motivating",
    "JudgingPlayerAbility",
    "JudgingPlayerPotential",
    "JudgingStaffAbility",
    "Negotiating",
    "TacticalKnowledge",
    "Physiotherapy",
    "SportsScience",
    "Authority",
    "Adaptability",
    "DataAnalysis",
    "WorkingWithYoungsters",
    "GoalkeepingDistribution",
    "GoalkeepingHandling",
    "GoalkeepingReflexes",
];

pub const BASIC_FIELD_IDS: &[&str] = &[
    "name",
    "age",
    "birth_year",
    "birth_day_of_year",
    "nationality",
    "nation_uid",
    "gender",
    "club",
    "division",
    "ca",
    "pa",
    "job_id",
    "wage",
    "contract_year",
    "contract_day",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricField {
    Attribute(&'static str),
    Role(&'static StaffRoleDefinition),
}

impl MetricField {
    pub fn parse(id: &str) -> Result<Self, String> {
        if let Some(key) = id.strip_prefix("attr.") {
            return STAFF_ATTRIBUTE_KEYS
                .iter()
                .copied()
                .find(|candidate| *candidate == key)
                .map(Self::Attribute)
                .ok_or_else(|| format!("unknown staff attribute: {key}"));
        }
        if let Some(role_id) = id.strip_prefix("role.") {
            return all_staff_roles()
                .iter()
                .find(|role| role.role_id == role_id)
                .map(Self::Role)
                .ok_or_else(|| format!("unknown staff role: {role_id}"));
        }
        Err(format!("unknown staff metric: {id}"))
    }

    pub fn id(self) -> String {
        match self {
            Self::Attribute(key) => format!("attr.{key}"),
            Self::Role(role) => format!("role.{}", role.role_id),
        }
    }

    pub fn sql_expression(self, alias: &str) -> String {
        match self {
            Self::Attribute(key) => {
                format!("json_extract({alias}.staff_attributes_json, '$.{key}')")
            }
            Self::Role(role) => format!(
                "(SELECT srs.score FROM staff_role_scores srs WHERE srs.snapshot_id = {alias}.snapshot_id AND srs.uid = {alias}.uid AND srs.role_id = '{}')",
                role.role_id
            ),
        }
    }
}

pub fn parse_requested_fields(ids: &[String]) -> Result<Vec<MetricField>, String> {
    if ids.len() > MAX_REQUESTED_FIELDS {
        return Err(format!(
            "requested field count exceeds maximum of {MAX_REQUESTED_FIELDS}"
        ));
    }
    let mut fields = Vec::new();
    for id in ids {
        if BASIC_FIELD_IDS.contains(&id.as_str()) {
            continue;
        }
        let field = MetricField::parse(id)?;
        if !fields.contains(&field) {
            fields.push(field);
        }
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_requested_field_counts_above_the_input_bound() {
        let fields = vec!["ca".to_string(); MAX_REQUESTED_FIELDS + 1];

        assert_eq!(
            parse_requested_fields(&fields).unwrap_err(),
            "requested field count exceeds maximum of 256"
        );
    }
}
