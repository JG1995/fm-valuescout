# Early Alpha Release Readiness

## Status

Completed — documentation reconciled; ready for publication

## Intent

Prepare FM ValueScout for a narrow, unsigned Windows early-alpha release that the author can dogfood and a public-repository visitor can understand and install at their own risk. The release path must build the bridge from the same source revision as the desktop app, fail closed when release identity or artifacts are wrong, preserve the local-data contract, and publish only a prepared prerelease from a verified `main` revision.

## Delivered behavior

- The Dashboard no longer exposes the template health status, demo SQLite value, or simulated-error controls. Migration v22 removes only the obsolete `demo_value` table; migration v1 remains historical input for existing databases.
- `./scripts/dev package-windows` restores the locked bridge, builds it from the checked-out source, rejects missing, malformed, placeholder, or version-mismatched DLLs, and produces one unsigned Windows x64 NSIS installer plus a SHA-256 sidecar under `.release/windows/<version>/`. The tracked placeholder is not a release input and is not modified.
- Release builds retain bounded local operational logs. The feature adds no telemetry, remote logging, analytics, account, cloud-data, or background-network requirement.
- The root product documentation, bridge guide, security policy, and early-alpha runbook describe the supported Windows Steam FM26/BepInEx environment, men's-database boundary, manual backup and update path, diagnostics, unsigned-installer warning, and support limits.
- The repository-local `create-pr` skill is the single human pull-request procedure. It records exactly one `none`, `patch`, `minor`, or `major` intent. Release-bearing preparation updates the five durable version owners, the dated Keep a Changelog section, and `release-preparation.json`; `none` leaves the prepared release record unchanged.
- `./scripts/dev release-metadata` validates the prerelease identity and returns the exact dated changelog section without writing files, invoking Git, or calling GitHub.
- The Dependabot policy offers only guarded stable pnpm patches and Cargo patches within a compatible release line, including `0.y.z` lines where `y > 0`. It does not grant release authority or auto-merge pnpm pre-`1.0.0`, Cargo `0.0.z`, prerelease, non-patch, mixed, maintainer-modified, GitHub Actions, or NuGet updates.
- The Release workflow follows only a successful required `Check` run caused by a push to `main`, checks out that run's exact `head_sha`, and uses the same package command as the candidate path. A changed, matching `release-preparation.json` record authorizes publication for that exact SHA.
- A later `none` or Dependabot `main` push defers the prepared version. An unchanged release is a successful no-op. A stale prepared draft or tag, a mismatched SHA or identity, or orphaned publication state fails closed before packaging or mutation. A same-SHA draft retry may repair the temporary draft. If the source must change, the unpublished draft and matching tag must be deleted before a new release-bearing preparation can publish the corrected source.
- The final staged publication job is the only release job with `contents: write`. It uploads one installer and checksum, verifies the exact release metadata and assets, and publishes a prerelease with `draft=false` and the exact dated changelog section as its complete body.

## Final architecture

- Release identity is owned by `src-tauri/tauri.conf.json` and checked against `package.json`, `src-tauri/Cargo.toml`, the root `app` package in `src-tauri/Cargo.lock`, and `bridge/FmDataBridge.csproj`. `release-preparation.json` records the explicit release intent, sequence, and prepared version.
- `scripts/release-metadata.mjs` is the read-only SemVer and changelog validator. `scripts/release-package-validation.mjs` and `scripts/dev package-windows` own the non-publishing Windows candidate path. `scripts/release-publication-policy.mjs` chooses the only allowed publication state before release mutation.
- `.github/workflows/check.yml` validates release metadata and stores the candidate artifact when release inputs change. `.github/workflows/release.yml` performs exact-SHA verified-main evaluation, no-op/defer decisions, temporary draft staging, asset verification, and automatic prerelease publication.
- `.agents/skills/create-pr/SKILL.md` owns human release-intent preparation and repository-template completion. GitHub Actions does not infer intent, rewrite changelog prose, commit version changes, or push to `main`.
- `.github/dependabot.yml`, `.github/workflows/dependabot-automerge.yml`, and the tested policy evaluator keep privileged dependency automation metadata-only and trusted-branch based. The strict `check` remains the merge gate.
- React, Rust, C#, scripts, and GitHub Actions retain separate ownership of presentation, migration and local diagnostics, bridge binaries, deterministic build commands, and publication orchestration. No new bridge protocol or write action was added.

## Important decisions

- Support one unsigned Windows x64 NSIS candidate for the early alpha. macOS, Linux, signing, store distribution, automatic updates, BepInEx bootstrap, and unsupported FM editions remain out of scope.
- Build the bridge from source at package time. Do not treat the tracked generated placeholder as release source of truth or commit generated release binaries.
- Keep `src-tauri/tauri.conf.json` as the canonical version owner and require exact SemVer agreement across every release input.
- Use one universal PR procedure with explicit release intent. CI evaluates only the prepared source record and the successful verified `main` SHA.
- Use temporary draft state only for fail-closed asset staging. Never retarget a draft, tag, or published release to a different source SHA.
- Preserve append-only migration history. Remove only the obsolete template table through a forward migration.
- Keep release diagnostics local, bounded, and redacted. No new remote endpoint or WebView privilege was needed.
- No ADR or debug report was added during reconciliation. The durable release boundary is owned by `ARCHITECTURE.md`, the early-alpha runbook, the PR skill, and this completed record.

## Migration and operational implications

- Existing databases upgrade through migration v22. Product saves, snapshots, Planner state, Academy state, CSV enrichment, and other persisted data remain intact; only the isolated `demo_value` table is removed.
- The supported operating boundary is Windows x64, Steam FM26.3, and BepInEx 6 IL2CPP already installed. FM patches can break memory compatibility. The two player-boost actions remain optional exact-build operations and require a fresh supported Load Data result and a save backup.
- Release logs remain under `%LOCALAPPDATA%\app.fmvaluescout\logs\fm-valuescout.log` with bounded rotation. Bridge diagnostics and dump files remain local and must not be attached to public issues without redaction.
- Database recovery is a closed-app copy and restore of the complete application-data directory. Updates are manual and forward-only; automatic updates and downgrade support remain out of scope.
- No GitHub tag, release, hosted Windows candidate, hosted `Check`, or hosted `Release` execution has been recorded. The first release-bearing PR must complete the runbook's candidate acceptance before merge because a successful merge can publish automatically.

## Validation

- The exact final implementation and workflow set below cleared the Sol xhigh feature-complete review with no blocking finding. The final correction `fcbf0217670841d3677738f38e710a8a9c74866d` is included in that reviewed set.
- The focused release test run passed 31 tests with normal host process permissions.
- `git diff --check 0c9c10e41b59941d08b90a5f493283836d149830..fcbf0217670841d3677738f38e710a8a9c74866d` was clean for the implementation range.
- No hosted Windows candidate or GitHub `Check`/`Release` run has been executed. No actual installer, installed-app diagnostic inspection, native recovery test, or live FM acceptance has been recorded. Browser, Rust, bridge-unit, and static workflow tests do not replace those checks.

## Exact implementation refs

Planning context is excluded from the feature-complete implementation set:

| Ref | Subject | Role |
| --- | --- | --- |
| `58e28a3ff37139bd481d3955182690209da6b19d` | `docs(release): plan early alpha distribution` | Planning ledger; supporting context only |

The final PR contains the following eleven commits:

| # | Ref | Subject | Implementation / review |
| --- | --- | --- | --- |
| 1 | `7a37ae49407c8c7e5646e15faf8813752f7287ba` | `fix(health): remove template health scaffold` | Terra xhigh / Sol High; accepted with no blocking findings |
| 2 | `8ddad7acedff7221c2457665e1af22c931d41435` | `chore(deps): lock release build inputs` | Terra xhigh / Sol High; accepted, clean-Windows restore remains unproved |
| 3 | `5e6769852af7a1ea6f48c8456a80423261ae037d` | `ci(deps): automate guarded dependency patches` | Terra xhigh / Sol High; accepted after two corrective review rounds; live GitHub settings remain unproved |
| 4 | `ab2f56b730b51b11b4516729b500e5741c8a8ba5` | `build(release): package the bridge from source` | Terra xhigh / Sol High; accepted after corrective review; native package and extraction remain unproved |
| 5 | `61ac81e48ece0233005522ad8b92d5758e4533d2` | `feat(diagnostics): retain local release logs` | Luna Max / Sol High; accepted, packaged log creation and privacy inspection remain unproved |
| 6 | `4cbf10b2f935c1f5b1ff074ff92d641b35862229` | `docs(release): define the early alpha contract` | Luna Max / Sol Medium; accepted after corrective review; installed-app, recovery, and live-FM acceptance remain unproved |
| 7 | `f730f139495d080e51a1bfb39b93f715d6070ce2` | `build(release): prepare pull request release metadata` | Terra xhigh / Sol High; accepted after corrective review; first hosted candidate and release evidence remain unproved |
| 8 | `a01e0c8488591555612caa9205df06ee0c9d1453` | `ci(release): publish prereleases from verified main` | Terra xhigh / Sol High; accepted after corrective review; hosted candidate and first-push release evidence remain unproved |
| 9 | `3a4d19124611f4afb7eb68b864eced604fa9a311` | `docs(release): mark implementation complete` | Documentation state transition; included in the final PR |
| 10 | `43f4de986b1ead490b5eded4dfb341a50e5e0831` | `docs(release): narrow candidate acceptance scope` | Runbook correction; included in the final PR |
| 11 | `fcbf0217670841d3677738f38e710a8a9c74866d` | `fix(release): bind publication to prepared source` | Final release-workflow correction; included in the feature-complete review and accepted with no blocking finding |

## Final publication

~~~yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/early-alpha-release-readiness
base_branch: main
base_ref: 0c9c10e41b59941d08b90a5f493283836d149830
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_checks: strict_check
required_check_name: check
pr_count: 1
earlier_prs: none
build_feature_loop_profile: terra_xhigh
feature_close_out: current
feature_review_profile: sol_xhigh
feature_review_blocking: false
feature_review_critical: none
feature_review_high: none
feature_review_medium: none
feature_review_nitpick: none
feature_review_recommendation: accept
ci_repair_rounds: 0
planning_ref: 58e28a3ff37139bd481d3955182690209da6b19d
implementation_refs:
  - 7a37ae49407c8c7e5646e15faf8813752f7287ba
  - 8ddad7acedff7221c2457665e1af22c931d41435
  - 5e6769852af7a1ea6f48c8456a80423261ae037d
  - ab2f56b730b51b11b4516729b500e5741c8a8ba5
  - 61ac81e48ece0233005522ad8b92d5758e4533d2
  - 4cbf10b2f935c1f5b1ff074ff92d641b35862229
  - f730f139495d080e51a1bfb39b93f715d6070ce2
  - a01e0c8488591555612caa9205df06ee0c9d1453
  - 3a4d19124611f4afb7eb68b864eced604fa9a311
  - 43f4de986b1ead490b5eded4dfb341a50e5e0831
  - fcbf0217670841d3677738f38e710a8a9c74866d
close_out_documentation_ref: Pending record
~~~

## Feature close-out

**State:** Current. The exact final implementation and workflow set passed feature-complete review, and documentation now matches the implemented release boundary. The branch remains local and unpublished; no PR, tag, release, or merge is claimed. Current means the repository's implemented behavior is reconciled, not that the early-alpha installer has passed hosted or native acceptance.

## Follow-up

- Keep the release-bearing branch unpublished until a native Windows candidate, checksum, managed bridge identity, installed-app diagnostic check, closed-app recovery check, and live FM acceptance pass.
- Run the required hosted `Check` and verified-main `Release` workflows on the exact prepared source. Confirm the changed `release-preparation.json` record authorizes one prerelease, and confirm later `none` or Dependabot pushes defer without mutation.
- If source correction is needed after a failed staging attempt, delete the unpublished temporary draft and matching tag before creating the new release-bearing preparation.
- Enable and verify repository auto-merge, Dependabot alerts, security updates, private vulnerability reporting, and any immutable-release setting as described by the runbook before first publication.
