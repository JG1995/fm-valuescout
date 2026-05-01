# Scouting Feature - Design Spec

## Context

FM ValueScout is a moneyball-style scouting tool for Football Manager players. The core feature is "Moneyball Scouting" — analyzing player data from CSV imports to rank suitability for positions based on configurable archetypes.

**Relationship to existing features:**

- Depends on: CSV Parser (already implemented, parses FM exports into `ParsedPlayer`)
- Depends on: Database Integration (already implemented, stores saves/seasons/players)
- Informs: Player Profile (displays detailed metrics for a selected player)
- Future integration: My Squad (role preferences can optionally sync)

## Problem Statement

Currently, FM moneyball players manually export data to Excel, format it, and calculate scores themselves. The scouting feature automates this by:

1. Loading player data from imported CSVs
2. Scoring players against position-specific archetypes using weighted metrics
3. Presenting results in an intuitive pitch-based UI with a top-3 podium

## Assumptions and Constraints

- **CSV-only data source**: Phase 1 focuses on CSV upload. "PC memory reading" is deferred.
- **No predefined role setup from My Squad**: Scouting has its own role management; future sync is possible.
- **Single season data**: Players are scored against metrics from one season at a time.
- **Local SQLite storage**: Archetypes stored in existing database (SQLite), not LocalStorage.
- **Single-user desktop app**: No concurrent access concerns.

### Hard Constraints

- Must not break existing CSV parser or database integration features
- Existing `ParsedPlayer` type must remain compatible (no changes to stored data)
- All scoring calculations happen client-side in Svelte (no new Tauri commands needed for MVP)
- Archetypes persisted to SQLite using existing patterns

## Anti-Requirements

- **No "PC memory reading"**: Deferred to future version
- **No My Squad integration**: Scouting manages its own roles; sync is future work
- **No league/minutes filtering in MVP**: Assume user uploads relevant data
- **No ML/probability-based scoring**: Simple weighted percentile sums only

## Feature Scope

### Must-haves (Phase 1 MVP)

#### 1. Pitch View UI

- **Purpose**: Interactive football pitch showing all 11 position slots
- **Behavior**:
  - Displays a stylized pitch (4-4-2 or similar standard formation)
  - Each position slot shows the selected archetype name (or "Click to select" if none)
  - Clicking a position opens an archetype selector (dropdown/modal)
  - Pitch always visible at top, results table below
- **Edge cases**:
  - If no archetype selected, show "Default" or placeholder text
  - Position slot sizing should be responsive
- **Decisions made**: Click on position → opens selector with available archetypes for that role

#### 2. Archetype Management

- **Purpose**: Define and persist scoring configurations per position
- **Behavior**:
  - Pre-populated with default archetypes from metrics.md (loaded from embedded defaults)
  - User can create, edit, delete custom archetypes
  - Each archetype contains: name, position (Role), metrics with weights, in/out of possession combined
  - Stored in SQLite `archetypes` table
- **Default archetypes** (from metrics.md):
  - GK: Traditional Goalkeeper, Ball-Playing Goalkeeper
  - CB: Traditional Center Back, Ball-Playing Center Back
  - FB: Full Back, Offensive Full Back
  - DM: Defensive Midfielder, Playmaker
  - WB: Wing Back, Offensive Wing Back
  - CM: All-Rounder, Box-to-Box, Playmaker
  - Winger: Traditional Winger, Goalscoring Winger, Inside Forward
  - AM: Running Attacking Mid, Playmaker
  - ST: Creative Forward, Goalscoring Forward
- **Edge cases**:
  - Deleting an archetype in use removes it from pitch positions
  - Cannot delete all archetypes for a position (must have at least one)
- **Decisions made**: Archetype selection is single (not separate in/out possession); each archetype contains both

#### 3. Scoring Algorithm

- **Purpose**: Score each player against the selected archetype
- **Algorithm**:
  1. For each metric in archetype: compute percentile within loaded dataset (0-100)
  2. For INVERTED metrics (from metrics.md): use (100 - percentile)
  3. Weighted sum: `score = sum(percentile * weight)` for all metrics
  4. Result is 0-100 score
  5. Value-adjusted: `score / (transfer_value / median_value)`
     - If player has no transfer value: use median transfer value of dataset
- **Edge cases**:
  - Player with no stats for a metric: use 0 percentile (worst score)
  - Player with no transfer value: fallback to median
  - Player with multiple positions: score for each position independently
- **Decisions made**: Percentile-based normalization; combined in/out possession in single archetype

#### 4. Results Podium (Top 3)

- **Purpose**: Highlight the best-scoring players for the selected role
- **Behavior**:
  - Classic 3-2-1 podium layout: 1st center (tallest), 2nd left, 3rd right
  - Each podium position shows: player name, club, raw score, value-adjusted score
  - Ties broken by value-adjusted score (cheaper is better)
- **Decisions made**: Podium style with 1st tallest center

#### 5. Results Table

- **Purpose**: Full unranked list of players for the selected archetype
- **Behavior**:
  - Shows: name, club, all positions, age, transfer value, raw score, value-adjusted score, key metrics for archetype
  - Sortable by any column
  - Columns can be hidden (not removed)
  - Click row → navigate to Player Profile
- **Edge cases**:
  - Empty table when no archetype selected (or show all players in DB view)
  - Very long player lists: virtualized scroll

#### 6. Full Database View (No Role Selected)

- **Purpose**: Browse all players when no archetype is selected
- **Behavior**:
  - Table shows: name, club, all positions, age, nationality, transfer value
  - Color-coded overall scores per position (best-fit archetype for each position)
  - Sortable and filterable
- **Decisions made**: All positions shown; overall best-fit score per position

### Nice-to-have (Phase 2+)

- Archetype creation modal with metric picker
- My Squad sync for role preferences
- Customizable possession weight (75/25 default)
- Export results to CSV
- Comparison view (side-by-side players)

## Integration Points

### Existing Code Affected

- `src-tauri/src/parser/types.rs` — No changes; `ParsedPlayer` used as-is
- `src-tauri/src/storage/` — Add `archetypes` table for persistence
- `src-tauri/src/commands/storage.rs` — Add Tauri commands for archetype CRUD
- `src/routes/+page.svelte` — Replace placeholder with scouting UI

### New Code Required

- `src/lib/components/pitch/` — Pitch visualization component
- `src/lib/components/archetype/` — Archetype selector, editor components
- `src/lib/components/scouting/` — Results table, podium components
- `src/lib/stores/` — Archetype store, scoring store
- `src-tauri/src/storage/archetypes.rs` — Archetype CRUD operations
- `src-tauri/src/commands/archetypes.rs` — Tauri command wrappers

### Data Changes

- New SQLite table: `archetypes` (id, name, role, metrics_json, created_at, updated_at)
- New SQLite table: `scouting_preferences` (id, save_id, position_roles_json)

## Edge Cases and Boundary Conditions

1. **No players loaded**: Show empty state with "Import a CSV to start scouting"
2. **Player missing stats**: Use 0 percentile for missing metrics (worst case)
3. **Player missing transfer value**: Use median transfer value from dataset
4. **No archetype selected**: Show full database view with all players
5. **Multiple positions on player**: Score each position independently, show all in table
6. **Duplicate player names**: Use `uid` as primary identifier, name for display
7. **Tied scores**: Sort by value-adjusted score (cheaper is better)
8. **Archetype deleted while selected**: Clear that position, show prompt to reselect
9. **Empty archetype metrics**: Prevent saving; require at least one metric
10. **Very long archetype names**: Truncate with ellipsis in UI

## Failure Modes and Degradation

- **CSV import fails**: Show error, no players loaded; user must reimport
- **Database connection lost**: Show error, disable save operations
- **Archetype computation fails**: Skip player, log warning, continue with others
- **Missing metric in data**: Use 0 percentile (player ranks lower)
- **All players filtered out**: Show "No players match criteria" state

## Architecture

### Component Hierarchy

```
ScoutingPage/
├── PitchView/
│   ├── PositionSlot[] (clickable)
│   └── ArchetypeSelector (modal/dropdown on click)
├── ResultsSection/
│   ├── PodiumView (top 3)
│   │   ├── PodiumPosition (1st, 2nd, 3rd)
│   │   └── PlayerCard
│   └── ResultsTable (virtualized)
│       ├── ColumnHeader (sortable)
│       └── PlayerRow (clickable → PlayerProfile)
└── ArchetypeEditor/ (for creating/editing)
```

### Data Flow

```
CSV Import → parse_csv → ParsedPlayer[]
                                ↓
                        get_players_for_season (from DB)
                                ↓
                    ScoutingStore (holds loaded players)
                                ↓
                    When archetype selected:
                        scorePlayer(player, archetype, allPlayers)
                                ↓
                    Update ResultsStore
                                ↓
                    Render: Podium + Table
```

### Scoring Flow

```
1. Load players from DB (via Tauri command)
2. Compute percentiles for all metrics across dataset (once, on load)
3. User clicks position → select archetype
4. For each player:
   a. For each position player can play:
      - Get archetype for that role
      - Compute score: sum(percentile[metric] * weight[metric])
      - Compute value-adjusted: score / (transfer_value / median_value)
   b. Store scores for all positions
5. Sort by selected position's value-adjusted score
6. Render podium (top 3) and table (remaining)
```

### State Management

- **ScoutingStore**: loaded players, selected season, percentile cache
- **ArchetypeStore**: all archetypes (loaded from DB), selected per position
- **ResultsStore**: computed scores, sort order, hidden columns

## Error Handling

- **No players in season**: Empty state with import prompt
- **Archetype computation error**: Log + skip player, show warning count
- **Database error on archetype save**: Show toast, revert UI, log error
- **Invalid archetype weights**: Validate on save (must sum to ~1.0 or normalize)

## Invariants

1. **At least one archetype per position**: Cannot delete last archetype for a role
2. **Archetype weights sum to 1.0**: Enforced on save (normalized)
3. **All loaded players are scored**: No filtering before scoring (filtered in display)
4. **Percentiles computed from loaded data**: Not hardcoded thresholds
5. **Value-adjusted uses median fallback**: Transfer value of 0 handled gracefully

## Tech Stack

- **Frontend**: SvelteKit + TypeScript + Vite
- **State**: Svelte 5 runes ($state, $derived)
- **UI**: CSS Grid/Flexbox (no component library specified)
- **Backend**: Rust + Tauri + SQLite (rusqlite)
- **Data**: Existing ParsedPlayer type, existing storage module

## Testing Strategy

### Unit Tests

- Scoring algorithm: percentile calculation, weight normalization, inverted metrics
- Archetype validation: weights sum, required fields

### Integration Tests

- Tauri commands for archetype CRUD
- Full scoring pipeline: load players → compute scores → verify podium

### E2E Tests

- CSV import → select archetype → verify podium updates
- Create custom archetype → verify it appears in selector
- Delete archetype → verify UI updates

## Success Criteria

1. **Pitch renders correctly**: All 11 positions visible, clickable
2. **Archetype selection works**: Click position → selector appears → select archetype → shows on pitch
3. **Scoring produces correct results**: Verified against manual calculation for known players
4. **Podium shows top 3**: Sorted by score, 3 players displayed in podium layout
5. **Table shows all results**: Scrollable, sortable, clickable rows
6. **Value-adjusted scoring works**: Cheaper players with good scores rank higher
7. **Archetype persistence**: Create/edit/delete survives app restart

## Risks and Mitigations

| Risk                                          | Mitigation                               |
| --------------------------------------------- | ---------------------------------------- |
| Percentile computation slow on large datasets | Pre-compute once on load, cache in store |
| Many archetypes cause UI clutter              | Group by position in selector            |
| Score ties cause inconsistent results         | Secondary sort by name                   |

## Decision Log

| Decision                | Options Considered          | Chosen                               | Rationale                               |
| ----------------------- | --------------------------- | ------------------------------------ | --------------------------------------- |
| Data source             | CSV + PC memory             | CSV only                             | PC memory reading not feasible Phase 1  |
| DB vs Role view         | Two tabs vs unified         | Unified (no role = full DB)          | Simpler UX                              |
| Archetype structure     | Separate in/out vs combined | Combined (single archetype has both) | Simpler selection                       |
| Scoring normalization   | Raw values vs percentiles   | Percentiles                          | Fair comparison across different scales |
| Value-adjusted formula  | Various                     | score / (value / median)             | Intuitive: "score per unit cost"        |
| Missing value fallback  | Error vs default            | Median fallback                      | Graceful degradation                    |
| Multi-position handling | Best only vs all            | Score all positions                  | More information for user               |
| Podium layout           | Various                     | 3-2-1 style                          | Recognizable pattern                    |
| Archetype storage       | LocalStorage vs SQLite      | SQLite                               | Consistent with existing data model     |
| Row click behavior      | Preview vs navigate         | Navigate to profile                  | Cleaner flow                            |
