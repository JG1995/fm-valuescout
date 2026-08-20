# Moneyball Role Scores

## Intent

Add transparent, versioned performance role scores to the existing Moneyball import without combining them with attribute-based role scores.

## Delivered behavior

- The Rust-owned version-1 catalog contains 88 IP/OOP role definitions across ten position families. Definitions remain distinct by position family even when their five metrics and weights match. The catalog validates identities, metric keys, weights, and lower-is-better metadata before use.
- A Moneyball role score is the rounded weighted mean of five nullable imported metric percentiles. Zero remains a valid score. A missing contributing percentile makes the role unavailable. The scorer does not calculate raw rates, invert percentiles again, apply a minutes threshold, or persist a derived cache.
- Moneyball Player Profile uses the active snapshot's full imported cohort. It shows best playable IP/OOP role summaries, a position-filtered and sortable role-fit panel, and keyboard-reachable score disclosures with metric contributions, direction, catalog version, and comparison basis. The raw statistics panel remains available.
- Moneyball Search offers every role as an optional, grouped ScoreBadge column, numeric filter, and sort field. Full CSV composes scores from persisted import percentiles. Filtered recomputes the required metric percentiles over the complete comparison cohort before scoring. Role filters run after scoring; mixed OR role filters use the full import. Nulls sort last, and totals and pagination follow post-score filtering.
- General player profiles use the expanded presentation inventory. Entries without an attribute-role mapping render `—` for Current and Potential. They do not enter attribute ingest scoring, General Search, Planner, tactics, or potential-score materialization. JAY-31 owns the missing attribute formulas.

## Final architecture

- `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` supplies the built-in catalog. `role_catalog.rs` validates and loads it, and `role_score.rs` owns the shared pure score and explanation calculation.
- `get_player_moneyball` reads persisted full-import percentiles and returns role results. `get_player` uses the catalog only to present mapped attribute results and explicit unavailable placeholders.
- `features/search` accepts the closed `moneyball_role.<id>` field only in Moneyball view. It calculates only roles and metric keys requested by visible fields, filters, sort, or the profile. Replacing the Moneyball import changes subsequent results because no independent score rows exist.
- React mirrors catalog identity metadata for Search registration only. It does not calculate scores or explain score arithmetic.

## Important decisions

- Keep Moneyball performance scores separate from attribute role scores and potential projection.
- Use the full imported cohort for profiles and either the persisted full import or the complete filtered cohort for Search.
- Keep role definitions versioned and position-family-specific. Do not merge matching wide-position definitions.
- No ADR or debug report is needed. The feature extends existing Moneyball, Rust query, and presentation boundaries without a durable new structural decision or confirmed reusable failure pattern.

## Validation and limitations

- Feature validation and the feature-complete review cleared the exact implementation set below. The final review had Blocking **No**, recommendation **Accept**, project fit **Conforms**, and no retained findings.
- Manual actual 200% browser zoom and narrow viewport verification was not possible in this environment.
- Existing Rust tests prove import, profile, page-invariance, and replacement separately. They do not prove one end-to-end imported-fixture derivation followed by replacement that changes role results.

## Exact implementation refs

**Base:** `c7dfd983a1304026a6ddcf6f8528a1dfce19871c`

There are no earlier PR merge refs and no correction commit. The close-out documentation commit is `c05f3a1b7fae0e61bdfd2c9bfbbb0a583b4f08dd`.

| Ref | Subject | Role |
| --- | --- | --- |
| `e041c157166fc117578aba1b35f44fb8959bb7d7` | `docs(moneyball): plan role scoring feature` | Planning record |
| `1a8046b6a533cb873fd1e9584ee3540dbe5ef8fd` | `feat(moneyball): define role score catalog` | Version-1 catalog, validation, scorer, and explanations |
| `0678e1ba6c7d1c49ce641ac91ad142766f072972` | `feat(profile): show Moneyball role fit` | Full-import profile scores, General placeholders, and role-fit UI |
| `527c4e0b6c66b90560bedf519460f9ff482aee05` | `feat(search): query Moneyball role scores` | Comparison-pool scoring, post-score filters, sorting, totals, and pagination |
| `ae80761bf260a42cd42859a30fb9c4e5820f709b` | `feat(search): expose Moneyball role columns` | Search catalog mirror, columns, filters, and browser coverage |
| `c05f3a1b7fae0e61bdfd2c9bfbbb0a583b4f08dd` | `docs(moneyball): archive role scoring feature` | Close-out documentation reconciliation |

Release preparation is `Pending record` until the `0.10.0` preparation change is committed.

## Delivery status

- **PR 1 — Add Moneyball role scoring:** Ready for publication. It has not been published or merged.
- **Commit 4 — Expose Search role columns and filters:** Completed at `ae80761bf260a42cd42859a30fb9c4e5820f709b`.

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/moneyball-role-scores
base_branch: main
base_ref: c7dfd983a1304026a6ddcf6f8528a1dfce19871c
publication_provider: GitHub
release_intent: minor
release_version: 0.10.0
release_tag: v0.10.0
required_check_name: check
ci_repair_rounds: 0 of 2
earlier_pr_merge_refs: none
correction_ref: none
close_out_documentation_ref: c05f3a1b7fae0e61bdfd2c9bfbbb0a583b4f08dd
release_preparation_ref: Pending record
implementation_range: "c7dfd983a1304026a6ddcf6f8528a1dfce19871c..ae80761bf260a42cd42859a30fb9c4e5820f709b"
final_pr_commit_set:
  - e041c157166fc117578aba1b35f44fb8959bb7d7
  - 1a8046b6a533cb873fd1e9584ee3540dbe5ef8fd
  - 0678e1ba6c7d1c49ce641ac91ad142766f072972
  - 527c4e0b6c66b90560bedf519460f9ff482aee05
  - ae80761bf260a42cd42859a30fb9c4e5820f709b
  - c05f3a1b7fae0e61bdfd2c9bfbbb0a583b4f08dd
  - Pending record (release preparation for 0.10.0)
```

## Feature close-out

**State:** Current. Validation, feature-complete review, and documentation reconciliation cleared the exact implementation set. The final PR is ready for publication but remains unpublished and unmerged.

## Follow-up

- Publish only through the GitHub publication workflow when the branch is ready for review. Resolve the pending release-preparation ref from Git after this change is committed.
- Run the missing 200% zoom and narrow viewport checks in a supported browser environment.
- Add an end-to-end Rust fixture only if replacement-driven role-result regression coverage becomes necessary.
