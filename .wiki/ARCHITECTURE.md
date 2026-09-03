# Architecture — FM ValueScout

> Authority: This document describes the implemented stack, application layout, and tooling for FM ValueScout.

This document describes how **FM ValueScout** is constructed: stack, thin-frontend / thick-backend boundaries, build and test pipeline, and conventions enforced by tooling.

Application layout follows [Bulletproof React](https://github.com/alan2207/bulletproof-react) adapted for TanStack Router and Query on the frontend, and feature modules under `src-tauri/src/features/` on the backend. Line-level rules come from the installed `coding-standards` skill and its React, Tauri, Rust, and Vite references.

For product purpose, see [CONCEPT.md](./CONCEPT.md). For rationale behind each default choice, see [.wiki/decisions/](./decisions/README.md).

---

## 1. Top-Level Shape

**FM ValueScout** is a Tauri desktop application built on the React + Tauri v2 stack below, with a Pi workflow (tracked project settings and repository skills under `.pi/`, globally installed PI_SETUP skills and direct subagents, wiki, `./scripts/dev`), an implemented **FM26 memory-read bridge** (C# BepInEx plugin + Rust file protocol — [ADR-0016](./decisions/0016-csharp-bepinex-fm26-bridge.md), [completed record](./features/completed/fm26-memory-read.md)), **snapshot ingest and history management** (multi-save slots, retained Load Data snapshots, date-selected current reads, and Settings save/snapshot management — [history record](./features/completed/snapshot-history.md)), a **CSV enrichment import** (bounded Youth Tracker and Moneyball parsing, exact current-snapshot UID matching, snapshot-owned Moneyball persistence, and save-owned Youth persistence — [completed record](./features/completed/csv-enrichment-persistence.md)), **role scoring** (FM26 IP/OOP scores computed and persisted on ingest — [completed record](./features/completed/role-scoring-engine.md)), **player search** (virtualized Search page, operator filters, global Ctrl+K name suggest — [completed record](./features/completed/player-search.md)), **configurable player tables** (shared full-height virtual paging, validated dynamic metrics, persisted current-snapshot potential roles, persisted per-table layouts, and offline nationality flags — [completed record](./features/completed/configurable-player-tables.md)), and the **Squad workspace and Planner tools** (My Club managed-club selection, bounded Squad overview, dual-phase tactic, managed-club depth matrix, and exact automatic allocation in the My Club workspace at `/my-club`, with `/planner` retained as a compatibility redirect).

**Client / UI:** React 19 in a Tauri WebView — presentation layer only

**Bundler / dev server:** Vite 8 with `@vitejs/plugin-react` — bundles the WebView frontend

**Routing:** TanStack Router (file-based routes, typed search params, loaders coordinated with Query)

**Server / async state:** TanStack Query v5 — caches **IPC command results**, not HTTP responses. Global defaults in `src/app/router.tsx` disable focus/reconnect refetch and query retry for local IPC ([ADR-0005](./decisions/0005-tanstack-query.md)).

**Client UI state:** Zustand v5 (modals, layout chrome, selections not in the URL). `useMoneyballPreferences` persists one app-local General or Moneyball default; Shortlist requires explicit `view=shortlist` and is never a default. It is not save- or snapshot-scoped.

**Styling:** Tailwind CSS v4 via `@tailwindcss/vite`; design tokens bridge to [DESIGN.md](./DESIGN.md). IBM Plex Sans/Mono self-hosted via `@fontsource`; Lucide icons via `lucide-react`. Shared primitives in `src/components/ui/` (Button, Panel, StatusChip, EmptyState, TextField, SelectField, **Modal**, **ScoreBadge**). App shell: `AppNavRail` + `AppTopBar` (**GlobalPlayerSearch**, active save, snapshot freshness, optional Load Data player-cap toggle/limit, **Load Data**, and a dismissible context-bound Load Data outcome); `useLayoutStore` persists nav-rail expansion; `useLoadDataPreferences` persists the Load Data cap toggle and limit. Route workspaces size from the shell main area, so an outcome banner reduces available panel height instead of creating nested page scrolling. Player search results use **@tanstack/react-virtual** for row virtualization.

**Language:** TypeScript (strict) on the frontend; Rust on the backend

**Package manager:** pnpm (pinned in `packageManager`)

**Runtime for tooling:** Node.js 24

**Desktop shell:** Tauri v2 — native window, WebView, IPC runtime, capabilities ACL. The main window starts maximized while retaining normal OS window controls.

**Backend / computation:** Rust in `src-tauri/` — commands, services, SQLite queries, validation at trust boundaries

**Data:** SQLite via **rusqlite** (bundled) in Rust — migrations (`PRAGMA user_version`) and queries; WebView never opens the database directly. Live FM26 dumps land on disk via the bridge file protocol (`%LOCALAPPDATA%\fm-valuescout\fm-bridge\`); **Load Data** is a native async Tauri command where the frontend hook captures the active save ID and context token and `api/load-data.ts` constructs the typed `Channel`; Rust verifies the supplied `saveId`/`contextToken` and echoes them on every progress event, validates preparation outside the database mutex, and publishes atomically on the fresh file `app-v2.db` (legacy `app.db` is never opened). Migrations v2–v40 cover `saves`, retained `snapshots`, `players` (`potential_attributes_json` plus current-only compact projection), `staff`, compact `player_role_metrics` (79 current + 79 potential nullable columns; 162 columns total) and `staff_role_metrics` (21 nullable columns) for the effective current snapshot only, immutable save/snapshot context tokens, optional snapshot names, shared snapshot-level boost recovery requirements, snapshot-owned Moneyball rows and cohort percentile scores, save-scoped Planner and Academy data, save-owned Youth career and staff shortlist rows, the save-owned v17 Moneyball quarantine, the save-owned `planner_teams` configuration, save-scoped `staff_assignment_targets`, `managed_club_settings`, and the shared `saves.reveal_hidden_information` constrained boolean. Migration v38 creates `player_role_metrics` and `staff_role_metrics` with the immutable 68/68/21 column inventory, model-version and score checks, and cascades; migration v39 drops the normalized `player_role_scores`, `player_potential_role_scores`, and `staff_role_scores` tables and indexes; migration v40 additively appends exactly 22 nullable checked generic OOP role columns (11 current + 11 potential) with no score backfill. The catalog is 79 static FM26 roles. `SCORE_MODEL_VERSION=2` and `PROJECTION_MODEL_VERSION=2`. Migrated v1 rows remain physically preserved but compact-dependent current or potential reads reject them until a normal Load Data/current lifecycle writer materializes v2. Historical snapshots retain raw facts only; promotion rebuilds compact state. `app-v2.db` only; `app.db` remains untouched. Migration v29 preserves each prior primary-club selection while renaming its table and column, and drops obsolete attached-source rows; v30 adds nullable Moneyball percentile JSON without backfilling prior imports.

**FM26 bridge:** C# BepInEx 6 IL2CPP plugin in `bridge/` — memory layouts, safe block-based heap scanning (`TryReadBlock`), and `status.json` / `dump.json` / diagnostics with phase timings. On the live-validated exact FM 26.3.2 build, it exposes two bounded player-boost operations and one fixed +10, PA-capped staff CA operation through the local file protocol; the bridge owns all process-memory writes. Separate in-memory player and staff candidate indexes bind every action to its producing live scan, and one work gate serializes scans and mutations. Rust `features/memory_read` serializes requests for each bridge directory and sends only the closed boost requests. Rust owns sequential managed-club orchestration for Squad and Staff without adding bridge batch payloads. Rust `features/player` and `features/staff` derive snapshot-bound values, wait without the Db mutex, and reconcile each verified result into SQLite. React exposes guarded individual and managed-club boost controls. Rust `features/memory_read` also orchestrates dumps, validates dump shape, and installs the plugin DLL into Steam `BepInEx/plugins`; React `features/memory-read` shows install controls and bridge status. **Load Data** lives in `AppTopBar`. Windows Steam FM26 only. See [bridge/README.md](../bridge/README.md), [bridge scan performance](./features/completed/bridge-scan-performance.md), and [bridge-plugin-install](./features/completed/bridge-plugin-install.md).

**Snapshot ingest and management:** Rust `features/snapshot` owns save slots, transactional retained ingest from `dump.json`, the shared current-snapshot selector, metadata and save/snapshot management IPC, and token-bound destructive policy; React `features/snapshot` owns the save switcher, metadata-only snapshot overview, ordered history panel, and rename/delete flows. The frontend supplies the invocation-captured active save ID and immutable context token to `load_data`; Rust acquires the load lease, verifies that context under a brief Db lock, runs the bridge scan without holding the Db mutex, captures a private copy only while `status.json` still identifies that ready request, then revalidates and ingests it with the request ID. Every successful ingest keeps prior rows, selects the greatest valid in-game date (then load timestamp and ID), and returns both stored and effective-current metadata. See [snapshot-history](./features/completed/snapshot-history.md).

**CSV enrichment import:** Rust `features/csv_import` owns standards-compliant Youth Tracker and Moneyball parsing, fixture-backed header contracts, numeric UID validation, null-preserving normalization, bounded regular-file reads (8 MiB and 10,000 player rows), and safe errors. `import_csv` captures the active save, its immutable context token, the effective current snapshot, that snapshot token, and snapshot UIDs under a brief Db lock; it parses and prepares Moneyball rows outside that lock, optionally requires the detected format selected by a modal before persistence, revalidates the same context inside its write transaction, and returns only the detected format plus total, stored, and skipped player counts. It atomically stores complete format-owned rows only for UIDs in the captured current snapshot. Youth Tracker values are all-time career appearances, goals, assists, and caps and remain save-owned. Moneyball values are snapshot-owned: each non-empty import upserts its matched players into the cumulative cohort for the captured current snapshot. Included rows fully replace that player's format-owned fields, including nulls; omitted rows remain unchanged. The transaction then recomputes and persists full-cohort percentiles for every resulting row. Empty and zero-match imports are successful no-ops. Migration v18 preserves all v17 Moneyball rows in the unread save-owned `player_moneyball_stats_legacy` quarantine; it does not infer a source snapshot, and new imports never use the quarantine. Snapshot deletion removes current-format Moneyball rows; save deletion removes both formats. CSV data never creates a player or replaces a memory-owned field. React `features/csv-import` exposes the Moneyball upload in Search and the shared Moneyball upload path in My Club Squad, alongside Youth Academy upload. Browse uses the dialog plugin; an open modal temporarily listens for one native Tauri WebView drop path. Each choice or drop carries its opening context generation, so a stale result or delayed drop cannot write or restore feedback after a save or snapshot replacement. The UI never displays the selected path; a successful Youth import invalidates Academy, while a Moneyball import invalidates Search and player Moneyball queries. The My Club Squad path trusts the Football Manager export and does not filter by managed-club membership. The main capability remains `dialog:allow-open`; native WebView drag events need no extra capability. See [csv-enrichment-persistence](./features/completed/csv-enrichment-persistence.md) and the parser foundation in [csv-player-reconciliation](./features/completed/csv-player-reconciliation.md).

**Staff shortlist import:** `features/csv_import::staff_shortlist` separately imports semicolon-delimited staff CSVs with `Unique ID`, `Preferred Job`, `Club Job`, and `Coaching Qualifications`. It accepts up to 10,000 rows within the same 8 MiB trusted-file bound, ignores non-authoritative CSV columns, and parses outside the database lock. The import captures and revalidates active save and current-snapshot context, matches exact staff UIDs, rejects a zero-match upload, and atomically replaces only that save's shortlist rows. `list_staff_shortlist` joins those save-owned rows to the effective current snapshot, applies Preferred Job and unemployment predicates before paging, and returns the CSV metadata only in Shortlist scope. React owns the context-bound native-picker flow, replacement warning, success summary, filters, and derived score-column views. A context change closes the modal and suppresses its late result. See [Staff Shortlist CSV Enrichment](./features/completed/staff-shortlist.md).

**Role scoring:** Rust `features/scoring` owns a static FM26 IP/OOP catalog (79 roles), `score_role`, `combine_role_scores`, and the pure CA-to-PA visible-attribute projection. The projection returns current attributes unchanged for players aged 29 or older. The compact `features/player_metrics::compact` and `features/staff::scoring` writers persist one current-only projected visible-attribute JSON map on `players` alongside one `player_role_metrics` row (79 current + 79 potential) per current player and one `staff_role_metrics` row (21 roles) per current staff, using the v38 inventory plus additive v40 columns (`SCORE_MODEL_VERSION=2`, `PROJECTION_MODEL_VERSION=2`). Sparse source omissions become null after supplied visible-domain validation. The writer projects each player once and derives every role from that map. Ingest preparation occurs outside the Db mutex; the final transaction, current-snapshot selection and demotion, deletion promotion, and supported boosts own these writes atomically. Catalog, compact writer/ingest, Profile, Search resolver, Planner options/tactic lanes, and frontend display mirrors all consume all 79 roles. All 88 Moneyball presentation definitions map to known attribute roles. Tactic coverage is exactly 119/129 with ten intentional uncovered pairs: holding_wing_back_oop+DL/DR, pressing_wing_back_oop+DL/DR, box_to_box_midfielder_ip+MC, box_to_box_playmaker_ip+MC, deep_lying_playmaker_ip+MC, second_striker_ip+ST, wing_back_oop+DL/DR. See [role-scoring-engine](./features/completed/role-scoring-engine.md), [potential-role-scores](./features/completed/potential-role-scores.md), [completed record](./features/completed/ingest-potential-scores.md), [ADR-0026](./decisions/0026-eager-current-potential-scoring.md) (superseded), [ADR-0027](./decisions/0027-scoped-potential-read-validation.md) (partially superseded), and [ADR-0028](./decisions/0028-compact-current-snapshot-metrics.md).

**Player search:** Rust `features/search` owns `search_players` and `suggest_players` — parameterized SQLite queries against the active save's current snapshot (`players` with `potential_attributes_json` and one `player_role_metrics` row per current player holding 79 nullable current and 79 nullable potential role columns plus model versions). The shared `features/player_metrics` resolver validates closed catalog IDs via safe snake_case mapping to the checked-in compact columns and decodes typed dynamic values. Completeness is validated against the compact row/version contract before reads; no materialization or repair occurs on read. An optional Moneyball query mode joins only current-snapshot rows with persisted percentiles. It accepts only the closed Moneyball metric, context, and version-1 role catalogs. Raw values and individual metric percentiles remain separate from nullable derived role scores. Full CSV mode composes roles from persisted import percentiles. Filtered mode computes the required metric percentiles over the complete comparison cohort in Rust, then scores only requested roles; role filters run after scoring, and a mixed OR role filter uses the full import as its comparison cohort. Role sorting, null-last ordering, totals, and pagination occur after this bounded post-score path. No derived Moneyball role-score cache or migration exists, so replacing an import changes later results directly. **Shortlist** is a third view (`General → Moneyball → Shortlist`) that reuses the General read model and metric catalog but restricts rows to the exact current-snapshot Moneyball cohort via `INNER JOIN player_moneyball_stats` on `(snapshot_id, player_uid)` without a `percentiles_json` gate; no new table, column, or migration exists. Search and Squad result ownership stays in committed/requested Query observers; route loaders do not start player-page result reads. Sort replacements retain committed rows, while supported app-owned context mutations cancel and remove the exact Search/Squad player-page roots before Tauri and block result controllers through owner refresh or error. React `features/search` owns the `/search` route, its General, Moneyball, and Shortlist tabs (keyboard-operable tablist in General, Moneyball, Shortlist order; selecting any view clears filters and resets to that view's default sort), the shared full-height virtualized player table, the categorized metric picker, the compact filter strip + staged editor modal, and top-bar global name search. Moneyball Search uses its own persisted layout and closed catalog of identity/context, raw Moneyball metrics, and optional role fields; the role fields use ScoreBadge while unavailable values render `—`. It offers Full CSV or filtered-cohort scoring without pagination. Shortlist keeps an independent persisted layout (`shortlist`, schema v6) with General default columns and `CA` descending, shows Moneyball-free General columns/filters/sorts only, maps row activation to `view=general`, and distinguishes empty states solely from `total` and `filters.length` (unfiltered `total === 0` → neutral “No shortlist yet” guidance to Moneyball, filtered `total === 0` → “No players match these filters” regardless of cohort emptiness). A valid `view` URL parameter wins; an absent view uses `useMoneyballPreferences` (General or Moneyball only) for both loader queries and first render. General, Moneyball, and Shortlist Search rows write their active view explicitly, with Shortlist mapping to General for profile navigation, while top-bar global search leaves it absent. Filter rules compile to a flat AND|OR AST in Rust; applied filters, combine mode, view, pool, and sort live in TanStack Router search params, while each table's ordered visible metric IDs and widths persist separately in Zustand. See [player-search](./features/completed/player-search.md), [configurable-player-tables](./features/completed/configurable-player-tables.md), [Moneyball Role Scores](./features/completed/moneyball-role-scores.md), and [Player Shortlist](./features/completed/player-shortlist.md).

Player-table sorting uses migration v33's six directional PA/Age/Value indexes and managed-club membership index while retaining Name and CA indexes. Current-role, potential-role, and Club DNA sorts use missing-preserving relation ordering; Club DNA remains null-last with UID ties. Potential-role rows are complete for the effective current snapshot and are read-only at query time.

**Search tactic columns:** `Search` General, `Moneyball`, and `Shortlist` can show 11 synthetic tactic lanes (`tactic_current.*` / `tactic_potential.*`) as optional display-only columns. Lane identity, prefixes, and the 11 lane allowlist live in neutral `src/utils/tactic-ids.ts` (single source; `src/features/planner/types/tactic.ts` re-exports it). The route passes ordered lanes and `laneLabels` into Search components; shared code and stores never import features, and Search helpers never import Planner. Layouts persist per `tableId` (`search`, `moneyball-search`, `shortlist`) in `usePlayerTableStore` via one atomic `replaceLayout` that validates synthetic IDs, prunes widths, and keeps the surviving tactic block contiguous at the far right; Squad has no tactic columns and tactic IDs never enter filters or the metric picker. Tactic IPC is immutable save-context scoped (`{ saveId, contextToken }`) with Rust token validation and matched-snapshot gating, so no `get_planner_tactic` fires without a matched snapshot. Rust owns scoring: blended IP/OOP `combine_role_scores` with familiarity and foot adjustments, `base_position` normalization, deterministic Moneyball mapping via `(attribute_role_id, base_position)` (10 uncovered combos render as `"—"`), and null-last sort with `uid` tie-breaker.

**Player profiles:** Rust `features/player` owns `get_player`, the shared `set_hidden_information_revealed`, and the UID-only `boost_current_ability` and `boost_wonderkid_mentality` commands. `get_player` reads the active save's shared reveal preference with its current snapshot; players aged 29 or older receive current visible attributes and role scores as their potential values. The explicit setter updates only that active save and returns its persisted value. React invalidates `playerKeys.all` after a successful setter, and keys pending or error feedback by both player UID and active save ID. This is a presentation preference, not an authorization boundary: the DTO can contain complete player data, but concealed values do not render. The profile has four canonical tabs. Outfield players default to Outfield, which groups Technical, Mental, and Physical with Set Pieces under Technical. Players with GK familiarity of at least 15 default to Goalkeeping, which includes goalkeeper attributes, First Touch, Passing, Technique, Mental, and Physical; their Outfield tab holds the remaining Technical attributes and Set Pieces. Legacy visible-group URL values normalize to Outfield. The compact summary renders four fixed summaries — Current IP, Current OOP, Potential IP, and Potential OOP — after filtering roles to positions with familiarity at least 15 and partitioning catalog metadata by phase. Potential summaries use concealed placeholders when the preference is off. Concealment also omits PA, projected attributes, potential role scores, Hidden and Personality values, and development actions whose states disclose those values. The pitch filters bounded role rows by exact catalog position tag and shows both sortable score bases only while revealed. The route is `/players/$uid`; its validated `tab` search param selects the attribute group. A valid analysis `view` URL parameter wins; an absent view uses the shared app-local default for both loader data and render. Shared **ScoreBadge** (`table` / `card` / `hero` / `muted`) remains in `src/components/ui/score-badge/`. Search row activation writes its active analysis view, while GlobalPlayerSearch and a direct profile URL leave it absent so the configured default applies. See [player-profiles](./features/completed/player-profiles.md), [potential-role-scores](./features/completed/potential-role-scores.md), and [Player Profile Information Controls and Layout](./features/completed/player-profile-information-controls.md).

**Staff workspace and profiles:** Rust `features/staff` owns UID-only `get_staff` for the active save's effective current snapshot. It returns staff identity, employment and contract fields, the complete current attribute map, all 21 catalog-labelled current job-fit scores in stable catalog order, and the same save-scoped reveal preference used by `get_player`. The bounded Search, managed-club Staff, and save-owned Shortlist commands all read current snapshot staff; only Shortlist joins save-owned CSV metadata by UID. React `features/staff` owns Staff Search at `/staff` and the profile outlet at `/staff/$uid`; the canonical My Club route composes the managed-club Staff and Staff Shortlist views. Staff Shortlist persists its All jobs layout separately, filters by exact Preferred Job and unemployment, and derives role-score columns without changing the saved layout. A mapped job, including Manager, shows its single score and ranks it descending; Coach shows six outfield coaching scores without choosing a score sort; unrecognized jobs show no added score and rank by CA. Migration v36 performs a one-time delete of saved `staff_assignment_targets` rows while retaining saves and Planner state. Migration v37 converts any configured Set Piece Coach target to one Club slot and removes its squad rows. Rust owns the exact 28-pair target catalog, section and maximum metadata, Club semantic persistence, the canonical Configure/result order, 0/1 club-wide and lead bounds, 0/50 squad count bounds, transactional complete replacement, and the 959-result limit. Its allocator uses the approved Preferred Job score pools, lead-before-ordinary allocation, and Recruitment Analyst support. Coach counts use General at 1, add Goalkeeping at 2, add Fitness at 3, fill through 8 with General, then repeat Goalkeeping, Fitness, and six General. A compact supported-bound matcher selects General coaches by cardinality, score, and lexicographic order. It enforces global UID uniqueness and returns typed Coach requirement and vacancy evidence. React consumes typed metadata and keeps only presentation and collapse state; Rust and IPC remain authoritative. `/staff/$uid` presents the compact summary, Coaching/Mental/Knowledge current attributes, an internally virtualized catalog-ranked Role fit list, shared concealment control, and fixed +10 Boost CA action. Staff attributes use the shared FM-scale four-tier value treatment; available job-fit values in the profile and all three staff tables use the shared four-tier `ScoreBadge` ramp. Missing attributes and scores remain neutral and never become zero. The command does not redact PA or any attribute; the profile owns presentation concealment and excludes player-only pitch, potential, and Wonderkid surfaces.

**Youth Academy:** Rust `features/academy` owns save-scoped `academy_classes`, `academy_memberships`, and one-to-one `academy_member_outcomes`, plus typed commands for class, membership, and outcome mutations, candidate eligibility, and current-snapshot member resolution. Every save gets one protected automatic Class of 2025. A trusted date on the effective current snapshot can add or promote its observed-year class during ingest or current-snapshot promotion; storing a non-current snapshot does not create a class, and existing classes and memberships remain save-scoped. New memberships require an exact current-club match with the save's managed club; existing memberships retain UID and last-known name across snapshot changes. Academy joins optional Youth career enrichment by the membership's save and UID. It reports all-time career appearances, goals, assists, caps, and graduate status, even when the current player is unresolved. Current identity fields remain memory-owned. An optional manual outcome records a sale (buying club plus non-negative whole-euro fee) or release; it never derives from snapshot data, can be cleared by restoring the player to Still at club, and cascades when its membership is removed. React `features/academy` owns the `/academy` route's typed Academy IPC/query layer, URL-backed Overview / Class / Graduates workspace shell, class creation and destructive deletion flow, first-use states, a searchable managed-club candidate picker, the current/departed/unresolved class roster with assignment, outcome correction, and removal, nullable statistic cards, and reported all-time career statistics. The candidate picker and class roster display only positions with familiarity 16 or higher, ordered strongest-first with canonical pitch-order ties. A player is a graduate with at least one reported career appearance. Aggregates and the Graduates workspace remain unavailable when required career data is missing. A class view with existing classes but no or invalid `classId` returns to Overview. Summary presentation operates only on the bounded, typed member DTOs; it does not access SQLite or recreate persistence and candidate-eligibility rules.

**Managed club:** Rust `features/managed_club` owns the save-scoped `managed_club_settings` row, exact current-snapshot club discovery, missing-selection status, and the `get_managed_club`, `list_managed_club_options`, and `set_managed_club` commands. The latest effective snapshot is authoritative. Squad, Planner, Academy, and managed-club Staff require an exact current-club match; Staff Shortlist remains save-owned and is not managed-club filtered. Planner shares that whole player pool across its Senior, Reserves, and Youth categories; imported `team_level` remains optional FM metadata and does not control Planner eligibility. Managed-club status retains the count of null or unsupported team-level values as diagnostic metadata, but My Club does not display it because users cannot act on it. The saved club survives snapshot replacement and remains visible when absent. React `features/managed-club` owns the single selector in the My Club header at `/my-club#managed-club`. Settings keeps save, snapshot, and bridge management; `/settings#managed-club` replaces itself with the canonical My Club anchor. App-shell save switching and Load Data invalidate managed-club and downstream membership queries.

**Club DNA:** Rust `features/club_dna` owns one save-owned definition, while `features/player_metrics` owns the pure scorer and persisted nullable score rows. Migrations v31 and v32 store the definition version and `club_dna_scores` identity `(snapshot_id, uid, definition_version, score_model_version)` with cascades. Definition create/edit eagerly rescoring every player in every retained snapshot is one atomic transaction. Snapshot ingest scores each new player inside the ingest transaction when a definition exists. A separate boost-reconciliation transaction recomputes the affected player and maps an eager failure to `SnapshotSync` for Load Data recovery. Snapshot promotion performs no backfill. Search and Squad read the fixed score only from exact persisted rows: Search supports display, filter, and sort; Squad supports display and sort. Sort keeps null values last with UID ties. The WebView does not calculate scores. Club DNA has no lazy materializer, cache invalidation path, background job, or frontend scoring. The My Club action crosses the context-token IPC boundary and Rust remains authoritative for save identity and validation.

**Planner tactic:** Rust `features/planner` owns eleven ordered, save-scoped `planner_tactic_lanes` rows. Each lane links compatible IP and OOP positions and roles, owns a 0–1 IP weight, one unique optional importance rank from 1 through 11, and a preferred-foot rule. Central placements store an explicit right, centre, or left value (`DCR` / `DC` / `DCL`, `DMCR` / `DM` / `DMCL`, `MCR` / `MC` / `MCL`, `AMCR` / `AMC` / `AMCL`, or `STCR` / `STC` / `STCL`); role validation and optimizer familiarity normalize each placement to its catalog base position. A qualified placement can appear only once per phase. Existing repeated base positions normalize on read with their prior visual order and persist as qualified placements on the next save; legacy groups larger than three remain loadable but must be resolved before save. This extends the existing text values and requires no schema migration. Migration v8 resets only tactic rows and removes the obsolete tactic parent table; migration v9 adds the nullable unique rank; migration v10 adds preferred foot (`any`, `left`, `right`, or `both`) and a Preferred or Strict mode. Planner assignments remain because they reference stable lane IDs. `get_planner_tactic` seeds a validated 4-3-3 DM In-Possession / 4-1-4-1 DM Out-of-Possession tactic; `get_planner_tactic_options` exposes catalog-backed placements and phase/position metadata; `save_planner_tactic` validates and replaces the complete lane set. React `features/planner` loads the tactic and options through TanStack Query. The editor composes a pitch area with one selected-position inspector: pitches render and select linked tactical positions, while the inspector owns global position settings and the visible phase position and role controls. All linked phase edits and selected-position drafts remain local until save.

**Planner depth and optimizer:** Rust `features/planner` owns save-scoped `planner_teams`, `planner_strings`, and `planner_assignments`. Migration v7 adds assignment provenance: existing rows and manual assign or move mutations are `manual`; optimized rows are `optimizer`. Migration v28 adds `planner_teams`: row presence makes the stable `senior`, `reserves`, or `youth` category available and stores its bounded display name. Existing saves receive all three canonical rows. A save with no configuration initializes all three rows and one string for each; later reads never recreate a removed category. `save_planner_teams` validates and atomically replaces the one-to-three category/name configuration. One combined impact and confirmation covers assignments and staffing targets before a team is removed. Confirmed removal atomically deletes that team's targets, strings, assignments, and team row while retaining targets for Club and other teams; restoring the category adds its canonical display name and one empty string with zero staffing targets. Depth, picker, and string mutations reject unavailable categories. Player UIDs remain unique across a save, and assignments resolve against the active snapshot. A resolved assignment has current identity, a current combined score, and a potential combined score read from persisted lane-role scores and combined with the lane's IP weight; an outside-pool assignment still resolves but no longer matches the managed club; an unresolved assignment retains its last-known name when its UID is absent. Missing phase scores remain unknown. `optimize_planner_depth` accepts a validated `current` or `potential` score basis and runs one database transaction: it retains manual rows, removes earlier optimizer rows, then allocates eligible players from the shared exact-managed-club pool for available teams and ordered strings in canonical Senior, Reserves, Youth order. Existing category age limits still apply: Senior has no age limit, Reserves allows players through age 23, and Youth through age 18. Current candidates use persisted role scores; potential candidates use persisted potential role scores before both modes share ranked allocation, exact matching, foot handling, replacement, global player uniqueness, and rollback. `clear_planner_depth` requires confirmation and removes every assignment in the active save in one transaction. `get_planner_depth` returns available teams in canonical order with their display names, while `get_planner_slot_candidates` remains current-score-only and ranks matching managed-club players by the Rust-computed lane-weighted combined score with any current assignment location.

React `features/planner` owns query, picker, confirmation, focus, menu, and presentation state. React `features/squad` owns the Squad adapter for the shared player table and its typed IPC query layer. The `/my-club` route owns validated URL-backed `view` plus independent `squadSort`/`squadDir`, `staffSort`/`staffDir`, and Staff Shortlist sort and filter state for five workspaces: Squad, Planner, Tactic, Staff, and Staff Shortlist; `/planner` replaces itself with the equivalent My Club URL. The canonical route defaults to Squad and CA descending and keeps the Planner depth, Tactic, managed-club Staff, and Staff Shortlist components mounted inside labelled hidden tab panels after the snapshot is available. When a snapshot exists, the route composes the single managed-club selector at `/my-club#managed-club` and invalidates the established Planner, Academy, and Staff roots after a successful selection. The Squad overview uses the same validated metric catalog and dynamic-value DTO as Search, keeps an independent persisted ordered column layout and widths, and requests bounded virtual pages through the shared full-height table. Its committed/requested Query observers retain rows only for sort replacement; route loaders do not own player-page result reads. It has no filters, but its sort may use any sortable metric, and the whole row opens the player profile. Unconfigured Squad, Academy, and managed-club Staff recovery links target the My Club selector. Settings retains current-context invalidation for save and snapshot changes, and its old managed-club anchor redirects to My Club. Workspace changes replace the current search state and do not change Planner data or mutation ownership. The depth workspace derives grouped headings and keyboard-operable tabs from available teams. Its **Manage teams** Modal edits only the stable three-category set and save-scoped display names, requires confirmation for populated removals, and reconciles depth and candidate caches after success. If removal changes the selected category, React selects a remaining canonical category and moves focus to its tab or the management action. Both Optimize squads and Optimize by potential actions remain beside Manage teams and Clear all in the shared toolbar. Assigned cells show distinct accessible Current and Potential scores, while the two optimizer actions report basis-specific pending, success, and error states. The matrix keeps sticky position and multi-row header context, readable string widths, compact rows, and bounded horizontal and vertical overflow; header menus remain available by button or right-click. Optimize and Clear all preserve their pending/error behavior and reconcile the returned depth cache and slot candidates, while the workspace exposes one latest successful-action status. Picker and string mutations reconcile the depth cache and invalidate candidate queries; tactic saves invalidate both because roles and weight change their results. Load Data, active-save changes, and managed-club saves invalidate the entire Planner query tree. React displays Rust-provided unresolved, outside-pool, and unknown-score states without recomputing domain values. Squad boost commands accept a command-scoped typed Tauri channel and report Rust-derived cohort progress after each terminal player outcome; channel delivery remains best effort and the final result remains authoritative.

**Auth:** None — FM ValueScout is a local desktop application with no account or remote authorization boundary.

**Distribution:** One unsigned Windows x64 NSIS installer and checksum. An explicit release-preparation PR changes `release-preparation.json`; after it merges, the Release workflow waits for that exact `main` Check, then builds, verifies, and publishes the exact-SHA release. Ordinary and version-tag pushes are not publication triggers. See the repository-local [`create-release` skill](../.pi/skills/create-release/SKILL.md).

**Testing:** Vitest + jsdom + React Testing Library with `mockIPC` (`./scripts/dev test`); Playwright smoke with IPC stub (`./scripts/dev smoke`, `e2e/smoke.spec.ts`); Rust unit tests (`cargo test` inside `./scripts/dev check`); C# bridge unit tests (`./scripts/dev bridge-test` in Windows CI)

**Client env validation:** not shipped in the template default — forks can add `src/config/env.ts` with Zod for `VITE_*` when needed (follow the Vite reference in the installed `coding-standards` skill; `.env.example` documents optional variables)

**Lint / format / types:** Biome + `tsc -b`; secretlint in `./scripts/dev check`; Rust `cargo fmt`, `clippy`, and `test` in the same gate

**Secret scanning:** secretlint (`./scripts/dev secrets`, included in check)

**Observability:** None in the template default

```text
┌─────────────────────────────────────────────────────────────┐
│  Tauri WebView — React 19 components + Tailwind v4          │
├─────────────────────────────────────────────────────────────┤
│  TanStack Router — routes, search params, loaders           │
├─────────────────────────────────────────────────────────────┤
│  TanStack Query — IPC result cache                          │
│  Zustand — client UI state                                  │
│  useState — local widget state                              │
├─────────────────────────────────────────────────────────────┤
│  src/lib/tauri-client.ts — sole invoke wrapper              │
├─────────────────────────────────────────────────────────────┤
│  IPC (invoke) — frontend/backend boundary                   │
├─────────────────────────────────────────────────────────────┤
│  Rust — features/<name>/commands.rs → service.rs → db/     │
│  SQLite — rusqlite migrations + queries                     │
├─────────────────────────────────────────────────────────────┤
│  Vite 8 — WebView bundle; Tauri — native shell + installers │
├─────────────────────────────────────────────────────────────┤
│  Vitest + mockIPC — unit/component tests                    │
│  Playwright + IPC stub — browser smoke                      │
│  cargo test — Rust unit tests                               │
│  Biome + tsc + secretlint + cargo fmt/clippy — gate         │
│  scripts/dev — stable product-test command surface          │
└─────────────────────────────────────────────────────────────┘

Fork chooses: auth, signing, auto-update, additional plugins
```

**Architecture rules:**

- **Thin frontend, thick backend** — React owns UI and presentation; Rust owns computation, aggregation, file/DB I/O, and validation at trust boundaries.
- Put **async data from IPC** in TanStack Query, not in Zustand.
- Put **URL-shareable state** in TanStack Router search params when practical.
- Put **client UI state** in Zustand only when it does not belong in the URL or Query cache.
- Organize **product code in `src/features/`** on the frontend and **`src-tauri/src/features/`** on the backend; keep `src/app/routes/` thin.
- **Do not import across features** — compose features in route files.
- **One invoke wrapper** — `src/lib/tauri-client.ts` is the sole `invoke` import site; feature `api/` folders call through it.
- **No WebView SQL** — do not use `@tauri-apps/plugin-sql` from JavaScript for product features.
- Use `./scripts/dev` for test and check commands — do not bypass with ad-hoc npm scripts in CI. `check-app` is the frontend-only CI gate; `check` remains the full local gate.

---

## 2. Project Layout

### 2.0 Repository layout

```text
your-repo/
├── .pi/               # Project Pi settings and repository skills
├── .wiki/             # Durable docs (this file, ADRs, TODO)
├── .husky/            # Git hooks (pre-commit → check-fast + conditional check-rust)
├── scripts/
│   └── dev            # test | check | bridge-test | format | smoke | mutate | bridge-install
├── bridge/            # C# BepInEx FM26 plugin (see bridge/README.md, DUMP_SCHEMA.md)
├── src/               # WebView frontend (see below)
├── src-tauri/         # Rust backend + Tauri config (see below)
├── public/            # Static assets served as-is
├── index.html         # Vite HTML entry
├── e2e/               # Playwright smoke specs (excluded from Vitest)
├── .env.example       # Documented optional VITE_* variables
├── playwright.config.ts
├── vite.config.ts     # Vite + React + Tailwind + Router + Vitest + Tauri
├── biome.json         # Lint, format, import zones
├── tsconfig.json      # TypeScript project references
├── package.json       # pnpm scripts → scripts/dev + pnpm tauri
├── pnpm-lock.yaml     # Locked dependency tree
├── AGENTS.md          # Development contract
└── README.md
```

Frontend source follows Bulletproof React: features-first, unidirectional imports, app shell for routing.

```text
src/
├── app/                    # Application shell — compose features here
│   ├── routes/             # TanStack Router file routes (thin wiring)
│   ├── components/         # App-shell UI (AppShellLayout, AppNavRail, AppTopBar, not-found)
│   ├── provider.tsx        # Global providers (Query, Router)
│   └── router.tsx          # Router factory when needed
├── features/               # Primary code home — one folder per feature
│   └── <feature>/
│       ├── api/            # queryOptions, IPC fetchers, mutations
│       ├── components/
│       ├── hooks/
│       ├── stores/         # Feature-scoped Zustand when needed
│       ├── types/
│       ├── utils/
│       └── assets/
├── components/             # Shared UI — ui/ (Button, Panel, StatusChip, EmptyState, field/), error-boundary/
├── hooks/                  # Shared hooks
├── lib/                    # tauri-client.ts (sole invoke wrapper)
├── config/                 # Env exports, app constants
├── types/                  # Shared app types
├── utils/                  # Shared presentation helpers (format.ts)
├── assets/                 # Static imports (images, fonts)
├── stores/                 # Global UI Zustand only
├── testing/                # Vitest setup, mockIPC helpers
├── styles/                 # Global CSS, Tailwind @theme → DESIGN.md
├── main.tsx                # Entry — mount app
└── routeTree.gen.ts        # Generated by TanStack Router plugin (do not edit)
```

Rust backend follows feature modules with shared database helpers:

```text
src-tauri/
├── Cargo.toml              # Rust dependencies (rusqlite bundled, etc.)
├── build.rs                # Tauri build script
├── tauri.conf.json         # Product identity, CSP, build hooks
├── capabilities/
│   └── default.json        # Deny-by-default ACL for the main window
├── icons/                  # App icons for installers
└── src/
    ├── main.rs             # Thin entry — calls lib::run()
    ├── lib.rs              # App shell — plugins, setup, invoke_handler only
    ├── db/
    │   ├── mod.rs          # DB path resolution, open + migrate, APP_DB_FILE
    │   └── migrations.rs   # PRAGMA user_version migration registry
    └── features/
        ├── mod.rs
        └── <feature>/
            ├── mod.rs
            ├── commands.rs # #[tauri::command] handlers
            ├── service.rs    # Business logic, rusqlite queries (when I/O appears)
            └── …             # e.g. memory_read/dump_validation.rs for dump ingestibility checks
```

**Import alias:** `@/` → `src/` (declared in `vite.config.ts` and `tsconfig.json`).

**Dependency direction (frontend):** shared (`components`, `config`, `hooks`, `lib`, `types`, `utils`, `stores`) → `features` → `app`. No reverse imports.

**Naming:** kebab-case files and folders; PascalCase component exports. Frontend feature names match backend feature folders when both sides exist (`snapshot` ↔ `snapshot`).

### 2.1 Source layout rules

| Rule | Enforcement | Effect on code |
| --- | --- | --- |
| File routes under `src/app/routes/` | TanStack Router plugin | New pages add route files; `routeTree.gen.ts` updates on build |
| Feature code under `src/features/<feature>/` | Convention + review | Product logic colocated per feature |
| Query options and IPC fetchers in `features/<feature>/api/` | Convention + review | Single invoke wrapper in `lib/tauri-client.ts` |
| No cross-feature imports | Biome zones + reviewer | Compose features in `app/routes/` |
| Unidirectional imports (shared → features → app) | Biome zones + reviewer | Predictable dependency flow |
| Global UI Zustand in `src/stores/` | Convention + review | Feature UI stores in `features/<feature>/stores/` |
| Shared UI in `src/components/` (especially `ui/`) | Convention + review | Route files stay thin |
| kebab-case file and folder names | Biome + reviewer | `discussion-list.tsx`, not `DiscussionList.tsx` |
| No barrel `index.ts` re-exports | Convention + reviewer | Direct imports for tree-shaking |
| Design tokens in CSS `@theme`, sourced from DESIGN.md | Convention + review | Prefer token classes over ad-hoc hex in components |
| Vitest setup and mockIPC in `src/testing/` | Convention + `vite.config.ts` | `setup.ts` registers IPC mocks |
| Rust commands in `features/<name>/commands.rs` | Convention + review | Not as bare `#[tauri::command]` in `lib.rs` |
| Business logic in `features/<name>/service.rs` | Convention + review | Commands stay thin; services own rusqlite queries |
| Shared DB helpers in `src-tauri/src/db/` | Convention + review | Path resolution, connection, migration registry |

### 2.2 State and reactivity patterns

- **Component-local state** — `useState` / `useReducer` for state inside one component (toggle, open section).
- **Derived state** — compute in the component, or derive in a Zustand selector; do not duplicate Query cache in Zustand.
- **URL / route state** — TanStack Router params and validated search params (filters, tabs, shareable view state).
- **Server / remote state** — TanStack Query (`useQuery`, `useMutation`, query options). Route loaders call `queryClient.ensureQueryData` or `prefetchQuery` to seed the cache before render. Fetchers call `invokeCommand` — not HTTP.
- **Client-only shared state** — Zustand (nav rail expansion, command palette, ephemeral multi-step UI before submit). `useLayoutStore` persists `railExpanded` across launches.
- **Form state** — local state for trivial fields. Add React Hook Form + Zod when the first non-trivial form arrives (not shipped in the template).
- **Low-velocity global** — React Context for theme or auth display snapshot; not high-frequency updates.
- **Side effects** — React `useEffect` for non-data subscriptions; Query handles fetch lifecycle; Router loaders handle navigation-time prefetch.
- **Devtools** — `@tanstack/react-query-devtools` and `@tanstack/react-router-devtools` render only when `import.meta.env.DEV` is true.

### 2.3 Interface contract (fork boundary)

The template ships IPC commands as the frontend/backend contract. Forked projects define contracts in matched feature folders:

- Frontend: query options name the command, key, and stale behavior in `features/<feature>/api/`.
- Backend: `#[tauri::command]` handlers in `src-tauri/src/features/<feature>/commands.rs` return bounded DTOs.
- Types live in `features/<feature>/types/` on the frontend and as Rust structs in commands or `types.rs` on the backend.
- Mutations invalidate Query keys explicitly — document cross-key invalidation in the feature ledger when one mutation clears multiple caches.
- Validate inputs in Rust `service.rs` or commands — the WebView is untrusted.

---

## 3. Build, Test, and Gate Pipeline

### 3.1 Build commands

| Command | Purpose |
| --- | --- |
| `pnpm install` | Install Node dependencies from lockfile; Husky hooks via `prepare` |
| `pnpm tauri dev` | **Default dev loop** — WebView + Rust IPC, real backend |
| `pnpm dev` | Frontend-only Vite dev server; IPC calls fail unless stubbed |
| `pnpm build` | Production WebView bundle to `dist/` (plain Vite — no Tauri platform env) |
| `pnpm tauri build` | Full desktop build — Vite bundle + Rust compile + OS installer |
| `pnpm preview` | Serve production WebView build locally (no Rust backend) |
| `./scripts/dev test` | Vitest (`vitest run`); no args runs full suite |
| `./scripts/dev test <pattern>` | Vitest with file or name filter |
| `./scripts/dev format` | Biome lint/format fixes (`biome check --write`), then `cargo fmt` in `src-tauri/`; optional path args forward to Biome only |
| `./scripts/dev secrets` | secretlint full-tree scan; `--staged` scans staged files only |
| `./scripts/dev check` | Code-quality gate — Biome + `tsc -b` + secretlint + Rust |
| `./scripts/dev check-app` | Frontend code-quality checks — Biome + `tsc -b` + secretlint |
| `./scripts/dev check-fast` | Fast pre-commit path — Biome + `tsc -b` + secretlint `--staged` |
| `./scripts/dev check-rust` | `cargo fmt --check`, clippy, and test in `src-tauri/` |
| `./scripts/dev bridge-test` | C# bridge unit tests; requires the .NET 6 SDK |
| `./scripts/dev smoke` | Playwright (`e2e/smoke.spec.ts`); starts Vite via `playwright.config.ts` when needed |
| `./scripts/dev bridge-install` | Build `bridge/` and copy `FmDataBridge.dll` into Steam `BepInEx/plugins` (Windows path via `FM_BRIDGE_PLUGINS` / `FM_STEAM_ROOT` / WSL default) |
| `./scripts/dev package-windows` | Windows-only non-publishing release validation: build the locked bridge from source, bundle one unsigned x64 NSIS installer, and write its SHA-256 sidecar under `.release/windows/<version>/` |

### 3.2 Validation gate

1. **Biome** — verify lint and format (`biome check`); fail on violations. Autofix via `./scripts/dev format` (also runs `cargo fmt`), not in `check`.
2. **TypeScript** — `tsc -b`; fail on type errors.
3. **secretlint** — `./scripts/dev secrets` (full tree, respects `.gitignore`); included in `check`. Optional `./scripts/dev secrets --staged` without lint-staged.
4. **Rust** — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` in `src-tauri/`; gated behind `require_rust_toolchain` (requires `cargo` on PATH).
5. **Vitest** — `./scripts/dev test`; CI runs the full suite when frontend or CI files change.
6. **Playwright smoke** — `./scripts/dev smoke`; CI installs Chromium and runs it when frontend or CI files change. Requires `pnpm exec playwright install chromium` once after install locally.
7. **Bridge tests** — `./scripts/dev bridge-test`; CI runs the C# unit suite on Windows when bridge or CI files change. Full FM attach tests remain manual on Windows.

`mutate` remains unconfigured until mutation targets exist.

### 3.3 Git hooks

Pre-commit runs a **fast local gate** (`check-fast`); local pre-merge validation runs the full code-quality gate, while CI selects applicable product suites from changed paths.

| Piece | Choice | Notes |
| --- | --- | --- |
| Hook runner | **Husky** | Installs on `pnpm install` via `prepare` script |
| Pre-commit | `./scripts/dev check-fast` (+ `check-rust` when `src-tauri/` staged) | Full-tree Biome + `tsc`; staged secretlint only |
| Code-quality gate (manual) | `./scripts/dev check` | Biome + TypeScript + full secretlint + Rust |
| lint-staged | **Not used** | Avoid split between staged lint and full gate |

Bypass for one commit: `git commit --no-verify`. Do not disable hooks globally.

### 3.4 Commit message convention

[Conventional Commits 1.0.0](https://www.conventionalcommits.org/). Load the installed `conventional-commits` skill before writing a message.

---

## 4. Configuration Files Reference

| File | Role |
| ---- | ---- |
| `package.json` | Dependencies, `packageManager`, scripts (`tauri`, `dev`, `build`) |
| `pnpm-lock.yaml` | Locked dependency tree |
| `vite.config.ts` | Vite, React plugin, Tailwind, Router plugin, Vitest, Tauri integration |
| `playwright.config.ts` | Playwright smoke — webServer, Chromium project |
| `e2e/` | Playwright specs (`smoke.spec.ts`, `tauri-ipc-stub.ts`) |
| `.env.example` | Optional `VITE_*` variables |
| `tsconfig.json` / `tsconfig.app.json` | TypeScript strict options, path aliases |
| `biome.json` | Lint and format (Biome only — no ESLint/Prettier) |
| `.secretlintrc.json` | secretlint rules (`@secretlint/secretlint-rule-preset-recommend`) |
| `.secretlintignore` | secretlint allowlist (`pnpm-lock.yaml`, `src-tauri/Cargo.lock`) |
| `.editorconfig` | LF + 2-space default; 4-space for `*.rs` (rustfmt) |
| `.husky/pre-commit` | Runs `check-fast`; runs `check-rust` when staged paths include `src-tauri/` |
| `index.html` | Vite HTML entry |
| `src-tauri/tauri.conf.json` | Product identity, CSP, build hooks |
| `src-tauri/capabilities/default.json` | Main-window capability ACL |
| `src-tauri/Cargo.toml` | Rust crate dependencies and features |
| `.github/workflows/check.yml` | CI — selects frontend, browser, Rust, bridge, and CI checks from changed paths; required `check` aggregates applicable results |
| `.github/workflows/release.yml` | Explicit Windows release publication after a `release-preparation.json` change reaches `main` and its exact `Check` succeeds |
| `scripts/dev` | Stable `test` / `check` / `check-app` / `bridge-test` / `format` / `secrets` / `smoke` / `mutate` surface |
| `.pi/settings.json` | Project Pi settings; machine package and preference settings remain global |
| `.pi/skills/create-pr/SKILL.md` | Repository ordinary pull-request preparation procedure |
| `.pi/skills/create-release/SKILL.md` | Explicit release preparation and verification procedure |
| `.vscode/extensions.json` | Recommended Biome, rust-analyzer, Even Better TOML, and Repowise extensions |
| `.vscode/settings.json` | Format on save (Biome / rust-analyzer); rust-analyzer linked to `src-tauri` |
| `.gitignore` | Build, test, and tool artifacts; generated or local Pi state; `.tanstack/` and `.repowise/` caches; `.env.*` except `.env.example`; `src-tauri/target/`; editor noise (`.idea/`, vim swap) |
| `.gitattributes` | LF for text sources; binary for images including `.icns` / `.ico` |

---

## 5. Data Flow

Product routes use feature-owned query and command paths. Each path follows the same React → IPC → Rust → SQLite boundary.

### 5.0 App shell (layout chrome)

```text
AppShellLayout (all routes via __root)
  → AppNavRail — Dashboard + Player Search + Staff Search + My Club + Youth Academy + Settings; railExpanded persisted in useLayoutStore (localStorage)
  → AppTopBar — Back, Forward, GlobalPlayerSearch (Ctrl+K / Meta+K), ActiveSaveSelect,
                SnapshotFreshnessChip, Load Data cap toggle/limit, Load Data + LoadDataOutcome banner
  → Main content — route Outlet (Dashboard, /search, /settings, …)
  → Skip link to #main-content on first Tab
```

Presentation formatters (`formatRelativeAge`, `formatAbsoluteUtc`, `formatCount`, `formatMissable`) live in `src/utils/format.ts` per [DESIGN.md](./DESIGN.md).

### 5.1 Settings read path (IPC + SQLite)

```text
User navigates to /settings
  → TanStack Router matches the Settings route and runs its loader
  → loader prefetches save, snapshot, and bridge queries without throwing section failures
  → Query fetchers invoke bounded commands through lib/tauri-client
  → Rust command reads SQLite and returns a typed DTO
  → independent Save data and Bridge sections render the query results with Tailwind and DESIGN tokens
```

### 5.2 Settings write path

```text
User changes a save, snapshot, or bridge-install value
  → feature mutation invokes its typed Tauri command
  → Rust validates the request and writes through a parameterized SQLite operation
  → on success, the feature invalidates or replaces the affected query data
  → the Settings section updates from the Query cache
  → on failure, the feature shows safe inline error feedback
```

### 5.3 Database bootstrap

```text
App startup (lib.rs setup):
  1. db::resolve_db_path joins APP_DB_FILE on app_data_dir
  2. db::open creates parent dirs, opens rusqlite Connection
  3. migrations::apply runs pending versions via PRAGMA user_version
  4. app.manage(Db(Mutex<Connection>)) for IPC commands
```

Migrations apply on open — there is no separate plugin preload step.

If a mutation must clear more than one cache key, document the invalidation map in the feature ledger.

### 5.4 Memory read path (FM26 bridge status)

```text
User opens Settings
  → BridgeStatusPanel: useSuspenseQuery(bridgeStatusQueryOptions)
  → invokeCommand("get_bridge_status")
  → Rust memory_read: resolve %LOCALAPPDATA%\fm-valuescout\fm-bridge\, parse status.json
  → Panel shows ready / missing / error / unsupported platform

Dump contract: [bridge/DUMP_SCHEMA.md](../bridge/DUMP_SCHEMA.md) schema v8 (frozen). A dump contains players plus staff, optional human-manager metadata, player-database scope, date basis, an exact nullable 15-slot position-familiarity map, and fixed nullable staff `Authority` and `Adaptability` attributes. Rust rejects stale schemas and malformed fixed maps before ingest. The scan writes `dump.json` on disk; ingest reads it in Rust (§5.5). Existing snapshots remain readable, but a new schema-v8 scan is required for complete staff scoring attributes.
```

For an unlimited concurrent reader, `PersonScanner` uses at most `min(regionCount, clamp(processorCount - 1, 1, 8))` worker-local 32 MiB buffers. Available physical memory below 2 GiB reduces that bound to two; capped or non-concurrent readers remain serial. The bridge counts requested, readable, unread, and internal-failure bytes, and fails closed when unread bytes exceed ten percent. Only then may it take one Windows PSS VA clone, after a separate 2 GiB available-commit check; cancellation, failed retries, and incomplete retries leave the prior dump and snapshot intact. `diagnostics.txt` records the source, retry count, quality, worker bound, phase timings, and aggregate memory-read volume. It can also include save-derived samples, so durable documentation uses only aggregate fields; module addresses are never emitted. Failed status errors replace machine-local paths with generic failure text before they are written.

### 5.4.1 Player and staff boost paths

The Rust client exposes UID-only `boost_current_ability` and `boost_wonderkid_mentality` commands plus the closed `boost_squad_current_ability` and `boost_squad_wonderkid_mentality` commands. Squad commands accept no player list, increment, or target values. They acquire the boost operation lease (boost + load + context gates) before capturing/freezing the distinct managed-club cohort from the active current snapshot (`player/commands.rs` `execute_squad_player_boost_with`), and process each eligible player sequentially through the one-player request path. The operation lease separates concerns: boosts exclude loads and context switches; one load is exclusive with boosts via the load gate; active-save switches can coexist with a load (load and context gates may be held together) but not with boosts; save and snapshot deletion remain boost-exclusive. This prevents a boost from racing a load or save switch without holding SQLite during bridge polling. Each verified result is reconciled into SQLite before the next request. Local ineligibility is skipped, and only a proven player-local no-write rejection can continue. A timeout, unverified rollback, unexpected bridge result, context change, or failed reconciliation latches the affected snapshot and stops the batch before another request. Both profile and Squad commands reject a latched current snapshot until Load Data establishes a new effective current snapshot.

A successful **live** full dump on exact build `26.3.2` retains one process-private candidate index keyed by its request ID. Snapshot-backed scans and plugin restarts leave no writable index. `boost-current-ability` accepts only source provenance, a UID, expected CA/PA, and a fixed increment of `5` or `10`; it caps CA at PA and `200`. `wonderkid-mentality` accepts known snapshot values for Ambition, Professionalism, and Determination, leaves null or above-threshold values unchanged, and generates any eligible `11..20` targets inside the bridge.

Before a write, the plugin resolves the exact layout, reopens the live reader, and validates the UID and expected values. It then uses typed one- or two-byte writes with readback and rollback reporting. One gate serializes dumps and boosts. Optional status fields report only verified scalar results and never include an address or UID. A proven expected-value mismatch or at-limit rejection preserves both live candidate indexes and their advertised capabilities so another player or staff request from the same scan can continue; an unsafe or uncertain failure clears both indexes. The player summary exposes the fixed **Boost CA** action and the secondary **Wonderkid Mentality** action. The Squad overview exposes confirmed **Boost all CA** and **Make all Wonderkids** actions, which report truthful partial counts and share the same recovery boundary. React previews only snapshot eligibility; the bridge chooses the random Wonderkid targets and the UI reports its verified values. After verified success, the route invalidates snapshot, search, player, Planner, and Academy query roots under [ADR-0017](./decisions/0017-action-specific-fm26-player-boosts.md).

The UID-only `boost_staff_current_ability` command uses a separate live staff candidate index and accepts no increment or target. Rust derives expected CA/PA from the effective current snapshot; the bridge computes `min(CA + 10, PA, 200)`, validates the source, identity, and values, and returns only a verified scalar result. Rust then updates only the matching current `staff.ca`; compact `staff_role_metrics` for that snapshot remain as scored from attributes (no staff metric rewrite on CA boost). `boost_my_staff_current_ability` captures the full managed-club staff cohort in Rust and applies the same closed request sequentially with aggregate progress. It skips capped staff, continues only after staff-local proven no-write failures, returns global bridge errors, and stops with partial counts when recovery becomes required. Player and staff actions share the application mutation gate and `snapshots.boost_recovery_required`. A timeout, unverified write, context change, or failed local reconciliation latches that snapshot for both action families until Load Data establishes a fresh effective current snapshot. Proven pre-write and live-value rejections do not latch it. See [ADR-0020](./decisions/0020-action-specific-fm26-staff-ca-boost.md) and [ADR-0021](./decisions/0021-sequential-club-family-staff-ca-boost.md).

### 5.5 Load Data and snapshot ingest

**Load Data** is one native async Tauri command with command-scoped best-effort progress. The dump body never crosses IPC.

```text
User clicks Load Data (AppTopBar)
  → useLoadData mutation creates a typed Channel<LoadDataProgressDto> and captures
    activeSaveContext { id, contextToken } for this invocation
  → invokeCommand("load_data", { maxAccepted, onProgress, saveId, contextToken })
      maxAccepted omitted or null = unlimited (production default)
      positive integer = diagnostic cap (UI toggle via useLoadDataPreferences)
      channel is command-scoped; backend sends invocation save ID/token with every event
  → Rust snapshot/commands::load_data (async, off the main thread):
      Acquire the load lease (`LOAD_GATE`, exclusive with boosts via `inProgress`, not with context switches) before checking whether the supplied save ID/token is still the active save; a concurrent load/boost conflict therefore rejects as `inProgress` before a stale-context `saveChanged`
      Verify invocation save ID/token is still the active save; if stale, fail with scan `saveChanged` without starting the bridge
      memory_read::request_player_dump — no Db lock during scan:
        writes request.json (30s TTL), polls status.json until terminal (120s default)
        Bridge plugin (off Unity main thread): writes scanning status → block heap scan → atomically replaces dump.json + diagnostics.txt → ready status
      Emit best-effort progress events: scan (indeterminate), preparing (determinate only when total truthful),
        scoring, saving, finalizing — ordered, phase-local, disjoint timings; failed sends ignored
  → On scan failure: LoadDataError { phase: "scan", kind, message }; prior snapshot unchanged
  → On scan success: capture dump to a private temporary path, then re-read status.json; reject if request ID or ready state changed
  → Prepare outside Db: validate dump, normalize raw rows, project attributes, and score compact
    player/staff metrics without holding the Db mutex; scoring is lock-free
  → Re-verify invocation save ID/token is still active immediately before publication under the Db lock;
    stale invocation fails with Ingest SaveChanged and publishes nothing, even though the active-save
    switch itself succeeded concurrently (load and context leases may coexist)
  → One final transaction: insert retained snapshot with nullable bridge source request ID + raw players/staff,
    compute/select the effective current snapshot by valid game date, load timestamp, then ID, demote/clear derived rows from a displaced current, and only if the newly stored snapshot is the winner write its compact `player_role_metrics`/`staff_role_metrics` rows; non-winners remain raw-only,
    ensure the valid trusted (`memory` or `derived`) in-game year's automatic Academy class only when this row is current
      On ingest failure: roll back; prior current snapshot remains
  → Returns LoadDataResult { requestId, playersFound, scanTruncated, maxAccepted, storedSnapshot, effectiveSnapshot,
      timings: { scanMs, prepareMs, scoringMs, saveMs, finalizeMs, totalMs, ingestMs } } — the five phase buckets are disjoint; `ingestMs = saveMs + finalizeMs` is a compatibility aggregate, and `totalMs` is total elapsed rather than another phase bucket
  → Frontend keeps Search and Squad mounted during the command and on failure; a successful matching current
    replacement cancels/removes the exact Search/Squad roots under a continuation guard, then schedules current-owner invalidations; mutation settlement does not await those refetches, suppressing stale progress/outcome;
    historical non-winner only refreshes history; the banner suppresses stale invocation contexts
  → Dismissible LoadDataOutcome banner in AppTopBar: stable polite text plus adjacent native <progress>
    (indeterminate for scan, determinate only when completed/total truthful); button shows phase-specific label
    with fixed width; success shows detailed disjoint timings and stored-versus-latest copy; bound to the
    captured save token and cleared/replaced by a generic busy state when that context is no longer active
  → Snapshot panels show ingest outcome (player count, truncated banner when scanTruncated) and ordered history
```

Load Data is `#[tauri::command] pub async fn load_data`;

**Saves model** (migrations v2–v40, `src-tauri/src/db/migrations.rs`):

| Table | Role |
| --- | --- |
| `saves` | App-side game save slots (not FM save files). Exactly one row has `is_active = 1` (partial unique index). Default save is created when the DB has none. Each row has an immutable internal context token used to reject stale asynchronous work and destructive targets. `reveal_hidden_information` is a constrained `0 \| 1` profile preference shared by player and staff reads and defaults to `1`. |
| `snapshots` | Retained snapshots per save, with at most one `is_current = 1` row (partial unique index per `save_id`). Stores schema/game/bridge versions, nullable game date and date basis, player database scope, scan state, player and staff counts, optional manager metadata, load time, nullable bridge request provenance, an optional custom name, an immutable internal context token, and a shared 0/1 boost recovery requirement. The shared selector orders valid dated rows greatest-first, then load timestamp and ID; undated rows follow dated rows. |
| `players` | Rows keyed by `(snapshot_id, uid)` with `attributes_json` plus current-only `potential_attributes_json` and `potential_projection_model_version`. Scalars for list/search foundations include nullable nation UID, gender, club reputation, and team type; attribute maps and arrays remain JSON text. `null` in dump JSON means unknown — never coerced to 0 on ingest. |
| `staff` | Rows keyed by `(snapshot_id, uid)`, with stable scalar metadata and one `staff_attributes_json` object. They cascade when the owning snapshot is deleted. Bounded `search_staff` and `list_my_staff` commands read the effective current snapshot; Staff Search and the My Club Staff workspace expose configurable current-snapshot tables, managed-club Staff offers one fixed +10 PA/200-capped CA boost, and `/staff/$uid` presents the current Staff Profile with shared concealment and the individual fixed action. |
| `player_role_metrics` | One row per current player per effective current snapshot: `snapshot_id, uid, score_model_version, projection_model_version` plus 79 nullable current role columns and 79 matching potential columns (0–100, validated SQL identifier mapping; 162 columns total). Historical snapshots have no row. FK to `players` with `ON DELETE CASCADE`. |
| `staff_role_metrics` | One row per current staff per effective current snapshot: `snapshot_id, uid, score_model_version` plus 21 nullable role columns. Historical snapshots have no row. FK to `staff` with `ON DELETE CASCADE`. |
| `staff_shortlist_entries` | One CSV shortlist row per `(save_id, staff_uid)`. It stores a non-empty preferred job plus raw Club Job and Coaching Qualifications strings. It belongs to the app save, not a snapshot or extracted staff row, so it survives snapshot replacement and cascades only when its save is deleted. An index on `(save_id, preferred_job COLLATE NOCASE)` supports shortlist job queries. |
| `player_youth_career_stats` | One latest all-time Youth Tracker row per `(save_id, player_uid)`: nullable career appearances, international caps, goals, and assists. It references the save, not a snapshot, so Academy can retain data for tracked unresolved members. |
| `player_moneyball_stats` | One latest Moneyball cohort per snapshot, with one row per matched `(snapshot_id, player_uid)`: nullable asking price, starts, substitute appearances, minutes, a 138-key canonical statistics object, and a parallel 138-key nullable 0–100 percentile object. Composite foreign keys require the player to belong to that snapshot; deleting the snapshot cascades the row. |
| `player_moneyball_stats_legacy` | Save-owned, unread quarantine for every v17 Moneyball row whose source snapshot was not recorded. It preserves values and import timestamps, accepts no new imports, and cascades only when the save is deleted. |
| `managed_club_settings` | One optional managed-club selection per save. Stores the exact selected club name, survives snapshot replacement, and cascades with its save. |
| `planner_tactic_lanes` | Eleven ordered, stable lanes per save. Each lane links an IP placement and role to an OOP placement and role, owns a 0–1 IP weight, an optional unique 1–11 importance rank, a preferred-foot rule, and references the save directly. Both role references are validated against the scoring catalog. |
| `planner_strings` | Ordered depth-chart strings for Senior, Reserves, and Youth. Rows reference the save, not a snapshot, and each team keeps at least one string. |
| `planner_assignments` | Save-wide unique player assignments to a tactic lane and string. Rows retain the player UID and last-known name while current snapshot resolution changes. Migration v7 records `manual` or `optimizer` provenance; legacy rows migrate to `manual`. |
| `academy_classes` | Save-scoped positive `class_year` cohorts, unique by year within a save. The automatic marker protects baseline and observed-year classes from deletion; automatic generation only adds or promotes a matching row and never replaces its memberships. |
| `academy_memberships` | One player UID may belong to one class per save. Stores last-known name and uses a composite `(save_id, class_id)` foreign key to prevent a cross-save class reference. |
| `academy_member_outcomes` | One optional outcome per save-scoped membership. `sold` stores a non-empty buying club and non-negative whole-euro fee; `released` stores neither. Its composite foreign key cascades when the selected membership is removed. |

Migration v17 adds save-owned Youth career and Moneyball tables. Migration v18 transactionally preserves all v17 Moneyball rows in `player_moneyball_stats_legacy`, then creates an empty snapshot/player-owned `player_moneyball_stats` table for new imports; it never infers a legacy source snapshot. Migration v19 adds immutable save and snapshot context tokens plus optional snapshot names, backfills existing rows, and protects tokens with unique indexes and SQLite triggers. Migration v20 adds `snapshots.player_boost_recovery_required`, defaulting existing rows to 0; a terminal profile or Squad boost reconciliation failure sets it, and later boosts reject that snapshot until a fresh current snapshot is ingested. Migration v21 originally added the disposable `player_potential_role_scores` cache; its rows are now retired. Migration v22 removes only the obsolete `demo_value` table. Migration v23 adds the constrained `saves.reveal_hidden_player_information` boolean and defaults it to `1` for existing and new saves. Migration v24 originally added `staff_role_scores`; its rows are now retired. Migration v25 renames the snapshot recovery column to `boost_recovery_required` without clearing existing values, so player and staff writes share one fail-closed recovery boundary. Migration v26 renames the save preference to `reveal_hidden_information` without changing any stored `0|1` value; player and staff detail reads share it. Migration v27 adds save-owned `staff_shortlist_entries` without backfilling or coupling its rows to a snapshot. Migration v30 adds nullable JSON percentile scores to snapshot-owned Moneyball rows without changing or scoring existing cohorts. Migration v34 added the current-only `potential_attributes_json` owner used by the compact path; migration v38 creates the compact `player_role_metrics`/`staff_role_metrics` tables with the exact 68/68/21 inventory, model checks, and cascades; migration v39 drops `player_role_scores`, `player_potential_role_scores`, and `staff_role_scores` and their indexes; migration v40 additively appends exactly 22 nullable checked generic OOP role columns (11 current + 11 potential) with no score backfill. Snapshot deletion removes current-format Moneyball rows and cascades compact metric rows while preserving save-owned Youth career and staff shortlist data; save deletion removes every child table, including both Moneyball formats and staff shortlist entries. A successful supported player boost replaces that player's projected attributes and compact current/potential metric columns in the same transaction as its source update. Migration v16 adds the nullable `bridge_source_request_id` to `snapshots`. Existing snapshots retain `null` provenance and remain readable. Migration v15 adds the schema-v6 fields and the snapshot-owned `staff` table. Existing snapshots retain null or default values where the old dump had no equivalent field. Migration v14 backfills a missing 2025 baseline for every save and promotes a matching class to automatic for a current snapshot only when its date source is `memory` or `derived` and its date is valid and at least 2025. It does not replace class identifiers or memberships, and it ignores unknown, malformed, untrusted, pre-2025, or non-current dates.

**Query and save-management IPC** (`features/snapshot/commands.rs`):

```text
Active save (AppTopBar ActiveSaveSelect)
  → list_saves / set_active_save
  → set_active_save switches which save’s current snapshot queries target

Save switcher panel (Settings route)
  → create_save / rename_save (create and rename only; switch is in the top bar)

Snapshot freshness (AppTopBar SnapshotFreshnessChip)
  → derives tone from get_current_snapshot age and scanTruncated

Snapshot overview
  → get_current_snapshot → active save’s current snapshot metadata (or null)

Settings snapshot history
  → list_snapshots(save_id) → active save metadata ordered by the shared date/load/ID comparator
  → rename_snapshot(snapshot_id, context_token, custom_name?) → organization metadata only; order and current selection do not change
  → delete_snapshot(snapshot_id, context_token) → atomic snapshot cascade and current promotion when needed
  → delete_save(save_id, context_token) → atomic save cascade, deterministic fallback activation, or blank active `Default save` recreation

The Settings route prefetches each section's owning queries without promoting one panel failure to a page failure. Snapshot/save mutations invalidate the local snapshot tree; the route-owned context callback invalidates Search, Player, Planner, Academy, and Staff consumers only when the active save or effective current snapshot changes.

```

**Managed-club IPC** (`features/managed_club/commands.rs`):

```text
User opens My Club
  → route loader prefetches current snapshot and managed-club status/options
  → no snapshot: show Load Data guidance without the selector
  → ManagedClubSelector at /my-club#managed-club reads get_managed_club and list_managed_club_options
  → save: invokeCommand("set_managed_club", { clubName })
  → Rust validates one exact current-snapshot option and upserts only that save's selection
  → selector refreshes managed-club status; My Club invalidates Planner, Academy, and Staff roots
User opens Settings#managed-club
  → replace redirect to /my-club#managed-club
  → Load Data and active-save changes invalidate managed-club and downstream membership queries
```

**Planner tactic IPC** (`features/planner/commands.rs`):

```text
User opens /my-club (legacy /planner replaces itself here)
  → route loader: ensureQueryData(get_planner_tactic + get_planner_tactic_options)
  → get_planner_tactic creates the default tactic for a save when none exists
  → save_planner_tactic receives the complete 11-lane draft with one IP weight, optional rank, preferred foot, and foot preference per lane
  → Rust rejects incomplete lanes, unknown or phase-incompatible roles, unsupported positions, lane weights outside 0–1, duplicate or out-of-range ranks, and invalid foot rules
  → planner query keys remain save-scoped and are invalidated with the rest of the planner tree on save/snapshot changes
```

**Planner depth IPC** (`features/planner/commands.rs`):

```text
User opens /my-club (legacy /planner replaces itself here)
  → route loader: ensureQueryData(get_planner_depth)
  → get_planner_depth returns available teams in canonical order, with each display name, strings,
      current assignment state, current combined score, and potential combined score
  → React renders one grouped table when the matrix container can preserve readable string widths; otherwise it renders one selected available team at a time and tabs change presentation state only
  → save_planner_teams receives the complete one-to-three available category/name set; Rust validates it and atomically deletes removed strings and assignments or creates an added category's first empty string
  → get_planner_slot_candidates(team, laneId, search) returns Rust-ranked candidates from the exact managed club
  → add_planner_string, remove_planner_string, clear_planner_assignment, assign_planner_player, and move_planner_player validate and mutate in Rust
  → optimize_planner_depth(score_basis: current | potential) returns reconciled depth after
      transactional, ordered allocation; clear_planner_depth requires confirmation and returns
      reconciled depth after clearing every assignment in the active save
  → player UIDs are unique per save; each available team retains at least one string; populated string or team removal is confirmed in React and still validated in Rust
  → successful depth mutations reconcile the depth cache and invalidate candidate queries; tactic saves invalidate both
  → Load Data, active-save changes, and managed-club saves invalidate the Planner query tree
```

`request_player_dump` remains registered for tests and low-level scan-only use; the **Load Data** button in `AppTopBar` calls `load_data`.

### 5.6 Current role scoring on ingest

Current role scores are computed in Rust during snapshot ingest — not in the WebView and not as a separate post-ingest job. Potential role scores are persisted eagerly for each effective current snapshot; product reads only validate and use those persisted values.

```text
ingest preparation (outside Db):
  → scoring::catalog::all_roles() — 79 static FM26 IP/OOP roles (SortItOutSI Key/Preferred; dump PascalCase keys)
  → staff::scoring::all_staff_roles() — 21 staff roles
  → for each player: parse attributes_json → score_role per role (current 79)
      + projection (CA→PA) → potential 79; missing source → null; scale /20×100
  → for each staff: score_all_staff_roles per current attributes
  → bounded owned PreparedSnapshot (raw + compact values) passed to final transaction
ingest transaction (one final Db transaction):
  → INSERT raw players + staff
  → compute/select the effective current snapshot; demote/clear derived rows from a displaced current; rebuild promotion from raw facts when needed
  → only if the newly stored snapshot is the winner, INSERT `player_role_metrics` (one wide row per current player) + `staff_role_metrics` (one wide row per current staff); non-winners remain raw-only

Pure helpers (no IPC yet):
  → combine_role_scores(ip, oop, ip_weight) — default 0.5; null if either input null or weight ∉ [0, 1]
```

Position suitability does not enter role scores. Planner tactic lanes persist the caller-supplied combined IP/OOP weights. Compact current-only persistence is authoritative; historical snapshots remain raw-only. Full-matrix 184k-player ingest test is `#[ignore]`; gate keeps a 2k scored ingest timing check.

#### Potential role projection

`features/scoring/projection.rs` is a pure Rust projection module. It projects nullable visible attributes from CA to PA using natural-position groups and age factors, preserves nulls, rounds and caps values at 20, and applies the result through `score_role` and `combine_role_scores`. `PA <= CA` and age 29 or older are identity projections. The compact writer validates supplied visible values in the 1–20 domain, normalizes sparse omissions to null, and stores a current-only projected JSON map on `players` alongside one `player_role_metrics` row per current player.

`features/player_metrics::compact` and `features/staff::scoring` own the closed 79/79/21 mapping to the v38 inventory plus additive v40 columns (`SCORE_MODEL_VERSION=2`, `PROJECTION_MODEL_VERSION=2`). Current selection clears non-current compact rows and materializes the winner, deletion promotion rebuilds compact rows from retained raw facts before commit, and supported player boosts replace one player's projected attributes and compact current/potential columns atomically. Search, Squad, Profile, and Planner read named compact columns. Historical snapshots retain raw facts only.

### 5.7 Player search

Search reads the **active save's effective current snapshot** only. The WebView never opens SQLite; all filtering, sorting, dynamic metric resolution, invariant validation, and bounded page selection run in Rust.

```text
User opens Search (nav rail or /search)
  → Search result controller: committed/requested Query observers for the first page
  → validateSearch normalizes sort, dir, filters[], combine in URL search params
  → SearchFilterBar — compact strip + SearchFilterEditorModal (shared Modal primitive)
  → shared VirtualizedPlayerTable — one full-height vertical scroll owner; useQueries fetches 50-row windows (offset/limit)
      as the virtualizer scrolls; total match count from IPC for scrollbar extent; fixed rows and bounded overscan
  → Whole-row click or Enter on a focused row navigates to /players/$uid;
      Arrow Up/Down move row focus within the virtualized list

search_players IPC (features/search/commands.rs)
  → offset (default 0), limit (default 50, max 200), validated sortBy, sortDir, requestedFields
  → optional filters[] + filterCombine ("and" | "or") — max 32 rules
  → filter.rs: validate field/op/value per field kind; compile FilterAst to parameterized WHERE
  → features/player_metrics/resolver.rs: validate and deduplicate the closed metric catalog via safe snake_case mapping to compact columns, build SQL expressions, and decode typed dynamicValues;
      role.* and potential_role.* read named `player_role_metrics` columns with version checks;
      attr./hidden./personality.* use json_extract; nationality uses json_each; position emits a stable strongest-first list
  → query.rs: validate compact row/version contract for requested metric kinds, then run read-only count, filter, sort, and page queries
  → Returns { players[], total }

suggest_players IPC
  → query string (trimmed; blank → []), optional limit (default 10, max 20)
  → Rank: exact name → prefix → contains (COLLATE NOCASE), then CA desc; escape_like on patterns
  → Returns { uid, name, ca }[]

GlobalPlayerSearch (AppTopBar, all routes)
  → Ctrl+K / Meta+K focus; 200ms debounce → suggest_players
  → Combobox + listbox; Escape clears input before closing
  → Selecting a hit navigates to /players/$uid

Cache invalidation: Load Data invalidates snapshot, Search, Player, Planner, Academy, and Staff query roots. Active-save switching updates the snapshot context and invalidates Search, Player, Planner, Academy, and Staff. Snapshot history deletion invalidates those current-only feature roots only when the deleted row was current and a different row becomes effective current; active-save deletion or final-save recreation invalidates the same route-owned context tree. A successful supported player boost atomically replaces the affected player's projected map and complete potential-role set, then invalidates the owning query roots.
```

**Invariants:** `null` dump values never coerce to 0 for filter or display; role scores come from Rust and SQLite (not computed in the WebView); available current and potential role-score cells use the shared four-tier ScoreBadge while missing scores and other dynamic metrics remain neutral; every requested metric and sort ID is validated before SQL construction; requested fields remain bounded and deduplicated; a potential filter or sort never runs against a partial required cohort. Player-table layout version 6 defaults Search, Squad, and Shortlist to Name, Age / DOB, Nationality, CA, PA, and Value; Moneyball Search starts with Name, Age / DOB, Nationality, Minutes, Average Rating, Goals / 90, Assists / 90, xG / 90, and xA / 90. Shortlist owns an independent persisted `shortlist` layout that migrates from version 5 without mutating `search` or `moneyball-search`. Hydration removes visible Club and Division entries and their widths only from pre-version-5 Search, Moneyball Search, and Squad layouts. Club and Division remain sortable, picker-available metrics that users can add, remove, resize, and reorder independently per table. Applied filters, combine mode, and sort remain URL state. The filter editor keeps a local draft until Done; Cancel, close, Escape, and backdrop dismissal do not apply it, while Done adds each filtered metric once and then filters and columns remain independent. Selecting any Search view clears all filters and resets to that view's default sort/direction.

Truncated-scan warning: `SnapshotFreshnessChip` in the top bar reflects `scanTruncated`; Search results count line does not yet append a cap annotation — see [player-search](./features/completed/player-search.md) follow-up.

### 5.7.1 Squad overview table

The Squad workspace uses the same `VirtualizedPlayerTable`, `PlayerTableHeader`, metric catalog, and Rust `features/player_metrics` resolver as Search. Rust retains Squad ownership of exact managed-club membership and returns bounded `list_squad_players` pages with the requested dynamic fields. Squad sort and direction remain validated in `/my-club` URL search state as `squadSort` and `squadDir`; filters are not added to the route. The six default columns are only the initial layout. The per-table Zustand store persists Squad's visible metric IDs, order, and clamped widths separately from Search. Header sorting, pointer or keyboard resize, and Move left or Move right menu actions change presentation state without changing the requested metric set or resetting the virtual window. A row click or Enter opens the player profile, and Arrow Up/Down moves focus across page boundaries without exposing Previous or Next controls.

### 5.8 Player profile read path

Profile reads the **active save's current snapshot** only. The WebView never opens SQLite; role scores are not recomputed in the WebView.

```text
User opens /players/$uid (from Search row, Enter on focused row, or GlobalPlayerSearch hit)
  → Route loader: ensureQueryData(current snapshot + get_player); view=moneyball also prefetches get_player_moneyball
  → validateSearch accepts canonical tabs and normalizes legacy technical | mental | physical to outfield;
      missing or invalid values remain unset until the loaded player determines the default; view is general unless exactly moneyball
  → Suspense fallback mirrors the summary plus two-panel workspace
  → summary remains visible; PlayerProfileTabs selects one attribute group

get_player IPC (features/player/commands.rs)
  → uid from route param
  → query.rs: SELECT player scalars + JSON attribute maps for current snapshot;
      validate this player's complete projected map and compact row version contract;
      SELECT compact `player_role_metrics` row; map 79 current + 79 potential columns via closed catalog order
      (displayName, phase, positionTags); missing/null score stays null;
      map persisted potential scores without projection or scoring;
      missing player row → null response (not-found empty state)
  → Returns PlayerDetailDto (identity, current attributes, projected visible attributes,
      hidden, personality, roleScores[] with current and potential values, and the active save's hiddenInformationRevealed preference)

get_player_moneyball IPC (features/moneyball/commands.rs)
  → uid from route param
  → query.rs resolves the active save's current snapshot and verifies the current player UID before it reads player_moneyball_stats
  → no matching Moneyball row returns noData; a row without percentiles_json returns needsReimport and hides its old raw payload
  → for a scored row, query.rs parses the player's `positions_json` familiarity map. Exact familiarity 20 selects natural positions.
  → no natural position returns raw imported context and metrics with null percentile and role-score fields plus an `unavailableNoNaturalPosition` comparison basis.
  → natural positions load one active-snapshot scored Moneyball-row set joined to player familiarity maps. Rust keeps peers that share at least one exact-20 natural position, deduplicates by player UID, calculates the existing null-aware metric percentiles in memory, then derives the 88 version-1 role scores and explanations from those values. The `available` basis returns natural positions and the unique comparison-player count.
  → persisted import-wide percentiles remain the Moneyball Search basis; profile recomputation never writes SQLite.
  → unknown UID, no active current snapshot, and older-snapshot rows return null

set_hidden_information_revealed IPC
  → explicit revealed state from the profile route
  → service updates only the active saves row and returns the persisted state
  → route invalidates playerKeys.all; pending and error feedback is keyed by player UID and active save ID

Summary
  → identity block + fixed Current IP, Current OOP, Potential IP, and Potential OOP hero summaries;
      each uses catalog-order ties after familiarity ≥ 15 filtering and phase partitioning
  → concealed preference removes PA, projected/potential values, hidden/personality values, and development actions;
      potential summary slots remain as concealed placeholders
  → preferredFoot title-cased for display
  → Boost CA and Wonderkid Mentality keep their closed confirmation and mutation flow;
      snapshot previews and disabled reasons move to focusable action tooltips

Attributes panel
  → four canonical tabs; outfield players use Outfield first, while players with GK familiarity ≥ 15
      use Goalkeeping first and default to it
  → static attribute-groups.ts membership (Technical / Mental / Physical / Goalkeeping, Hidden, Personality);
      goalkeeper profiles show Goalkeeping with Mental and Physical, move First Touch, Passing,
      and Technique into the alphabetized goalkeeper list, and keep only the remaining Technical
      attributes and Set Pieces under Outfield
  → revealed visible rows show Current → Potential from the DTO; concealed visible rows show current values only,
      while Hidden and Personality show an explicit concealed state
  → known 1–20 values map to four FM-style presentation bands; raw values remain unchanged
  → null → —

Role fit panel
  → pitch defaults to the strongest recorded position, then the best current role position
  → selected exact positionTags filter the bounded 79-role DTO in React
  → the pitch omits unsupported SW and de-emphasizes red-tier familiarity values 1–5;
      revealed role rows expose sortable Current and Potential headers; concealed rows expose Current only;
      unavailable scores stay last and catalog order breaks ties
  → revealed rows use card ScoreBadge pairs for Current and Potential; concealed rows use Current only; rolePhaseLabel maps in_possession/out_of_possession → IP/OOP

Moneyball role fit panel
  → the Rust-owned version-1 catalog defines 88 position-family-specific IP/OOP roles; each profile score is the rounded weighted mean of five natural-position-cohort metric percentiles
  → the Moneyball summary selects best playable IP and OOP scores; the position picker filters the role table, unavailable scores render `—`, and a disclosure shows metric direction, weight, percentile contribution, and catalog version. The IPC response carries the current comparison basis.
  → All 88 Moneyball presentation definitions map to known attribute roles; General profile presentation shows real scores once version-2 compact data is materialized.

Cache invalidation: Load Data invalidates snapshot, Search, Player, Moneyball, Planner, Academy, and Staff query roots. Active-save switching updates the snapshot context and invalidates Search, Player, Moneyball, Planner, Academy, and Staff. A verified player boost replaces persisted potential data atomically, then invalidates snapshot, Search, Player, Planner, and Academy. Snapshot current promotion and save replacement use the Settings route's context callback to refresh Search, Player, Moneyball, Planner, Academy, and Staff.
```

**Invariants:** `null` dump/DB values never display as `0`. One scoring model shared with Search. No cross-feature component imports — routes compose; Search/GlobalPlayerSearch navigate by route path only.

### 5.9 Bridge plugin install path

```text
User opens Settings
  → BridgePluginInstallSection: useSuspenseQuery(bridgeInstallStatusQueryOptions)
  → invokeCommand("get_bridge_install_status")
  → Rust memory_read/install.rs: resolve Steam BepInEx/plugins path, check FmDataBridge.dll presence

User clicks Install / Update plugin
  → useMutation → invokeCommand("install_bridge_plugin")
  → Rust copies bundled src-tauri/resources/FmDataBridge.dll → plugins/
  → User restarts FM so BepInEx loads the DLL

User clicks Remove plugin
  → useMutation → invokeCommand("remove_bridge_plugin")
  → Rust deletes only FmDataBridge.dll (never BepInEx core or other plugins)

Path resolution: FM_BRIDGE_PLUGINS → FM_STEAM_ROOT/BepInEx/plugins → default Windows Steam path
(same order as ./scripts/dev bridge-install). Developer build-and-copy from source stays on bridge-install.
```

Non-Windows hosts return `unsupportedPlatform` for bridge install commands. Full FM attach tests are manual on Windows. CI runs Rust, frontend, browser, and bridge checks only when their source paths or CI configuration change.

---

## 6. Testing Strategy

### 6.1 How tests are organised

- **Component and hook tests** — colocated `*.test.tsx` or `*.test.ts` beside source; Vitest + jsdom.
- **Integration tests** — feature flows under `features/<feature>/` or `app/routes/`; preferred over shallow unit tests for confidence.
- **IPC mocks** — `mockIPC` in `src/testing/setup.ts`; prefer over ad-hoc invoke stubs.
- **E2E / smoke** — Playwright in `e2e/` with `tauri-ipc-stub.ts`; `./scripts/dev smoke` runs application smoke checks. Vitest excludes `e2e/**`.
- **Rust unit tests** — `#[cfg(test)]` modules in `src-tauri/src/`; run via `cargo test` in the gate. CSV parser and import tests use checked-in Youth Tracker and Moneyball fixtures plus temporary files and SQLite databases; they cover dialect/header detection, null and malformed values, UID validation, file limits, stale context, and unchanged database state.
- **Bridge unit tests** — `bridge/Tests/` run through `./scripts/dev bridge-test` in Windows CI.

### 6.2 What each layer covers

- **Presentational components** — RTL queries by role/label; user-visible outcomes.
- **Hooks and stores** — Vitest with minimal mocks; test Zustand store actions and selectors.
- **Query logic** — test query options and IPC fetchers; `mockIPC` when integration matters.
- **Routes** — smoke critical navigation in component tests and Playwright; avoid testing framework router internals.
- **Rust services** — unit tests against temp SQLite files with migrations applied.

### 6.3 Test quality guidelines

Test behaviour the user sees, not implementation details. Do not assert on Zustand or Query internal cache shape unless the contract is the subject. Follow the testing reference in the installed `coding-standards` skill.

### 6.4 Playwright smoke scope

`./scripts/dev smoke` runs Playwright against the **Vite dev server** in Chromium, not `pnpm tauri dev`. `e2e/tauri-ipc-stub.ts` injects `window.__TAURI_INTERNALS__` before the app loads so IPC calls never reach Rust.

| Playwright smoke covers | Playwright smoke does not cover |
| --- | --- |
| Vite shell loads; TanStack Router renders home, 404, and layout chrome | Real Tauri WebView runtime or platform WebView differences |
| Application UI with stubbed IPC: app shell (nav rail with Search, Squad, and Settings; top bar with global search), bridge status, Search route, minimal Dashboard, Settings management sections, and Squad no-snapshot, first-use, CSV import, tactic, and three-team string-add paths | Real `#[tauri::command]` handlers in Rust |
| User-visible navigation and form interaction in Chromium | SQLite persistence, migrations, or `app_data_dir` file I/O |
| Stub IPC for `get_bridge_status`, `get_bridge_install_status`, `install_bridge_plugin`, `remove_bridge_plugin`, `request_player_dump`, `list_saves`, `create_save`, `rename_save`, `set_active_save`, `list_snapshots`, `rename_snapshot`, `delete_snapshot`, `delete_save`, `get_current_snapshot`, `import_csv`, `search_players`, `suggest_players`, `get_player`, managed-club and Planner tactic commands, `get_planner_depth`, `add_planner_string`, `remove_planner_string`, `optimize_planner_depth`, and `load_data` | Capabilities ACL, plugin permissions, native OS file dialogs, or menu/tray integration |
| Bridge panel, save switcher, snapshot overview/history, Squad CSV modals, plugin install section, top-bar save selector, and Load Data button render with stubbed IPC | Real BepInEx plugin, FM attach, LocalAppData file protocol, SQLite ingest, native CSV dialog/file read, or Steam-folder DLL install |

| Concern | Owner in this template |
| --- | --- |
| Frontend IPC wiring and React UI around commands | Vitest + `mockIPC` (`./scripts/dev test`) |
| Command validation, services, migrations, SQLite | `cargo test` in `./scripts/dev check` |
| Bridge scan, dump writers, file protocol | `./scripts/dev bridge-test` in Windows CI (fakes; no FM attach) |
| Full-stack manual verification | `pnpm tauri dev` |
| Automated real WebView e2e | Deferred — see [BACKLOG.md](./BACKLOG.md) (tauri-driver) |

Green smoke does **not** prove SQLite persistence works in production. Rust unit tests own the database; smoke owns browser UI with a stub.

---

## 7. Deployable Artifacts

- **Development** — Install Node 24, pnpm, and the Rust toolchain, then `pnpm install`, `pnpm exec playwright install chromium` (once), then `pnpm tauri dev`. On Linux/WSL, install WebKitGTK and related system packages (see §11). WSLg or an X server is required for the native window on WSL.
- **Release validation (Windows)** — `./scripts/dev package-windows` runs only on Windows. It restores the locked bridge, validates its managed DLL and shared version, bundles one unsigned x64 NSIS installer from that source-built DLL, and writes the installer plus SHA-256 sidecar under `.release/windows/<version>/`. It never publishes anything. The required Check does not run this release-only validation.
- **Published release (Windows)** — `.github/workflows/release.yml` starts only when an explicit release PR changes `release-preparation.json` on `main`. Its read-only job waits for the exact `Check` run for that SHA before it validates the version and changelog. Its final job uses the same package command to stage and verify one matching draft release with the exact dated changelog section and checksum, then publishes the Windows x64 asset as a normal GitHub release. Only the final job has `contents: write`.
- **WebView bundle only** — `pnpm build` produces static files in `dist/` for frontend-only checks; this is not the shipped desktop artifact.
- **Source maps** — default `build.sourcemap: "hidden"` for plain Vite builds (maps on disk, not linked from the public bundle). Tauri production builds use platform-conditional settings when `TAURI_ENV_PLATFORM` is set.
- **Signing** — not configured in the template. Unsigned installers trigger OS security warnings on first run. Add platform signing secrets before shipping a real product.
- **Network / telemetry** — No telemetry in the template. Forks choose online-only or offline-first per product.

---

## 8. Lint & Architecture Enforcement Matrix

### 8.1 TypeScript / React

| Rule | Mechanism | Enforcement |
| ---- | --------- | ----------- |
| Type errors | `tsc -b` | Hard error in `./scripts/dev check` |
| Lint and format | Biome | Hard error in `./scripts/dev check` |
| No IPC data in Zustand | Convention + reviewer | Manual — not lint-enforced |
| Query for async IPC results | Convention + reviewer | Manual |
| Import alias `@/` | `tsconfig` paths | Hard error if path wrong |
| kebab-case filenames | Biome `useFilenamingConvention` when configured | Hard error or reviewer |
| No cross-feature imports | Biome `noRestrictedImports` + reviewer | Hard error in `src/features/**` |
| Unidirectional imports | Biome `noRestrictedImports` + reviewer | Hard error in shared folders |
| No barrel re-exports | Convention + reviewer | Manual |
| Sole invoke wrapper | Convention + reviewer | Manual — only `lib/tauri-client.ts` imports `invoke` |

### 8.2 Rust

| Rule | Mechanism | Enforcement |
| ---- | --------- | ----------- |
| Format | `cargo fmt --check` | Hard error in `./scripts/dev check` |
| Lint | `cargo clippy -D warnings` | Hard error in `./scripts/dev check` |
| Unit tests | `cargo test` | Hard error in `./scripts/dev check` |
| Commands in feature modules | Convention + reviewer | Manual |
| Parameterized queries | Convention + reviewer | Manual — no string-concat SQL |

**Known tooling gaps:** Layer boundaries (Query vs Zustand vs URL state) are convention-only — not lint-enforced. Full `jsx-a11y` lint is not in the default gate — use `ui-design` skill and review; add ESLint only if a product requirement needs plugin coverage Biome lacks.

---

## 9. Notable Trade-offs and Decisions

Each item links to an ADR with alternatives and consequences.

| Decision | ADR |
| --- | --- |
| React for UI | [0001](./decisions/0001-react-for-ui.md) |
| TypeScript | [0002](./decisions/0002-typescript.md) |
| Vite SPA | [0003](./decisions/0003-vite-spa.md) |
| TanStack Router | [0004](./decisions/0004-tanstack-router.md) |
| TanStack Query | [0005](./decisions/0005-tanstack-query.md) |
| Zustand for client state | [0006](./decisions/0006-zustand-client-state.md) |
| Tailwind CSS v4 | [0007](./decisions/0007-tailwind-css-v4.md) |
| Vitest and RTL | [0008](./decisions/0008-vitest-and-rtl.md) |
| Biome | [0009](./decisions/0009-biome.md) |
| pnpm | [0010](./decisions/0010-pnpm.md) |
| Husky (no lint-staged) | [0011](./decisions/0011-husky-git-hooks.md) |
| Secretlint in check | [0012](./decisions/0012-secretlint.md) |
| Tauri v2 desktop shell | [0013](./decisions/0013-tauri-v2-desktop-shell.md) |
| Rust backend and IPC boundary | [0014](./decisions/0014-rust-backend-ipc-boundary.md) |
| SQLite (Rust-owned) | [0015](./decisions/0015-sqlite-rust-owned.md) |
| C# BepInEx FM26 bridge | [0016](./decisions/0016-csharp-bepinex-fm26-bridge.md) |

**@tanstack/react-virtual** is in the stack for the player search results table. TanStack Table, Form, and TanStack Start remain intentionally **not** in the default stack — add per feature when needed. The FM26 bridge is implemented per [ADR-0016](./decisions/0016-csharp-bepinex-fm26-bridge.md), [fm26-memory-read](./features/completed/fm26-memory-read.md), and [bridge-plugin-install](./features/completed/bridge-plugin-install.md); dump schema v8 is frozen in [bridge/DUMP_SCHEMA.md](../bridge/DUMP_SCHEMA.md).

---

## 10. Where to Look Next

- **Add a feature:** Create `src/features/<feature>/` and `src-tauri/src/features/<feature>/` with the subfolders each side needs. Register commands in `lib.rs` via `.invoke_handler(tauri::generate_handler![...])`. Add route wiring in `src/app/routes/`.
- **Add a page:** Create a file under `src/app/routes/`, add Query options in `features/<feature>/api/` if the page loads IPC data.
- **Add client UI state:** Global store in `src/stores/`; feature-scoped store in `features/<feature>/stores/`. Do not store IPC responses in Zustand.
- **Add shared UI:** `src/components/ui/` for primitives (see [DESIGN.md](./DESIGN.md) component specs); wrap third-party components there.
- **Change visual language:** update [DESIGN.md](./DESIGN.md) first, then mirror tokens in `src/styles/global.css` `@theme`.
- **Add persistence:** Migration in `db/migrations.rs`, service in `features/<feature>/service.rs`, commands in `commands.rs`. Open path stays `app_data_dir` + `APP_DB_FILE` via `db::open`.
- **Change stack defaults:** Read ADRs, update decisions, then reconcile this file and scaffold configs.
- **Coding standards detail:** load the installed `coding-standards` skill and its React, Tauri, Rust, and Vite references

---

## 11. Operational Notes

### Prerequisites (WSL Ubuntu or any Linux/macOS dev machine)

Install on the **host OS** before `pnpm install`:

| Tool | Why |
| --- | --- |
| **Node.js 24** | Runs Vite, Vitest, Biome, and all build tooling |
| **pnpm** | Package manager for this template (`corepack enable` after Node, or `npm install -g pnpm`) |
| **Rust toolchain** | `rustc` and `cargo` for the Tauri backend — install via [rustup](https://rustup.rs/) |
| **git** | Version control |

On **Linux and WSL**, install Tauri system dependencies before `pnpm tauri dev` or `pnpm tauri build`:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
```

On **WSL**, you also need a display server for the native window:

- **WSLg** (Windows 11) — GUI apps work out of the box when WSLg is enabled.
- **X server** (older setups) — run an X server on Windows and set `DISPLAY` before `pnpm tauri dev`.

Headless gate commands (`./scripts/dev check`, `cargo test`) do not require a display.

**FM26 bridge:** Build and install the plugin on a **Windows** host with .NET 6 SDK and BepInEx 6 IL2CPP on the Steam FM26 folder. End users can install the bundled DLL from the app (**Bridge plugin install** in Settings); developers use `./scripts/dev bridge-install` or manual copy from WSL when `FM_STEAM_ROOT` or the default Windows Steam path is set. See [bridge/README.md](../bridge/README.md).

### What `pnpm install` does

`pnpm install` reads `package.json` and `pnpm-lock.yaml` and downloads **Node packages** into `node_modules`. It also runs the Husky `prepare` script to install Git hooks. It does not install Node itself, pnpm itself, the Rust toolchain, or system libraries.

### Typical first-time setup on clean WSL

```bash
# Install Node 24 (example: nvm)
nvm install 24
nvm use 24
corepack enable
corepack prepare pnpm@latest --activate

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install Tauri Linux dependencies (see apt list above)

# Clone template, then in repo root
pnpm install
pnpm exec playwright install chromium
./scripts/dev check
./scripts/dev test
pnpm tauri dev
```

Husky runs `./scripts/dev check-fast` on every commit (and `check-rust` when `src-tauri/` is staged). Run `./scripts/dev check` before merge — CI selects the applicable product suites.

### CI parity

GitHub Actions selects product checks from changed paths. Frontend changes run `./scripts/dev check-app` and `./scripts/dev test`, then browser smoke. Rust changes install the Rust toolchain and Tauri Linux dependencies before `./scripts/dev check-rust`. Bridge changes run `./scripts/dev bridge-test` on Windows. The required `check` status aggregates every applicable job. Match local Node major version for fewer surprises.

Release evaluation follows each successful required Check run caused by a `main` push. Metadata and validation jobs are read-only; a newer validated version receives one unsigned Windows x64 release only after the final job stages and verifies its checksum and exact changelog body.
