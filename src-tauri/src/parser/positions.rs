use crate::parser::types::{Position, Role, Side};

/// Parse an FM position string into a list of Position structs.
///
/// Examples:
///   "GK"              → [Position { role: GK, sides: [] }]
///   "D (LC)"          → [Position { role: D, sides: [L, C] }]
///   "AM (C), ST (C)"  → [Position { role: AM, sides: [C] }, Position { role: ST, sides: [C] }]
///   "D/WB (L)"        → [Position { role: D, sides: [L] }, Position { role: WB, sides: [L] }]
///   "M/AM (C)"        → [Position { role: M, sides: [C] }, Position { role: AM, sides: [C] }]
///   "AM (RLC)"        → [Position { role: AM, sides: [R, L, C] }]
///
/// Returns Err(reason) if the string cannot be parsed at all.
pub fn parse_positions(input: &str) -> Result<Vec<Position>, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Empty position string".to_string());
    }

    let mut positions = Vec::new();

    // Split on ", " to get individual position entries
    for entry in input.split(", ") {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let parsed = parse_single_position(entry)?;
        positions.extend(parsed);
    }

    if positions.is_empty() {
        return Err(format!("No valid positions found in '{}'", input));
    }

    Ok(positions)
}

/// Parse a single position entry like "D (LC)" or "D/WB (L)".
fn parse_single_position(entry: &str) -> Result<Vec<Position>, String> {
    // Extract sides from parens if present: "D (LC)" → roles="D", sides="LC"
    let (roles_part, sides) = if let Some(paren_start) = entry.find('(') {
        let roles_part = entry[..paren_start].trim();
        let paren_content = &entry[paren_start + 1..];
        let sides_str = paren_content.trim_end_matches(')').trim();
        let sides = parse_sides(sides_str)?;
        (roles_part, sides)
    } else {
        (entry.trim(), Vec::new())
    };

    // Split on "/" for combined roles: "D/WB" → ["D", "WB"]
    let role_strs: Vec<&str> = roles_part.split('/').map(|s| s.trim()).collect();

    let mut result = Vec::new();
    for role_str in role_strs {
        let role = parse_role(role_str).ok_or_else(|| {
            format!("Unrecognized role '{}' in position string '{}'", role_str, entry)
        })?;
        result.push(Position {
            role,
            sides: sides.clone(),
        });
    }

    Ok(result)
}

fn parse_role(s: &str) -> Option<Role> {
    match s {
        "GK" => Some(Role::GK),
        "D" => Some(Role::D),
        "WB" => Some(Role::WB),
        "DM" => Some(Role::DM),
        "M" => Some(Role::M),
        "AM" => Some(Role::AM),
        "ST" => Some(Role::ST),
        _ => None,
    }
}

fn parse_sides(s: &str) -> Result<Vec<Side>, String> {
    let mut sides = Vec::new();
    for ch in s.chars() {
        match ch {
            'L' => sides.push(Side::L),
            'C' => sides.push(Side::C),
            'R' => sides.push(Side::R),
            _ => return Err(format!("Unrecognized side character '{}' in '{}'", ch, s)),
        }
    }
    Ok(sides)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goalkeeper() {
        let result = parse_positions("GK").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, Role::GK);
        assert!(result[0].sides.is_empty());
    }

    #[test]
    fn defender_with_sides() {
        let result = parse_positions("D (LC)").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, Role::D);
        assert_eq!(result[0].sides, vec![Side::L, Side::C]);
    }

    #[test]
    fn multiple_positions() {
        let result = parse_positions("AM (C), ST (C)").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].role, Role::AM);
        assert_eq!(result[0].sides, vec![Side::C]);
        assert_eq!(result[1].role, Role::ST);
        assert_eq!(result[1].sides, vec![Side::C]);
    }

    #[test]
    fn combined_role() {
        let result = parse_positions("D/WB (L)").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Position { role: Role::D, sides: vec![Side::L] });
        assert_eq!(result[1], Position { role: Role::WB, sides: vec![Side::L] });
    }

    #[test]
    fn midfielder_combined() {
        let result = parse_positions("M/AM (C)").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].role, Role::M);
        assert_eq!(result[1].role, Role::AM);
    }

    #[test]
    fn triple_side() {
        let result = parse_positions("AM (RLC)").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].sides, vec![Side::R, Side::L, Side::C]);
    }

    #[test]
    fn plain_defender_no_sides() {
        let result = parse_positions("D").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, Role::D);
        assert!(result[0].sides.is_empty());
    }

    #[test]
    fn empty_string_is_error() {
        assert!(parse_positions("").is_err());
        assert!(parse_positions("   ").is_err());
    }

    #[test]
    fn unrecognized_role_is_error() {
        assert!(parse_positions("XYZ").is_err());
    }

    #[test]
    fn goalkeeper_with_side() {
        // GK typically has no side, but FM might produce "GK (C)"
        let result = parse_positions("GK (C)").unwrap();
        assert_eq!(result[0].role, Role::GK);
        assert_eq!(result[0].sides, vec![Side::C]);
    }
}
