# Task 02 — Country Code Lookup Table

## Overview

Create a static lookup table mapping 3-letter FIFA country codes to full country names using `phf` (compile-time perfect hash functions). This is used when parsing the `Nation` column from FM CSV exports.

## Files to Create/Modify

- Create: `src-tauri/src/parser/countries.rs`

## Steps

- [ ] **Step 1: Write failing tests**

In `src-tauri/src/parser/countries.rs`:

```rust
use phf::phf_map;

/// Static map of 3-letter FIFA country codes to full names.
/// Covers all ~200 FIFA member associations plus common FM codes.
/// Source: https://en.wikipedia.org/wiki/List_of_FIFA_country_codes
static COUNTRY_CODES: phf::Map<&'static str, &'static str> = phf_map! {
    "AFG" => "Afghanistan",
    "ALB" => "Albania",
    "ALG" => "Algeria",
    "AND" => "Andorra",
    "ANG" => "Angola",
    "ANT" => "Antigua and Barbuda",
    "ARG" => "Argentina",
    "ARM" => "Armenia",
    "ARU" => "Aruba",
    "ASA" => "American Samoa",
    "AUS" => "Australia",
    "AUT" => "Austria",
    "AZE" => "Azerbaijan",
    "BAH" => "Bahamas",
    "BAN" => "Bangladesh",
    "BAR" => "Barbados",
    "BDI" => "Burundi",
    "BEL" => "Belgium",
    "BEN" => "Benin",
    "BER" => "Bermuda",
    "BFA" => "Burkina Faso",
    "BHR" => "Bahrain",
    "BHU" => "Bhutan",
    "BIH" => "Bosnia and Herzegovina",
    "BLR" => "Belarus",
    "BLZ" => "Belize",
    "BOL" => "Bolivia",
    "BOT" => "Botswana",
    "BRA" => "Brazil",
    "BRB" => "Barbados",
    "BRU" => "Brunei",
    "BUL" => "Bulgaria",
    "CAM" => "Cambodia",
    "CAN" => "Canada",
    "CAY" => "Cayman Islands",
    "CGO" => "Republic of the Congo",
    "CHA" => "Chad",
    "CHI" => "Chile",
    "CHN" => "China PR",
    "CIV" => "Ivory Coast",
    "CMR" => "Cameroon",
    "COD" => "DR Congo",
    "COK" => "Cook Islands",
    "COL" => "Colombia",
    "COM" => "Comoros",
    "CPV" => "Cape Verde",
    "CRC" => "Costa Rica",
    "CRO" => "Croatia",
    "CUB" => "Cuba",
    "CYP" => "Cyprus",
    "CZE" => "Czech Republic",
    "DEN" => "Denmark",
    "DJI" => "Djibouti",
    "DMA" => "Dominica",
    "DOM" => "Dominican Republic",
    "ECU" => "Ecuador",
    "EGY" => "Egypt",
    "EQG" => "Equatorial Guinea",
    "ERI" => "Eritrea",
    "ESA" => "El Salvador",
    "ESP" => "Spain",
    "EST" => "Estonia",
    "ETH" => "Ethiopia",
    "FIJ" => "Fiji",
    "FIN" => "Finland",
    "FRA" => "France",
    "FSM" => "Micronesia",
    "GAB" => "Gabon",
    "GAM" => "Gambia",
    "GBR" => "United Kingdom",
    "GEO" => "Georgia",
    "GER" => "Germany",
    "GHA" => "Ghana",
    "GRE" => "Greece",
    "GRN" => "Grenada",
    "GUA" => "Guatemala",
    "GUI" => "Guinea",
    "GUM" => "Guam",
    "GUY" => "Guyana",
    "HAI" => "Haiti",
    "HON" => "Honduras",
    "HKG" => "Hong Kong",
    "HUN" => "Hungary",
    "IDN" => "Indonesia",
    "IND" => "India",
    "IRL" => "Republic of Ireland",
    "IRN" => "Iran",
    "IRQ" => "Iraq",
    "ISL" => "Iceland",
    "ISR" => "Israel",
    "ITA" => "Italy",
    "JAM" => "Jamaica",
    "JOR" => "Jordan",
    "JPN" => "Japan",
    "KAZ" => "Kazakhstan",
    "KEN" => "Kenya",
    "KGZ" => "Kyrgyzstan",
    "KOR" => "South Korea",
    "KSA" => "Saudi Arabia",
    "KUW" => "Kuwait",
    "KVX" => "Kosovo",
    "LAO" => "Laos",
    "LAT" => "Latvia",
    "LBR" => "Liberia",
    "LBY" => "Libya",
    "LCA" => "Saint Lucia",
    "LES" => "Lesotho",
    "LIB" => "Lebanon",
    "LIE" => "Liechtenstein",
    "LTU" => "Lithuania",
    "LUX" => "Luxembourg",
    "MAD" => "Madagascar",
    "MAS" => "Malaysia",
    "MAR" => "Morocco",
    "MDA" => "Moldova",
    "MDV" => "Maldives",
    "MEX" => "Mexico",
    "MKD" => "North Macedonia",
    "MLI" => "Mali",
    "MLT" => "Malta",
    "MNE" => "Montenegro",
    "MNG" => "Mongolia",
    "MOZ" => "Mozambique",
    "MRI" => "Mauritius",
    "MTN" => "Mauritania",
    "MYA" => "Myanmar",
    "NAM" => "Namibia",
    "NCA" => "Nicaragua",
    "NED" => "Netherlands",
    "NEP" => "Nepal",
    "NGA" => "Nigeria",
    "NIG" => "Niger",
    "NOR" => "Norway",
    "NZL" => "New Zealand",
    "OMA" => "Oman",
    "PAK" => "Pakistan",
    "PAN" => "Panama",
    "PAR" => "Paraguay",
    "PER" => "Peru",
    "PHI" => "Philippines",
    "PLE" => "Palestine",
    "PLW" => "Palau",
    "PNG" => "Papua New Guinea",
    "POL" => "Poland",
    "POR" => "Portugal",
    "PRK" => "North Korea",
    "PUR" => "Puerto Rico",
    "QAT" => "Qatar",
    "ROU" => "Romania",
    "RUS" => "Russia",
    "RWA" => "Rwanda",
    "SAM" => "Samoa",
    "SCO" => "Scotland",
    "SDN" => "Sudan",
    "SEN" => "Senegal",
    "SEY" => "Seychelles",
    "SIN" => "Singapore",
    "SLB" => "Solomon Islands",
    "SLE" => "Sierra Leone",
    "SLV" => "Slovenia",
    "SMR" => "San Marino",
    "SOL" => "Solomon Islands",
    "SOM" => "Somalia",
    "SRB" => "Serbia",
    "SRI" => "Sri Lanka",
    "SSD" => "South Sudan",
    "STP" => "São Tomé and Príncipe",
    "SUR" => "Suriname",
    "SVK" => "Slovakia",
    "SWE" => "Sweden",
    "SWZ" => "Eswatini",
    "SYR" => "Syria",
    "TAN" => "Tanzania",
    "TCA" => "Turks and Caicos Islands",
    "TGA" => "Tonga",
    "THA" => "Thailand",
    "TOG" => "Togo",
    "TPE" => "Chinese Taipei",
    "TTO" => "Trinidad and Tobago",
    "TUN" => "Tunisia",
    "TUR" => "Turkey",
    "TKM" => "Turkmenistan",
    "UAE" => "United Arab Emirates",
    "UGA" => "Uganda",
    "UKR" => "Ukraine",
    "URU" => "Uruguay",
    "USA" => "United States",
    "UZB" => "Uzbekistan",
    "VAN" => "Vanuatu",
    "VEN" => "Venezuela",
    "VGB" => "British Virgin Islands",
    "VIE" => "Vietnam",
    "VIN" => "Saint Vincent and the Grenadines",
    "WAL" => "Wales",
    "YEM" => "Yemen",
    "ZAM" => "Zambia",
    "ZIM" => "Zimbabwe",
    // FM-specific codes seen in exports
    "ENG" => "England",
    "NIR" => "Northern Ireland",
    "WAL" => "Wales",
};

/// Look up a 3-letter country code. Returns the full name if found.
/// If not found, returns None (caller decides fallback behavior).
pub fn lookup_country(code: &str) -> Option<&'static str> {
    COUNTRY_CODES.get(code.trim().to_uppercase().as_str()).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_codes() {
        assert_eq!(lookup_country("ENG"), Some("England"));
        assert_eq!(lookup_country("GER"), Some("Germany"));
        assert_eq!(lookup_country("UKR"), Some("Ukraine"));
        assert_eq!(lookup_country("BRA"), Some("Brazil"));
        assert_eq!(lookup_country("ESP"), Some("Spain"));
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(lookup_country("eng"), Some("England"));
        assert_eq!(lookup_country("Ger"), Some("Germany"));
        assert_eq!(lookup_country("ukr"), Some("Ukraine"));
    }

    #[test]
    fn unknown_code_returns_none() {
        assert_eq!(lookup_country("XXX"), None);
        assert_eq!(lookup_country("ZZZ"), None);
    }

    #[test]
    fn whitespace_trimmed() {
        assert_eq!(lookup_country(" ENG "), Some("England"));
    }

    #[test]
    fn sample_csv_nation_codes() {
        // Codes from actual sample CSV rows
        assert!(lookup_country("UKR").is_some()); // Trubin
        assert!(lookup_country("GER").is_some()); // Woltemade
        assert!(lookup_country("GEO").is_some()); // Mamardashvili
        assert!(lookup_country("ITA").is_some()); // Donnarumma
    }

    #[test]
    fn total_entries_reasonable() {
        // Verify we have at least 150 entries (FIFA has ~211 members)
        let count = COUNTRY_CODES.entries().count();
        assert!(count >= 150, "Expected >= 150 country codes, got {count}");
    }
}
```

- [ ] **Step 2: Create stub files for compilation**

Create empty files so `parser/mod.rs` can find all submodules:
- `src-tauri/src/parser/positions.rs` — empty
- `src-tauri/src/parser/headers.rs` — empty
- `src-tauri/src/parser/fields.rs` — empty
- `src-tauri/src/parser/metrics.rs` — empty

- [ ] **Step 3: Run tests**

Run: `cd src-tauri && cargo test --lib parser::countries`
Expected: All 6 tests PASS.

## Dependencies

- Task 01 (types must exist for the module to compile; countries itself doesn't depend on types but parser/mod.rs references all submodules).

## Success Criteria

- `countries.rs` compiles with `phf` static map.
- All 6 unit tests pass.
- `lookup_country` handles case-insensitivity, whitespace, and unknown codes correctly.

## Tests

### Test 1: Known codes resolve to country names
**Feasibility:** ✅ Can be tested

### Test 2: Case-insensitive lookup
**Feasibility:** ✅ Can be tested

### Test 3: Unknown code returns None
**Feasibility:** ✅ Can be tested

### Test 4: Whitespace is trimmed before lookup
**Feasibility:** ✅ Can be tested

### Test 5: Sample CSV nation codes all resolve
**Feasibility:** ✅ Can be tested

### Test 6: Table has sufficient coverage (>= 150 entries)
**Feasibility:** ✅ Can be tested
