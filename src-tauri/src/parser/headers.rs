use crate::parser::types::ColumnStatus;

/// Result of header parsing.
#[derive(Debug)]
pub struct HeaderMap {
    /// Map from lowercase header name to column index.
    pub map: std::collections::HashMap<String, usize>,
    /// Original header names in order.
    pub original_names: Vec<String>,
    /// Detected delimiter character.
    pub delimiter: u8,
    /// Columns found (with original case).
    pub columns_found: Vec<ColumnStatus>,
    /// Required columns that are missing.
    pub missing_required: Vec<String>,
}

/// Required column names (lowercase for matching).
const REQUIRED_COLUMNS: &[&str] = &["unique id", "player", "minutes", "position"];

/// Strip UTF-8 BOM from the start of a string.
pub fn strip_bom(input: &str) -> &str {
    input.strip_prefix('\u{feff}').unwrap_or(input)
}

/// Detect the most likely delimiter from the header row.
/// Counts `;`, `,`, `\t` occurrences and returns the most frequent.
/// Defaults to `;` on tie.
pub fn detect_delimiter(header: &str) -> u8 {
    let semicolons = header.chars().filter(|&c| c == ';').count();
    let commas = header.chars().filter(|&c| c == ',').count();
    let tabs = header.chars().filter(|&c| c == '\t').count();

    if tabs > semicolons && tabs > commas {
        b'\t'
    } else if commas > semicolons {
        b','
    } else {
        b';'
    }
}

/// Parse the header row into a HeaderMap.
/// Returns Err if fewer than 3 columns detected (not a valid CSV).
pub fn parse_headers(header_raw: &str) -> Result<HeaderMap, String> {
    let header = strip_bom(header_raw);
    let delimiter = detect_delimiter(header);

    let original_names: Vec<String> = header
        .split(delimiter as char)
        .map(|s| s.trim().to_string())
        .collect();

    // Reject files with fewer than 3 columns — not a valid FM export
    if original_names.len() < 3 {
        return Err(
            "File is not a valid Football Manager export.".to_string()
        );
    }

    // Build lowercase→index map
    let mut map = std::collections::HashMap::new();
    let mut columns_found = Vec::new();
    for (idx, name) in original_names.iter().enumerate() {
        let lower = name.to_lowercase();
        map.insert(lower, idx);
        columns_found.push(ColumnStatus {
            name: name.clone(),
            index: idx,
        });
    }

    // Check required columns
    let missing_required: Vec<String> = REQUIRED_COLUMNS
        .iter()
        .filter(|&&col| !map.contains_key(col))
        .map(|&col| {
            // Return a nice display name
            match col {
                "unique id" => "Unique ID",
                "player" => "Player",
                "minutes" => "Minutes",
                "position" => "Position",
                _ => col,
            }
            .to_string()
        })
        .collect();

    Ok(HeaderMap {
        map,
        original_names,
        delimiter,
        columns_found,
        missing_required,
    })
}

/// Check if a column exists in the header map (case-insensitive).
pub fn has_column(header_map: &HeaderMap, name: &str) -> bool {
    header_map.map.contains_key(&name.to_lowercase())
}

/// Get the column index for a given name (case-insensitive).
pub fn get_column_index(header_map: &HeaderMap, name: &str) -> Option<usize> {
    header_map.map.get(&name.to_lowercase()).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_header() -> &'static str {
        "Unique ID;Player;Nation;2nd Nat;Club;Position;Age;Height;Left Foot;Right Foot;CA;PA;Transfer Value;Wage;Expires;Appearances;Minutes"
    }

    #[test]
    fn detects_semicolon_delimiter() {
        assert_eq!(detect_delimiter(sample_header()), b';');
    }

    #[test]
    fn detects_comma_delimiter() {
        let header = "Unique ID,Player,Nation,Position,Minutes";
        assert_eq!(detect_delimiter(header), b',');
    }

    #[test]
    fn detects_tab_delimiter() {
        let header = "Unique ID\tPlayer\tNation\tPosition\tMinutes";
        assert_eq!(detect_delimiter(header), b'\t');
    }

    #[test]
    fn defaults_to_semicolon_on_tie() {
        // Single comma, single semicolon → tie, default semicolon
        let header = "A;B,C";
        assert_eq!(detect_delimiter(header), b';');
    }

    #[test]
    fn strips_bom() {
        let with_bom = "\u{feff}Unique ID;Player;Position";
        assert_eq!(strip_bom(with_bom), "Unique ID;Player;Position");
    }

    #[test]
    fn strips_bom_only_at_start() {
        let no_bom = "Unique ID;Player;Position";
        assert_eq!(strip_bom(no_bom), no_bom);
    }

    #[test]
    fn parses_sample_header() {
        let result = parse_headers(sample_header()).unwrap();
        assert_eq!(result.delimiter, b';');
        assert!(result.missing_required.is_empty());
        assert!(result.map.contains_key("unique id"));
        assert!(result.map.contains_key("player"));
        assert!(result.map.contains_key("position"));
        assert!(result.map.contains_key("minutes"));
        assert_eq!(result.columns_found.len(), 17);
    }

    #[test]
    fn case_insensitive_lookup() {
        let result = parse_headers(sample_header()).unwrap();
        assert!(has_column(&result, "unique id"));
        assert!(has_column(&result, "UNIQUE ID"));
        assert!(has_column(&result, "Unique ID"));
        assert_eq!(get_column_index(&result, "player"), Some(1));
    }

    #[test]
    fn rejects_fewer_than_3_columns() {
        let result = parse_headers("A;B");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "File is not a valid Football Manager export.");
    }

    #[test]
    fn detects_missing_required_columns() {
        let header = "Unique ID;Player;Age"; // Missing Minutes, Position
        let result = parse_headers(header).unwrap();
        assert_eq!(result.missing_required.len(), 2);
        assert!(result.missing_required.contains(&"Minutes".to_string()));
        assert!(result.missing_required.contains(&"Position".to_string()));
    }

    #[test]
    fn handles_bom_with_headers() {
        let header = "\u{feff}Unique ID;Player;Position;Minutes";
        let result = parse_headers(header).unwrap();
        assert!(result.missing_required.is_empty());
        assert!(has_column(&result, "Unique ID"));
    }


}
