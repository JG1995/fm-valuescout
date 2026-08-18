# Settings and Managed Club

## Intent

Combine Linear JAY-26 and JAY-27 in one feature. Move operational management from Dashboard to Settings, and replace configured club families with one save-scoped managed club.

## Delivered behavior

- The top bar remains the only active-save selector and location for snapshot freshness, player-cap controls, and **Load Data**.
- Settings at `/settings` contains separate Save data, Managed club, and Bridge sections. It owns save and snapshot management, bridge status, and bridge plugin install, update, and removal.
- Dashboard remains at `/` with only its `Dashboard` heading and `Placeholder.`. The Dashboard CSV importer and sanity-player IPC path are removed. Format-specific Moneyball and Youth Academy imports remain in Squad.
- Each save stores one managed club. Settings lists exact current-club names from the effective current snapshot and keeps a saved selection visible when a later snapshot no longer contains it.
- Squad, Planner, Academy, My Staff, and club-wide player and staff boosts use exact managed-club membership from the effective current snapshot. `team_level` remains imported metadata but does not restrict Planner eligibility or appear as a user-facing diagnostic.
- Planner teams share one managed-club player pool. Existing Reserves and Youth age limits, canonical Senior-to-Youth allocation order, and save-wide player uniqueness still apply.
- A missing managed club produces empty current cohorts without deleting Planner assignments, Academy records, tactics, strings, shortlists, enrichment, snapshots, or saves.

## Final architecture

- Rust `features/managed_club` owns selection validation, save-scoped persistence, current-snapshot club options, and exact player and staff cohort queries. React `features/managed-club` owns its typed IPC and Settings panel.
- Migration v29 renames `planner_club_settings` to `managed_club_settings`, renames `primary_club` to `club_name`, and drops obsolete `planner_club_sources`. It preserves the primary selection and unrelated save, snapshot, Planner, Academy, shortlist, and enrichment data.
- Routes compose managed-club invalidation with snapshot context changes. Changing the managed club, active save, or effective current snapshot refreshes downstream membership consumers.
- The bridge dump and ingest retain nullable FM `team_level`. Only the managed-club contract determines Planner membership.

## Important decisions

- Use one exact managed-club selection rather than attached clubs, fuzzy matching, or `parent_club` inference.
- Treat the effective current snapshot as authoritative. A missing saved club remains a recoverable selection, not a reason to delete retained user data.
- Do not add a new ADR. The feature replaces an obsolete product contract within existing Rust-owned persistence and cohort boundaries. [ADR-0018](../../decisions/0018-squad-wide-player-boosts.md) and [ADR-0021](../../decisions/0021-sequential-club-family-staff-ca-boost.md) retain their write and recovery decisions, while their former club-family cohort selection is superseded.

## Migration and operational implications

- Databases at v28 migrate to v29 with their existing primary club retained as the managed club. Attached-source rows are intentionally removed.
- Users select a replacement in Settings if the managed club is absent from the latest snapshot. Current views become empty, while saved Planner and Academy records retain their existing outside-pool or unresolved states.
- FM can provide a null or unsupported `team_level`. The value remains available for future consumers but does not exclude a player from Planner or optimizer candidates.

## Validation

- `./scripts/dev format` passed.
- `./scripts/dev test` passed 487 frontend tests in 46 files.
- `./scripts/dev check` passed, including 481 Rust tests and two intentional ignores.
- `./scripts/dev smoke` passed 44 browser tests.
- The v28-to-v29 migration proof seeds and compares Planner, Academy, shortlist, Youth enrichment, and Moneyball enrichment data. A deliberate deletion of Academy outcomes made that proof fail before the mutation was removed.
- The Sol xhigh feature-complete review cleared after one correction round with Blocking **No** and no retained findings.
- Native Tauri route, focus, unsupported-platform bridge, and minimum-window checks remain unavailable in this headless environment. Repowise remained stale after its index refresh did not complete; direct source, test, Git, and review evidence support this record.

## Exact implementation refs

**Feature range:** `ad5c12ff386274057dd2f06b2f03e4adcbe9dbfb..09b60dd1c050f1826852602b69bbea0676db7b09`

| Ref | Subject | Role |
| --- | --- | --- |
| `fce4c716edbc5ababf3c56eef94aef3220890b9d` | `docs(settings): plan managed-club configuration` | Planning record |
| `f536ac2f78dad38a4c8bc0129f9edc850e1a719c` | `feat(settings): move app management from Dashboard` | Settings route, placeholder Dashboard, and removal of the Dashboard importer and sanity path |
| `1bc7717e620cded6ea20fdb32f517a5dadf8bd19` | `feat(club): derive managed membership from FM data` | v29 migration, managed-club contract, exact cohorts, and removal of club-family APIs and UI |
| `98e56df1a65ab65224ed5304395341bea3b578f0` | `fix(planner): share managed club pool across teams` | Removed team-level Planner eligibility after live data showed nullable values excluded the complete cohort |
| `950d0baaed6f9926b631b53502ace7dbdaa49df9` | `fix(settings): remove non-actionable team-level warning` | Removed Settings diagnostic copy while retaining native metadata |
| `09b60dd1c050f1826852602b69bbea0676db7b09` | `fix(settings): preserve data through managed-club migration` | Added complete v28-to-v29 preservation proof and aligned empty-state and top-bar lifecycle coverage |

Documentation reconciliation commit: Pending record.

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/settings-managed-club
base_branch: main
base_ref: ad5c12ff386274057dd2f06b2f03e4adcbe9dbfb
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_check_name: check
pr_count: 1
earlier_prs: none
feature_close_out: current
feature_review_profile: sol_xhigh
feature_review_blocking: false
feature_review_recommendation: accept
feature_review_findings: none
feature_review_correction_rounds: 1
ci_repair_attempts: 0
publication_range: "ad5c12ff386274057dd2f06b2f03e4adcbe9dbfb..09b60dd1c050f1826852602b69bbea0676db7b09"
feature_review_scope: "f536ac2f78dad38a4c8bc0129f9edc850e1a719c, 1bc7717e620cded6ea20fdb32f517a5dadf8bd19, 98e56df1a65ab65224ed5304395341bea3b578f0, 950d0baaed6f9926b631b53502ace7dbdaa49df9, 09b60dd1c050f1826852602b69bbea0676db7b09"
final_pr_commit_set:
  - fce4c716edbc5ababf3c56eef94aef3220890b9d
  - f536ac2f78dad38a4c8bc0129f9edc850e1a719c
  - 1bc7717e620cded6ea20fdb32f517a5dadf8bd19
  - 98e56df1a65ab65224ed5304395341bea3b578f0
  - 950d0baaed6f9926b631b53502ace7dbdaa49df9
  - 09b60dd1c050f1826852602b69bbea0676db7b09
close_out_documentation_ref: Pending record
```

## Feature close-out

**State:** Current. The exact implementation set passed final validation, the Sol xhigh feature review, and documentation reconciliation. The feature branch is ready for publication; no pull request or merge ref exists.

## Follow-up

- Publish through the repository pull-request workflow when requested. Record the pull request URL and merge ref only after GitHub provides evidence.
- Run native Tauri/WebView route, focus, unsupported-platform bridge, and minimum-window checks when a desktop environment is available.
