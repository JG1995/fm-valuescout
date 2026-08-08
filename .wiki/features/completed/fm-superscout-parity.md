# FM SuperScout reader parity

## Intent

Bring the FM26 reader to direct-data parity with the permitted FMSuperScout memory plugin, then harden scans without changing accepted values, ordering, or snapshot replacement semantics.

## Delivered behavior

- The bridge emits schema v6 and Rust validates and ingests it transactionally. The contract includes the remaining player metadata, complete staff records, optional human-manager metadata, player-database scope, selected-team data, and an explicit date basis.
- Same-pass club discovery supplies bounded, structurally validated candidates. Contract-derived clubs remain the safe fallback. Player facets win over duplicate staff facets by UID, and manager selection is deterministic.
- Production scans remain unlimited by default. Capped and non-concurrent reads keep the serial path. Unlimited concurrent reads use bounded worker-local buffers and deterministic result merging.
- The bridge accounts for requested, readable, unread, and internal-failure bytes. More than ten percent unread coverage fails closed. One guarded Windows PSS VA-clone retry may run after a live incomplete read when available commit memory passes the fixed safety check.
- Cancellation, failed retries, incomplete reads, and failed writes do not replace the prior dump or SQLite snapshot. Dump writes use a temporary file, and failed status errors remove machine-local paths before publication.
- Existing Search, Player, Planner, Optimizer, and Youth Academy screens remain unchanged. No staff UI, gender control, or new player filters were added.

## Final architecture

The C# bridge owns FM memory access, layout pins, typed discovery, extraction, scan diagnostics, and schema-v6 dump writing. Rust owns schema validation, migration v15, transactional snapshot and staff ingest, and Load Data orchestration. React remains outside the data-contract change. The boundary remains [ADR-0016 — C# BepInEx bridge for FM26 memory read](../../decisions/0016-csharp-bepinex-fm26-bridge.md).

The scan limits are fixed: up to eight 32 MiB worker buffers for an unlimited concurrent scan, reduced to two below the documented physical-memory boundary; one snapshot retry after a materially incomplete live attempt; and no user tuning controls. Diagnostics expose aggregate quality, source, retry, worker, timing, and memory-read data without publishing addresses or machine-local paths.

## Important decisions

- Define parity against direct memory-backed values and required discovery behavior, not FMSuperScout's JSON shape, UI, estimates, or derived models.
- Keep the existing bridge, Rust, and SQLite ownership boundaries. No new ADR was required.
- Use schema v6 as one atomic contract transition. Reject stale schema-v5 dumps and never validate a field that ingest silently discards.
- Keep `men` as the app-generated database-scope default while accepting `women` and `both` in the file protocol. Record schedule consensus as derived `next-fixture-consensus` data.
- Keep scan hardening separate from the schema baseline. Retain only deterministic, bounded mechanics that preserve the PR 1 semantic result; one healthy live run is not a general performance claim.

## Migration and operational implications

- Migration v15 adds schema-v6 snapshot fields and the snapshot-owned `staff` table. Existing snapshots keep null or default values where the older dump had no equivalent field.
- A current bridge DLL must be installed before Load Data. The supported runtime remains Windows Steam FM26 with BepInEx 6 IL2CPP and the .NET 6 bridge toolchain.
- The bridge and Rust tests cover failed, cancelled, incomplete, retry, replacement, and rollback paths. Live validation used aggregate and sanitized evidence only; no names, UIDs, paths, raw dumps, or memory addresses belong in the repository.

## Validation

- `./scripts/dev format` completed cleanly.
- `./scripts/dev bridge-test` passed 162 tests with 3 expected Windows-only skips.
- `./scripts/dev test` passed 205 tests.
- `./scripts/dev check` passed Biome, TypeScript, secretlint, and Rust validation with 240 Rust tests passed and 2 ignored.
- One healthy unlimited Windows FM26.3.2 Load Data run after installing the final DLL and restarting FM matched the PR 1 aggregate baseline: 247,781 players, 134,316 staff, manager metadata present, `men` scope, derived `next-fixture-consensus` date basis, no truncation, zero player/staff overlap, and a 491,761,405-byte dump. The live attempt used eight workers, 99.9034% readable coverage, zero retries, and 23.618 seconds of bridge time. Representative player, staff, manager, and club comparisons matched the baseline. The evidence was sanitized and did not retain private live artifacts.
- Feature-complete review used Sol xhigh and accepted the exact implementation set after one correction round. The correction commit added deterministic cancellation and prior-dump preservation proofs and status-path privacy proofs.

## Publication metadata

### PR 1 — Add SuperScout direct-data parity

| Field | Value |
| --- | --- |
| Status | Merged |
| PR ref | https://github.com/JG1995/fm-valuescout/pull/34 |
| Merge ref | `d3f1cad1f6d9f33155f51a8cb74a43b5a77d09d7` |
| Branch | `feature/fm-superscout-parity` |
| Base branch | `main` |
| Publication provider | GitHub |
| PR template | `.github/pull_request_template.md` |
| Merge method | squash |
| Required checks | strict `check` |
| Feature close-out | Not required for the intermediate PR |
| CI repair rounds | 0 |
| Provisional PR title | `feat(memory-read): add SuperScout reader parity` |
| Build-feature-loop profile | Terra Max |

| Commit | Git ref | Implementation profile | Review profile | Result |
| --- | --- | --- | --- | --- |
| Discover non-player people | `0f0dc83` | Terra xhigh | Sol High | Accepted after correction review |
| Discover the complete club graph | `fce0d07` | Terra xhigh | Sol High | Accepted after alignment correction review |
| Read remaining player metadata | `92823ca` | Terra xhigh | Sol High | Accepted |
| Extract non-player records | `3e68e09` | Terra Max | Sol High | Accepted after correction review |
| Publish and persist dump schema v6 | `4ea5a43` | Terra Max | Sol xhigh | Accepted after C5-01 and C5-02 correction review |
| Validate SuperScout data parity | `8553e9f` | Luna Max | Sol High | Accepted after correction review |

### PR 2 — Harden FM memory scans

| Field | Value |
| --- | --- |
| Status | Ready for publication |
| PR ref | Not published |
| Merge ref | Not merged |
| Branch | `feature/fm-superscout-scan-hardening` |
| Base branch | `main` |
| Publication provider | GitHub |
| PR template | `.github/pull_request_template.md` |
| Merge method | squash |
| Required checks | strict `check` |
| Feature close-out | Current; final feature review accepted |
| CI repair rounds | 0 |
| Provisional PR title | `perf(memory-read): harden FM memory scans` |
| Build-feature-loop profile | Terra Max |

| Commit | Git ref | Implementation profile | Review profile | Result |
| --- | --- | --- | --- | --- |
| Measure scan read quality | `0de1985` | Terra xhigh | Sol High | Accepted after stale-buffer correction review |
| Parallelize deterministic region scanning | `b377aed` | Terra Max | Sol xhigh | Accepted after low-memory boundary correction review |
| Retry incomplete scans from a frozen snapshot | `4c46f41` | Terra Max | Sol xhigh | Accepted after correction review |
| Validate hardened scan behavior | `4968e1d` plus correction `b38135d` | Luna Max | Sol High | Accepted after correction; cancellation, prior-dump, and status-privacy proofs added |

## Feature close-out

**State:** Current.

The exact PR 1 merge ref and PR 2 commit set are recorded above. PR 2 remains unpublished and unmerged by design; no publication or merge is claimed here.

## Follow-up

- Publish PR 2 only when its branch is intentionally handed to the GitHub publication workflow. Do not create or publish it as part of documentation reconciliation.
- Keep the current live result as a supported-save semantic baseline, not a promise of general speedup. Revalidate after FM patches or layout repins.
- Staff query APIs, staff UI, additional database-scope controls, and snapshot history remain outside this feature.
