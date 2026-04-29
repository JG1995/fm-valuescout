# FM ValueScout

## Purpose

FM ValueScout is a companion application for Football Manager players who want to apply moneyball-style scouting to their save. Moneyball scouting identifies players who are undervalued by the transfer market but excel at statistical metrics relevant to a specific role. The tool replaces the manual workflow of exporting data to spreadsheets by providing purpose-built analysis, ranking, and squad management features in one place.

## Data Input

The application ingests player data from CSV files exported from Football Manager. Each CSV represents a snapshot of the game world at a point in time — typically one in-game season.

**Import lifecycle.** Each CSV import adds a new season layer to the database. The app accumulates seasons over time, building a longitudinal record of every player's career. This enables timeline-based analysis: the user can scroll back through a player's history and observe how their metrics evolved season to season.

**In-game date.** The user manually provides the current in-game date at the time of each import. This is used to compute contract status (time remaining until expiry) and other date-dependent context.

**CSV parsing.** The CSV uses semicolons as delimiters and contains 80+ columns covering biographical data, contract details, playing time, and statistical performance across all facets of the game. The parser must handle:

- Variable currency symbols and magnitude suffixes (K, M) in transfer values and wages.
- Wage denominations (per week, per month, per year) — preserved because they affect downstream calculations.
- Transfer value ranges (e.g. `€160M - €210M`) — only the upper bound is stored.
- Optional columns (CA, PA) that may be absent from some exports.
- Numeric values with varying precision and embedded units (e.g. `199 cm`, `312.7km`).
- Nationality as a 3-letter code (mapped to a full name) alongside 2nd nationality as a full name.
- Position strings parsed into arrays.
- Footedness as qualitative strings (e.g. "Very Strong", "Reasonable") optionally mapped to scores.

For the full column specification, see `csv-parsing.md`.

## Metrics

The application works with 80+ statistical metrics derived from or computed against the CSV data. Metrics are organized into eight categories:

1. **Attacking & Finishing** — goals, xG, shots, conversion rate, penalties.
2. **Creativity & Chance Creation** — assists, xA, key passes, crosses, chances created.
3. **Transition & Ball Progression** — passing, progressive passes, dribbles, distance, possession lost.
4. **Defensive Actions** — tackles, interceptions, pressures, blocks, clearances.
5. **Aerial Presence** — headers won/lost, aerial challenges, key headers.
6. **Goalkeeping & Shot Stopping** — saves, expected goals prevented, penalty saving.
7. **Discipline & Error Margins** — fouls, cards, offsides, mistakes leading to goal.
8. **Match Impact & Availability** — average rating, player of the match, win/draw/loss ratios, team goals.

Each metric has a **total** variant and (where applicable) a **per-90** variant. Some metrics are ratios computed from other metrics (e.g. pass completion ratio from passes attempted/completed). Metrics can be **inverted** — meaning a lower value is better (e.g. possession lost, fouls made, minutes per goal).

Every player has access to every metric regardless of position — a striker still has tackling data available.

For the full metric definitions, sources, and inversion flags, see `metrics.md`.

## Archetypes

Archetypes define how a player is evaluated for a specific role. Each archetype is a weighted set of metrics. The app ships with default archetypes covering all standard Football Manager positions:

- Goalkeeper (Traditional, Ball-Playing)
- Center Back (Traditional, Ball-Playing)
- Full Back (Standard, Offensive)
- Wing Back (Standard, Offensive)
- Defensive Midfielder (Defensive, Playmaker)
- Central Midfielder (All-Rounder, Box-to-Box, Playmaker)
- Winger (Traditional, Goalscoring, Inside Forward)
- Attacking Midfielder (Running, Playmaker)
- Striker (Creative Forward, Goalscoring Forward, Pressing Forward)

Each position has both an **in-possession** and **out-of-possession** archetype variant. When computing an overall score, the default weighting is 75% in-possession and 25% out-of-possession, but this is user-adjustable.

Users can create **custom archetypes** by choosing their own metric selections and weightings. Custom archetypes can be built from scratch or derived from an existing archetype (default or custom) as a starting point.

For the full list of default archetypes and their metric weights, see `metrics.md`.

## Core Features

### 1. My Squad

The squad module ties the user's own team into all analysis.

**Club selection.** On first use, the user selects the club they manage in their Football Manager save. The squad is then populated from the loaded database.

**Tactics board.** A visual pitch where the user sets their formation — either from presets or by freely positioning role slots. Each slot is assigned an archetype (e.g. "ball-playing center back" for one CB, "traditional center back" for the other). The tactics board is also the place where in-possession and out-of-possession archetypes are assigned per position.

**Lineup suggestions.** Based on the formation and archetypes selected, the app suggests an optimal starting eleven. Suggestions can factor in pure metrics or optionally weight in-game ability (CA).

**Squad overview.** A table view of the full squad, colour-coded by metric performance. Which metrics are shown depends on the roles assigned on the tactics board. The user can switch between Senior Squad, U21, U18, or all players.

For the full squad specification, see `squad.md`.

### 2. Moneyball Scouting

The core scouting feature, accessed from the Scouting tab, with two modes:

**Database view.** A large table of every player in the loaded database, displaying colour-coded percentile scores for each metric. Clicking a player opens their Player Profile. This is the "browse everything" mode.

**Role search.** A visual pitch shows the user's formation and assigned archetypes (from the Squad tab; defaults shown if none set). Clicking a position slot searches the database for players who score well for that archetype and presents:

- A **Top 3 podium** — a hero section ranking the three best signings. The user can toggle between "best by quality" and "best by value for money."
- A **results table** below — unranked, showing player info and key metrics for the archetype. Sortable, with the ability to add, remove, or reorder visible columns.

Role search applies filters by default: only players with 1,000+ minutes in the most recent season, and only players in leagues at or near the user's managed club's level. Both thresholds are user-adjustable.

For the full scouting specification, see `scouting.md`.

### 3. Player Profile

A dedicated page for a single player, providing:

**Biographical panel.** Photo, nationality, club, height, age, footedness, wage, transfer value, contract expiry, and positional eligibility.

**Statistical analysis.** Percentile rankings, configurable charts and diagrams, and full metric breakdowns across all eight categories. Every metric is available regardless of position.

**Role fit matrix.** If the user has set up their squad, a pitch diagram shows the player's suitability score for each role in the user's formation.

**Career timeline.** A vertical timeline with a node per season. The user can scroll back through the player's career, viewing how any metric evolved over time. This relies on the accumulated seasonal data from repeated CSV imports.

For the full player profile specification, see `player-profile.md`.

## Data Persistence

All data is stored locally on the user's machine. The persisted dataset includes:

- **Player database** — all imported player records, keyed by unique ID, with full seasonal history.
- **Season snapshots** — each CSV import is retained as a distinct season layer, enabling timeline analysis.
- **Squad configuration** — the user's managed club, selected formation, assigned archetypes per position slot.
- **Custom archetypes** — user-created archetype definitions with their metric selections and weightings.
- **User preferences** — in-possession/out-of-possession weight split, scouting filter thresholds, visible columns, and other configurable parameters.

No data is sent to external servers. The application is fully offline-capable.

## Design Principles

- **Value-first.** Every ranking and suggestion surfaces both quality and cost, because moneyball scouting is about finding undervalued production.
- **Configurable but opinionated.** Default archetypes, filters, and weightings provide a strong starting point. Power users can customize everything, but the defaults should work well out of the box.
- **Positional nuance.** In-possession and out-of-possession phases are scored separately, reflecting how real tactical systems assign different responsibilities to the same position.
- **Longitudinal awareness.** Accumulating seasons over time means the tool gets more valuable the longer the user's save runs.
- **Full metric access.** No metric is hidden based on position. A goalkeeper's tackling numbers are available; a striker's aerial data is available. This supports unexpected comparisons and custom archetype creation.
