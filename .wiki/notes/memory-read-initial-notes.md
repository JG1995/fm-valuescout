````markdown
# FM26 Data Bridge — MVP Implementation Plan

## Objective

Build a read-only integration capable of extracting basic player information from a running Football Manager 26 save and making it available to a Tauri desktop application.

The MVP should extract:

- Player ID
- Name
- Date of birth and age
- Nationality
- Height
- Preferred foot
- Natural positions
- Current Ability
- Potential Ability
- Visible attributes
- Hidden attributes
- Personality attributes
- Current club
- Parent club
- Loan status
- Division
- Team level
- Contract expiry
- Weekly wage
- Transfer status
- Market value
- Player reputation

Performance statistics, match data, tactical familiarity, injuries, happiness and other advanced data are explicitly out of scope for the initial MVP.

---

# High-Level Architecture

```text
Football Manager 26 — fm.exe
│
├── BepInEx 6 IL2CPP
│   └── FM Data Bridge plugin — C#
│       ├── Detects requests from the desktop app
│       ├── Reads FM memory safely
│       ├── Finds player, contract, team and club objects
│       ├── Serializes extracted data
│       └── Writes status and dump files
│
└── %LOCALAPPDATA%\YourApp\fm-bridge\
    ├── request.json
    ├── status.json
    ├── dump.json
    └── diagnostics.txt

Tauri desktop application
│
├── Rust backend
│   ├── Installs and verifies the FM plugin
│   ├── Requests new dumps
│   ├── Watches scan progress
│   ├── Validates and imports dump data
│   └── Stores/query players through SQLite
│
└── Frontend
    ├── Displays bridge and FM status
    ├── Starts a new data scan
    ├── Searches and filters players
    └── Displays player profiles
````

---

# Core Principles

* The integration must be read-only.
* Never write to FM memory.
* Avoid direct unsafe pointer dereferencing where possible.
* Use `ReadProcessMemory` even inside `fm.exe` so invalid addresses fail safely.
* Never overwrite a valid dump with an empty or failed scan.
* Keep memory-layout knowledge outside the frontend.
* Treat every major FM patch as a separately supported memory layout.
* Do not pass the complete player database through Tauri events.
* Store and query the extracted data in the Rust backend.
* Keep all scanning work off Unity's main thread.

---

# Suggested Repository Structure

```text
project-root/
├── bridge/
│   ├── FmDataBridge.csproj
│   ├── Plugin.cs
│   │
│   ├── Protocol/
│   │   ├── BridgeRequest.cs
│   │   ├── BridgeStatus.cs
│   │   └── BridgePaths.cs
│   │
│   ├── Memory/
│   │   ├── IMemoryReader.cs
│   │   ├── WindowsMemoryReader.cs
│   │   ├── MemoryRegion.cs
│   │   ├── RegionEnumerator.cs
│   │   ├── ModuleLocator.cs
│   │   └── ModuleImageCache.cs
│   │
│   ├── Layouts/
│   │   ├── IFmMemoryLayout.cs
│   │   ├── LayoutRegistry.cs
│   │   └── Fm263Layout.cs
│   │
│   ├── Scanning/
│   │   ├── PersonCandidate.cs
│   │   ├── PersonScanner.cs
│   │   ├── ClassOffsetResolver.cs
│   │   └── ScanProgress.cs
│   │
│   ├── Extraction/
│   │   ├── PlayerReader.cs
│   │   ├── StaffReader.cs
│   │   ├── NameReader.cs
│   │   ├── NationReader.cs
│   │   ├── ContractReader.cs
│   │   ├── ClubResolver.cs
│   │   ├── SquadResolver.cs
│   │   └── FmDateDecoder.cs
│   │
│   ├── Models/
│   │   ├── ExtractedPlayer.cs
│   │   ├── ExtractedStaff.cs
│   │   ├── ExtractedClub.cs
│   │   └── DumpDocument.cs
│   │
│   ├── Output/
│   │   ├── DumpWriter.cs
│   │   ├── StatusWriter.cs
│   │   └── DiagnosticsWriter.cs
│   │
│   └── Tests/
│
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands.rs
│   │   ├── bridge/
│   │   │   ├── mod.rs
│   │   │   ├── request.rs
│   │   │   ├── status.rs
│   │   │   └── watcher.rs
│   │   ├── database/
│   │   │   ├── mod.rs
│   │   │   ├── migrations.rs
│   │   │   ├── importer.rs
│   │   │   └── player_queries.rs
│   │   ├── installation/
│   │   │   ├── fm_locator.rs
│   │   │   ├── bepinex.rs
│   │   │   └── plugin_installer.rs
│   │   └── models/
│   ├── resources/
│   │   ├── bridge/
│   │   └── bepinex/
│   └── migrations/
│
├── src/
│   ├── lib/
│   │   ├── api/
│   │   ├── components/
│   │   ├── stores/
│   │   └── types/
│   └── routes/
│
├── docs/
│   ├── architecture.md
│   ├── memory-layouts.md
│   ├── offset-discovery.md
│   ├── protocol.md
│   └── troubleshooting.md
│
└── README.md
```

---

# Workstream 1 — BepInEx Plugin Bootstrap

## Tasks

* [ ] Install a known-compatible BepInEx 6 IL2CPP build into FM26.
* [ ] Create a .NET 6 C# class-library project.
* [ ] Reference the required BepInEx and IL2CPP assemblies.
* [ ] Implement a minimal `BasePlugin`.
* [ ] Confirm that the plugin loads inside `fm.exe`.
* [ ] Log the plugin version and FM process information.
* [ ] Detect whether `GameAssembly.dll` is loaded.
* [ ] Detect whether `game_plugin.dll` is loaded.
* [ ] Write a basic `status.json` file on successful load.
* [ ] Ensure plugin errors are written to the BepInEx log.

## Acceptance criteria

* FM26 starts successfully with the plugin installed.
* The plugin logs a successful load.
* The plugin reports whether the required FM modules are present.
* Removing the plugin DLL restores the original game state.

---

# Workstream 2 — Bridge Request Protocol

## Tasks

* [ ] Define a versioned request schema.
* [ ] Define a versioned status schema.
* [ ] Create the bridge data directory under `%LOCALAPPDATA%`.
* [ ] Poll for requests on a normal background thread.
* [ ] Reject stale requests.
* [ ] Prevent more than one scan from running simultaneously.
* [ ] Write explicit `idle`, `scanning`, `complete` and `error` states.
* [ ] Include a unique request ID in every status update.
* [ ] Write request and status files atomically.

## Example request

```json
{
  "protocolVersion": 1,
  "requestId": "1bf16615-cc25-4ddf-97e7-d55770bad0db",
  "createdAtUtc": "2026-07-28T18:30:00Z",
  "operation": "full-dump"
}
```

## Example status

```json
{
  "protocolVersion": 1,
  "requestId": "1bf16615-cc25-4ddf-97e7-d55770bad0db",
  "state": "scanning",
  "phase": "player-scan",
  "progress": 0.45,
  "playersFound": 23104,
  "staffFound": 0,
  "updatedAtUtc": "2026-07-28T18:30:06Z"
}
```

## Acceptance criteria

* The desktop app can trigger a request without using an in-game hotkey.
* A stale request does not run after restarting FM.
* Duplicate requests do not start parallel scans.
* Scan failures result in a readable error status.

---

# Workstream 3 — Safe Memory Access

## Tasks

* [ ] Wrap `GetCurrentProcess`.
* [ ] Wrap `ReadProcessMemory`.
* [ ] Wrap `VirtualQuery`.
* [ ] Implement typed safe reads:

  * [ ] Pointer
  * [ ] Byte
  * [ ] Unsigned 16-bit integer
  * [ ] Unsigned 32-bit integer
  * [ ] Signed 32-bit integer
  * [ ] Byte arrays
* [ ] Enumerate committed private read/write memory regions.
* [ ] Exclude guard and no-access pages.
* [ ] Add maximum-region-size sanity limits.
* [ ] Locate `GameAssembly.dll`.
* [ ] Locate `game_plugin.dll`.
* [ ] Cache module images for fast metadata reads.
* [ ] Add cancellation checks between scan blocks.
* [ ] Add unit tests using a fake memory reader.

## Acceptance criteria

* Invalid reads return failure instead of crashing FM.
* The scanner can enumerate relevant memory regions.
* Module base addresses and versions are recorded in diagnostics.
* The memory reader can be tested without launching FM.

---

# Workstream 4 — Versioned Memory Layouts

## Tasks

* [ ] Create an `IFmMemoryLayout` abstraction.
* [ ] Store offsets by supported FM version.
* [ ] Implement an initial FM 26.3 layout.
* [ ] Detect the loaded `game_plugin.dll` file version.
* [ ] Resolve the correct layout from the game version.
* [ ] Refuse to create a new dump for unsupported major/minor versions.
* [ ] Preserve the most recent valid dump after a version mismatch.
* [ ] Record all active offsets in diagnostics.
* [ ] Document the process for repinning offsets after FM updates.

## Layout categories

```text
Object header
Person fields
Player fields
Player attributes
Player positions
Staff fields
Contract fields
Nation fields
Team fields
Club fields
Competition fields
```

## Acceptance criteria

* The bridge clearly reports the detected FM version.
* Unsupported versions fail closed.
* Offset constants are not scattered across extraction code.
* Supporting a new FM patch requires adding or updating a layout.

---

# Workstream 5 — Player Object Discovery

## Tasks

* [ ] Scan selected heap regions in fixed-size chunks.
* [ ] Examine aligned pointer-sized values.
* [ ] Reject values outside the FM module address ranges.
* [ ] Resolve class metadata from candidate vtables.
* [ ] Determine the dynamic class offset.
* [ ] Identify:

  * [ ] Pure player objects
  * [ ] Player/staff objects
  * [ ] Staff objects
  * [ ] Human-manager objects
* [ ] Read and validate the object UID.
* [ ] Read and validate CA and PA.
* [ ] Reject impossible values.
* [ ] Deduplicate candidates by UID.
* [ ] Cap scanner worker count.
* [ ] Give each worker its own read buffer and result collections.
* [ ] Merge results after scanning completes.
* [ ] Generate a class-offset histogram for diagnostics.

## Initial candidate validation

A candidate player should only be accepted when:

* Its vtable points into a known FM module.
* Its class metadata produces a recognized player offset.
* Its UID is neither zero nor `0xFFFFFFFF`.
* CA is within `1..200`.
* PA is within `1..200`.
* The candidate has a readable person block.
* Additional sanity checks do not detect invalid data.

## Acceptance criteria

* A loaded save produces a plausible player count.
* The same player is not emitted more than once.
* No-save or startup states do not overwrite valid data.
* Known players have correct CA and PA values.

---

# Workstream 6 — Basic Player Extraction

## Tasks

* [ ] Decode player names.
* [ ] Decode common names.
* [ ] Decode dates of birth.
* [ ] Calculate age using the in-game date where available.
* [ ] Decode nationality.
* [ ] Decode height.
* [ ] Decode preferred foot.
* [ ] Decode natural positions.
* [ ] Decode visible attributes.
* [ ] Decode hidden performance attributes.
* [ ] Decode personality attributes.
* [ ] Decode player reputation.
* [ ] Normalize FM's attribute storage to the `1..20` scale.
* [ ] Record unknown or invalid values as `null`.
* [ ] Avoid silently replacing invalid data with zero.

## Minimum player model

```json
{
  "id": 123456,
  "name": "Example Player",
  "birthYear": 2005,
  "birthDayOfYear": 142,
  "age": 21,
  "nationalities": ["Denmark"],
  "heightCm": 186,
  "preferredFoot": "right",
  "positions": {
    "DC": 20,
    "DM": 14
  },
  "currentAbility": 132,
  "potentialAbility": 168,
  "reputation": {
    "current": 5400,
    "world": 4200
  },
  "attributes": {
    "Acceleration": 13,
    "Pace": 14,
    "Strength": 15,
    "Passing": 11
  },
  "hiddenAttributes": {
    "Consistency": 12,
    "ImportantMatches": 14,
    "InjuryProneness": 7
  },
  "personality": {
    "Ambition": 16,
    "Professionalism": 15,
    "Pressure": 13,
    "Loyalty": 10
  }
}
```

## Acceptance criteria

* At least 20 known players match the values displayed in FM.
* Attribute values are consistently normalized.
* Names containing non-ASCII characters are serialized correctly.
* Invalid pointers do not cause partial or corrupted player records.

---

# Workstream 7 — Contract Extraction

## Tasks

* [ ] Resolve the player's full-contract object.
* [ ] Extract weekly wage.
* [ ] Extract contract expiry.
* [ ] Extract transfer-list status.
* [ ] Extract loan-list status.
* [ ] Extract not-for-sale status.
* [ ] Extract set-for-release status.
* [ ] Extract squad number where available.
* [ ] Extract FM's stored market or guide value.
* [ ] Preserve unknown values as `null`.
* [ ] Validate currency and unit assumptions.

## Acceptance criteria

* Contract dates match FM for known players.
* Weekly wages match FM's base stored currency.
* Listed and not-for-sale players are identified correctly.
* Players without contracts do not cause extraction failures.

---

# Workstream 8 — Club, Team and Loan Resolution

## Tasks

* [ ] Follow the contract-to-team-to-club object chain.
* [ ] Extract parent club from the contract chain.
* [ ] Discover club objects.
* [ ] Walk club team arrays.
* [ ] Walk team squad arrays.
* [ ] Associate squad entries with known person addresses.
* [ ] Handle squad-entry wrapper objects.
* [ ] Determine current playing club.
* [ ] Determine parent club.
* [ ] Detect loans when current club differs from parent club.
* [ ] Determine team level:

  * [ ] First team
  * [ ] Reserve team
  * [ ] Under-21 team
  * [ ] Under-19 or youth team
* [ ] Extract club reputation.
* [ ] Extract competition or division.
* [ ] Identify the human manager.
* [ ] Identify the human manager's current club.
* [ ] Define deterministic conflict resolution when a player appears in multiple squads.

## Important distinction

```text
Parent club:
Derived from the player's full contract.

Current club:
Derived from the squad in which the player currently appears.
```

For a loaned player:

```text
parentClub != currentClub
```

## Acceptance criteria

* Regular first-team players show the correct club.
* Reserve and youth players show the correct team level.
* Loaned-in players show both current and parent clubs.
* Loaned-out players show both current and parent clubs.
* The user's managed club is correctly identified.

---

# Workstream 9 — In-Game Date

## Tasks

* [ ] Investigate the current in-game date source.
* [ ] Attempt to resolve the date through the human manager's team.
* [ ] Implement an explicit date-source field.
* [ ] Fall back to an inferred season year when exact date discovery fails.
* [ ] Recalculate player ages after resolving the date.

## Example metadata

```json
{
  "gameDate": "2026-08-14",
  "gameDateSource": "memory"
}
```

Possible source values:

```text
memory
derived
unknown
```

## Acceptance criteria

* Player ages match FM on most dates.
* The dump clearly states whether the date was read or inferred.
* Failure to find the date does not block player extraction.

---

# Workstream 10 — Dump Serialization

## Tasks

* [ ] Define a stable versioned dump schema.
* [ ] Include bridge, protocol and game versions.
* [ ] Include extraction timestamps.
* [ ] Include manager and managed-club metadata.
* [ ] Stream output rather than building the entire JSON in memory.
* [ ] Write to a temporary file first.
* [ ] Flush and close the temporary file.
* [ ] Atomically replace the previous dump.
* [ ] Never replace the previous dump when zero players are found.
* [ ] Write diagnostics even after a failed scan.
* [ ] Optionally compute a dump checksum.
* [ ] Include record counts and dump size in completion status.

## Suggested dump metadata

```json
{
  "schemaVersion": 1,
  "generatedAtUtc": "2026-07-28T18:31:10Z",
  "gameVersion": "26.3.2.2329565",
  "supportedGameVersion": "26.3",
  "bridgeVersion": "0.1.0",
  "protocolVersion": 1,
  "gameDate": "2026-08-14",
  "gameDateSource": "memory",
  "manager": "Example Manager",
  "managedClub": "Example FC",
  "currency": "GBP",
  "counts": {
    "players": 52184,
    "staff": 24102
  }
}
```

## Acceptance criteria

* The desktop app never observes partially written JSON.
* A failed scan leaves the previous valid dump intact.
* Large saves can be serialized without excessive memory growth.
* The dump schema can evolve without breaking older clients silently.

---

# Workstream 11 — Diagnostics

## Tasks

* [ ] Record detected module addresses.
* [ ] Record detected module versions.
* [ ] Record scanned memory-region count.
* [ ] Record total scanned bytes.
* [ ] Record scan duration by phase.
* [ ] Record candidate count.
* [ ] Record accepted-player count.
* [ ] Record rejected-candidate reasons.
* [ ] Record class-offset histogram.
* [ ] Record player/staff deduplication counts.
* [ ] Record club and squad association counts.
* [ ] Include sample extracted players.
* [ ] Include active memory-layout values.
* [ ] Add repinning hints when the game version differs.
* [ ] Provide a desktop action to export diagnostics.

## Acceptance criteria

* A failed scan produces actionable information.
* Offset changes can be investigated without attaching a debugger.
* Users can export all relevant diagnostic files from the desktop app.

---

# Workstream 12 — Tauri Bridge Integration

## Tasks

* [ ] Resolve the bridge data directory in Rust.
* [ ] Add a `request_fm_dump` command.
* [ ] Add a `get_bridge_status` command.
* [ ] Add a background status watcher.
* [ ] Emit only small progress events to the frontend.
* [ ] Detect whether `fm.exe` is running.
* [ ] Detect whether the bridge plugin is installed.
* [ ] Detect whether BepInEx interop generation has completed.
* [ ] Validate dump schema before importing.
* [ ] Reject incomplete or unsupported dump versions.
* [ ] Display clear errors for:

  * [ ] FM not running
  * [ ] No save loaded
  * [ ] Plugin missing
  * [ ] BepInEx not initialized
  * [ ] Unsupported FM version
  * [ ] Scan failure
  * [ ] Dump parse failure

## Suggested Tauri commands

```text
get_fm_status
get_bridge_installation_status
install_fm_bridge
remove_fm_bridge
request_fm_dump
get_bridge_status
import_latest_dump
query_players
get_player
export_diagnostics
```

## Acceptance criteria

* The frontend never accesses arbitrary filesystem paths directly.
* The user can trigger and monitor a scan from the Tauri UI.
* Progress updates do not contain the actual player database.
* The app handles closing or restarting FM gracefully.

---

# Workstream 13 — SQLite Import and Queries

## Tasks

* [ ] Create SQLite migrations.
* [ ] Store dump metadata.
* [ ] Store players.
* [ ] Store attributes in queryable columns or a related table.
* [ ] Store positions.
* [ ] Store nationality relationships.
* [ ] Import dumps inside a transaction.
* [ ] Replace the current imported snapshot only after a successful import.
* [ ] Add indexes for common filters.
* [ ] Implement pagination.
* [ ] Implement sorting.
* [ ] Implement basic filters.
* [ ] Return player summaries rather than complete player records for list views.

## Suggested tables

```text
dump_metadata
players
player_attributes
player_positions
player_nationalities
clubs
competitions
```

## Initial filters

* Name
* Club
* Division
* Nationality
* Age
* Position
* Current Ability
* Potential Ability
* Market value
* Weekly wage
* Contract expiry
* Preferred foot
* Individual attribute minimums

## Acceptance criteria

* Large databases do not need to be loaded entirely into the webview.
* Player lists are paginated and responsive.
* Import failures leave the previous database intact.
* Common player searches complete quickly.

---

# Workstream 14 — Installation and Removal

## Tasks

* [ ] Detect common Steam FM26 installation paths.
* [ ] Detect additional Steam-library folders.
* [ ] Allow manual folder selection.
* [ ] Validate the selected FM installation.
* [ ] Detect an existing BepInEx installation.
* [ ] Avoid overwriting unrelated BepInEx configuration.
* [ ] Install only the files required by the bridge.
* [ ] Track files installed by the application.
* [ ] Explain that the first BepInEx launch may take longer.
* [ ] Detect when generated interop files are ready.
* [ ] Provide a clean plugin-removal action.
* [ ] Do not remove BepInEx when other plugins may use it.
* [ ] Request elevation only when required.

## Acceptance criteria

* Existing BepInEx users keep their current installation.
* Plugin installation is explicit and reversible.
* The app can verify whether installation succeeded.
* Removing the integration does not damage the FM installation.

---

# Workstream 15 — Testing and Validation

## Unit tests

* [ ] FM date decoding
* [ ] Attribute normalization
* [ ] Version-layout resolution
* [ ] String decoding
* [ ] Candidate validation
* [ ] Contract flag decoding
* [ ] Loan resolution
* [ ] Atomic file replacement
* [ ] Dump-schema validation
* [ ] SQLite import transactions

## Integration tests

* [ ] Plugin loads with FM26.
* [ ] Request file triggers a scan.
* [ ] Scan status progresses correctly.
* [ ] Known player values match FM.
* [ ] Known contracts match FM.
* [ ] Known loans are resolved correctly.
* [ ] First-team, reserve and youth players are distinguished.
* [ ] Unsupported game version does not overwrite data.
* [ ] Closing FM during a scan produces a clean error.
* [ ] Large databases import successfully.

## Manual verification set

Maintain a small list of known players containing examples of:

* High CA and PA
* Low CA and high PA
* Left-footed player
* Two-footed player
* Multiple natural positions
* No club
* Expiring contract
* Transfer-listed player
* Loaned-in player
* Loaned-out player
* Reserve player
* Youth player
* Player with non-ASCII name
* Player with multiple nationalities

---

# MVP Milestones

## Milestone 1 — Plugin Proof of Life

Deliverables:

* BepInEx plugin loads.
* Required modules are detected.
* Status file is written.
* Desktop app can see plugin status.

## Milestone 2 — CA/PA Scanner

Deliverables:

* Memory regions are scanned safely.
* Player candidates are detected.
* UID, CA and PA are dumped.
* Known players are manually verified.

## Milestone 3 — Player Identity and Attributes

Deliverables:

* Names
* Dates of birth
* Nationalities
* Height
* Preferred foot
* Positions
* Visible attributes
* Hidden attributes
* Personality attributes

## Milestone 4 — Contracts and Clubs

Deliverables:

* Wage
* Contract expiry
* Transfer status
* Market value
* Parent club
* Current club
* Loan detection
* Division
* Team level
* Managed club

## Milestone 5 — Tauri Data Pipeline

Deliverables:

* Scan requests
* Progress display
* Dump validation
* SQLite import
* Paginated player queries
* Basic player browser

## Milestone 6 — Installer and Diagnostics

Deliverables:

* FM path detection
* Bridge installation and removal
* BepInEx readiness checks
* Version compatibility checks
* Diagnostics export
* Troubleshooting documentation

---

# Definition of Done for the Initial MVP

The MVP is complete when:

* [ ] A user can install the FM integration from the Tauri application.
* [ ] FM26 starts normally with the plugin installed.
* [ ] The application detects whether FM and the bridge are running.
* [ ] The user can request a new data scan.
* [ ] Scan progress is visible in the desktop application.
* [ ] The bridge extracts a complete player database from a loaded save.
* [ ] Basic player identity information matches FM.
* [ ] CA, PA and attributes match FM.
* [ ] Contracts and transfer status match FM.
* [ ] Current club and parent club are distinguished.
* [ ] Loaned players are represented correctly.
* [ ] The dump is imported into SQLite.
* [ ] Players can be searched, filtered, sorted and paginated.
* [ ] Failed scans do not destroy previously valid data.
* [ ] Unsupported FM versions produce a clear compatibility error.
* [ ] Plugin installation is reversible.
* [ ] Diagnostics are sufficient to investigate offset changes.

---

# Known Risks

## FM patch compatibility

Memory offsets may change after major FM updates.

Mitigation:

* Version-specific layouts
* Fail closed on unsupported versions
* Class-offset histograms
* Detailed diagnostics
* Documented repinning process

## Game stability

Invalid native pointer access can crash FM.

Mitigation:

* Prefer `ReadProcessMemory`
* Validate every pointer and collection range
* Avoid direct unsafe dereferencing
* Keep scanning off the Unity thread
* Add strict sanity limits

## False-positive objects

Random memory may resemble a player object.

Mitigation:

* Validate vtable range
* Validate class metadata
* Validate UID
* Validate CA and PA
* Validate names, dates and related structures
* Deduplicate by UID

## Large saves

Large databases may produce dumps exceeding 100 MB.

Mitigation:

* Stream JSON output
* Limit scanner worker count
* Avoid complete frontend payloads
* Import directly into SQLite
* Paginate desktop queries

## Incorrect club relationships

Players may appear in several team or club structures because of loans, reserve teams or historical records.

Mitigation:

* Treat contract club and current squad club separately
* Preserve parent club
* Use deterministic squad conflict resolution
* Test loaned-in and loaned-out players explicitly

## Licensing and attribution

The reference implementation is publicly visible, but public source availability does not automatically establish unrestricted reuse.

Mitigation:

* Review the source repository's license
* Request permission where necessary
* Prefer independently structured code
* Document external research and offset provenance

---

# Recommended First Development Tasks

1. Create the C# BepInEx bridge project.
2. Confirm the plugin loads in FM26.
3. Write module and version information to `status.json`.
4. Implement `ReadProcessMemory` and `VirtualQuery` wrappers.
5. Enumerate candidate heap regions.
6. Implement module image caching.
7. Implement class-offset resolution from vtables.
8. Detect player candidates using UID and CA/PA validation.
9. Dump only player UID, CA and PA.
10. Compare the result with known players in FM.
11. Add names and dates of birth.
12. Add visible attributes and positions.
13. Add contracts.
14. Add current and parent clubs.
15. Integrate dump requests into Tauri.
16. Import the validated dump into SQLite.
17. Build the first searchable player table.

Do not begin with the installer, polished UI, statistics or advanced scouting features. First prove that the bridge can reliably identify and decode players across several saves.

```
```
