# Player Profiles

## Status

Active

## Intent

Give each player a dedicated profile page so the user can inspect identity, attributes (visible, hidden, personality), and all role scores — the traditional scouting path after Search. Names and search rows navigate to the same route from anywhere they appear in product surfaces (not the dashboard sanity list).

## User-visible behavior

- Route `/players/$uid` shows one player from the **active save’s current snapshot**.
- Page header: player name as title; browser Back returns to the previous view (Search URL state stays intact).
- Three tabs — **Overview** | **Attributes** | **Roles** — active tab in validated URL search params (`tab`).
- **Overview:** identity and list-style basics (name, age/DOB, nationality, club, division, CA, PA, market value, height, preferred foot, contract/transfer flags as available). Best-role Score Badge (hero) once Roles data is present.
- **Attributes:** visible attributes in FM-style groups (Technical / Mental / Physical / Goalkeeping), plus Hidden and Personality sections. Missing values render as `—`; never coerce null to 0.
- **Roles:** all 68 catalog role scores, grouped by **position family**, show every role (no mute/filter by familiarity). Score Badge per role; null scores as `—`.
- **Entry points:** Search results — click whole row (Enter on focused row) → profile; GlobalPlayerSearch (Ctrl+K) — activate a hit → profile. Dashboard sanity-list names stay plain text (dev-only).
- Empty/missing: no snapshot → same Load Data guidance as Search; unknown `uid` for current snapshot → not-found empty state (not a crash).
- Truncated-scan warning remains on the top-bar freshness chip (same as Search); no separate profile banner required in this feature.

## Invariants

- Profile reads the active save’s **current** snapshot only; WebView never opens SQLite.
- Role scores come from `player_role_scores` (ingest-time); WebView does not recompute the formula.
- `null` dump/DB values never display as `0`.
- One scoring model shared with Search (CONCEPT principle 4).
- No cross-feature component imports — routes compose; Search/GlobalPlayerSearch navigate by route path only.

## Non-goals

- Position suitability map UI
- Radar / comparison charts
- Player comparison / inspector compare controls
- Snapshot history or attribute trends
- Combined IP+OOP weight UI (squad planner)
- Wiring the dashboard sanity list
- Export, facepacks, crests

## Current-state map

- Relevant components: Search virtual table (row click no-op); `GlobalPlayerSearch` navigates to `/search` with name `is` filter; no `/players` route; Score Badge specced in DESIGN.md but not implemented.
- Data model: `players` + `player_role_scores`; scoring catalog has `position_tags` per role.
- Persistence and migrations: none required for MVP profile read.
- Existing behavioral assumptions: Search filters/sort live in URL; Load Data / `set_active_save` invalidate snapshot + search keys.
- Architectural seams: Rust `features/search` for list/suggest; new profile read IPC; React `features/player-profile` + thin route.
- Tests and validation: Vitest + mockIPC; `cargo test`; Playwright smoke stubs.
- Primary risks: position-family assignment for multi-tag roles; attribute group membership lists; keeping frontend role labels aligned with catalog.

## Feature architecture (this feature)

```text
Rust features/player (or profile)
  → get_player(uid) — active save current snapshot
  → DTO: identity scalars + attribute/hidden/personality maps + roleScores[]
     (roleId, displayName, phase, positionTags, score)

React features/player-profile
  → api/ query options
  → components: page shell, tab panels (Overview / Attributes / Roles)
  → utils: attribute groups; position-family grouping from positionTags

app/routes/players.$uid.tsx — thin wiring; validateSearch tab; loader ensureQueryData

Shared UI
  → ScoreBadge (table / card / hero) in src/components/ui/

Search / GlobalPlayerSearch
  → navigate to /players/$uid (no profile component imports)
```

**Position families** (ordered pitch groups; primary family = first tag’s family in this order):

| Family | Tags |
| --- | --- |
| Goalkeeper | GK |
| Centre-back | DC |
| Full-back / Wing-back | DL, DR, WBL, WBR |
| Defensive midfield | DM |
| Central midfield | MC |
| Wide midfield / Winger | ML, MR, AML, AMR |
| Attacking midfield | AMC |
| Striker | ST |

Within a family: stable catalog order (or IP then OOP, then name) — pick one in build and keep it consistent.

## Uncertainty register

### Known

- Dedicated route; three tabs; all roles by position family; attributes include visible + hidden + personality; entry from Search row + Ctrl+K; sanity list out; comparison/radar/suitability/history deferred.
- DESIGN.md deferred “Player profile layout” is filled for this feature.

### Assumptions

- Feature folder name `player-profile` (frontend) mirroring a Rust `player` feature module is fine; rename only if build discovers a collision.
- Overview “best role” = highest non-null score among all roles (ties: first in catalog order).
- Attribute Technical/Mental/Physical/Goalkeeping membership is a static frontend (or shared) list derived from known dump keys — not computed in Rust for MVP.

### Decisions

- Sole PR; five atomic commits (see Delivery plan).
- Tab state in URL (`tab=overview|attributes|roles`).
- IPC returns catalog metadata with each score so the WebView does not re-derive phase/tags from a second copy of the full Rust catalog (labels may still mirror for filters elsewhere).

### Unknowns

- Exact attribute-group membership for edge keys — resolve from dump/layout lists during Attributes commit; ask only if a key is ambiguous.

### Risks

- Large role list density on Roles tab — mitigate with family section headings and Score Badge `card` variant, not virtualization unless scrolling is painful.
- Ctrl+K behaviour change may surprise users who used suggest-as-search-filter — accepted per product decision.

## Walking skeleton

`get_player` IPC + `/players/$uid` route rendering Overview identity for a known uid from the active snapshot, with tab chrome and empty Attributes/Roles placeholders — then fill tabs and wire Search/Ctrl+K.

## Delivery plan

### PR 1 — Player profile page and entry points

**Status:** Active

**Provisional PR title:** `feat(profile): add player profile page with attributes and role scores`

**Purpose:** End-to-end traditional scouting path: detail route, tabs, and navigation from Search and global suggest.

**Merge to trunk when:** Gate green; Search row and Ctrl+K open profiles; Attributes and Roles tabs show real data.

**Depends on:** Player search (done), role scoring on ingest (done).

#### Commit 1 — get_player IPC for current-snapshot detail

**Status:** Active

**Work:** Add Rust `get_player` command/service that loads one player by `uid` from the active save’s current snapshot, including attribute JSON maps and all `player_role_scores` joined with catalog display fields (`displayName`, `phase`, `positionTags`). Return null/not-found cleanly when missing. Register command; invalidate with snapshot/save keys from existing Load Data / set_active_save paths when wiring frontend later (document key in this commit if only Rust). Frontend types + query options stub optional if needed for RED — prefer Rust tests first.

**Out of scope for this commit:**
- React route / UI
- Search navigation changes
- ScoreBadge component
- Migrations

**Validation:** `cargo test` for happy path, missing uid, active-save isolation, null attribute/score preservation; `./scripts/dev check` when Rust staged.

**Provisional commit:** `feat(profile): add get_player IPC for snapshot player detail`

#### Commit 2 — Profile route and Overview tab

**Status:** Pending

**Work:** Add `/players/$uid` route with validated `tab` search param; loader prefetches player; Overview tab shows identity/basic fields with shared formatters; empty states for no snapshot / not found; tab chrome (Attributes/Roles can be empty panels). Page title = player name.

**Out of scope for this commit:**
- Full Attributes / Roles content
- Search / Ctrl+K wiring
- ScoreBadge (optional placeholder for best-role slot)

**Validation:** Vitest route + Overview render with mockIPC; smoke stub for `get_player`; `./scripts/dev check`.

**Provisional commit:** `feat(profile): add player route with overview tab`

#### Commit 3 — Attributes tab

**Status:** Pending

**Work:** Attributes tab: grouped visible attributes, Hidden, Personality; `—` for null; tabular figures per DESIGN.

**Out of scope for this commit:**
- Roles tab
- Radar / suitability
- Entry-point wiring

**Validation:** Vitest for grouping and null display; gate.

**Provisional commit:** `feat(profile): show visible hidden and personality attributes`

#### Commit 4 — Roles tab and ScoreBadge

**Status:** Pending

**Work:** Shared ScoreBadge (`table` / `card` / `hero`); Roles tab groups all scores by position family; Overview best-role hero badge; accessible names include role + tier.

**Out of scope for this commit:**
- Muting by positional familiarity
- Combined IP/OOP weights
- Search wiring

**Validation:** Vitest for family grouping and badge a11y name; gate.

**Provisional commit:** `feat(profile): show role scores by position family`

#### Commit 5 — Open profile from Search and Ctrl+K

**Status:** Pending

**Work:** Search results: whole-row activation (click + Enter) navigates to `/players/$uid`; GlobalPlayerSearch hit activation navigates to profile instead of name filter; update tests and Playwright stubs/expectations; keyboard search-to-profile path.

**Out of scope for this commit:**
- Sanity-list links
- New nav-rail item

**Validation:** Vitest Search + GlobalPlayerSearch navigation; smoke; `./scripts/dev check`.

**Provisional commit:** `feat(profile): open player profile from search and suggest`

## Active work

**PR:** 1 — Player profile page and entry points

**Commit:** get_player IPC for current-snapshot detail

### RED test (active commit)

Rust service/command test: given a fixture snapshot with a known `uid`, `get_player` returns that player’s name and at least one role score row; unknown `uid` returns not-found; null attribute in JSON stays null in the DTO (not 0).

### Expected outcome

IPC `get_player` is registered and covered by Rust tests; no UI yet (or types only if required for compile).

### Explicit exclusions

No React profile page, no Search navigation change, no schema migration.

## Discoveries and replanning

- Product decisions locked 2026-07-31: dedicated route; Overview/Attributes/Roles tabs; all roles by position family; attributes = visible+hidden+personality (+ basics); no suitability/radar; entry = Search row + Ctrl+K; not sanity list; defer comparison/history/weights/export.

## Completed work

| PR | Commit | Hash | Notes |
| --- | --- | --- | --- |
| — | — | — | — |

## Final validation

At feature end: `./scripts/dev test`, `./scripts/dev check`, smoke; manual Search → profile → tabs; Ctrl+K → profile; Back restores Search filters.

## Documentation impact

- DESIGN.md: Player profile layout (done at plan time).
- ARCHITECTURE.md § player profile read path at `/finish-feature`.
- TODO.md: feature Active → Completed when finished.
- Completed ledger archive at finish.
