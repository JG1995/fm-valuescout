# Early-alpha release runbook

This runbook separates evidence required before a release-bearing pull request merges from the automatic publication that follows verified `main`. It is written for the solo maintainer and does not authorize a push, merge, tag, or release by itself.

## Current state

The Windows candidate command exists now:

```bash
./scripts/dev package-windows
```

Run it on a native Windows host. It builds the locked bridge from the checked-out source, produces one unsigned Windows x64 NSIS installer and its SHA-256 sidecar under `.release/windows/<version>/`, and never publishes anything.

The universal pull-request release-preparation procedure, read-only metadata validation, and verified-`main` publication automation exist now. Until the initial release-bearing pull request completes its acceptance evidence and merges, there is no released installer. Do not create a tag or GitHub release manually as a substitute.

## Candidate acceptance checklist

Complete this before merging the first release-bearing pull request and after any change to packaging, the bridge, persistent data, diagnostics, or release automation:

1. On Windows, run `./scripts/dev package-windows` from the exact commit under review.
2. Verify exactly one installer and its `.sha256` sidecar. Verify the checksum from a separate command.
3. Install the candidate. Confirm its bundled `FmDataBridge.dll` is a managed DLL with the same release version as the app.
4. Start the installed app. Confirm `%LOCALAPPDATA%\app.fmvaluescout\logs\fm-valuescout.log` contains startup and schema information, rotates within its limit, and contains no player rows, dump bodies, memory addresses, or local paths.
5. Complete the non-FM recovery checklist: closed-app backup, restore, update, and uninstall behavior.
6. With BepInEx already installed, install/update the bridge, restart FM26, and run one complete men's-database Load Data flow on the supported build.
7. Exercise Search, profiles, configurable tables, CSV enrichment, Planner, Academy, snapshot persistence, and the two guarded boost actions only after backing up the FM save.

Do not merge the initial release-bearing pull request if a required installed-app or FM validation is missing. Its successful merge will eventually publish automatically.

## Pull-request procedure

Every human-authored pull request uses the repository-local [`create-pr` skill](../../.agents/skills/create-pr/SKILL.md) and the repository template. Record exactly one release intent:

| Intent | Effect |
| --- | --- |
| `none` | No version change, dated changelog section, tag, or release. Relevant notes may remain under `Unreleased`. |
| `patch` | Prepare the next prerelease patch identity and a complete dated changelog section from all unreleased changes. |
| `minor` | Prepare the next prerelease minor identity and a complete dated changelog section from all unreleased changes. |
| `major` | Stop for a maintainer compatibility decision. |

`none` is a normal answer for an ordinary pull request; it is not a different PR type. The first prerelease uses `minor` with no prior release tag and prepares `0.1.0-alpha.1`. A patch after an alpha release advances the alpha counter. A minor release advances the minor version and begins a new `alpha.1` sequence. The procedure validates prepared release metadata before it pushes and opens the normal template-complete draft PR. It never merges, tags, or publishes.

Use `./scripts/dev release-metadata [latest-tag|none] [release-intent]` to validate version owners and the exact dated changelog section. For a pull-request intent, always pass both arguments; `none` by itself means that no prior tag exists. It is deterministic and read-only. The release workflow will call it without an intent after it discovers the latest tag.

## Verified-main publication

Every successful required `Check` run caused by a push to `main` evaluates release metadata for that exact SHA:

- An unchanged version is a successful no-op.
- A newer validated prerelease builds the same Windows candidate command, stages the installer and checksum in a matching temporary draft, uses the exact dated changelog section as the complete release body, verifies the assembled release, then publishes it as a prerelease.
- A mismatch among the SHA, version, tag, changelog, existing draft, or published release fails closed. A retry for the same release identity is idempotent.

The `Release` workflow has read-only defaults. Only its final Windows package-and-publish job receives `contents: write`; it checks out the successful `Check` run's exact `head_sha`, rather than a later `main` commit. The required `check` also validates release metadata and, when release inputs change, stores the same Windows candidate and checksum for the installed-app acceptance checklist.

If staging fails, keep the release unpublished, inspect the failed workflow and temporary draft only for the same SHA and version, correct the source through the normal PR process, and let a new verified `main` evaluation run. Do not retarget a draft to a different commit or overwrite a published release.

## Emergency withdrawal

If an automatically published prerelease is unsafe:

1. Stop further release-bearing merges.
2. Mark the GitHub prerelease as a draft or withdraw it through GitHub; do not delete the tag or release history before recording what happened.
3. Publish a short withdrawal notice that names the affected version and tells users not to install it. Do not disclose exploit details until a fix is available.
4. Prepare a corrective pull request under the normal release-intent procedure, rerun candidate acceptance, and allow the verified-main release flow to publish the replacement.
5. Create a private security advisory when the cause is a vulnerability and follow [SECURITY.md](../../SECURITY.md).

## Repository settings before first publication

Confirm these manually in GitHub before the first release-bearing merge:

- Repository auto-merge, Dependabot alerts, Dependabot security updates, and private vulnerability reporting are enabled.
- The maintainer watches repository security alerts.
- Required `check` is strict and up to date; direct pushes, force pushes, and branch deletion remain restricted.
- Only stable pnpm/Cargo patch Dependabot pull requests aged at least 14 days can receive auto-merge. Actions and NuGet routine updates remain disabled.
- The release path has no signing secret, updater token, or unreviewed write permission outside its final publication job.
