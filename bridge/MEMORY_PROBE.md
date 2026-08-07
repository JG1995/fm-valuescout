# Memory research probe

This is a developer-only workflow for finding possible FM26 player-memory locations. It does not add data to the app, change `dump.json`, or verify a production offset. Treat every correlation result as a hypothesis until it passes the evidence checklist below.

## Before you start

- Use a Windows Steam FM26 save supported by the bridge's current layout (`26.3`). Restart FM after `./scripts/dev bridge-install` so BepInEx loads the current DLL.
- Create the FM player view that supplies the CSV. Include `Unique ID` and the exact numeric fields you want to research.
- Keep the export and capture synchronized. Export the view, then capture it without advancing the game, changing saves, or changing the player set.
- Define each field's meaning before you correlate it. Academy `AT Apps`, `AT Gls`, and `Int Apps` mean all senior matches. Moneyball values are season-to-date at the capture point. Treat starts and substitute appearances as separate values. Assists are not a research target because FM does not export the required value.
- Keep the full exported UID set when it has 128 or fewer players. Do not trim it to make a convenient sample.

The plugin data directory is `%LOCALAPPDATA%\fm-valuescout\fm-bridge\`. When the CLI runs through WSL, pass its mounted location with `--bridge-dir`, for example `/mnt/c/Users/<Windows-user>/AppData/Local/fm-valuescout/fm-bridge`. Run the commands from the repository root. Create an absolute work directory for every CSV, capture, correlation report, and diagnostic:

```bash
repo_root="$(git rev-parse --show-toplevel)"
work_dir="$repo_root/.work/memory-probe"
bridge_dir=/mnt/c/Users/<Windows-user>/AppData/Local/fm-valuescout/fm-bridge
mkdir -p "$work_dir"
```

`.work/` is ignored by Git. Do not commit exports, `probe.json`, reports, screenshots, diagnostics, or machine-specific paths.

## Capture one synchronized export

Copy the FM export into the work directory. Use the CSV headers exactly as FM wrote them. Current exports use `Unique ID` and semicolon delimiters, so make both explicit in the command:

```bash
./scripts/dev memory-probe capture \
  --csv "$work_dir/academy.csv" \
  --uid-column "Unique ID" \
  --delimiter semicolon \
  --bridge-dir "$bridge_dir" \
  --request-id academy-a

cp "$bridge_dir/probe.json" "$work_dir/academy-a.probe.json"
```

`capture` writes `probe-request.json`, waits for its matching `probe-status.json`, and succeeds only after a fresh `ready` status points to a matching capture. Use a new request ID for every capture. A request ID cannot be reused while its matching status or capture remains.

Before correlation, check that the capture has the intended request ID, a supported game/layout version, exactly the CSV UID set, and a non-zero bounded player count. The bridge accepts at most 128 unique UIDs and captures at most 1,408 raw bytes per player (180,224 bytes total). A failed request writes only research status; it does not replace a prior successful `probe.json` or any production dump file.

Production `request.json` and `force-scan` requests take priority over a pending probe. Do not expect two scans to run at once.

## Correlate labeled CSV values

Start each capture with known anchors. The following command uses fields present in the Academy view:

```bash
./scripts/dev memory-probe correlate \
  --csv "$work_dir/academy.csv" \
  --capture "$work_dir/academy-a.probe.json" \
  --uid-column "Unique ID" \
  --delimiter semicolon \
  --field "uid=Unique ID" \
  --field "ca=CA" \
  --field "pa=PA" \
  --field "determination=Determination" \
  --field "all_senior_apps=AT Apps" \
  --field "all_senior_goals=AT Gls" \
  --field "international_apps=Int Apps" \
  > "$work_dir/academy-a.correlation.json"
```

Add only fields that have an explicit scalar meaning. Treat a localized `Transfer Value` range as display context, not as a market-value source value. A later feature may use a numeric market-value anchor only after its scalar semantics are independently established.

The default transform is `integer`. Declare a transform whenever the CSV display needs one:

```bash
--field "starts=Appearances" --transform "starts=appearances-starts"
--field "substitute_appearances=Appearances" --transform "substitute_appearances=appearances-subs"
--field "xg=xG" --transform "xg=decimal:<display-decimal-places>"
--field "distance=Distance" --transform "distance=unit-decimal:<unit>:<display-decimal-places>"
```

`decimal` and `unit-decimal` accept 0 through 6 display decimal places. Use the unit and precision shown by FM. Blank cells, whitespace, and `-` are missing values; the report lists their excluded UIDs instead of dropping them. Unsupported display text is an error, not a value to guess.

Read the JSON report before acting on it. It states the selected players, exclusions, candidate path, encoding, coverage, ambiguity, and evidence kind. Exact, rounded, and fixed-scale results are different evidence classes. A candidate needs varied truth across multiple eligible players; all-zero, one-player, or uniform fields are insufficient.

## Replicate before proposing a product field

One correlation report never proves an offset. Repeat the same CSV-view contract with an independently captured, disjoint UID set. For every capture:

1. Keep the export and capture synchronized to the same open save state.
2. Recover the known UID, CA, PA, Determination-times-five, and applicable scalar market-value anchors.
3. Correlate the target field with the same declared normalization.
4. Compare the reports. The same relative path and encoding must recur, no equally strong conflicting path may remain, and the field must have varied multi-player evidence in each set.

For sparse Moneyball values, also check that related statistics form a coherent nearby structure. Do not accept a rare-event result merely because many zeroes agree.

Before/after capture is optional supporting evidence. If a controlled change is available, use the same UIDs and compatible game/layout/bridge metadata:

```bash
./scripts/dev memory-probe diff \
  --before-csv "$work_dir/before.csv" \
  --after-csv "$work_dir/after.csv" \
  --before-capture "$work_dir/before.probe.json" \
  --after-capture "$work_dir/after.probe.json" \
  --uid-column "Unique ID" \
  --delimiter semicolon \
  --field "starts=Appearances" \
  --transform "starts=appearances-starts" \
  > "$work_dir/appearances.diff.json"
```

Do not require a before/after sample when FM cannot provide one. It cannot replace independent synchronized captures.

## Evidence required for later implementation

A later feature may propose a production field only after the field meaning is written down and the following evidence is retained in that feature's ledger:

- The CSV view, header, transform, and statistic semantics are explicit.
- Each capture reports the expected request ID, supported build metadata, exact UID set, and bounded bytes.
- Known anchors recover in every capture.
- The target path and encoding agree across independent synchronized UID sets, with no equally strong conflict.
- The report records whether the match is exact, rounded, fixed-scale, ambiguous, or unsupported.
- The candidate remains separate from the frozen schema-v5 product dump until a later feature plans, validates, and implements it.

If a request fails, read its `probe-status.json` error, correct the export/save-state problem, and capture again with a new request ID. Do not widen the capture bounds, add a guessed offset, or use raw memory as product data.

## Manual boundary checks

Use a disposable test state when you need to exercise failure paths. Confirm that a malformed request, unsupported layout, missing UID, or 129-player request fails only the research protocol and leaves `dump.json`, `status.json`, and the prior successful `probe.json` unchanged.

Run `./scripts/dev memory-probe --help` for the current command surface. The CLI reports hypotheses only; it never verifies a production memory offset.
