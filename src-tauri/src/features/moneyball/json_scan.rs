//! A scanner for the flat JSON objects stored in Moneyball rows and player position maps: keys
//! map to numbers or null. It finds values without building a JSON tree, which matters when a
//! request reads tens of thousands of stored objects.

/// Calls `found` with each key and raw value text of one flat JSON object whose values are
/// numbers or null.
pub(crate) fn scan_json_object(
    json: &str,
    mut found: impl FnMut(&str, &str) -> Result<(), String>,
) -> Result<(), String> {
    let malformed = || "malformed Moneyball JSON object".to_string();
    let bytes = json.as_bytes();
    let mut at = 0;
    let skip_whitespace = |at: &mut usize| {
        while bytes.get(*at).is_some_and(u8::is_ascii_whitespace) {
            *at += 1;
        }
    };
    skip_whitespace(&mut at);
    if bytes.get(at) != Some(&b'{') {
        return Err(malformed());
    }
    at += 1;
    skip_whitespace(&mut at);
    if bytes.get(at) == Some(&b'}') {
        return Ok(());
    }
    loop {
        skip_whitespace(&mut at);
        if bytes.get(at) != Some(&b'"') {
            return Err(malformed());
        }
        let key_start = at + 1;
        at = key_start;
        loop {
            match bytes.get(at) {
                Some(b'"') => break,
                Some(b'\\') => at += 2,
                Some(_) => at += 1,
                None => return Err(malformed()),
            }
        }
        let key = &json[key_start..at];
        at += 1;
        skip_whitespace(&mut at);
        if bytes.get(at) != Some(&b':') {
            return Err(malformed());
        }
        at += 1;
        skip_whitespace(&mut at);
        let value_start = at;
        while bytes
            .get(at)
            .is_some_and(|byte| !matches!(byte, b',' | b'}') && !byte.is_ascii_whitespace())
        {
            at += 1;
        }
        found(key, &json[value_start..at])?;
        skip_whitespace(&mut at);
        match bytes.get(at) {
            Some(b',') => at += 1,
            Some(b'}') => return Ok(()),
            _ => return Err(malformed()),
        }
    }
}

/// A stored value: `null`, or a number. Rust parses numbers to the nearest value, as SQLite's
/// JSON functions do for the at most 17 significant digits that stored values carry.
pub(crate) fn parse_json_value<T>(
    text: &str,
    parse: fn(&str) -> Option<T>,
) -> Result<Option<T>, String> {
    if text == "null" {
        return Ok(None);
    }
    parse(text)
        .map(Some)
        .ok_or_else(|| format!("malformed Moneyball value: {text}"))
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{parse_json_value, scan_json_object};

    #[test]
    fn scanned_values_equal_sqlite_json_extract() {
        let json = r#" { "goals" : 0.30000000000000004 , "own\"goals":7,"xg":-1.5e-3,
            "minutes":null, "saves" :12345678901234567 } "#;
        let keys = ["goals", "xg", "minutes", "saves", "assists"];
        let mut scanned = vec![None; keys.len()];
        scan_json_object(json, |key, text| {
            if let Some(index) = keys.iter().position(|wanted| *wanted == key) {
                scanned[index] = parse_json_value(text, |text| text.parse::<f64>().ok())?;
            }
            Ok(())
        })
        .expect("scan object");

        let conn = Connection::open_in_memory().expect("open database");
        let extracted = keys
            .iter()
            .map(|key| {
                conn.query_row(
                    &format!("SELECT json_extract(?1, '$.\"{key}\"')"),
                    [json],
                    |row| row.get::<_, Option<f64>>(0),
                )
                .expect("extract value")
            })
            .collect::<Vec<_>>();
        assert_eq!(scanned, extracted);
        assert_eq!(scanned[0], Some(0.30000000000000004));
        assert_eq!(scanned[2], None);
    }
}
