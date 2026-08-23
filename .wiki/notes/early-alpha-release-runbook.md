# Early-alpha release runbook

This runbook separates evidence required before a release-bearing pull request merges from the automatic publication that follows verified `main`. It is written for the solo maintainer and does not authorize a push, merge, tag, or release by itself.

## Current state

The Windows validation command exists now:

```bash
./scripts/dev package-windows
```

Run it on a native Windows host. It builds the locked bridge from the checked-out source, produces one unsigned Windows x64 NSIS installer and its SHA-256 sidecar under `.release/windows/<version>/`, and never publishes anything.

The universal pull-request release-preparation procedure, read-only metadata validation, and verified-`main` publication automation exist now. The historical `0.1.0-alpha.1` release remains immutable. Future release-bearing pull requests publish ordinary SemVer releases such as `0.2.0` and `0.2.1`; do not create a tag or GitHub release manually as a substitute.

## Routine automated release validation

For a release-bearing pull request, the required `Check` validates release metadata and runs `./scripts/dev package-windows` against the exact pull-request SHA. Check builds the Windows validation package, but it does not upload or retain that artifact.

After merge, `Check` repeats the same metadata and package coverage on the exact `main` SHA. The workflow-run `Release` job then checks out that verified SHA, builds the publishable installer, stages and verifies the release assets, and publishes the release.

Do not run a local Windows package only because release metadata or version owners changed. The automated Check and Release path covers that routine case.

## Risk-triggered manual validation

Run the following native Windows validation before merge only when the change affects packaging, the bridge, diagnostics, or release automation. The local artifact supports these installed-app checks; it is not a routine duplicate of the Check package.

1. On Windows, run `./scripts/dev package-windows` from the exact commit under review.
2. Verify exactly one installer and its `.sha256` sidecar. Verify the checksum from a separate command.
3. Install the validation artifact. Confirm its bundled `FmDataBridge.dll` is a managed DLL with the same release version as the app.
4. Start the installed app. Confirm `%LOCALAPPDATA%\app.fmvaluescout\logs\fm-valuescout.log` contains startup and schema information, rotates within its limit, and contains no player rows, dump bodies, memory addresses, or local paths.
5. Complete the non-FM recovery checklist: closed-app backup, restore, update, and uninstall behavior.
6. With BepInEx already installed, install/update the bridge, restart FM26, and run one complete men's-database Load Data flow on the supported build.
7. Exercise Search, profiles, configurable tables, CSV enrichment, Planner, Academy, snapshot persistence, and the two guarded boost actions only after backing up the FM save.

Do not merge when a required risk-triggered installed-app or FM validation is missing. Release-bearing changes that do not meet a risk trigger use the automated validation above.

## Data and migration checks

Do not repeat the full release validation checklist for routine data work.

- For feature work that reads or writes existing data without a schema migration, add focused behavior tests and run the normal feature validation.
- For additive or routine SQLite migrations, add representative old-schema fixtures and assert the required upgrade, preservation, rollback, and idempotency behavior.
- For a migration that deletes, transforms, reinterprets, or reassigns existing data, also run a small installed-app upgrade smoke from a closed-app backup. Confirm the affected data after the migration. This does not require the FM, bridge, or boost checks unless the change affects them.

Run the full release validation checklist only when the data change also affects packaging, the bridge, diagnostics, or release automation.

## Pull-request procedure

Every human-authored pull request uses the repository-local [`create-pr` skill](../../.pi/skills/create-pr/SKILL.md) and the repository template. Record exactly one release intent:

| Intent | Effect |
| --- | --- |
| `none` | No version change, dated changelog section, tag, or release. Relevant notes may remain under `Unreleased`. This also applies to an intermediate PR in a valid fingerprinted feature whose final PR owns one later release outcome. |
| `patch` | Prepare the next plain patch identity and a complete dated changelog section from all unreleased changes. |
| `minor` | Prepare the next plain minor identity and a complete dated changelog section from all unreleased changes. |
| `major` | Stop for a maintainer compatibility decision. |

`none` is a normal answer for an ordinary pull request; it is not a different PR type. A user-visible intermediate PR can use `none` only when an accepted schema 2 feature ledger has an unchanged Delivery fingerprint, marks that PR's Feature close-out as `Not required`, and assigns the complete non-`none` release contract to a later final PR. Record the ledger path and fingerprint in the intermediate PR Notes. The final PR prepares the complete unreleased range from the latest reachable tag; intermediate PRs do not change any release owner.

A first release uses `minor` with no prior release tag and prepares `0.1.0`. A compatible capability advances the minor version (`0.1.0` → `0.2.0`); a compatible fix advances the patch version (`0.2.0` → `0.2.1`). The historical `0.1.0-alpha.1` remains valid only as an existing release identity; there is no public alpha counter or release-candidate sequence. The procedure validates prepared release metadata before it pushes and opens the normal template-complete draft PR. It never merges, tags, or publishes.

A release-bearing pull request also updates `release-preparation.json` with the matching version and intent, and increments its sequence. A `none` pull request leaves that file unchanged. The Release workflow packages and publishes only when that record changed in the exact successful Check SHA. If a release attempt fails before its draft exists, prepare a new release-bearing pull request for a new release identity; do not rely on a later `none` or Dependabot push to retry it.

Use `./scripts/dev release-metadata [latest-tag|none] [release-intent]` to validate version owners and the exact dated changelog section. For a pull-request intent, always pass both arguments; `none` by itself means that no prior tag exists. It is deterministic and read-only. The release workflow will call it without an intent after it discovers the latest tag.

## Verified-main publication

Every successful required `Check` run caused by a push to `main` evaluates release metadata for that exact SHA:

- An unchanged version is a successful no-op.
- A newer validated release builds the same Windows validation artifact, stages the installer and checksum in a matching temporary draft, uses the exact dated changelog section as the complete release body, verifies the assembled release, then publishes it as a normal GitHub release.
- A mismatch among the SHA, version, tag, changelog, existing draft, or published release fails closed. A retry for the same release identity is idempotent.

The `Release` workflow has read-only defaults. Only its final Windows package-and-publish job receives `contents: write`; it checks out the successful `Check` run's exact `head_sha`, rather than a later `main` commit. The required `Check` validates release metadata and builds the Windows validation package, but it does not upload or retain that artifact. The `Release` workflow creates the publishable installer after verified `main`, stages and verifies it, and publishes it.

If staging fails after a temporary draft exists, keep it unpublished. Re-run the exact failed Release workflow only when the source SHA is unchanged; it can repair that matching draft. If source correction is required, remove the unpublished temporary draft and its matching tag before a new release-bearing PR produces a different SHA. Do not retarget a draft to a different commit or delete or overwrite a published release.

### Schema 2 release commands

A release-bearing schema 2 ledger must bind its post-merge Release phase to the synchronized final merge. The merge already triggers publication through verified `main`; the ledger's release command waits for that exact automatic workflow run instead of creating a tag or release.

Use this exact release-command shape:

```bash
bash -lc 'set -euo pipefail; sha=$(git rev-parse HEAD); for _ in {1..120}; do run=$(gh run list --repo JG1995/fm-valuescout --workflow Release --branch main --event workflow_run --limit 20 --json databaseId,headSha --jq ".[] | select(.headSha == \"$sha\") | .databaseId" | head -n1); if [[ -n "$run" ]]; then gh run watch "$run" --repo JG1995/fm-valuescout --exit-status; exit; fi; sleep 10; done; exit 1'
```

The Release verification must hard-code the ledger's exact SemVer target before plan review. Copy the command below into the ledger, replace `<exact-version>` once, and record the fully instantiated command in the Delivery fingerprint. It verifies that the tag points to synchronized `HEAD`, the published release has the exact title and changelog body, exactly one installer and checksum exist, GitHub reports the installer digest, and the downloaded checksum matches it.

```bash
bash -lc 'set -euo pipefail; version=<exact-version>; tag=v$version; sha=$(git rev-parse HEAD); tmp=$(mktemp -d); trap '\''rm -rf "$tmp"'\'' EXIT; test "$(gh api repos/JG1995/fm-valuescout/git/ref/tags/$tag --jq .object.sha)" = "$sha"; gh release view "$tag" --repo JG1995/fm-valuescout --json name,body,tagName,targetCommitish,isDraft,isPrerelease,assets > "$tmp/release.json"; gh release download "$tag" --repo JG1995/fm-valuescout --pattern "FM-ValueScout_${version}_x64-setup.exe.sha256" --dir "$tmp"; node --input-type=module -e '\''import { readFileSync } from "node:fs"; import { extractDatedSection } from "./scripts/release-metadata.mjs"; const [sha, version, releasePath, checksumPath] = process.argv.slice(1); const release = JSON.parse(readFileSync(releasePath, "utf8")); const tag = "v" + version; const installer = "FM-ValueScout_" + version + "_x64-setup.exe"; const checksum = installer + ".sha256"; const names = release.assets.map((asset) => asset.name).sort(); const binary = release.assets.find((asset) => asset.name === installer); const notes = extractDatedSection(readFileSync("CHANGELOG.md", "utf8"), version); const checksumText = readFileSync(checksumPath, "utf8").trim(); const valid = release.tagName === tag && release.name === "FM ValueScout " + tag && release.body === notes && release.targetCommitish === sha && release.isDraft === false && release.isPrerelease === false && names.length === 2 && names[0] === installer && names[1] === checksum && /^sha256:[0-9a-f]{64}$/.test(binary?.digest ?? "") && checksumText === binary.digest.slice(7) + " *" + installer; if (!valid) process.exit(1);'\'' "$sha" "$version" "$tmp/release.json" "$tmp/FM-ValueScout_${version}_x64-setup.exe.sha256"'
```

Run release verification first. If it passes, adopt the existing exact release and do not rerun publication. Otherwise run the release command once, then run verification again. Stop if the workflow fails, the exact run does not appear within 20 minutes, or any release evidence differs; do not create a manual substitute.

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
