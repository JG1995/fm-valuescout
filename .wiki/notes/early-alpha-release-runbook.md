# Early-alpha release runbook

This runbook separates evidence required before a release-bearing pull request merges from the automatic publication that follows verified `main`. It is written for the solo maintainer and does not authorize a push, merge, tag, or release by itself.

## Current state

The Windows validation command exists now:

```bash
./scripts/dev package-windows
```

Run it on a native Windows host. It builds the locked bridge from the checked-out source, produces one unsigned Windows x64 NSIS installer and its SHA-256 sidecar under `.release/windows/<version>/`, and never publishes anything.

The universal pull-request release-preparation procedure, read-only metadata validation, and verified-`main` publication automation exist now. The historical `0.1.0-alpha.1` release remains immutable. Future release-bearing pull requests publish ordinary SemVer releases such as `0.2.0` and `0.2.1`; do not create a tag or GitHub release manually as a substitute.

## Release validation checklist

Complete this before merging a release-bearing pull request. Repeat the full checklist after a change to packaging, the bridge, diagnostics, or release automation. Use the data and migration checks below for data changes.

1. On Windows, run `./scripts/dev package-windows` from the exact commit under review.
2. Verify exactly one installer and its `.sha256` sidecar. Verify the checksum from a separate command.
3. Install the validation artifact. Confirm its bundled `FmDataBridge.dll` is a managed DLL with the same release version as the app.
4. Start the installed app. Confirm `%LOCALAPPDATA%\app.fmvaluescout\logs\fm-valuescout.log` contains startup and schema information, rotates within its limit, and contains no player rows, dump bodies, memory addresses, or local paths.
5. Complete the non-FM recovery checklist: closed-app backup, restore, update, and uninstall behavior.
6. With BepInEx already installed, install/update the bridge, restart FM26, and run one complete men's-database Load Data flow on the supported build.
7. Exercise Search, profiles, configurable tables, CSV enrichment, Planner, Academy, snapshot persistence, and the two guarded boost actions only after backing up the FM save.

Do not merge a release-bearing pull request if a required installed-app or FM validation is missing. Its successful merge will publish automatically.

## Data and migration checks

Do not repeat the full release validation checklist for routine data work.

- For feature work that reads or writes existing data without a schema migration, add focused behavior tests and run the normal feature validation.
- For additive or routine SQLite migrations, add representative old-schema fixtures and assert the required upgrade, preservation, rollback, and idempotency behavior.
- For a migration that deletes, transforms, reinterprets, or reassigns existing data, also run a small installed-app upgrade smoke from a closed-app backup. Confirm the affected data after the migration. This does not require the FM, bridge, or boost checks unless the change affects them.

Run the full release validation checklist only when the data change also affects packaging, the bridge, diagnostics, or release automation.

## Pull-request procedure

Every human-authored pull request uses the repository-local [`create-pr` skill](../../.agents/skills/create-pr/SKILL.md) and the repository template. Record exactly one release intent:

| Intent | Effect |
| --- | --- |
| `none` | No version change, dated changelog section, tag, or release. Relevant notes may remain under `Unreleased`. |
| `patch` | Prepare the next plain patch identity and a complete dated changelog section from all unreleased changes. |
| `minor` | Prepare the next plain minor identity and a complete dated changelog section from all unreleased changes. |
| `major` | Stop for a maintainer compatibility decision. |

`none` is a normal answer for an ordinary pull request; it is not a different PR type. A first release uses `minor` with no prior release tag and prepares `0.1.0`. A compatible capability advances the minor version (`0.1.0` → `0.2.0`); a compatible fix advances the patch version (`0.2.0` → `0.2.1`). The historical `0.1.0-alpha.1` remains valid only as an existing release identity; there is no public alpha counter or release-candidate sequence. The procedure validates prepared release metadata before it pushes and opens the normal template-complete draft PR. It never merges, tags, or publishes.

A release-bearing pull request also updates `release-preparation.json` with the matching version and intent, and increments its sequence. A `none` pull request leaves that file unchanged. The Release workflow packages and publishes only when that record changed in the exact successful Check SHA. If a release attempt fails before its draft exists, prepare a new release-bearing pull request for a new release identity; do not rely on a later `none` or Dependabot push to retry it.

Use `./scripts/dev release-metadata [latest-tag|none] [release-intent]` to validate version owners and the exact dated changelog section. For a pull-request intent, always pass both arguments; `none` by itself means that no prior tag exists. It is deterministic and read-only. The release workflow will call it without an intent after it discovers the latest tag.

## Verified-main publication

Every successful required `Check` run caused by a push to `main` evaluates release metadata for that exact SHA:

- An unchanged version is a successful no-op.
- A newer validated release builds the same Windows validation artifact, stages the installer and checksum in a matching temporary draft, uses the exact dated changelog section as the complete release body, verifies the assembled release, then publishes it as a normal GitHub release.
- A mismatch among the SHA, version, tag, changelog, existing draft, or published release fails closed. A retry for the same release identity is idempotent.

The `Release` workflow has read-only defaults. Only its final Windows package-and-publish job receives `contents: write`; it checks out the successful `Check` run's exact `head_sha`, rather than a later `main` commit. The required `check` also validates release metadata and, when release inputs change, stores the same Windows validation artifact and checksum for the installed-app checklist.

If staging fails after a temporary draft exists, keep it unpublished. Re-run the exact failed Release workflow only when the source SHA is unchanged; it can repair that matching draft. If source correction is required, remove the unpublished temporary draft and its matching tag before a new release-bearing PR produces a different SHA. Do not retarget a draft to a different commit or delete or overwrite a published release.

## Emergency withdrawal

If an automatically published release is unsafe:

1. Stop further release-bearing merges.
2. Mark the GitHub release as a draft or withdraw it through GitHub; do not delete the tag or release history before recording what happened.
3. Publish a short withdrawal notice that names the affected version and tells users not to install it. Do not disclose exploit details until a fix is available.
4. Prepare a corrective pull request under the normal release-intent procedure, rerun release validation, and allow the verified-main release flow to publish the replacement.
5. Create a private security advisory when the cause is a vulnerability and follow [SECURITY.md](../../SECURITY.md).

## Repository settings for publication

Confirm these manually in GitHub before the next release-bearing merge:

- Repository auto-merge, Dependabot alerts, Dependabot security updates, and private vulnerability reporting are enabled.
- The maintainer watches repository security alerts.
- Required `check` is strict and up to date; direct pushes, force pushes, and branch deletion remain restricted.
- Only stable pnpm patches and Cargo patches within a compatible release line (`>=1.0.0`, or `0.y.z` where `y > 0`) can receive auto-merge after aging at least 14 days. Cargo `0.0.z`, Actions, and NuGet routine updates remain disabled.
- The release path has no signing secret, updater token, or unreviewed write permission outside its final publication job.
