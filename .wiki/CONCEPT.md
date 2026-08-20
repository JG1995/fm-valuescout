# Product Concept Document: FM ValueScout

> Authority: This document owns product purpose, users, principles, and boundaries. It does not own the implementation backlog or current architecture.

---

## The Idea

FM ValueScout is a desktop companion for Football Manager 26 that reads live game data and turns it into searchable player and staff databases with role-aware scoring and squad planning tools. It bridges the gap between in-game scouting and structured analysis: you load a snapshot while FM is running, explore players by position and role fit, compare staff by job fit, plan your squad around your tactic, and optimize lineups against a combined team score.

The central insight is that FM's default interface supports scouting and squad building, but not always in one workflow. ValueScout unifies traditional scouting (browse profiles, compare candidates) with moneyball-style scouting (rank by quantified role fit) and tactical squad planning (see gaps, optimize the XI). The app stays beside the game—you make decisions in FM; ValueScout gives you the lens.

## Problem Statement

Scouting and squad planning in Football Manager's default UI involve a lot of context switching. Finding players who fit a specific role in your tactic means juggling search filters, attribute views, reports, and mental math about how a signing changes the squad. Squad planning is separate from transfer search: you notice a gap in the tactics screen, then hunt for candidates elsewhere with no shared scoring model.

Spreadsheets and third-party scouts (Genie Scout, SuperScout) help with data export and attribute tables, but they are often disconnected from your current save state, your tactic, and a squad optimizer. You either maintain data manually or use tools that excel at lists but not at "who fills this role best for *my* system right now?"

Trade-offs today: rich in-game immersion vs. analytical depth; static exports vs. live world state; player lists vs. tactic-aware planning.

## Solution

ValueScout solves this with four pillars tied to how FM players actually work:

### 1. Live data snapshots

Load the current game world from a running FM26 session. One action captures clubs, squads, contracts, and attributes to match what is in memory. The primary workflow needs no manual export step. Successful snapshots stay in the active app save, and the snapshot with the greatest valid in-game date is current for normal product reads. Snapshots are explicit (you click **Load Data** when you want fresh data), which keeps the model simple and predictable.

Optional format-specific Squad imports supplement the current snapshot with supported Youth Tracker and Moneyball CSV values that the memory pipeline does not supply. Imports use exact numeric player IDs, stay scoped to the active app save, and never create players or replace live memory data. The Staff workspace can also import one save-owned shortlist from a staff CSV; it matches exact staff IDs and keeps Preferred Job, Club Job, and qualifications as recruitment context.

Player Profile and Player Search have optional Moneyball views for current-snapshot players in a scored Moneyball import. General remains the default view and keeps the familiar attribute, role-fit, concealment, and development tools. Moneyball shows playing-time context and raw performance metrics with imported-cohort percentile scores; Search can compare scores against either the filtered result cohort or the full CSV import.

The app supports multiple **save slots** (separate scouting databases—for example, different FM careers). Exactly one slot is active; **Load Data** stores a new snapshot in that slot, while Search, profiles, Planner, Academy, and CSV matching use the slot's current snapshot. Slots are app-side labels, not FM save files.

### 2. Searchable databases with role scores

Every loaded player lives in a searchable database with scores per position and per role (e.g. defensive midfielder / deep-lying playmaker). Staff have current-ability job-fit scores derived from the attributes required by each job. Sort and filter by the fit you need, inspect detailed player or staff profiles, review everyone at the selected managed club in Staff, and narrow an imported Staff Shortlist by Preferred Job or unemployment.

### 3. Squad planner

Model your squad and tactic in the app. See who fits where, where roles are thin, and how the current group lines up against your system. Planning stays linked to the same snapshot and scoring model used in search.

### 4. Squad optimizer

Given your tactic, optimize the lineup to maximize combined team score across positions and roles. Surface concrete gaps—e.g. missing a defensive midfielder who excels as a deep-lying playmaker—so transfer search has a clear target.

## Target Audience

> Solo hobbyist project scope. Built primarily for the author; shared with other FM players who want the same workflow. Agents calibrate ceremony, testing, and review accordingly.

| Segment | Description | Key Need |
| --- | --- | --- |
| **Primary user (author)** | FM26 player who scouts actively and tweaks tactics | Fast snapshot → gap analysis → targeted transfer search in one tool |
| **Analytical FM players** | Players who use spreadsheets or external scouts today | Live data and role scores without maintaining sheets |
| **Tactic-focused managers** | Players who build around roles and system fit | Optimizer and role-ranked search aligned to their tactic |

## Differentiation

| Competitor | Their Strength | Our Advantage |
| --- | --- | --- |
| **FM Genie Scout** | Mature scouting database, familiar to the community | Live FM26 memory snapshot; integrated squad planner and optimizer tied to your tactic |
| **FM SuperScout** | Scouting-focused companion features | Role scoring plus squad optimization in one workflow; offline desktop app |
| **Spreadsheets** | Full control, custom formulas | Live snapshots stay in sync with the loaded game world; an optional CSV import supplements supported exported values without replacing memory-backed data |
| **In-game FM UI** | Official, immersive, always current while playing | Searchable DB, cross-player role ranking, and lineup optimizer FM does not provide |

## Core Principles

1. **Offline-first:** Core use works without network access. Online connectivity is only for app updates.
2. **Live game as source of truth:** Data comes from the running FM26 session via memory read. Supported CSV imports supplement only values the memory pipeline does not provide and never become a parallel identity source.
3. **Explicit refresh:** The user loads data when they want a new snapshot—after signings, sales, or weekly progression—not silent background sync.
4. **One scoring model:** The same position and role scores drive search, profiles, squad planning, and optimization.
5. **Companion, not replacement:** Decisions (bids, contracts, team selection) happen in FM; ValueScout informs them.

## Success Looks Like

You have FM26 open mid-save. You launch ValueScout and click **Load Data**. The app snapshots the world: your squad, league players, attributes, and contracts match the game.

You open **Squad Planner**, set your tactic, and click **Optimize**. The optimizer shows your best XI by combined score and flags a gap: you lack a defensive midfielder who rates highly as a deep-lying playmaker.

You go to transfer search, filter available players, and sort by defensive midfielder / deep-lying playmaker score. You open a few profiles, compare role fit and attributes, and pick a target. You switch back to FM, negotiate, and sign the player.

A week later in-game, the signing completes. You click **Load Data** again. The new player appears in your squad; search results and planner state reflect the updated world. You re-run the optimizer and see the gap closed—or the next priority for the window.

## Scope Boundaries

### In Scope (MVP)

- Read player and world data from a running **FM26** session (memory read)
- **Load Data** action to create and refresh a local snapshot
- Searchable player database from the snapshot
- Position and role scores per player
- Detailed player profile view
- Transfer search with sort/filter by role scores
- Staff Search with sort/filter by current job-fit scores
- Staff overview for everyone at the selected managed club
- Save-owned Staff Shortlist CSV import with exact staff-ID matching, Preferred Job filtering, and unemployment filtering
- Detailed Staff Profile with current attributes and job fit
- Squad planner aligned to the user's tactic
- Squad optimizer that maximizes combined team score for the tactic
- Optional supported Youth Tracker and Moneyball CSV enrichment import matched to the current memory snapshot; imported career data can feed Youth Academy views
- Offline use; online only for application updates

### Out of Scope (MVP)

- FM editions other than FM26
- Save-file import or parsing as an alternative data source
- Historical Moneyball seasons, season selection, trends, comparisons, historical player views, and analytics beyond the supported current-snapshot contract
- Automatic or background sync while FM runs (refresh is manual)
- Accounts, cloud sync, or multi-user collaboration
- Executing transfers or general edits inside FM from the app. The only accepted exceptions are the two action-specific player boosts in [ADR-0017](./decisions/0017-action-specific-fm26-player-boosts.md), the fixed staff CA boost in [ADR-0020](./decisions/0020-action-specific-fm26-staff-ca-boost.md), and sequential managed-club orchestration of those closed operations in [ADR-0018](./decisions/0018-squad-wide-player-boosts.md) and [ADR-0021](./decisions/0021-sequential-club-family-staff-ca-boost.md). The bridge still exposes only three fixed one-person actions; transfers and general editing remain out of scope.
- Mobile or web clients
- Community databases, facepacks, or mod management
- Advanced moneyball analytics beyond defined position/role scores (e.g. xG models, custom ML pipelines)

## Risk Assumptions

1. **FM26 memory layout stays discoverable.** Patches or minor versions may shift offsets and break reads until updated. Mitigation: version-aware read logic and clear failure when the game build is unsupported.
2. **Role scores correlate with real in-game fit.** A scoring model that feels wrong will undermine trust. Mitigation: tune against known players and tactics; expose scores transparently on profiles.
3. **Users accept manual Load Data.** Requiring FM to be running and an explicit refresh is simpler but less seamless than continuous sync. Mitigation: make refresh fast and obvious after signings or weekly ticks.
4. **Memory reading remains viable for personal use.** Game EULAs and anti-cheat posture may affect distribution or long-term maintenance. Mitigation: treat as a personal/companion tool; reassess if sharing widely.
