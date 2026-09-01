use super::metrics::MetricField;
use super::scoring::{staff_role_column, STAFF_METRICS_ALIAS};
use rusqlite::types::Value;

pub const MAX_FILTER_RULES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombineMode {
    And,
    Or,
}
impl CombineMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "and" => Ok(Self::And),
            "or" => Ok(Self::Or),
            _ => Err(format!("unknown staff filter combine mode: {value}")),
        }
    }
    fn sql(self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterValue {
    Text(String),
    Integer(i64),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterRule {
    pub field: String,
    pub op: String,
    pub value: FilterValue,
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

pub fn filter_value_from_json(value: serde_json::Value) -> Result<FilterValue, String> {
    match value {
        serde_json::Value::String(value) => Ok(FilterValue::Text(value)),
        serde_json::Value::Number(value) => value
            .as_i64()
            .map(FilterValue::Integer)
            .ok_or_else(|| "staff filter number out of range".to_string()),
        _ => Err("staff filter value must be a string or integer".to_string()),
    }
}

pub fn parse_filter_ast(
    rules: Vec<FilterRule>,
    combine: Option<&str>,
) -> Result<FilterAst, String> {
    if rules.len() > MAX_FILTER_RULES {
        return Err(format!(
            "staff filter rule count exceeds maximum of {MAX_FILTER_RULES}"
        ));
    }
    Ok(FilterAst {
        combine: combine
            .map(CombineMode::parse)
            .transpose()?
            .unwrap_or(CombineMode::And),
        rules,
    })
}

enum FieldKind {
    Text(&'static str),
    Integer(&'static str),
    Nationality,
    Metric(MetricField),
}
fn resolve_field(field: &str) -> Result<FieldKind, String> {
    match field {
        "name" => Ok(FieldKind::Text("staff.name")),
        "club" => Ok(FieldKind::Text("staff.club")),
        "division" => Ok(FieldKind::Text("staff.division")),
        "gender" => Ok(FieldKind::Text("staff.gender")),
        "age" => Ok(FieldKind::Integer("staff.age")),
        "birth_year" => Ok(FieldKind::Integer("staff.birth_year")),
        "birth_day_of_year" => Ok(FieldKind::Integer("staff.birth_day_of_year")),
        "nation_uid" => Ok(FieldKind::Integer("staff.nation_uid")),
        "ca" => Ok(FieldKind::Integer("staff.ca")),
        "pa" => Ok(FieldKind::Integer("staff.pa")),
        "job_id" => Ok(FieldKind::Integer("staff.job_id")),
        "wage" => Ok(FieldKind::Integer("staff.weekly_wage_gbp")),
        "contract_year" => Ok(FieldKind::Integer("staff.contract_expiry_year")),
        "contract_day" => Ok(FieldKind::Integer("staff.contract_expiry_day_of_year")),
        "nationality" => Ok(FieldKind::Nationality),
        other => Ok(FieldKind::Metric(MetricField::parse(other)?)),
    }
}

pub fn compile_filters(ast: &FilterAst, start_index: usize) -> Result<CompiledFilter, String> {
    let mut clauses = Vec::with_capacity(ast.rules.len());
    let mut params = Vec::new();
    let mut index = start_index;
    for rule in &ast.rules {
        let (clause, values) = compile_rule(resolve_field(&rule.field)?, rule, &mut index)?;
        clauses.push(clause);
        params.extend(values);
    }
    Ok(CompiledFilter {
        sql: if clauses.is_empty() {
            String::new()
        } else {
            format!("({})", clauses.join(&format!(" {} ", ast.combine.sql())))
        },
        params,
    })
}

fn compile_rule(
    field: FieldKind,
    rule: &FilterRule,
    index: &mut usize,
) -> Result<(String, Vec<Value>), String> {
    match field {
        FieldKind::Text(column) => compile_text(column, rule, index),
        FieldKind::Integer(column) => compile_integer(column.to_string(), rule, index, Vec::new()),
        FieldKind::Nationality => {
            let text = text_value(&rule.value)?;
            let p = placeholder(index);
            let (exists, compare, bound) = match rule.op.as_str() {
                "contains" => ("EXISTS", "LIKE", format!("%{}%", escape_like(&text))),
                "not_contains" => ("NOT EXISTS", "LIKE", format!("%{}%", escape_like(&text))),
                "is" => ("EXISTS", "=", text),
                "is_not" => ("NOT EXISTS", "=", text),
                _ => return Err(format!("invalid staff nationality operator: {}", rule.op)),
            };
            let escape = if compare == "LIKE" {
                " ESCAPE '\\'"
            } else {
                ""
            };
            Ok((format!("{exists} (SELECT 1 FROM json_each(staff.nationalities_json) n WHERE typeof(n.value) = 'text' AND n.value {compare} {p}{escape} COLLATE NOCASE)"), vec![Value::Text(bound)]))
        }
        FieldKind::Metric(MetricField::Attribute(key)) => compile_integer(
            format!("json_extract(staff.staff_attributes_json, '$.{key}')"),
            rule,
            index,
            Vec::new(),
        ),
        FieldKind::Metric(MetricField::Role(role)) => {
            let column = staff_role_column(role.role_id).map_err(|error| error.to_string())?;
            let expr = format!("{STAFF_METRICS_ALIAS}.{column}");
            compile_integer(expr, rule, index, Vec::new())
        }
    }
}

fn compile_integer(
    expr: String,
    rule: &FilterRule,
    index: &mut usize,
    mut params: Vec<Value>,
) -> Result<(String, Vec<Value>), String> {
    let number = integer_value(&rule.value)?;
    let compare = match rule.op.as_str() {
        "gt" => ">",
        "lt" => "<",
        "eq" => "=",
        "neq" => "!=",
        _ => return Err(format!("invalid staff integer operator: {}", rule.op)),
    };
    let p = placeholder(index);
    params.push(Value::Integer(number));
    Ok((
        format!("({expr} {compare} {p} AND {expr} IS NOT NULL)"),
        params,
    ))
}
fn compile_text(
    column: &str,
    rule: &FilterRule,
    index: &mut usize,
) -> Result<(String, Vec<Value>), String> {
    let text = text_value(&rule.value)?;
    let p = placeholder(index);
    let (predicate, value) = match rule.op.as_str() {
        "contains" => (
            format!("{column} LIKE {p} ESCAPE '\\' COLLATE NOCASE"),
            format!("%{}%", escape_like(&text)),
        ),
        "not_contains" => (
            format!("{column} NOT LIKE {p} ESCAPE '\\' COLLATE NOCASE"),
            format!("%{}%", escape_like(&text)),
        ),
        "is" => (format!("{column} = {p} COLLATE NOCASE"), text),
        "is_not" => (format!("{column} != {p} COLLATE NOCASE"), text),
        _ => return Err(format!("invalid staff text operator: {}", rule.op)),
    };
    Ok((
        format!("({predicate} AND {column} IS NOT NULL)"),
        vec![Value::Text(value)],
    ))
}
fn placeholder(index: &mut usize) -> String {
    let value = format!("?{index}");
    *index += 1;
    value
}
fn integer_value(value: &FilterValue) -> Result<i64, String> {
    if let FilterValue::Integer(value) = value {
        Ok(*value)
    } else {
        Err("staff integer filter requires integer".to_string())
    }
}
fn text_value(value: &FilterValue) -> Result<String, String> {
    if let FilterValue::Text(value) = value {
        Ok(value.clone())
    } else {
        Err("staff text filter requires text".to_string())
    }
}
pub fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rule(field: &str, op: &str, value: FilterValue) -> FilterRule {
        FilterRule {
            field: field.into(),
            op: op.into(),
            value,
        }
    }
    #[test]
    fn compiles_bound_attribute_and_score_filters() {
        let ast = parse_filter_ast(
            vec![
                rule("attr.Authority", "gt", FilterValue::Integer(15)),
                rule("role.coach_fitness", "gt", FilterValue::Integer(70)),
            ],
            Some("and"),
        )
        .unwrap();
        let compiled = compile_filters(&ast, 2).unwrap();
        assert!(compiled.sql.contains("json_extract"));
        assert!(compiled.sql.contains("staff_metrics.coach_fitness"));
        assert_eq!(compiled.params, [Value::Integer(15), Value::Integer(70)]);
    }
    #[test]
    fn enforces_limit_and_rejects_unknown_or_injection_shaped_fields() {
        let rules = (0..33)
            .map(|_| rule("ca", "gt", FilterValue::Integer(1)))
            .collect();
        assert!(parse_filter_ast(rules, None).is_err());
        let ast = parse_filter_ast(
            vec![rule(
                "attr.Authority') OR 1=1 --",
                "gt",
                FilterValue::Integer(1),
            )],
            None,
        )
        .unwrap();
        assert!(compile_filters(&ast, 2).is_err());
        assert!(parse_filter_ast(Vec::new(), Some("xor")).is_err());
    }
    #[test]
    fn nulls_are_excluded_from_negative_integer_filters() {
        let ast =
            parse_filter_ast(vec![rule("age", "neq", FilterValue::Integer(40))], None).unwrap();
        assert!(compile_filters(&ast, 2)
            .unwrap()
            .sql
            .contains("IS NOT NULL"));
    }
}
