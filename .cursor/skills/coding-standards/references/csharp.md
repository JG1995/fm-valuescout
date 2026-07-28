# C# — BepInEx IL2CPP bridge (`bridge/`)

Load this file when you implement or review C# under `bridge/`, `.csproj` files for the FM26 plugin, or BepInEx/IL2CPP interop wiring.

Read `references/universal.md` first. Read `references/testing.md` when the change adds or changes tests. The Rust file-protocol client lives in `src-tauri/` — see `references/rust.md` and ADR-0016. Security audit depth for bridge sinks lives in `security-audit/references/csharp.md`.

External references (read for depth, do not duplicate here):

- [C# coding conventions](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/coding-style/coding-conventions) — language style
- [.NET code-style rule options](https://learn.microsoft.com/en-us/dotnet/fundamentals/code-analysis/code-style-rule-options) — EditorConfig and analyzers
- [BepInEx plugin tutorial](https://docs.bepinex.dev/articles/dev_guide/plugin_tutorial/index.html) — plugin lifecycle, logging, config
- [BepInEx IL2CPP plugin start](https://docs.bepinex.dev/master/articles/dev_guide/plugin_tutorial/2_plugin_start.html) — project references, attributes
- [Il2CppInterop](https://github.com/BepInEx/Il2CppInterop) — IL2CPP ↔ .NET interop
- [Memory and Span usage](https://learn.microsoft.com/en-us/dotnet/standard/memory-and-spans/memory-t-usage-guidelines) — hot-path buffers
- [C# performance](https://learn.microsoft.com/en-us/dotnet/csharp/advanced-topics/performance/) — allocation discipline

Calibrate depth to hobbyist solo-dev scope in `AGENTS.md`. Structural quality stays high; skip paranoid padding on paths that cannot fail in practice.

## Stack defaults

| Piece | Role |
| --- | --- |
| .NET 6 SDK | Plugin target (`net6.0` class library) — matches BepInEx 6 IL2CPP template |
| BepInEx 6 IL2CPP | Plugin host inside `fm.exe` |
| Il2CppInterop | Generated interop assemblies for FM types — machine-local, not vendored |
| `System.Text.Json` | `request.json`, `status.json`, `dump.json` — no `BinaryFormatter` |
| xUnit (or NUnit) | Unit tests under `bridge/Tests/` — fakes for memory and filesystem |
| `dotnet build` / `dotnet test` | Windows host with local path overrides — Linux gate does not require `dotnet` |

Do not vendor BepInEx, FM interop DLLs, or game assemblies in git. Use `Directory.Build.props.example` + untracked `Directory.Build.user.props` for machine paths (ADR-0016).

## Architecture principles

The bridge is an **in-process, read-only** FM26 memory reader. Rust owns orchestration, validation, and SQLite; the WebView never touches game memory or bridge files directly.

- **Read-only** — never write to FM memory. Invalid addresses must fail safely.
- **File protocol** — exchange `request.json`, `status.json`, `dump.json`, and `diagnostics.txt` under `%LOCALAPPDATA%\<app>\fm-bridge\`. No sockets or Tauri IPC inside the plugin.
- **Off the Unity main thread** — polling, scanning, and dump writes run on a background thread. Do not block game frames.
- **Versioned layouts** — offset tables live in `Layouts/` per FM build. Unsupported versions fail closed with a clear status.
- **Fail-safe output** — never replace a valid dump with an empty or failed scan. Write diagnostics on failure.
- **Bounded work** — paginate or chunk only when needed for memory; full player dumps go to disk, not in-process queues unbounded forever.

Pick **one** owner per concern. Path resolution in `Protocol/BridgePaths.cs`, memory primitives in `Memory/`, layout pins in `Layouts/`, extraction in `Extraction/`, writers in `Output/`.

## Project structure

```text
bridge/
├── FmDataBridge.csproj
├── Plugin.cs                    # BepInEx entry — thin; wires lifecycle + background worker
├── Directory.Build.props.example
├── README.md
├── Protocol/
│   ├── BridgeRequest.cs
│   ├── BridgeStatus.cs
│   └── BridgePaths.cs
├── Memory/
│   ├── IMemoryReader.cs
│   ├── WindowsMemoryReader.cs   # ReadProcessMemory wrapper
│   └── ...
├── Layouts/
│   ├── IFmMemoryLayout.cs
│   ├── LayoutRegistry.cs
│   └── Fm263Layout.cs           # one file per supported FM build pin
├── Scanning/
├── Extraction/
├── Models/
├── Output/
│   ├── StatusWriter.cs
│   ├── DumpWriter.cs
│   └── DiagnosticsWriter.cs
└── Tests/
```

Only add folders a feature needs. Do not scaffold empty `Scanning/` or `Extraction/` types before the ledger commit calls for them.

### Plugin entry (`Plugin.cs`)

- Inherit `BasePlugin` (BepInEx 6 Unity IL2CPP).
- Decorate with `[BepInPlugin(GUID, Name, Version)]` — GUID must stay stable across releases.
- `Load()` starts the background request loop; `Unload()` signals cancellation and joins cleanly.
- No business logic in `Load()` beyond wiring — same rule as thin `main.rs` / `Plugin.cs` shell.

Reference game assemblies from a **local `lib/` copy**, not directly from the Steam folder ([BepInEx docs](https://docs.bepinex.dev/master/articles/dev_guide/plugin_tutorial/2_plugin_start.html)). Do not reference .NET core assemblies from the game directory.

### Rust and frontend parity

| Concern | Owner |
| --- | --- |
| Write `request.json`, read `status.json` / dump metadata | `src-tauri/src/features/.../bridge/` (Rust) |
| Memory scan, layout resolution, dump serialization | `bridge/` (C#) |
| UI status and scan trigger | `src/features/...` (React) |

Keep protocol DTO field names aligned with Rust serde models once the schema freezes. Document breaking changes in the feature ledger and a new layout pin — not silent renames.

## Memory reading

- Prefer `IMemoryReader` with a `WindowsMemoryReader` implementation using `ReadProcessMemory` (or equivalent safe read API) so invalid pointers return failure instead of crashing FM.
- Avoid direct `unsafe` pointer dereference on game addresses. When `unsafe` is unavoidable for interop, keep it inside `Memory/` with documented invariants.
- Centralize pointer/size validation — one helper for bounds and alignment checks.
- Record module bases and layout version in diagnostics on every scan.
- Layout offsets live only in `Layouts/` — not scattered through extractors.

# ponytail: synchronous scan on a single background thread
# Upgrade to a dedicated worker with progress reporting and cancellation tokens when scan duration blocks the next request poll — not at status-writer scaffold.

## Threading and lifecycle

| Rule | Reason |
| --- | --- |
| Poll `request.json` on a background thread | Keeps Unity main thread responsive |
| Use `CancellationToken` for shutdown | `Unload()` must not leave a hung thread |
| Do not touch Unity/IL2CPP objects from the worker unless API requires main thread | IL2CPP thread affinity |
| Atomic or file-rename writes for output JSON | Readers must not see half-written dumps |

Prefer `Task.Run` + long-running loop or `Thread` with explicit naming (`FmBridge-Worker`) for the poll loop. Avoid `async void`. Use `ConfigureAwait(false)` in library-style helpers if you introduce `async` I/O for files.

## File output

- Write to a temp file in the same directory, then rename/move to the final name — readers see complete files only.
- `DumpWriter` — never overwrite an existing valid dump with empty or partial output on failure.
- `StatusWriter` — always reflect terminal state (`idle`, `scanning`, `complete`, `failed`) with a human-readable message.
- `DiagnosticsWriter` — append or rewrite on every scan attempt, including failures.

Path roots come from `BridgePaths` (LocalAppData). Do not hardcode user profile strings in multiple files.

## Naming

Follow [.NET naming guidelines](https://learn.microsoft.com/en-us/dotnet/standard/design-guidelines/naming-guidelines):

| Item | Convention | Example |
| --- | --- | --- |
| Namespaces | PascalCase, match folder structure | `FmDataBridge.Protocol` |
| Types, methods, properties | PascalCase | `BridgeStatus`, `ReadInt32` |
| Private fields | `_camelCase` | `_cancellationSource` |
| Static fields | `s_camelCase` | `s_layoutRegistry` |
| Interfaces | `I` prefix | `IMemoryReader`, `IFmMemoryLayout` |
| Async methods | `Async` suffix | `WriteStatusAsync` |

Use `string` and `int` keywords, not `System.String` / `System.Int32`. Spell out types when `var` would hide intent from a reader without IDE tooltips.

## Error handling

- Return `bool`, `Result`-like types, or nullable outputs from memory reads — do not throw across the game boundary for expected invalid addresses.
- Catch only exceptions you can handle at the worker boundary; log and write `failed` status — never swallow without diagnostics.
- Do not catch bare `Exception` unless the top-level worker loop converts it to status + diagnostics (last resort).
- Use `ArgumentNullException.ThrowIfNull` at public entry points — not null-forgiving `!` to silence the compiler.

Plugin logging: use BepInEx `Log` (`Log.LogInfo`, `Log.LogError`). Do not `Console.WriteLine` in production paths.

## Performance

Scanning thousands of players is allocation-sensitive.

| Practice | When |
| --- | --- |
| `ReadOnlySpan<byte>` / `Span<byte>` on synchronous parse paths | Hot loops reading primitive fields from a buffer |
| `ReadOnlyMemory<byte>` when data crosses `async` or is stored on a class | Background worker hand-off |
| `ArrayPool<byte>.Shared` for large scratch buffers | Repeated scan buffers — return in `finally` |
| `stackalloc` for small fixed buffers | Short-lived primitives under ~1 KB — watch stack limits |
| Reuse `JsonSerializerOptions` instances | Static readonly options for output writers |
| `struct` for small scan intermediates | Reduce GC pressure in inner loops |
| `sealed` on implementation classes | Default for non-extensible types |

Avoid LINQ in hot scan paths — prefer explicit loops. Do not allocate new `byte[]` per field read when a shared buffer suffices.

Do not prematurely micro-optimize status-only commits. Optimize when profiling shows GC or scan time problems.

## JSON and serialization

- Use `System.Text.Json` with explicit DTO types — not `dynamic` or `JsonDocument` for the frozen protocol schema.
- `JsonSerializerOptions`: camelCase for wire format if Rust/TS expect it; document the choice in the ledger.
- `[JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]` on optional fields to keep dumps smaller.
- Never use `BinaryFormatter`, `SoapFormatter`, or other binary deserializers with polymorphic type graphs.
- Deserialize `request.json` with a typed `BridgeRequest` — validate enum values and reject unknown `action` strings.

## Nullable reference types

Enable `<Nullable>enable</Nullable>` in the `.csproj`. Treat nullability warnings as errors in new code (`<WarningsAsErrors>nullable</WarningsAsErrors>` when the project matures).

## Analyzers and format

- Add `.editorconfig` at `bridge/` (or repo root) with C# style rules — Allman braces, `_camelCase` private fields, file-scoped namespaces.
- Enable `EnforceCodeStyleInBuild` when the bridge project stabilizes.
- Run `dotnet format` before checkpoint on Windows when C# changed.

Linux `./scripts/dev check` intentionally does not build the bridge — bridge validation is `dotnet build` / `dotnet test` on a Windows machine with local props configured.

## Testing

| Kind | Location | Purpose |
| --- | --- | --- |
| Unit | `bridge/Tests/` | Status serialization, path logic, layout registry, fake `IMemoryReader` |
| Fake memory | Test doubles implementing `IMemoryReader` | Extraction logic without `fm.exe` |
| Integration | Manual on Windows | BepInEx install, FM launch, file protocol end-to-end |

Conventions:

- One behavior per test; name tests like sentences: `WriteStatus_never_overwrites_valid_dump_on_failure`.
- Colocate fakes in test project, not in production `Memory/`.
- Do not reference FM interop assemblies in tests unless a commit explicitly needs them — prefer fakes.

Full attach tests cannot run in Linux CI. Document manual verification steps in `bridge/README.md`.

## Optional upgrades (not defaults)

| Tool | Trigger to add |
| --- | --- |
| `Microsoft.CodeAnalysis.NetAnalyzers` | Bridge grows beyond scaffold — enforce CA rules in build |
| `dotnet list package --vulnerable` in CI | Third-party NuGet deps beyond BepInEx template |
| Source generators for protocol types | Rust and C# DTOs drift repeatedly — prefer ledger schema freeze first |
| Harmony patches | Only when IL2CPP API requires hooking — prefer file-protocol polling default |

Prefer Harmony only when required ([BepInEx preloader guidance](https://docs.bepinex.dev/v6.0.0-pre.1/articles/dev_guide/preloader_patchers.html)). Minimize hook count for game stability.

## What not to do

- Write to FM memory or call game mutators from the bridge.
- Block the Unity main thread with scans or file I/O.
- Vendor BepInEx or FM assemblies in git.
- Reference game DLLs directly from the Steam install path in the `.csproj`.
- Mix patcher DLL and plugin DLL in one assembly.
- Replace a good `dump.json` with an empty file on scan failure.
- Scatter layout offsets through `Extraction/` — use `Layouts/`.
- Use `BinaryFormatter` or other insecure deserializers.
- Expose bridge protocol handling in the React layer.
- Copy SuperScout source — study publicly, reimplement independently (ADR-0016).

## Cross-boundary checklist

When a feature touches bridge + Rust + UI:

1. Freeze or version protocol types in the ledger before wide refactors.
2. Load **`csharp.md` + `rust.md` + `tauri.md`** — file protocol spans C# output, Rust read/write, and IPC to the WebView.
3. Add a layout pin (`Fm263Layout.cs`, etc.) when FM patches — do not silently change offsets in place without a new layout class.
