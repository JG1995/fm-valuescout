using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;
using FmDataBridge.Output;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;

namespace FmDataBridge.Scanning;

public sealed class CapADumpPipeline
{
    private readonly LayoutRegistry _layouts;

    public CapADumpPipeline(LayoutRegistry? layouts = null)
    {
        _layouts = layouts ?? LayoutRegistry.CreateDefault();
    }

    public CapADumpResult Run(
        IMemoryReader reader,
        string bridgeDirectory,
        string gameVersion,
        string bridgeVersion,
        ModuleBounds gameAssembly,
        ModuleBounds? gamePlugin = null)
    {
        ArgumentNullException.ThrowIfNull(reader);
        Directory.CreateDirectory(bridgeDirectory);

        var diagnostics = new ScanDiagnostics
        {
            GameVersion = gameVersion,
            GamePlugin = gamePlugin is { } gp
                ? new ModuleBoundsSnapshot(gp.BaseAddress, gp.EndAddress)
                : null,
        };

        if (!_layouts.TryResolveFromGameVersion(gameVersion, out var layout))
        {
            diagnostics.FailureReason =
                $"unsupported FM version '{gameVersion}'; no layout for major.minor key";
            DiagnosticsWriter.Write(bridgeDirectory, DiagnosticsWriter.Format(diagnostics));
            return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
        }

        var regions = RegionEnumerator.GetCandidateRegions(reader);
        var candidates = PersonScanner.Scan(
            reader,
            layout,
            gameAssembly,
            gamePlugin,
            regions,
            diagnostics);

        if (candidates.Count == 0)
        {
            diagnostics.FailureReason = "scan produced zero player candidates";
            DiagnosticsWriter.Write(bridgeDirectory, DiagnosticsWriter.Format(diagnostics));
            return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
        }

        var players = candidates
            .Select(c => new DumpPlayer { Uid = c.Uid, Ca = c.Ca, Pa = c.Pa })
            .ToList();

        var document = new DumpDocument
        {
            SchemaVersion = BridgeProtocol.DumpSchemaVersion,
            GeneratedAtUtc = DateTimeOffset.UtcNow.ToString("O"),
            GameVersion = gameVersion,
            SupportedGameVersion = layout.VersionKey,
            BridgeVersion = bridgeVersion,
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            PlayerCount = players.Count,
            Players = players,
        };

        var replaced = DumpWriter.TryWriteReplaceOnSuccess(bridgeDirectory, document);
        DiagnosticsWriter.Write(bridgeDirectory, DiagnosticsWriter.Format(diagnostics));

        return replaced
            ? CapADumpResult.Succeeded(players.Count)
            : CapADumpResult.Failed("dump write did not replace file", dumpReplaced: false);
    }
}

public readonly record struct CapADumpResult(bool Success, string? Error, int PlayerCount, bool DumpReplaced)
{
    public static CapADumpResult Succeeded(int playerCount) =>
        new(true, null, playerCount, DumpReplaced: true);

    public static CapADumpResult Failed(string error, bool dumpReplaced) =>
        new(false, error, 0, dumpReplaced);
}
