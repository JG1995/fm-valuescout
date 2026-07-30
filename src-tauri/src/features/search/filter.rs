use rusqlite::types::Value;

/// Maximum filter rules accepted at the trust boundary.
pub const MAX_FILTER_RULES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombineMode {
    And,
    Or,
}

impl CombineMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "and" => Ok(Self::And),
            "or" => Ok(Self::Or),
            _ => Err(format!("unknown filter combine mode: {value}")),
        }
    }

    fn sql_keyword(self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterRule {
    pub field: String,
    pub op: String,
    pub value: FilterValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterValue {
    Text(String),
    Integer(i64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterAst {
    pub combine: CombineMode,
    pub rules: Vec<FilterRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledFilter {
    pub sql: String,
    pub params: Vec<Value>,
}

/// FM26 dump position keys (bridge `PositionEntries`).
const POSITION_KEYS: &[&str] = &[
    "GK", "SW", "DL", "DC", "DR", "DM", "ML", "MC", "MR", "AML", "AMC", "AMR", "ST", "WBL", "WBR",
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum FieldKind {
    String {
        column: &'static str,
        case_insensitive: bool,
    },
    Integer {
        column: &'static str,
        nullable: bool,
    },
    Boolean {
        column: &'static str,
    },
    Enum {
        column: &'static str,
        allowed: &'static [&'static str],
    },
    /// Integer in a JSON object column (`attributes_json`, etc.).
    JsonInteger {
        column: &'static str,
        key: String,
    },
    /// String list JSON array (`nationalities_json`) — any element may match.
    StringList {
        column: &'static str,
    },
    /// Position key presence / key match against `positions_json`.
    PositionPresence,
}

fn is_safe_json_object_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|ch| ch.is_ascii_alphanumeric())
}

fn resolve_prefixed_json_integer(
    field: &str,
    prefix: &str,
    column: &'static str,
) -> Result<Option<FieldKind>, String> {
    let Some(key) = field.strip_prefix(prefix) else {
        return Ok(None);
    };
    if !is_safe_json_object_key(key) {
        return Err(format!("invalid filter field key: {key}"));
    }
    Ok(Some(FieldKind::JsonInteger {
        column,
        key: key.to_string(),
    }))
}

fn resolve_position_suitability(field: &str) -> Result<Option<FieldKind>, String> {
    let Some(key) = field.strip_prefix("pos.") else {
        return Ok(None);
    };
    let Some(canonical) = POSITION_KEYS
        .iter()
        .copied()
        .find(|candidate| *candidate == key)
    else {
        return Err(format!("unknown position key: {key}"));
    };
    Ok(Some(FieldKind::JsonInteger {
        column: "positions_json",
        key: canonical.to_string(),
    }))
}

fn resolve_field(field: &str) -> Result<FieldKind, String> {
    match field {
        "name" => Ok(FieldKind::String {
            column: "name",
            case_insensitive: true,
        }),
        "club" => Ok(FieldKind::String {
            column: "current_club",
            case_insensitive: true,
        }),
        "division" => Ok(FieldKind::String {
            column: "division",
            case_insensitive: true,
        }),
        "parent_club" => Ok(FieldKind::String {
            column: "parent_club",
            case_insensitive: true,
        }),
        "age" => Ok(FieldKind::Integer {
            column: "age",
            nullable: true,
        }),
        "ca" => Ok(FieldKind::Integer {
            column: "ca",
            nullable: false,
        }),
        "pa" => Ok(FieldKind::Integer {
            column: "pa",
            nullable: false,
        }),
        "height" => Ok(FieldKind::Integer {
            column: "height_cm",
            nullable: true,
        }),
        "wage" => Ok(FieldKind::Integer {
            column: "weekly_wage_gbp",
            nullable: true,
        }),
        "value" => Ok(FieldKind::Integer {
            column: "market_value_gbp",
            nullable: true,
        }),
        "reputation" => Ok(FieldKind::Integer {
            column: "reputation_current",
            nullable: true,
        }),
        "world_reputation" => Ok(FieldKind::Integer {
            column: "reputation_world",
            nullable: true,
        }),
        "birth_year" => Ok(FieldKind::Integer {
            column: "birth_year",
            nullable: false,
        }),
        "contract_year" => Ok(FieldKind::Integer {
            column: "contract_expiry_year",
            nullable: true,
        }),
        "transfer_listed" => Ok(FieldKind::Boolean {
            column: "transfer_listed",
        }),
        "loan_listed" => Ok(FieldKind::Boolean {
            column: "loan_listed",
        }),
        "not_for_sale" => Ok(FieldKind::Boolean {
            column: "not_for_sale",
        }),
        "set_for_release" => Ok(FieldKind::Boolean {
            column: "set_for_release",
        }),
        "on_loan" => Ok(FieldKind::Boolean { column: "on_loan" }),
        "preferred_foot" => Ok(FieldKind::Enum {
            column: "preferred_foot",
            allowed: &["left", "right", "either"],
        }),
        "team_level" => Ok(FieldKind::Enum {
            column: "team_level",
            allowed: &["senior", "reserve", "youth"],
        }),
        "nationality" => Ok(FieldKind::StringList {
            column: "nationalities_json",
        }),
        "position" => Ok(FieldKind::PositionPresence),
        _ => {
            if let Some(kind) = resolve_prefixed_json_integer(field, "attr.", "attributes_json")? {
                return Ok(kind);
            }
            if let Some(kind) =
                resolve_prefixed_json_integer(field, "hidden.", "hidden_attributes_json")?
            {
                return Ok(kind);
            }
            if let Some(kind) =
                resolve_prefixed_json_integer(field, "personality.", "personality_json")?
            {
                return Ok(kind);
            }
            if let Some(kind) = resolve_position_suitability(field)? {
                return Ok(kind);
            }
            Err(format!("unknown filter field: {field}"))
        }
    }
}

pub fn filter_value_from_json(value: serde_json::Value) -> Result<FilterValue, String> {
    match value {
        serde_json::Value::String(text) => Ok(FilterValue::Text(text)),
        serde_json::Value::Number(number) => number
            .as_i64()
            .map(FilterValue::Integer)
            .ok_or_else(|| "filter value number out of range".to_string()),
        serde_json::Value::Bool(value) => Ok(FilterValue::Bool(value)),
        serde_json::Value::Null => Err("filter value cannot be null".to_string()),
        _ => Err("filter value must be a string, number, or boolean".to_string()),
    }
}

pub fn parse_filter_ast(
    rules: Vec<FilterRule>,
    combine: Option<&str>,
) -> Result<FilterAst, String> {
    if rules.len() > MAX_FILTER_RULES {
        return Err(format!(
            "filter rule count exceeds maximum of {}",
            MAX_FILTER_RULES
        ));
    }

    let combine = match combine {
        None => CombineMode::And,
        Some(value) => CombineMode::parse(value)?,
    };

    Ok(FilterAst { combine, rules })
}

pub fn compile_filters(ast: &FilterAst, start_index: usize) -> Result<CompiledFilter, String> {
    if ast.rules.is_empty() {
        return Ok(CompiledFilter {
            sql: String::new(),
            params: Vec::new(),
        });
    }

    let mut clauses = Vec::with_capacity(ast.rules.len());
    let mut params = Vec::new();
    let mut next_index = start_index;

    for rule in &ast.rules {
        let field_kind = resolve_field(&rule.field)?;
        let (clause, rule_params) =
            compile_rule(field_kind, &rule.op, &rule.value, &mut next_index)?;
        clauses.push(clause);
        params.extend(rule_params);
    }

    let joined = clauses.join(&format!(" {} ", ast.combine.sql_keyword()));
    Ok(CompiledFilter {
        sql: format!("({joined})"),
        params,
    })
}

fn compile_rule(
    field: FieldKind,
    op: &str,
    value: &FilterValue,
    next_index: &mut usize,
) -> Result<(String, Vec<Value>), String> {
    match field {
        FieldKind::String {
            column,
            case_insensitive,
        } => compile_string_rule(column, case_insensitive, op, value, next_index),
        FieldKind::Integer { column, nullable } => {
            compile_integer_rule(column, nullable, op, value, next_index)
        }
        FieldKind::Boolean { column } => compile_boolean_rule(column, op, value, next_index),
        FieldKind::Enum { column, allowed } => {
            compile_enum_rule(column, allowed, op, value, next_index)
        }
        FieldKind::JsonInteger { column, key } => {
            compile_json_integer_rule(column, &key, op, value, next_index)
        }
        FieldKind::StringList { column } => compile_string_list_rule(column, op, value, next_index),
        FieldKind::PositionPresence => compile_position_presence_rule(op, value, next_index),
    }
}

fn json_extract_expr(column: &str, key: &str) -> String {
    // Key is validated alphanumeric — safe to embed in the JSON path literal.
    format!("json_extract({column}, '$.{key}')")
}

fn compile_json_integer_rule(
    column: &str,
    key: &str,
    op: &str,
    value: &FilterValue,
    next_index: &mut usize,
) -> Result<(String, Vec<Value>), String> {
    let number = value_as_integer(value)?;
    let placeholder = next_placeholder(next_index);
    let compare = match op {
        "gt" => ">",
        "lt" => "<",
        "eq" => "=",
        "neq" => "!=",
        _ => return Err(format!("invalid integer filter operator: {op}")),
    };
    let extract = json_extract_expr(column, key);
    let clause = format!("({extract} {compare} {placeholder} AND {extract} IS NOT NULL)");
    Ok((clause, vec![Value::Integer(number)]))
}

fn compile_string_list_rule(
    column: &str,
    op: &str,
    value: &FilterValue,
    next_index: &mut usize,
) -> Result<(String, Vec<Value>), String> {
    let text = value_as_text(value)?;
    let placeholder = next_placeholder(next_index);

    let (exists_op, bound) = match op {
        "contains" => (
            format!(
                "EXISTS (SELECT 1 FROM json_each({column}) AS n WHERE typeof(n.value) = 'text' AND n.value LIKE {placeholder} ESCAPE '\\' COLLATE NOCASE)"
            ),
            Value::Text(format!("%{}%", escape_like(&text))),
        ),
        "not_contains" => (
            format!(
                "NOT EXISTS (SELECT 1 FROM json_each({column}) AS n WHERE typeof(n.value) = 'text' AND n.value LIKE {placeholder} ESCAPE '\\' COLLATE NOCASE)"
            ),
            Value::Text(format!("%{}%", escape_like(&text))),
        ),
        "is" => (
            format!(
                "EXISTS (SELECT 1 FROM json_each({column}) AS n WHERE typeof(n.value) = 'text' AND n.value = {placeholder} COLLATE NOCASE)"
            ),
            Value::Text(text),
        ),
        "is_not" => (
            format!(
                "NOT EXISTS (SELECT 1 FROM json_each({column}) AS n WHERE typeof(n.value) = 'text' AND n.value = {placeholder} COLLATE NOCASE)"
            ),
            Value::Text(text),
        ),
        _ => return Err(format!("invalid string filter operator: {op}")),
    };

    Ok((exists_op, vec![bound]))
}

fn compile_position_presence_rule(
    op: &str,
    value: &FilterValue,
    next_index: &mut usize,
) -> Result<(String, Vec<Value>), String> {
    let _ = next_index;
    let text = value_as_text(value)?;
    let Some(canonical) = POSITION_KEYS
        .iter()
        .copied()
        .find(|candidate| candidate.eq_ignore_ascii_case(&text))
    else {
        return Err(format!("unknown position key: {text}"));
    };
    let extract = json_extract_expr("positions_json", canonical);

    // Presence is exact key match only — never substring LIKE (MC must not match AMC).
    let clause = match op {
        "is" | "contains" => format!("{extract} IS NOT NULL"),
        "is_not" | "not_contains" => format!("{extract} IS NULL"),
        _ => return Err(format!("invalid string filter operator: {op}")),
    };
    Ok((clause, Vec::new()))
}

fn compile_string_rule(
    column: &str,
    case_insensitive: bool,
    op: &str,
    value: &FilterValue,
    next_index: &mut usize,
) -> Result<(String, Vec<Value>), String> {
    let text = value_as_text(value)?;
    let placeholder = next_placeholder(next_index);

    let clause = match op {
        "contains" => {
            if case_insensitive {
                format!("{column} LIKE {placeholder} ESCAPE '\\' COLLATE NOCASE")
            } else {
                format!("{column} LIKE {placeholder} ESCAPE '\\'")
            }
        }
        "not_contains" => {
            if case_insensitive {
                format!("({column} NOT LIKE {placeholder} ESCAPE '\\' COLLATE NOCASE AND {column} IS NOT NULL)")
            } else {
                format!("({column} NOT LIKE {placeholder} ESCAPE '\\' AND {column} IS NOT NULL)")
            }
        }
        "is" => {
            if case_insensitive {
                format!("({column} = {placeholder} COLLATE NOCASE AND {column} IS NOT NULL)")
            } else {
                format!("({column} = {placeholder} AND {column} IS NOT NULL)")
            }
        }
        "is_not" => {
            if case_insensitive {
                format!("({column} != {placeholder} COLLATE NOCASE AND {column} IS NOT NULL)")
            } else {
                format!("({column} != {placeholder} AND {column} IS NOT NULL)")
            }
        }
        _ => return Err(format!("invalid string filter operator: {op}")),
    };

    let bound = match op {
        "contains" | "not_contains" => Value::Text(format!("%{}%", escape_like(&text))),
        _ => Value::Text(text),
    };

    Ok((clause, vec![bound]))
}

fn compile_integer_rule(
    column: &str,
    nullable: bool,
    op: &str,
    value: &FilterValue,
    next_index: &mut usize,
) -> Result<(String, Vec<Value>), String> {
    let number = value_as_integer(value)?;
    let placeholder = next_placeholder(next_index);
    let compare = match op {
        "gt" => ">",
        "lt" => "<",
        "eq" => "=",
        "neq" => "!=",
        _ => return Err(format!("invalid integer filter operator: {op}")),
    };

    let clause = if nullable {
        format!("({column} {compare} {placeholder} AND {column} IS NOT NULL)")
    } else {
        format!("{column} {compare} {placeholder}")
    };

    Ok((clause, vec![Value::Integer(number)]))
}

fn compile_boolean_rule(
    column: &str,
    op: &str,
    value: &FilterValue,
    next_index: &mut usize,
) -> Result<(String, Vec<Value>), String> {
    let bool_value = value_as_bool(value)?;
    let placeholder = next_placeholder(next_index);
    let stored = if bool_value { 1 } else { 0 };

    let clause = match op {
        "is" => format!("({column} = {placeholder} AND {column} IS NOT NULL)"),
        "is_not" => format!("({column} != {placeholder} AND {column} IS NOT NULL)"),
        _ => return Err(format!("invalid boolean filter operator: {op}")),
    };

    Ok((clause, vec![Value::Integer(stored)]))
}

fn compile_enum_rule(
    column: &str,
    allowed: &[&str],
    op: &str,
    value: &FilterValue,
    next_index: &mut usize,
) -> Result<(String, Vec<Value>), String> {
    let text = value_as_text(value)?;
    if !allowed.iter().any(|candidate| *candidate == text) {
        return Err(format!("invalid value for {column}: {text}"));
    }

    let placeholder = next_placeholder(next_index);
    let clause = match op {
        "is" => format!("({column} = {placeholder} AND {column} IS NOT NULL)"),
        "is_not" => format!("({column} != {placeholder} AND {column} IS NOT NULL)"),
        _ => return Err(format!("invalid enum filter operator: {op}")),
    };

    Ok((clause, vec![Value::Text(text)]))
}

fn next_placeholder(next_index: &mut usize) -> String {
    let placeholder = format!("?{}", *next_index);
    *next_index += 1;
    placeholder
}

fn value_as_text(value: &FilterValue) -> Result<String, String> {
    match value {
        FilterValue::Text(text) => Ok(text.clone()),
        _ => Err("filter value must be a string".to_string()),
    }
}

fn value_as_integer(value: &FilterValue) -> Result<i64, String> {
    match value {
        FilterValue::Integer(number) => Ok(*number),
        FilterValue::Text(text) => text
            .parse::<i64>()
            .map_err(|_| format!("filter value must be an integer: {text}")),
        _ => Err("filter value must be an integer".to_string()),
    }
}

fn value_as_bool(value: &FilterValue) -> Result<bool, String> {
    match value {
        FilterValue::Bool(value) => Ok(*value),
        FilterValue::Text(text) => match text.to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(format!("filter value must be a boolean: {text}")),
        },
        FilterValue::Integer(number) => match *number {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(format!("filter value must be a boolean: {number}")),
        },
    }
}

fn escape_like(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' | '%' | '_' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(field: &str, op: &str, value: FilterValue) -> FilterRule {
        FilterRule {
            field: field.to_string(),
            op: op.to_string(),
            value,
        }
    }

    #[test]
    fn rejects_more_than_max_filter_rules() {
        let rules = (0..=MAX_FILTER_RULES)
            .map(|i| rule("ca", "gt", FilterValue::Integer(i as i64)))
            .collect::<Vec<_>>();

        let error = parse_filter_ast(rules, None).expect_err("cap");
        assert!(error.contains("filter rule count exceeds maximum"));
    }

    #[test]
    fn rejects_unknown_field() {
        let ast = parse_filter_ast(
            vec![rule(
                "ca; DROP TABLE players",
                "eq",
                FilterValue::Integer(100),
            )],
            None,
        )
        .expect("parse ast");

        let error = compile_filters(&ast, 2).expect_err("unknown field");
        assert!(error.contains("unknown filter field"));
    }

    #[test]
    fn rejects_invalid_operator_for_string_field() {
        let ast = parse_filter_ast(
            vec![rule("name", "gt", FilterValue::Text("10".to_string()))],
            None,
        )
        .expect("parse ast");

        let error = compile_filters(&ast, 2).expect_err("invalid op");
        assert!(error.contains("invalid string filter operator"));
    }

    #[test]
    fn compiles_string_contains_with_parameter() {
        let ast = parse_filter_ast(
            vec![rule(
                "name",
                "contains",
                FilterValue::Text("son".to_string()),
            )],
            None,
        )
        .expect("parse ast");

        let compiled = compile_filters(&ast, 2).expect("compile");
        assert!(compiled.sql.contains("name LIKE ?2 ESCAPE"));
        assert_eq!(compiled.params, vec![Value::Text("%son%".to_string())]);
    }

    #[test]
    fn compiles_integer_gt_with_null_guard() {
        let ast = parse_filter_ast(vec![rule("age", "gt", FilterValue::Integer(25))], None)
            .expect("parse ast");

        let compiled = compile_filters(&ast, 2).expect("compile");
        assert!(compiled.sql.contains("age > ?2 AND age IS NOT NULL"));
        assert_eq!(compiled.params, vec![Value::Integer(25)]);
    }

    #[test]
    fn compiles_boolean_is_not_with_null_guard() {
        let ast = parse_filter_ast(
            vec![rule("transfer_listed", "is_not", FilterValue::Bool(true))],
            None,
        )
        .expect("parse ast");

        let compiled = compile_filters(&ast, 2).expect("compile");
        assert!(compiled
            .sql
            .contains("transfer_listed != ?2 AND transfer_listed IS NOT NULL"));
    }

    #[test]
    fn compiles_boolean_is_yes() {
        let ast = parse_filter_ast(
            vec![rule("transfer_listed", "is", FilterValue::Bool(true))],
            None,
        )
        .expect("parse ast");

        let compiled = compile_filters(&ast, 2).expect("compile");
        assert!(compiled
            .sql
            .contains("transfer_listed = ?2 AND transfer_listed IS NOT NULL"));
        assert_eq!(compiled.params, vec![Value::Integer(1)]);
    }

    #[test]
    fn compiles_enum_is_with_allowed_value() {
        let ast = parse_filter_ast(
            vec![rule(
                "preferred_foot",
                "is",
                FilterValue::Text("right".to_string()),
            )],
            None,
        )
        .expect("parse ast");

        let compiled = compile_filters(&ast, 2).expect("compile");
        assert!(compiled.sql.contains("preferred_foot = ?2"));
        assert_eq!(compiled.params, vec![Value::Text("right".to_string())]);
    }

    #[test]
    fn accepts_dump_team_level_values() {
        for value in ["senior", "reserve", "youth"] {
            let ast = parse_filter_ast(
                vec![rule(
                    "team_level",
                    "is",
                    FilterValue::Text(value.to_string()),
                )],
                None,
            )
            .expect("parse ast");
            compile_filters(&ast, 2)
                .unwrap_or_else(|error| panic!("expected team_level {value} to compile: {error}"));
        }
    }

    #[test]
    fn rejects_invalid_enum_value() {
        let ast = parse_filter_ast(
            vec![rule(
                "team_level",
                "is",
                FilterValue::Text("veteran".to_string()),
            )],
            None,
        )
        .expect("parse ast");

        let error = compile_filters(&ast, 2).expect_err("invalid enum");
        assert!(error.contains("invalid value for team_level"));
    }

    #[test]
    fn combines_rules_with_and() {
        let ast = parse_filter_ast(
            vec![
                rule("ca", "gt", FilterValue::Integer(100)),
                rule("name", "contains", FilterValue::Text("a".to_string())),
            ],
            Some("and"),
        )
        .expect("parse ast");

        let compiled = compile_filters(&ast, 2).expect("compile");
        assert!(compiled.sql.contains(" AND "));
        assert_eq!(compiled.params.len(), 2);
    }

    #[test]
    fn combines_rules_with_or() {
        let ast = parse_filter_ast(
            vec![
                rule("ca", "eq", FilterValue::Integer(100)),
                rule("ca", "eq", FilterValue::Integer(200)),
            ],
            Some("or"),
        )
        .expect("parse ast");

        let compiled = compile_filters(&ast, 2).expect("compile");
        assert!(compiled.sql.contains(" OR "));
    }

    #[test]
    fn escapes_like_wildcards_in_contains() {
        let ast = parse_filter_ast(
            vec![rule(
                "name",
                "contains",
                FilterValue::Text("100%_".to_string()),
            )],
            None,
        )
        .expect("parse ast");

        let compiled = compile_filters(&ast, 2).expect("compile");
        assert_eq!(
            compiled.params,
            vec![Value::Text("%100\\%\\_%".to_string())]
        );
    }

    #[test]
    fn compiles_attribute_integer_via_json_extract() {
        let ast = parse_filter_ast(
            vec![rule("attr.Acceleration", "gt", FilterValue::Integer(12))],
            None,
        )
        .expect("parse ast");

        let compiled = compile_filters(&ast, 2).expect("compile");
        assert!(
            compiled
                .sql
                .contains("json_extract(attributes_json, '$.Acceleration')"),
            "expected attributes_json extract, got {}",
            compiled.sql
        );
        assert!(compiled.sql.contains("IS NOT NULL"));
        assert_eq!(compiled.params, vec![Value::Integer(12)]);
    }

    #[test]
    fn compiles_hidden_and_personality_attribute_fields() {
        let hidden = compile_filters(
            &parse_filter_ast(
                vec![rule("hidden.Consistency", "eq", FilterValue::Integer(10))],
                None,
            )
            .expect("parse"),
            2,
        )
        .expect("compile hidden");
        assert!(hidden
            .sql
            .contains("json_extract(hidden_attributes_json, '$.Consistency')"));

        let personality = compile_filters(
            &parse_filter_ast(
                vec![rule("personality.Ambition", "lt", FilterValue::Integer(15))],
                None,
            )
            .expect("parse"),
            2,
        )
        .expect("compile personality");
        assert!(personality
            .sql
            .contains("json_extract(personality_json, '$.Ambition')"));
    }

    #[test]
    fn rejects_attribute_field_with_unsafe_key() {
        let ast = parse_filter_ast(
            vec![rule(
                "attr.Acceleration'; DROP TABLE players--",
                "gt",
                FilterValue::Integer(1),
            )],
            None,
        )
        .expect("parse ast");

        let error = compile_filters(&ast, 2).expect_err("unsafe key");
        assert!(error.contains("unknown filter field") || error.contains("invalid"));
    }

    #[test]
    fn compiles_nationality_contains_via_json_each() {
        let ast = parse_filter_ast(
            vec![rule(
                "nationality",
                "contains",
                FilterValue::Text("ENG".to_string()),
            )],
            None,
        )
        .expect("parse ast");

        let compiled = compile_filters(&ast, 2).expect("compile");
        assert!(
            compiled.sql.contains("json_each(nationalities_json)"),
            "expected json_each on nationalities, got {}",
            compiled.sql
        );
        assert!(compiled.sql.contains("EXISTS"));
        assert_eq!(compiled.params, vec![Value::Text("%ENG%".to_string())]);
    }

    #[test]
    fn compiles_position_presence_and_suitability() {
        let presence = compile_filters(
            &parse_filter_ast(
                vec![rule("position", "is", FilterValue::Text("MC".to_string()))],
                None,
            )
            .expect("parse"),
            2,
        )
        .expect("compile presence");
        assert!(
            presence
                .sql
                .contains("json_extract(positions_json, '$.MC')")
                || presence.sql.contains("json_each(positions_json)"),
            "expected positions presence SQL, got {}",
            presence.sql
        );

        let suitability = compile_filters(
            &parse_filter_ast(vec![rule("pos.MC", "gt", FilterValue::Integer(15))], None)
                .expect("parse"),
            2,
        )
        .expect("compile suitability");
        assert!(
            suitability
                .sql
                .contains("json_extract(positions_json, '$.MC')"),
            "expected position suitability extract, got {}",
            suitability.sql
        );
        assert!(suitability.sql.contains("IS NOT NULL"));
        assert_eq!(suitability.params, vec![Value::Integer(15)]);
    }

    #[test]
    fn rejects_unknown_position_key() {
        let ast = parse_filter_ast(vec![rule("pos.XYZ", "gt", FilterValue::Integer(10))], None)
            .expect("parse ast");

        let error = compile_filters(&ast, 2).expect_err("unknown pos");
        assert!(error.contains("unknown") || error.contains("invalid"));
    }
}
