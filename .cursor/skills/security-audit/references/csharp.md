# C# — BepInEx IL2CPP bridge security audit

Load this file when recon finds C# under `bridge/`, BepInEx plugin entry points, IL2CPP interop, or the FM file protocol writers/readers on the C# side.

Read `references/universal.md` first. Rust protocol client validation, dump ingest, and IPC to the WebView live in `security-audit/references/rust.md` and `security-audit/references/tauri.md`. This file covers **C#-side sinks and idioms**: memory safety, file I/O, deserialization, interop, diagnostics leakage, and plugin integrity.

Adapted from [OWASP .NET Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/DotNet_Security_Cheat_Sheet.html), [BinaryFormatter security guidance](https://learn.microsoft.com/en-us/dotnet/standard/serialization/binaryformatter-security-guide), BepInEx plugin docs, and FM bridge constraints (ADR-0016, feature ledger).

## Ownership vs `rust.md` / `tauri.md`

| Topic | Owner |
| --- | --- |
| **`csharp.md` (this file)** | In-process memory reads, `unsafe`/P/Invoke, `request.json` deserialization, bridge file writes, diagnostics content, Harmony hooks, NuGet CVEs |
| **`rust.md`** | Rust read of dump/status, path resolution, dump validation before SQLite, parameterized SQL |
| **`tauri.md`** | WebView → IPC → Rust; capabilities; XSS escalating to filesystem via Rust commands |

When all three load, trace: WebView trigger → Rust writes `request.json` → C# reads request → C# writes dump/status → Rust reads dump → IPC. Audit each hop once.

## Threat model for this bridge

The plugin runs **inside `fm.exe`** with the game's privileges. Primary assets:

- **Game integrity** — memory must stay read-only; crashes harm availability.
- **Local user data** — dumps and diagnostics under LocalAppData may contain PII (player names, wages).
- **Cross-process file protocol** — any local process that can write `request.json` may trigger scans.

This is not a multi-tenant web API. Calibrate findings to **local desktop** reachability:

- Malicious **other local processes** writing the bridge directory → untrusted `request.json` (Medium when validation is weak).
- Malicious **same user** with FM running → already has game access; focus on accidental corruption and data leaks, not remote RCE fantasies.
- **Supply chain** — compromised NuGet or copied interop DLLs → High if unchecked.

Do not report generic web-app checks (CSRF, JWT) with no foothold in `bridge/`.

## What C# / .NET protect and do not protect

.NET memory safety reduces many classic buffer overflows in **managed** code. `unsafe`, P/Invoke, and incorrect interop reintroduce risk.

.NET does **not** protect against:

- Writing to game memory when code paths call mutators or raw writes
- Crashes from unhandled exceptions on worker threads inside the game process
- Insecure deserialization of `request.json` crafted by another local process
- Path traversal when output paths incorporate untrusted strings
- Secrets or PII in `diagnostics.txt` left world-readable
- Vulnerable **NuGet** dependencies — use `dotnet list package --vulnerable`

## Attack surface

| Layer | What attackers reach | Primary files |
| --- | --- | --- |
| **File protocol input** | `request.json` content | `Protocol/BridgeRequest.cs`, poll loop |
| **Memory reads** | Invalid pointers → crash or leak adjacent memory | `Memory/WindowsMemoryReader.cs` |
| **File protocol output** | `dump.json`, `status.json`, `diagnostics.txt` | `Output/*Writer.cs` |
| **Paths** | Bridge directory resolution | `Protocol/BridgePaths.cs` |
| **IL2CPP interop** | Game object access | Generated interop, `Plugin.cs` |
| **Harmony / patches** | Hooked game methods | Patch classes if present |
| **Logging** | BepInEx log file | `Plugin.cs`, scan workers |
| **Dependencies** | Transitive packages | `FmDataBridge.csproj`, `packages.lock.json` if present |

## Recon signals

```text
# Deserialization
BinaryFormatter|SoapFormatter|LosFormatter|NetDataContractSerializer
JsonSerializer.Deserialize|JsonDocument.Parse
TypeNameHandling|JsonPolymorphic

# Unsafe and interop
unsafe\s*\{|fixed\s*\(
DllImport|ReadProcessMemory|Marshal\.
IntPtr|void\*

# Process and shell
Process\.Start|ProcessStartInfo
cmd\.exe|powershell

# Files
File\.Write|File\.Read|StreamWriter
Path\.Combine\(.*request|Path\.Combine\(.*user
\.\./|Directory\.GetFiles

# Game integrity
WriteProcessMemory|Marshal\.Write
HarmonyPatch|AccessTools

# Secrets and PII
Log\.Log.*password|diagnostics.*wage|api[_-]?key
```

Map each worker entry: `request.json` → validation → scan → writers → final paths.

## Read-only memory and game integrity

Product invariant: **never write to FM memory** (feature ledger, ADR-0016).

| Pattern | Risk |
| --- | --- |
| `WriteProcessMemory`, `Marshal.Write*`, or direct pointer store to game address | Game corruption, anti-cheat triggers, save damage |
| IL2CPP call that mutates game state as side effect | Same — audit every game API invoked from bridge |
| Harmony postfix that changes return values affecting game logic | Unintended gameplay mutation |

Safe pattern: `ReadProcessMemory` (or wrapper) with failure on invalid read; read-only traversal of structures.

**Availability:** unhandled exception on background thread may still tear down the plugin or destabilize FM — treat missing top-level handler as Medium for local DoS, not remote RCE.

## Command and process execution

| Pattern | Risk |
| --- | --- |
| `Process.Start` with user-influenced executable or arguments | Arbitrary code execution |
| Shelling out to `cmd.exe` / PowerShell with interpolated paths | Command injection |
| Launching the Tauri app or other tools from the plugin | Unexpected attack surface expansion |

This bridge should not spawn processes. Flag any `Process.Start` unless the ledger explicitly requires it.

## Path traversal and filesystem

Bridge files live under `%LOCALAPPDATA%\<app>\fm-bridge\`.

| Pattern | Risk |
| --- | --- |
| `Path.Combine(bridgeRoot, request.RelativePath)` without validation | Write dump outside intended directory |
| Trusting `request.json` field that overrides output directory | Redirect dumps or diagnostics |
| World-readable ACL on dump/diagnostics | Local information disclosure (other OS users) |
| Predictable temp file names in shared temp | TOCTOU / leak |

Safe pattern:

- Resolve bridge root once in `BridgePaths` — fixed subpaths only (`status.json`, `dump.json`, etc.).
- Reject request fields that change root or file names outside an allowlist.
- Atomic write via temp + rename inside the bridge directory only.

Rust-side path checks must agree with C# writers — audit both when paths change.

## Deserialization (`request.json`)

Treat `request.json` as **untrusted** at the C# boundary — another local process may race the Tauri app.

| Pattern | Risk |
| --- | --- |
| `BinaryFormatter`, `NetDataContractSerializer`, polymorphic JSON with type names | Remote/local gadget chains — never use |
| `JsonSerializer.Deserialize<BridgeRequest>` without validation | Unexpected actions, oversized payloads |
| `JsonDocument` manual parse without length limits | DoS — huge file stalls worker |
| Deserializing into `object` or `dynamic` | Weak typing, unexpected shapes |

Safe patterns:

- Typed `BridgeRequest` with explicit properties only.
- `System.Text.Json` with **no** polymorphic type discriminators from wire data.
- Max file size before read; reject unknown `action` enum values.
- Optional: write request nonce from Rust; plugin ignores stale files.

`dump.json` is **output** — lower injection risk unless another tool consumes it with an unsafe deserializer. Rust ingest validation is the next trust boundary (`rust.md`).

## Unsafe code and P/Invoke

| Signal | Audit |
| --- | --- |
| `unsafe` blocks | Document invariant; minimize scope; keep inside `Memory/` |
| `DllImport` for `ReadProcessMemory` | Correct buffer size, return check, no write APIs |
| `IntPtr` arithmetic without bounds | Out-of-bounds read → crash or leak |
| `fixed` buffers with wrong length | Buffer over-read |
| `Marshal.PtrToStructure` on game memory | Struct layout mismatch → misread or crash |

Prefer centralized `IMemoryReader` so unsafe is auditable in one module.

## IL2CPP interop

| Risk | Check |
| --- | --- |
| Calling game methods with side effects | Mutations, network, save triggers |
| Holding IL2CPP references across threads | Undefined behaviour / crash |
| Generated interop from untrusted source | Supply chain — interop must come from local FM + BepInEx generation |
| Logging full game object `ToString()` | May leak internal paths or PII |

Minimize surface: prefer memory layout reads over invoking game APIs when the ledger allows.

## Harmony and preloader patchers

| Risk | Check |
| --- | --- |
| Unnecessary patches | Stability and upgrade fragility |
| Patch runs on hot path every frame | Performance DoS |
| Patch exposes new attack surface (extra logging of secrets) | Information disclosure |
| Patcher DLL mixed with plugin DLL | BepInEx load-order issues ([BepInEx docs](https://docs.bepinex.dev/v6.0.0-pre.1/articles/dev_guide/preloader_patchers.html)) |

Default bridge design avoids Harmony. Flag patches unless ledger-approved.

## Diagnostics and logging

| Risk | Example |
| --- | --- |
| Full memory addresses, module paths, offsets in user-exported diagnostics | Aids reverse engineering — usually Low; document intentional |
| Player PII in logs at `Info` | Wage, name in BepInEx log file |
| Exception stack traces with paths | Username leakage in `%USERPROFILE%` |
| `request.json` contents logged verbatim | May contain tokens if Rust adds them later |

Redact or gate verbose diagnostics behind a debug flag. Assume users may export `diagnostics.txt` from the app.

## Denial of service

| Pattern | Impact |
| --- | --- |
| Scan on main thread | FM freeze — availability |
| Unbounded scan without cancellation | Long hang |
| Huge `request.json` / malformed JSON retry loop | CPU spin |
| Memory bomb — allocate per candidate without cap | OOM inside game process |

Not always security findings at solo-dev scope — report as Medium when an **untrusted** writer can trigger them via `request.json`.

## Authentication and authorization

No network auth in the bridge. Authorization is **implicit**: whoever can write `request.json` can request a scan.

| Gap | Finding |
| --- | --- |
| No validation that request file was written by the Tauri app | Local cross-process trigger (Medium if sensitive) |
| Shared bridge directory across Windows users | Unlikely under LocalAppData — note if misconfigured |

Document intentional single-user desktop assumption in ADR/ledger. Recommend Rust write atomic requests with optional shared secret only if threat model tightens — not required for MVP.

## Dependency auditing

Gate does not run `dotnet` on Linux by default.

| Check | Tool |
| --- | --- |
| Known CVEs | `dotnet list package --vulnerable` on Windows CI or dev machine |
| Pin versions | `PackageReference` with explicit versions; commit `packages.lock.json` if enabled |
| Git or floating NuGet sources | Supply chain risk |

Flag copied FM interop DLLs from untrusted sources.

## Testing vs production

| Risk | Check |
| --- | --- |
| Test hook that disables read-only guard | `#if DEBUG` write paths shipped in Release |
| Fake `IMemoryReader` that logs real addresses in CI artifacts | Leak in build logs |
| Test `request.json` fixtures with absolute paths | Copy-paste into production config |

## Static audit methodology

1. **List entry points** — `Plugin.Load`, background poll loop, any Harmony patches.
2. **Trace request path** — read → deserialize → validate → action dispatch.
3. **Trace scan path** — memory reads only; no writes to game.
4. **Trace output path** — writers, atomic rename, fail-safe dump rules.
5. **Grep sinks** — `unsafe`, `Process.Start`, `BinaryFormatter`, `Path.Combine` with external input.
6. **Cross-check Rust** — matching path rules and dump validation (`rust.md`).
7. **Dependencies** — `FmDataBridge.csproj` package list.

## False positives

- `ReadProcessMemory` for read-only scan — expected core behaviour.
- `System.Text.Json` deserialize to a fixed DTO with validation — not insecure deserialization by itself.
- Diagnostics recording module base addresses — intentional for support; not a vulnerability unless combined with remote exposure (there is none).
- BepInEx `Log.LogInfo` for scan lifecycle — fine without PII.
- LocalAppData bridge path per user — correct isolation; not traversal.

## Sources

| Source | Use in this file |
| --- | --- |
| ADR-0016 | C# bridge + Rust protocol split |
| Feature ledger `fm26-memory-read` | Read-only, fail-safe dump, threading |
| [OWASP .NET Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/DotNet_Security_Cheat_Sheet.html) | Deserialization, validation |
| [BinaryFormatter security](https://learn.microsoft.com/en-us/dotnet/standard/serialization/binaryformatter-security-guide) | Banned APIs |
| BepInEx plugin docs | Plugin vs patcher separation |
| `coding-standards/references/csharp.md` | Module layout, performance, output rules |
