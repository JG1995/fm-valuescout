# Player profiles

## Intent

Give each player a dedicated profile page so the user can inspect identity, attributes (visible, hidden, personality), and all role scores — the traditional scouting path after Search. Names and search rows navigate to the same route from Search and global suggest (not the dashboard sanity list).

## Delivered behavior

- Route `/players/$uid` shows one player from the **active save's current snapshot**.
- Page header: player name as `headline-lg` title; browser Back returns to the previous view (Search URL state stays intact).
- Three tabs — **Overview** | **Attributes** | **Roles** — active tab in validated URL search params (`tab=overview|attributes|roles`).
- **Overview:** identity and list-style basics (name, age/DOB, nationality, club, division, CA, PA, market value, height, preferred foot title-cased, contract/transfer flags when present). **Hero** Score Badge for the best non-null role score once Roles data is present (highest score; catalog-order ties).
- **Attributes:** visible attributes in FM-style groups (Technical / Mental / Physical / Goalkeeping), plus Hidden and Personality sections. Missing values render as `—`; null never coerces to 0.
- **Roles:** all 68 catalog role scores grouped by **position family**; every role shown (no mute/filter by familiarity). **Card** Score Badge per role; phase label maps `in_possession` / `out_of_possession` to **IP** / **OOP**. Null scores as `—`.
- **Entry points:** Search results — whole-row click or Enter on focused row → profile; **GlobalPlayerSearch** (Ctrl+K) — activate a hit → profile. Dashboard sanity-list names stay plain text (dev-only).
- Empty/missing: no snapshot → Load Data guidance (same as Search); unknown `uid` for current snapshot → not-found empty state (not a crash).
- Truncated-scan warning remains on the top-bar freshness chip only (no separate profile banner).
- Suspense fallback uses tab-shaped loading skeletons matching the active tab layout.

## Final architecture

```text
Rust features/player
  → commands.rs — get_player(uid)
  → query.rs — load one player from active save current snapshot; load player_role_scores
               then merge with in-process all_roles() (displayName, phase, positionTags);
               catalog order; null scores preserved
  → DTO: identity scalars + attribute/hidden/personality maps + roleScores[]

React features/player-profile
  → api/ — playerKeys ["player", uid]; getPlayerQueryOptions; fetch-get-player
  → components/ — overview, attributes, roles panels; profile tabs
  → utils/ — attribute-groups (static FM group membership); position-families; role-phase (IP/OOP labels)

app/routes/players.$uid.tsx — thin wiring; validateSearch tab; loader ensureQueryData;
                              snapshot + player queries; tab-shaped Suspense skeletons

Shared UI
  → ScoreBadge (table / card / hero / muted) in src/components/ui/score-badge/

Search / GlobalPlayerSearch
  → navigate to /players/$uid (no profile component imports)
```

**Position families** (ordered pitch groups; primary family = first known tag in this order): Goalkeeper (GK) → Centre-back (DC) → Full-back / Wing-back (DL, DR, WBL, WBR) → Defensive midfield (DM) → Central midfield (MC) → Wide midfield / Winger (ML, MR, AML, AMR) → Attacking midfield (AMC) → Striker (ST). Within a family: catalog order.

## Important decisions

- Sole PR; six delivery commits (five planned + finish-feature remediation).
- Tab state in URL (`tab` search param).
- IPC returns catalog metadata with each role score so the WebView does not re-derive phase/tags from a second Rust catalog copy.
- Overview best role = highest non-null score among all roles (ties: first in catalog order).
- Visible attribute group membership is a static frontend list in `attribute-groups.ts` aligned to FM Technical/Mental/Physical/Goalkeeping and dump keys — not computed in Rust for MVP.
- Profile page composition in `app/routes/players.$uid.tsx` so features stay free of cross-feature imports.
- Ctrl+K behaviour change (navigate to profile instead of name filter) accepted per product decision.

## Migration and operational implications

- No new migrations — profile reads existing `players` and `player_role_scores` from the current snapshot.
- Query runs in Rust against SQLite; the WebView never opens the DB.
- Role scores are read from `player_role_scores` (ingest-time); the WebView does not recompute the formula.
- `playerKeys.all` (`["player"]`) invalidated on Load Data and `set_active_save` alongside snapshot and search keys.

## Validation

- `./scripts/dev test`, `./scripts/dev check`, Playwright smoke (`get_player` stub).
- Vitest: route, Overview/Attributes/Roles panels, attribute grouping, null display, position-family grouping, ScoreBadge a11y, Search/GlobalPlayerSearch navigation, role-phase IP/OOP mapping, tab skeletons, preferredFoot title-case.
- `cargo test`: happy path, missing uid, active-save isolation, null attribute/score preservation.
- Feature-complete review: Blocking **No** (remediation `a30e45a` for phase labels, tab skeletons, preferredFoot).

**Delivery commits (final hashes):** `9bab503`, `f318171`, `42f64cb`, `109456e`, `5b11eeb`, `a30e45a` (comparison base `26bd83c`).

## Follow-up

- **Next feature:** [Squad planner](../../TODO.md) (order 6) — tactic + squad slots; same scores as search.
- **Deferred (unchanged):** position suitability map UI, radar/comparison charts, player comparison inspector, snapshot history, combined IP+OOP weight UI, sanity-list links, export, facepacks, crests.
