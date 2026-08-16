# Staff Workspace

## Intent

Turn extracted staff into a first-class workspace for role-fit scouting, club-family oversight, and a tightly bounded staff CA action.

## Delivered behavior

- **Staff** navigation opens `/staff` on the URL-backed Search workspace. Search covers the effective current snapshot and provides bounded, virtualized, configurable staff tables with validated filters and sorting.
- Staff job-fit scores cover 20 jobs. Rust persists `round(mean(required 1–20 attributes) × 5)` only when all required attributes are present; React displays the stored 0–100 value with the shared accessible score ramp.
- **My Staff** lists the configured Senior, Reserves, and Youth club family. It has its own sort and column layout, no filters, and one confirmed **Boost all CA** action.
- `/staff/$uid` provides a Staff Profile with current attributes in Coaching, Mental, and Knowledge columns and a bounded, virtualized Role fit list. It excludes player-only pitch, potential, and Wonderkid surfaces.
- The save-scoped hidden-information preference applies to player and staff profiles. Concealed Staff Profiles hide PA and **Boost CA**, while current staff attributes, including Adaptability, and current job-fit scores remain visible.
- Individual and configured-family boosts use the fixed `+10`, capped at PA and 200. The WebView sends no target values or cohorts. Rust owns configured-family selection and serial execution; the bridge owns the closed live-memory operation. Partial bulk results remain visible if a later error occurs.
- The bridge retains its live indexes after proven no-write value rejections, while uncertain failures still fail closed. Load Data, active-save changes, and snapshot changes invalidate Staff data along with the affected current context.
- The bridge reads `WorkingWithYoungsters` as its raw 1–20 value. `Authority` remains the accepted FM26.3 mapping at `NPLO_ATTRS + 0x30`; a future live comparison can disprove that mapping.

## Final architecture

- The C# bridge publishes staff schema v8 with the required attributes and exposes one fixed staff CA operation. Rust validates dumps, persists `staff` and `staff_role_scores`, validates Staff query fields and filters, and owns boost policy, recovery, and configured-family membership.
- React uses staff-specific query keys, metric catalogs, and layouts on the shared virtual-table primitives. Search, My Staff, and profile views read persisted scores instead of recalculating them.
- Migrations 24 through 26 add persisted staff scores, one shared player/staff boost-recovery flag, and one shared `reveal_hidden_information` preference.

## Important decisions

- [ADR-0020 — Action-specific FM26 staff CA boost](../../decisions/0020-action-specific-fm26-staff-ca-boost.md) defines the closed individual memory-write operation.
- [ADR-0021 — Sequential club-family staff CA boost](../../decisions/0021-sequential-club-family-staff-ca-boost.md) defines Rust-owned configured-family orchestration and partial-result handling.
- No debug report was added. Focused regression tests cover the corrected bridge-index, cache-invalidation, sort-parity, and partial-progress contracts.

## Migration and operational implications

- Existing schema-v7 snapshots remain readable but cannot provide complete staff scores. Run **Load Data** with the schema-v8 bridge for complete scoring.
- The installed bridge must match the supported FM build before it advertises staff boosting. A fresh effective snapshot clears the shared recovery requirement after uncertain mutation outcomes.
- The accepted `Authority` mapping is not independently verified against FM. If a live comparison disproves it, correct the bridge mapping and regenerate affected snapshots.

## Validation

- `./scripts/dev format` made no changes.
- `./scripts/dev bridge-test` passed: 224 tests, with 3 platform skips.
- `./scripts/dev test` passed: 469 tests.
- `./scripts/dev check` passed, including 452 Rust tests with 2 intentional ignores.
- `CI=1 ./scripts/dev smoke` passed: 43/43 browser tests.
- `git diff --check` passed for the completed implementation range.
- The Sol xhigh feature review completed one correction round. Its final verdict retained no findings and accepted the feature.
- The developer explicitly accepted two remaining manual checks: assembled live-FM validation of score formulas, configured-family membership, and boosts; and native Tauri checks at 1280×800 and 1600×900.

## Exact implementation refs

PR 1 merged as [`2e192811a4501d7dfe4df7b1fc675252fc5e8564`](https://github.com/JG1995/fm-valuescout/commit/2e192811a4501d7dfe4df7b1fc675252fc5e8564). Its reviewed implementation commits were:

| Ref | Subject |
| --- | --- |
| `009c21f718bab5385e8c6aea5311ed9a8fa20b81` | `feat(memory-read): extract staff scoring attributes` |
| `94bc9215e5aa0ed2515663908f11205248de2397` | `feat(scoring): persist staff job scores` |
| `98cf8c5bc986e119ac8541468bd47c6937b9b42f` | `feat(staff): query scored staff pages` |
| `1932350eac38a2383b9b45ae9e391bbab74b8b10` | `feat(memory-read): support staff CA boosts` |
| `48e5c5a6a2891aa0ba41a093abb14e9387367d79` | `feat(staff): reconcile verified staff boosts` |
| `a8a35cd566b7d538b5d48528070692c974d53260` | `feat(staff): query staff profiles` |

The PR 2 implementation range is `2e192811a4501d7dfe4df7b1fc675252fc5e8564..d1df3618f513263247ba3ae1acf6b78260abe034`:

| Ref | Subject |
| --- | --- |
| `bd373ffdae161d815cf020396be07a729c897765` | `refactor(tables): share configurable table controls` |
| `12c212fe400c6e660e34a334f82ef52dd768a361` | `feat(staff): add staff search workspace` |
| `d37c93e0d32381776da8a711c7d5154c03be46b8` | `feat(staff): add club-family staff overview` |
| `2153fd223aa7098a6a571fffab9333dece39641a` | `feat(staff): add staff CA boost` |
| `18c28ce47acecef40cbc7582c8691da2640ccffd` | `feat(staff): add staff profiles` |
| `cba1bdfb337df7c309cbbf79abb18a617b5d07ba` | `feat(staff): add bulk CA boost` |
| `b1d33dc1adeaf2347909a5080db5cbb5c3e86bfd` | `refactor(staff): combine profile attributes` |
| `6d86e1303826aca0ed296e16762807b518a30588` | `feat(staff): virtualize profile role fit` |
| `70c27f42ebdd0496aeb25c17ebb08b90177eca3d` | `feat(staff): color role scores` |
| `1df2c58f3e368e937a5cda2c07c466c00a034d51` | `feat(search): color role scores` |
| `427c7ac776a48b9b61ee5130f844758afdae09d8` | `fix(staff): align profile attribute styling` |
| `ad70c90ce283505e42142e36e164709df783b947` | `fix(bridge): decode working with youngsters correctly` |
| `d1df3618f513263247ba3ae1acf6b78260abe034` | `fix(staff): preserve workspace lifecycle contracts` |

`09a82f23cd455f24b3ace9a9656019a42dfdca24`, `7f6147b0eb4ccb19c557832780ab181dec06726b`, and `22d6daa6f9c0be3bde66664dd08a2f19a2cdd142` record planning and checkpoint state. They are retained in the branch history but are not implementation commits.

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/staff-workspace
base_branch: main
base_ref: 2e192811a4501d7dfe4df7b1fc675252fc5e8564
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_check_name: check
pr_count: 2
earlier_prs:
  - https://github.com/JG1995/fm-valuescout/pull/57
feature_review_profile: sol_xhigh
feature_review_blocking: false
feature_review_recommendation: accept
feature_review_findings: none
feature_review_correction_rounds: 1
publication_range: "2e192811a4501d7dfe4df7b1fc675252fc5e8564..d1df3618f513263247ba3ae1acf6b78260abe034"
close_out_documentation_ref: Pending record
```

## Follow-up

Create the PR with the repository template. Record its URL and immutable merge reference only after those events occur.
