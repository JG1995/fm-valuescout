# CSV Parser Technical Deep-Dive

## Overview & Purpose

The CSV Parser is the foundational data ingestion feature of FM ValueScout. It reads Football Manager CSV exports, extracts player data across 80+ stat columns, computes derived metrics (per-90 stats, ratios), and returns structured player records with validation feedback.

**Key capabilities:**
- Parses FM CSV exports with auto-detected delimiter (semicolon, comma, or tab)
- Extracts 80+ stat fields across 8 categories (attacking, creativity, movement, defending, aerial, goalkeeping, discipline, match outcome)
- Computes per-90 metrics and ratios with zero-protection
- Validates required fields and provides detailed skip reasons
- Handles position strings like "D/WB (L)" and "AM (RLC)"
- Parses financial data (transfer value ranges, wage denominations)
- Returns structured `ParseResult` with players, skipped rows, warnings, and column status

**Architecture principle:** Pure function with no side effects. All parsing happens in memory; storage is a separate concern.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Tauri Frontend                              │
│  (invokes parse_csv command via Tauri IPC)                         │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    commands/csv_parser.rs                           │
│  parse_csv() - Tauri command wrapper                                │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       parser/mod.rs                                 │
│  parse_csv() - Main orchestration                                   │
│  - define_stats! macro generates stat column mapping               │
│  - Pre-computes column indices for O(1) lookup                     │
│  - Row-by-row validation and parsing                                │
└─────────────┬───────────┬──────────────┬──────────────┬──────────────┘
              │           │              │              │
              ▼           ▼              ▼              ▼
    ┌─────────────┐ ┌──────────────┐ ┌────────────┐ ┌─────────────┐
    │  headers.rs │ │ positions.rs │ │ countries  │ │  fields.rs  │
    │  BOM strip  │ │  Position    │ │  phf map   │ │  30+ field  │
    │  Delimiter  │ │  grammar     │ │  lookup    │ │  parsers    │
    │  detection  │ │              │ │            │ │             │
    └─────────────┘ └──────────────┘ └────────────┘ └─────────────┘
                                                                  │
                                                                  ▼
                                                        ┌─────────────────┐
                                                        │   metrics.rs    │
                                                        │  compute_metrics│
                                                        │  - per-90       │
                                                        │  - ratios       │
                                                        │  - zero-protection
                                                        └─────────────────┘
```

**Module responsibilities:**

| Module | Lines | Responsibility |
|--------|-------|----------------|
| `types.rs` | 337 | All data structures (enums, structs) |
| `mod.rs` | 432 | Main orchestration, stat column mapping, row processing |
| `fields.rs` | 714 | 30+ field parser functions |
| `headers.rs` | 200 | BOM stripping, delimiter detection, column mapping |
| `positions.rs` | 175 | FM position string grammar parser |
| `countries.rs` | 262 | Static FIFA country code map (phf) |
| `metrics.rs` | 216 | Per-90 and ratio computation |
| `commands/csv_parser.rs` | 18 | Tauri command wrappers |

## Data Flow

**Step-by-step from CSV file to ParseResult:**

```
1. File I/O
   └─> fs::read_to_string(path)
       └─> Error if file missing or open in another program

2. Header Row
   └─> strip_bom() removes UTF-8 BOM if present
   └─> detect_delimiter() counts ; , \t → most frequent
   └─> parse_headers() builds HashMap<String, usize> (lowercase name → index)
   └─> Check required columns (unique id, player, minutes, position)
       └─> Return Err if any missing

3. CSV Reader Setup
   └─> csv::ReaderBuilder::new()
       .delimiter(header_map.delimiter)
       .flexible(true)
   └─> Skip header row (already parsed)

4. Pre-compute Column Indices (performance optimization)
   └─> Identity fields: UID, Player, Nation, 2nd Nat, Club, Position, Age, Height, Feet
   └─> Financial: Transfer Value, Wage, Expires
   └─> Playing time: Appearances, Minutes
   └─> Stat columns: 80+ fields via STAT_COLUMNS loop
       └─> Build HashMap<&str, usize> for O(1) lookup

5. Row Processing Loop (for each data row)
   └─> MALFORMED ROW: Skip with "Malformed CSV row"

   └─> HARD REJECT: UID
       └─> Missing or non-integer → Skip with "Missing or invalid UID"
       └─> Duplicate UID → Skip with "Duplicate UID {uid}"

   └─> HARD REJECT: Name
       └─> Empty or whitespace → Skip with "Missing player name"

   └─> HARD REJECT: Position
       └─> Empty or parse error → Skip with "Invalid position: {reason}"

   └─> Build base player: ParsedPlayer::empty(uid, name, positions)

   └─> SOFT EXTRACT: Nation, 2nd Nat, Club, Age, Height
       └─> None if column missing or parse fails

   └─> SOFT EXTRACT WITH WARNING: Left Foot, Right Foot
       └─> Unrecognized label → Score 3 + warning

   └─> OPTIONAL COLUMNS: CA, PA
       └─> None if column missing

   └─> FINANCIAL FIELDS WITH WARNING: Transfer Value, Wage
       └─> Unparseable → all None + warning

   └─> DATE/PLAYING TIME: Expires (ISO format), Appearances (started, sub), Minutes

   └─> STAT FIELDS (80+ columns via STAT_COLUMNS)
       └─> For each stat with column present:
           └─> Parse as f64 (negative → None unless allow_negative)
           └─> Assign to player struct via assign_stat()

   └─> COMPUTE METRICS
       └─> compute_metrics(&mut player)
           └─> Per-90: value / minutes * 90 (None if minutes = 0 or None)
           └─> Ratios: numerator / denominator (None if denom = 0 or None)

   └─> Add player to results

6. Build Result
   └─> columns_missing: STAT_COLUMNS not in header_map
   └─> Return ParseResult with players, skipped_rows, warnings, column status
```

**Key design decision:** Pre-computing stat column indices outside the row loop (lines 174-180 in mod.rs) prevents O(n²) complexity. Without this optimization, looking up each stat column by name for every row would require 80+ HashMap lookups per row.

## Core Types

### Enums

```rust
/// Position role from FM position string (8 values)
pub enum Role { GK, D, WB, DM, M, AM, ST }

/// Side qualifier from FM position string (3 values)
pub enum Side { L, C, R }
```

### Structs

```rust
/// A single parsed position entry
pub struct Position {
    pub role: Role,
    pub sides: Vec<Side>,  // Empty for GK, typically [L], [C], [R], or [L,C,R]
}

/// Footedness with label and numeric score (1-5)
pub struct Footedness {
    pub label: String,  // "Very Strong", "Strong", "Fairly Strong", "Reasonable", "Weak"
    pub score: u8,      // 5, 4, 3, 2, 1
}

/// Nationality with optional 3-letter FIFA code
pub struct Nationality {
    pub code: Option<String>,  // Some("ENG") or None
    pub name: String,          // "England" or fallback to raw input
}
```

### Stat Category Structs (8 categories, 80+ fields)

All stat structs implement `Default` and are `Serialize`/`Deserialize`.

```rust
/// Attacking stats (15 fields + 5 per-90)
pub struct AttackingStats {
    // Raw fields
    pub goals: Option<f64>,
    pub goals_from_outside_box: Option<f64>,
    pub xg: Option<f64>,
    pub np_xg: Option<f64>,
    pub xg_overperformance: Option<f64>,  // Allows negative
    pub xg_per_shot: Option<f64>,
    pub shots: Option<f64>,
    pub shots_from_outside_box_per_90: Option<f64>,
    pub shots_on_target: Option<f64>,
    pub penalties_taken: Option<f64>,
    pub penalties_scored: Option<f64>,
    pub free_kick_shots: Option<f64>,

    // Per-90 computed fields
    pub goals_per_90: Option<f64>,
    pub xg_per_90: Option<f64>,
    pub np_xg_per_90: Option<f64>,
    pub shots_per_90: Option<f64>,
    pub shots_on_target_per_90: Option<f64>,
}

/// Chance creation / passing stats (12 fields + 4 per-90 + 1 ratio)
pub struct ChanceCreationStats {
    pub assists: Option<f64>,
    pub xa: Option<f64>,
    pub chances_created_per_90: Option<f64>,
    pub clear_cut_chances: Option<f64>,
    pub key_passes: Option<f64>,
    pub open_play_key_passes_per_90: Option<f64>,
    pub crosses_attempted: Option<f64>,
    pub crosses_completed: Option<f64>,
    pub open_play_crosses_attempted: Option<f64>,
    pub open_play_crosses_completed: Option<f64>,
    pub passes_attempted: Option<f64>,
    pub passes_completed: Option<f64>,
    pub progressive_passes: Option<f64>,
    pub pass_completion_rate: Option<f64>,  // Ratio

    // Per-90
    pub assists_per_90: Option<f64>,
    pub xa_per_90: Option<f64>,
    pub key_passes_per_90: Option<f64>,
    pub progressive_passes_per_90: Option<f64>,
}

/// Movement / physical stats (4 fields + 2 per-90)
pub struct MovementStats {
    pub dribbles: Option<f64>,
    pub distance_km: Option<f64>,
    pub sprints_per_90: Option<f64>,
    pub possession_lost_per_90: Option<f64>,
    pub dribbles_per_90: Option<f64>,
    pub distance_per_90: Option<f64>,
}

/// Defending stats (11 fields + 5 per-90 + 2 ratios)
pub struct DefendingStats {
    pub tackles_attempted: Option<f64>,
    pub tackles_completed: Option<f64>,
    pub key_tackles: Option<f64>,
    pub interceptions: Option<f64>,
    pub possession_won_per_90: Option<f64>,
    pub pressures_attempted: Option<f64>,
    pub pressures_completed: Option<f64>,
    pub blocks: Option<f64>,
    pub shots_blocked: Option<f64>,
    pub clearances: Option<f64>,
    pub tackles_per_90: Option<f64>,
    pub interceptions_per_90: Option<f64>,
    pub pressures_per_90: Option<f64>,
    pub clearances_per_90: Option<f64>,
    pub tackle_completion_rate: Option<f64>,
    pub pressure_completion_rate: Option<f64>,
}

/// Aerial / heading stats (4 fields + 1 per-90 + 1 ratio)
pub struct AerialStats {
    pub aerial_challenges_attempted: Option<f64>,
    pub aerial_challenges_won: Option<f64>,
    pub aerial_challenges_lost_per_90: Option<f64>,
    pub key_headers_per_90: Option<f64>,
    pub aerial_challenge_rate: Option<f64>,
    pub aerial_duels_per_90: Option<f64>,
}

/// Goalkeeping stats (10 fields, all pre-computed in FM)
pub struct GoalkeepingStats {
    pub clean_sheets: Option<f64>,
    pub goals_conceded: Option<f64>,
    pub saves_per_90: Option<f64>,
    pub expected_save_pct: Option<f64>,
    pub expected_goals_prevented: Option<f64>,  // Allows negative
    pub saves_held: Option<f64>,
    pub saves_parried: Option<f64>,
    pub saves_tipped: Option<f64>,
    pub penalties_faced: Option<f64>,
    pub penalties_saved: Option<f64>,
}

/// Discipline stats (6 fields + 2 per-90)
pub struct DisciplineStats {
    pub fouls_made: Option<f64>,
    pub fouls_against: Option<f64>,
    pub yellow_cards: Option<f64>,
    pub red_cards: Option<f64>,
    pub offsides: Option<f64>,
    pub mistakes_leading_to_goal: Option<f64>,
    pub fouls_made_per_90: Option<f64>,
    pub fouls_against_per_90: Option<f64>,
}

/// Match outcome / general stats (7 fields + 1 ratio)
pub struct MatchOutcomeStats {
    pub average_rating: Option<f64>,
    pub player_of_the_match: Option<f64>,
    pub games_won: Option<f64>,
    pub games_drawn: Option<f64>,
    pub games_lost: Option<f64>,
    pub team_goals: Option<f64>,
    pub win_rate: Option<f64>,
}
```

### Financial Types

```rust
/// Transfer value with optional range
pub struct TransferValue {
    pub currency_symbol: Option<String>,  // Some("€") or None
    pub low: Option<f64>,                 // Low end of range
    pub high: Option<f64>,                // High end of range (equal to low if single value)
    pub raw: Option<String>,              // Original string for display
}

/// Wage with denomination normalization
pub struct Wage {
    pub currency_symbol: Option<String>,
    pub raw_value: Option<f64>,          // Parsed numeric value before normalization
    pub wage_per_week: Option<f64>,      // Normalized to per-week
    pub denomination: Option<String>,     // "p/w", "p/m", "p/a"
    pub raw: Option<String>,
}
```

### Core Player Record

```rust
pub struct ParsedPlayer {
    // Identity (required)
    pub uid: u32,
    pub name: String,
    pub positions: Vec<Position>,

    // Identity (optional)
    pub nationality: Option<Nationality>,
    pub second_nationality: Option<Nationality>,
    pub club: Option<String>,
    pub age: Option<u16>,
    pub height: Option<u16>,
    pub left_foot: Option<Footedness>,
    pub right_foot: Option<Footedness>,

    // Ability (optional columns)
    pub ca: Option<u16>,
    pub pa: Option<u16>,

    // Financial
    pub transfer_value: TransferValue,
    pub wage: Wage,
    pub contract_expires: Option<String>,  // ISO date "yyyy-mm-dd"

    // Playing time
    pub appearances_started: Option<u16>,
    pub appearances_sub: Option<u16>,
    pub minutes: Option<u16>,

    // Stat categories (all Default)
    pub attacking: AttackingStats,
    pub chance_creation: ChanceCreationStats,
    pub movement: MovementStats,
    pub defending: DefendingStats,
    pub aerial: AerialStats,
    pub goalkeeping: GoalkeepingStats,
    pub discipline: DisciplineStats,
    pub match_outcome: MatchOutcomeStats,
}

// Builder helper
impl ParsedPlayer {
    pub fn empty(uid: u32, name: String, positions: Vec<Position>) -> Self {
        Self { uid, name, positions, ..Default::default() }
    }
}
```

### Result Types

```rust
/// Column metadata for UI feedback
pub struct ColumnStatus {
    pub name: String,
    pub index: usize,
}

/// Row skip reason with 1-indexed row number
pub struct SkippedRow {
    pub row_number: usize,
    pub reason: String,
}

/// Non-fatal parsing warning
pub struct ParseWarning {
    pub row_number: usize,
    pub field: String,
    pub message: String,
}

/// Complete parse result
pub struct ParseResult {
    pub players: Vec<ParsedPlayer>,
    pub skipped_rows: Vec<SkippedRow>,
    pub warnings: Vec<ParseWarning>,
    pub columns_found: Vec<ColumnStatus>,
    pub columns_missing: Vec<String>,
    pub total_rows: usize,
}
```

### Type Design Decisions

1. **Option wrapping everywhere**: FM CSV columns are frequently optional or missing. All stats are `Option<f64>` to distinguish "0" from "not recorded".

2. **Separate raw vs computed**: Raw stats (goals, shots) and computed stats (goals_per_90) live in the same struct. This keeps related data together but means the struct serves dual purposes.

3. **Financial types preserve raw input**: `TransferValue.raw` and `Wage.raw` store the original string. This allows the UI to display "€62M - €94M" even though we parse low/high as floats.

4. **Position as Vec<Position>**: FM players can have multiple positions (e.g., "AM (C), ST (C)"). We store all parsed positions, not just the first.

5. **Nationality fallback**: If the 3-letter code isn't in our phf map, we use the code as the name. This prevents data loss for obscure nations.

## Module Deep-Dives

### 1. Header Parsing (`headers.rs`)

**Responsibility:** Parse first CSV row, detect encoding/format, build column index map.

```rust
pub struct HeaderMap {
    pub map: HashMap<String, usize>,        // lowercase name → index
    pub original_names: Vec<String>,        // preserves case and order
    pub delimiter: u8,                      // b';', b',', or b'\t'
    pub columns_found: Vec<ColumnStatus>,
    pub missing_required: Vec<String>,
}
```

**BOM stripping:**
```rust
pub fn strip_bom(input: &str) -> &str {
    input.strip_prefix('\u{feff}').unwrap_or(input)
}
```
UTF-8 BOM (EF BB BF) is common in Windows exports. We strip it before delimiter detection.

**Delimiter detection:**
```rust
pub fn detect_delimiter(header: &str) -> u8 {
    let semicolons = header.chars().filter(|&c| c == ';').count();
    let commas = header.chars().filter(|&c| c == ',').count();
    let tabs = header.chars().filter(|&c| c == '\t').count();

    if tabs > semicolons && tabs > commas { b'\t' }
    else if commas > semicolons { b',' }
    else { b';' }  // Default on tie
}
```
Counts each delimiter and returns the most frequent. Semicolon is the default (FM's standard).

**Required columns check:**
```rust
const REQUIRED_COLUMNS: &[&str] = &["unique id", "player", "minutes", "position"];
```
Missing any of these returns `Err` immediately, before parsing any rows.

**Case-insensitive lookup:**
```rust
pub fn has_column(header_map: &HeaderMap, name: &str) -> bool {
    header_map.map.contains_key(&name.to_lowercase())
}
```
All header lookups are case-insensitive because FM's CSV casing varies.

### 2. Position Parsing (`positions.rs`)

**Responsibility:** Parse FM position strings into typed `Vec<Position>`.

**Grammar:**
```
position_list ::= single_position (", " single_position)*
single_position ::= role [ "(" sides ")" ]
role ::= "GK" | "D" | "WB" | "DM" | "M" | "AM" | "ST"
combined_role ::= role ( "/" role )*
sides ::= "L" | "C" | "R" | "LC" | "RLC"  (any combination)
```

**Examples:**
- `"GK"` → `[Position { role: GK, sides: [] }]`
- `"D (LC)"` → `[Position { role: D, sides: [L, C] }]`
- `"AM (C), ST (C)"` → `[Position { role: AM, sides: [C] }, Position { role: ST, sides: [C] }]`
- `"D/WB (L)"` → `[Position { role: D, sides: [L] }, Position { role: WB, sides: [L] }]`
- `"M/AM (C)"` → `[Position { role: M, sides: [C] }, Position { role: AM, sides: [C] }]`
- `"AM (RLC)"` → `[Position { role: AM, sides: [R, L, C] }]`

**Parser:**
```rust
pub fn parse_positions(input: &str) -> Result<Vec<Position>, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Empty position string".to_string());
    }

    let mut positions = Vec::new();
    for entry in input.split(", ") {
        let entry = entry.trim();
        if entry.is_empty() { continue; }
        let parsed = parse_single_position(entry)?;
        positions.extend(parsed);
    }

    if positions.is_empty() {
        return Err(format!("No valid positions found in '{}'", input));
    }
    Ok(positions)
}
```

**Side parsing:**
```rust
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
```
Returns `Err` on any unrecognized character, causing the row to be skipped.

### 3. Country Lookup (`countries.rs`)

**Responsibility:** Fast 3-letter FIFA country code → full name lookup.

**Implementation:**
```rust
use phf::phf_map;

static COUNTRY_CODES: phf::Map<&'static str, &'static str> = phf_map! {
    "AFG" => "Afghanistan",
    "ALB" => "Albania",
    // ... ~200 entries
    "ENG" => "England",   // FM-specific
    "NIR" => "Northern Ireland",  // FM-specific
    // ...
};

pub fn lookup_country(code: &str) -> Option<&'static str> {
    COUNTRY_CODES.get(code.trim().to_uppercase().as_str()).copied()
}
```

**Why phf?** Perfect hash function at compile time. O(1) lookup with no runtime hash computation. Faster than `HashMap` and more memory-efficient than `match` for 200+ entries.

**Coverage:** All ~211 FIFA members plus FM-specific codes (ENG, NIR, SCO, WAL). 207 entries as of current implementation.

**Fallback behavior:** If code not found, `lookup_country` returns `None`. The caller (`parse_nationality`) uses the raw code as the name.

### 4. Field Parsers (`fields.rs`)

**Responsibility:** 30+ individual field parser functions for identity, physical, financial, date, and stat fields.

**Helper functions:**
```rust
/// Get field by index, trim, filter empty
pub fn get_field(record: &csv::StringRecord, index: usize) -> Option<String> {
    record.get(index).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Extract field by optional index, apply parser
pub fn extract_field<U>(
    record: &csv::StringRecord,
    col_index: Option<usize>,
    parse: impl Fn(&str) -> Option<U>,
) -> Option<U> {
    col_index.and_then(|i| record.get(i).and_then(|s| parse(s)))
}

/// Extract with warning capture (for Transfer Value, Wage, Footedness)
pub fn extract_field_with_warning<T>(
    record: &csv::StringRecord,
    col_index: Option<usize>,
    row_number: usize,
    field_name: &str,
    parse: impl Fn(&str) -> (T, Option<String>),
    warnings: &mut Vec<ParseWarning>,
) -> Option<T>
```

**Identity parsers:**
```rust
pub fn parse_uid(record: &csv::StringRecord, index: usize) -> Result<u32, String>
pub fn parse_name(record: &csv::StringRecord, index: usize) -> Result<String, String>
pub fn parse_nationality(raw: &str) -> Option<Nationality>
pub fn parse_second_nationality(raw: &str) -> Option<Nationality>
pub fn parse_club(raw: &str) -> Option<String>
pub fn parse_age(raw: &str) -> Option<u16>
```

**Physical parsers:**
```rust
pub fn parse_height(raw: &str) -> Option<u16>  // Strip " cm"
pub fn parse_footedness(raw: &str) -> (Footedness, Option<String>)
```

**Footedness mapping:**
```rust
const FEETEDNESS_MAP: &[(&str, u8)] = &[
    ("very strong", 5),
    ("strong", 4),
    ("fairly strong", 3),
    ("reasonable", 2),
    ("weak", 1),
];
```
Unrecognized labels default to score 3 with a warning (e.g., "Unrecognized footedness label 'Injury', defaulting to score 3").

**Financial parsers:**

Transfer value formats:
- `"€62M - €94M"` → range
- `"€57M"` → single value
- `"€500"` → low value
- `"Not for Sale"` → all None + warning

```rust
pub fn parse_transfer_value(raw: &str) -> (TransferValue, Option<String>) {
    // 1. Extract currency symbol (leading non-digit chars)
    let (currency, numeric_part) = extract_currency_symbol(raw);

    // 2. Split on " - " for range
    let parts: Vec<&str> = numeric_part.split(" - ").collect();

    // 3. Parse each part with K/M multiplier
    match parts.len() {
        1 => { /* single value */ }
        2 => { /* range */ }
        _ => { /* unparseable → all None + warning */ }
    }
}

fn parse_money_value(s: &str) -> Option<f64> {
    let (num_str, multiplier) = if s.ends_with('M') || s.ends_with('m') {
        (&s[..s.len() - 1], 1_000_000.0)
    } else if s.ends_with('K') || s.ends_with('k') {
        (&s[..s.len() - 1], 1_000.0)
    } else {
        (s, 1.0)
    };
    num_str.trim().parse::<f64>().ok().map(|v| v * multiplier)
}
```

Wage formats with normalization:
- `"€74K p/w"` → wage_per_week = 74000
- `"€50K p/m"` → wage_per_week = 50000 / 4.33
- `"€600K p/a"` → wage_per_week = 600000 / 52
- `"€10K"` → no denomination, assume per-week

```rust
pub fn parse_wage(raw: &str) -> (Wage, Option<String>) {
    // 1. Extract currency symbol
    // 2. Detect denomination (p/w, p/m, p/a)
    // 3. Strip denomination, parse numeric value
    // 4. Normalize to per-week based on denomination
    let wage_per_week = match (raw_value, &denomination) {
        (Some(val), Some(d)) => match d.as_str() {
            "p/w" => val,
            "p/m" => val / 4.33,
            "p/a" => val / 52.0,
            _ => val,
        }
        _ => None,
    };
}
```

**Date parser:**
```rust
pub fn parse_date(raw: &str) -> Option<String> {
    // Parse "30/6/2028" or "30/6/28" as ISO "2028-06-30"
    // 2-digit years → assume 20XX
    let parts: Vec<&str> = raw.split('/').collect();
    if parts.len() != 3 { return None; }

    let day = parts[0].parse::<u32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let year_raw = parts[2].parse::<u32>().ok()?;
    let year = if year_raw >= 100 { year_raw as i32 } else { (year_raw + 2000) as i32 };

    let d = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    Some(d.format("%Y-%m-%d").to_string())
}
```

**Playing time parser:**
```rust
pub fn parse_appearances(raw: &str) -> (Option<u16>, Option<u16>) {
    // "46 (9)" → (Some(46), Some(9))
    // "51" → (Some(51), Some(0))
    // "" → (None, None)
    if let Some(paren_start) = raw.find('(') {
        let started_str = &raw[..paren_start].trim();
        let sub_str = raw[paren_start + 1..].trim_end_matches(')').trim();
        (started_str.parse::<u16>().ok(), sub_str.parse::<u16>().ok())
    } else {
        (raw.parse::<u16>().ok(), Some(0))
    }
}
```

**Stat parser:**
```rust
pub fn parse_stat(raw: &str, allow_negative: bool) -> Option<f64> {
    match parse_f64(raw, allow_negative) {
        Ok(Some(v)) => Some(v),
        _ => None,
    }
}

pub fn parse_f64(raw: &str, allow_negative: bool) -> Result<Option<f64>, String> {
    let raw = raw.trim();
    if raw.is_empty() { return Ok(None); }

    match raw.parse::<f64>() {
        Ok(v) => {
            if !allow_negative && v < 0.0 {
                Err(format!("Negative value {} not allowed", v))
            } else {
                Ok(Some(v))
            }
        }
        Err(_) => Err(format!("Cannot parse '{}' as number", raw)),
    }
}

pub fn parse_distance(raw: &str) -> Option<f64> {
    parse_f64_strip_suffix(raw, "km", false).ok().flatten()
}
```
Negative values are converted to `None` for normal stats (hard reject the value, not the row). Only `xG-OP` and `xGP` allow negative.

### 5. Metrics Computation (`metrics.rs`)

**Responsibility:** Compute all derived metrics (per-90 stats, ratios) with zero-protection.

**Entry point:**
```rust
pub fn compute_metrics(player: &mut ParsedPlayer) {
    let minutes: Option<f64> = player.minutes.map(|m| m as f64);

    // Attacking per-90
    player.attacking.goals_per_90 = per_90(player.attacking.goals, minutes);
    player.attacking.xg_per_90 = per_90(player.attacking.xg, minutes);
    // ... (19 per-90 computations total)

    // Ratios
    player.chance_creation.pass_completion_rate = ratio(
        player.chance_creation.passes_completed,
        player.chance_creation.passes_attempted,
    );
    // ... (4 ratio computations total)
}
```

**Per-90 formula:**
```rust
fn per_90(value: Option<f64>, minutes: Option<f64>) -> Option<f64> {
    match (value, minutes) {
        (Some(v), Some(m)) if m > 0.0 => Some(v / m * 90.0),
        _ => None,
    }
}
```
Returns `None` if:
- Raw value is `None`
- Minutes is `None`
- Minutes is 0 (avoid division by zero)

**Ratio formula:**
```rust
fn ratio(numerator: Option<f64>, denominator: Option<f64>) -> Option<f64> {
    match (numerator, denominator) {
        (Some(n), Some(d)) if d > 0.0 => Some(n / d),
        _ => None,
    }
}
```
Returns `None` if denominator is `None` or 0.

**Safe add (for win rate):**
```rust
fn safe_add(a: Option<f64>, b: Option<f64>, c: Option<f64>) -> Option<f64> {
    let vals = [a, b, c].into_iter().filter_map(|v| v);
    let mut sum = 0.0;
    let mut any = false;
    for v in vals {
        sum += v;
        any = true;
    }
    if any { Some(sum) } else { None }
}
```
Sum wins/draws/losses to compute total games for win rate.

### 6. Parser Orchestration (`mod.rs`)

**Responsibility:** Main entry point, stat column mapping, row processing loop, hard reject vs soft extract logic.

#### Stat Column Macro

**Problem:** Adding a new stat field requires:
1. Adding field to stat struct in `types.rs`
2. Adding parser logic in row processing
3. Tracking column index

**Solution:** `define_stats!` macro generates all three from one declarative entry.

```rust
macro_rules! define_stats {
    ($($field:literal => { csv: $csv:literal, neg: $neg:expr, access: $($access:ident).+ }),* $(,)?) => {
        // 1. Static column definition array
        const STAT_COLUMNS: &[(&str, ColumnDef)] = &[
            $(
                ($field, ColumnDef { csv_name: $csv, allow_negative: $neg }),
            )*
        ];

        // 2. Dynamic stat assignment function
        fn assign_stat(player: &mut ParsedPlayer, field_name: &str, value: Option<f64>) {
            match field_name {
                $(
                    $field => player.$($access).+ = value,
                )*
                _ => {}
            }
        }
    };
}
```

**Usage:**
```rust
define_stats! {
    "goals" => { csv: "Goals", neg: false, access: attacking.goals },
    "xg_overperformance" => { csv: "xG-OP", neg: true, access: attacking.xg_overperformance },
    "expected_goals_prevented" => { csv: "xGP", neg: true, access: goalkeeping.expected_goals_prevented },
    // ... 80+ entries
}
```

**Benefits:**
- Single source of truth for stat column mapping
- Compile-time generation of assignment logic
- Type-safe path access (`attacking.goals` is checked at compile time)

**Performance optimization (lines 174-180):**
```rust
// Pre-compute stat column indices for O(1) lookup
let mut stat_col_indices = std::collections::HashMap::new();
for &(field_name, ref col_def) in STAT_COLUMNS {
    if let Some(idx) = get_column_index(&header_map, col_def.csv_name) {
        stat_col_indices.insert(field_name, idx);
    }
}
```
Without this, each row would need 80+ `get_column_index` HashMap lookups. With pre-computation, we do 80 lookups once, then O(1) access per row.

#### Row Processing Loop

**Hard rejects (row skipped immediately):**
```rust
// 1. UID validation
let uid = match (col_uid, parse_uid_safe(&record, col_uid)) {
    (_, Some(Ok(u))) => u,
    _ => {
        skipped_rows.push(SkippedRow { row_number, reason: "Missing or invalid UID".to_string() });
        continue;
    }
};

// 2. Duplicate UID check
if seen_uids.contains(&uid) {
    skipped_rows.push(SkippedRow { row_number, reason: format!("Duplicate UID {}", uid) });
    continue;
}
seen_uids.insert(uid);

// 3. Name validation
let name = match col_player.and_then(|i| record.get(i).map(|s| s.trim().to_string())) {
    Some(n) if !n.is_empty() => n,
    _ => {
        skipped_rows.push(SkippedRow { row_number, reason: "Missing player name".to_string() });
        continue;
    }
};

// 4. Position validation
let positions = match col_position.and_then(|i| record.get(i)) {
    Some(pos_str) => match parse_position_field(pos_str) {
        Ok(p) => p,
        Err(reason) => {
            skipped_rows.push(SkippedRow { row_number, reason: format!("Invalid position: {}", reason) });
            continue;
        }
    },
    None => {
        skipped_rows.push(SkippedRow { row_number, reason: "Missing position".to_string() });
        continue;
    }
};
```

**Soft extracts (None if missing/invalid):**
```rust
player.nationality = extract_field(&record, col_nation, parse_nationality);
player.second_nationality = extract_field(&record, col_2nd_nat, parse_second_nationality);
player.club = extract_field(&record, col_club, parse_club);
player.age = extract_field(&record, col_age, parse_age);
player.height = extract_field(&record, col_height, parse_height);
```

**Soft extracts with warnings (value + warning pushed to warnings vec):**
```rust
player.left_foot = extract_field_with_warning(
    &record, col_left_foot, row_number, "Left Foot", parse_footedness, &mut warnings,
);
player.right_foot = extract_field_with_warning(
    &record, col_right_foot, row_number, "Right Foot", parse_footedness, &mut warnings,
);
```

**Stat fields loop:**
```rust
for &(field_name, ref col_def) in STAT_COLUMNS {
    if let Some(&idx) = stat_col_indices.get(field_name) {
        if let Some(raw) = record.get(idx) {
            let is_distance = field_name == "distance_km";
            let value = if is_distance {
                parse_distance(raw)
            } else {
                parse_stat(raw, col_def.allow_negative)
            };
            assign_stat(&mut player, field_name, value);
        }
    }
}
```

**Metrics computation:**
```rust
compute_metrics(&mut player);
```

## Tauri Integration

**Command wrapper (`commands/csv_parser.rs`):**
```rust
#[tauri::command]
pub fn parse_csv(file_path: String, _in_game_date: String) -> Result<ParseResult, String> {
    parser::parse_csv(&file_path)
}

#[tauri::command]
pub fn save_import(players: Vec<ParsedPlayer>, in_game_date: String) -> Result<(), String> {
    storage::save_import(players, &in_game_date)
}
```

**Wiring (`lib.rs`):**
```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::csv_parser::parse_csv,
            commands::csv_parser::save_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Design notes:**
- `_in_game_date` parameter is unused (reserved for future multi-snapshot logic)
- `parse_csv` is pure (no side effects)
- `save_import` is the bridge to the storage layer (currently stubbed)

## Storage Layer

**Current status:** Honest stub.

```rust
pub fn save_import(_players: Vec<ParsedPlayer>, _in_game_date: &str) -> Result<(), String> {
    Err("Storage is not yet implemented. Your data has not been saved.".to_string())
}
```

**Planned behavior (per type signature):**
- Idempotent: Players with same UID + in_game_date are skipped silently
- All-or-nothing: Returns `Err` if any player fails to insert

## Error Handling Philosophy

The parser distinguishes three severity levels:

### 1. Hard Reject (Row Skip)

**Condition:** Violates data integrity or uniqueness.

**Triggers:**
- Missing or invalid UID
- Duplicate UID within the same CSV
- Missing player name
- Missing or invalid position

**Behavior:** Row skipped immediately, added to `skipped_rows` with reason. Processing continues with next row.

**Example:**
```
Skipped row 42: Missing or invalid UID
Skipped row 57: Duplicate UID 71101334
```

### 2. Soft Extract (Field-Level None)

**Condition:** Optional column missing or parse fails for a non-critical field.

**Triggers:**
- Nation, 2nd Nat, Club, Age, Height column missing or invalid
- CA, PA column missing or invalid
- Stat column has non-numeric value (e.g., "-")

**Behavior:** Field set to `None`, row still parsed. No warning.

**Example:**
- CA/PA not in CSV → `player.ca = None`, `player.pa = None`
- Age is "N/A" → `player.age = None`

### 3. Warning (Non-Fatal Issue)

**Condition:** Parse succeeds but with fallback behavior.

**Triggers:**
- Unrecognized footedness label → defaults to score 3
- Unparseable transfer value → all None
- Unparseable wage → all None

**Behavior:** Value uses fallback, warning added to `warnings` vec.

**Example:**
```
Warning row 12 field Left Foot: Unrecognized footedness label 'Injury', defaulting to score 3
Warning row 45 field Transfer Value: Cannot parse transfer value: 'Not for Sale'
```

### Negative Value Handling

**General stats:** Negative → `None` (soft extract)
- Goals, shots, tackles, etc. with negative values are treated as missing

**Exception stats:** Negative allowed
- `xG-OP` (xG Overperformance) can be negative (underperforming xG)
- `xGP` (Expected Goals Prevented) can be negative (poor keeping)

## Test Coverage

### 1. Unit Tests (in each module)

**Purpose:** Test individual functions in isolation.

**Examples:**
- `headers.rs`: BOM stripping, delimiter detection, case-insensitive lookup
- `positions.rs`: All position grammar variations
- `countries.rs`: Code lookup, case insensitivity, whitespace trimming
- `fields.rs`: Each parser function with valid/invalid input
- `metrics.rs`: Per-90 and ratio formulas with edge cases

**Example from `metrics.rs`:**
```rust
#[test]
fn per_90_zero_minutes() {
    let mut p = make_player_with_minutes(0);
    compute_metrics(&mut p);
    assert!(p.attacking.goals_per_90.is_none());
    assert!(p.chance_creation.assists_per_90.is_none());
}
```

### 2. Integration Tests (`tests/integration_csv_parser.rs`)

**Purpose:** Test full pipeline against real sample CSV (258 lines = 1 header + 257 data rows).

**Test file:** `../docs/notes/test-files/Test_CSV_2026_04_29.csv`

**What it validates:**
- Correct player count (256 players, 1 skipped row)
- UID uniqueness across all parsed players
- Identity fields match sample (Trubin, Woltemade, Donnarumma, Mamardashvili)
- Stat values match sample (goals, xG, distance, etc.)
- Per-90 computations are accurate
- Warnings captured correctly
- Position parsing handles multiple positions

**Example:**
```rust
#[test]
fn sample_csv_woltemade_per_90() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv").unwrap();
    let woltemade = result.players.iter().find(|p| p.uid == 91187791).expect("Should find Woltemade");

    // Woltemade: 3674 minutes = 40.82 * 90
    // Goals: 23 → 23 / 40.82 = 0.5633...
    assert!(woltemade.attacking.goals_per_90.unwrap() > 0.56 && woltemade.attacking.goals_per_90.unwrap() < 0.57);
}
```

### 3. Edge Case Tests (`tests/edge_case_csv_parser.rs`)

**Purpose:** Test error handling, boundary conditions, and unusual but valid input.

**Categories:**

**File format:**
- Empty CSV (headers only)
- Non-CSV file (fewer than 3 columns)
- BOM-prefixed CSV
- Comma-delimited CSV
- Tab-delimited CSV
- File not found

**Column/header:**
- Missing optional columns (no CA/PA)
- Missing required column (no Position)

**Row validation (hard rejects):**
- Missing UID
- Missing name
- Missing position
- Invalid position
- Duplicate UID
- Mixed valid/invalid rows

**Data validation:**
- Zero minutes → all per-90 stats are None
- Single transfer value (low == high)
- Wage per-month/per-a normalization
- Negative stat → None for normal field
- Negative allowed for xG-OP, xGP

**Example:**
```rust
#[test]
fn zero_minutes_per90_all_none() {
    let csv = "Unique ID;Player;Position;Minutes;Goals\n12345;Test Player;ST (C);0;10\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert!(result.players[0].attacking.goals_per_90.is_none());
}
```

## Key Design Decisions

### 1. Pure Function with No Side Effects

**Decision:** `parse_csv` is pure — reads file, returns result, no database writes.

**Rationale:**
- Easy to test (no database setup/teardown)
- Frontend can preview parse result before committing
- Storage is separate concern (can be swapped without touching parser)

**Tradeoff:** Frontend must call `save_import` separately (two round trips instead of one).

### 2. Pre-compute Column Indices

**Decision:** Build `stat_col_indices` HashMap before row loop (lines 174-180).

**Rationale:** O(1) stat column lookup per row vs O(n) HashMap lookup per stat per row.

**Impact:** For 80 stats across 256 rows:
- Without pre-computation: 80 × 256 = 20,480 HashMap lookups
- With pre-computation: 80 (once) + 256 × 80 HashMap reads = ~20,480 HashMap reads
HashMap reads are faster than lookups (no hash computation on miss).

### 3. Hard Reject vs Soft Extract

**Decision:** Hard reject on UID/name/position issues; soft extract on all other fields.

**Rationale:**
- UID/name/position are required for identity — can't have partial identity
- Stats can be partial (player who took no shots still exists)
- Prevents garbage data from polluting the database

**Tradeoff:** Some valid data is lost in hard-reject rows. Alternative would be to parse all data and let UI decide, but this shifts responsibility to frontend.

### 4. Negative Value Handling

**Decision:** Negative values → `None` for normal stats; allowed for xG-OP/xGP.

**Rationale:**
- Most stats can't be negative (you can't score -3 goals)
- Negative values in FM usually mean "not recorded" or data error
- xG-OP/xGP are genuine negatives (underperformance)

**Alternative:** Hard reject rows with negative values. Rejected because data entry errors are common and shouldn't discard entire row.

### 5. Footedness Default Score

**Decision:** Unrecognized footedness labels default to score 3 with warning.

**Rationale:**
- FM may add new labels in future versions
- Default 3 (average) is reasonable middle ground
- Warning alerts user to check data

**Alternative:** Return `None`. Rejected because footedness is rarely critical for scouting decisions.

### 6. Position Grammar Combined Roles

**Decision:** "D/WB (L)" parses as two separate positions with same sides.

**Rationale:**
- FM uses this to indicate player can play either position
- Keeping both positions preserves information
- Scoring system needs to evaluate player in each role separately

**Alternative:** Parse as single combined role. Rejected because scoring system is position-specific.

### 7. Financial Normalization

**Decision:** All wages normalized to per-week; transfer values preserve raw string.

**Rationale:**
- Per-week is standard for comparison
- Preserving raw string allows UI to display original format
- Frontend can convert to other time periods if needed

**Tradeoff:** Loss of precision (division by 4.33 and 52 is approximate).

### 8. Stat Category Structs

**Decision:** 8 separate structs instead of one flat struct or generic map.

**Rationale:**
- Groups related fields (attacking stats together)
- Type-safe (compile-time checking)
- Easy to serialize/deserialize

**Tradeoff:** More verbose than generic map. Worth it for type safety and IDE autocomplete.

### 9. Static phf Map for Countries

**Decision:** Use `phf_map` instead of `HashMap` or `match`.

**Rationale:**
- Compile-time perfect hash (no runtime hash computation)
- O(1) lookup, faster than `HashMap`
- More maintainable than 200-case `match`

**Tradeoff:** Requires build-time code generation. Worth it for hot path (every row).

### 10. ISO Date Format

**Decision:** Parse and store dates as "yyyy-mm-dd" strings.

**Rationale:**
- Chrono would add dependency weight
- String format is sufficient for display and sorting
- Easy to parse in frontend (JavaScript `Date` constructor)

**Tradeoff:** No date arithmetic in Rust. Acceptable because date operations are UI concern (filtering by date range, etc.).

## Performance Characteristics

**Time complexity:** O(n) where n = number of rows
- Header parsing: O(h) where h = number of columns
- Row processing: O(n × s) where s = number of stat columns (constant 80+)
- Metrics computation: O(1) per player (constant 23 operations)

**Space complexity:** O(n + s)
- Player records: O(n)
- Column index maps: O(s) where s = number of columns (~100)

**Bottlenecks:**
- File I/O (`fs::read_to_string`) — cannot be avoided
- CSV parsing (`csv` crate) — highly optimized, negligible overhead
- String allocations — extensive use of `trim().to_string()` in hot path

**Optimization opportunities:**
- Zero-copy parsing (requires `csv` crate with lifetimes)
- Bulk stat assignment (currently one-by-one via `assign_stat` match)
- Parallel row processing (requires `Send` bounds and coordination overhead)

For typical FM exports (200-500 rows), current implementation is sufficiently fast (<100ms on modern hardware).

## Future Enhancements

**Planned:**
- Database storage layer (SQLite or IndexedDB)
- Multi-snapshot accumulation (compare players across seasons)
- Custom stat columns (user-defined fields)
- Incremental parsing (skip already-imported UIDs)

**Considered:**
- Streaming CSV parsing (for files >100MB) — rejected because FM exports are small
- Fuzzy matching for footedness labels — rejected in favor of strict matching + warning
- Automatic currency conversion — rejected because exchange rates fluctuate
