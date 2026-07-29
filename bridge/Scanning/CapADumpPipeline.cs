using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;
using FmDataBridge.Output;
using FmDataBridge.Protocol;

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

        var players = new List<DumpPlayer>(candidates.Count);
        foreach (var candidate in candidates)
        {
            var playerBase = candidate.ObjectAddress - (ulong)candidate.ClassOffset;
            var identity = PlayerIdentityReader.TryRead(
                reader,
                candidate.ObjectAddress,
                playerBase,
                layout,
                out var rejectReason);

            if (identity is null)
            {
                switch (rejectReason)
                {
                    case IdentityRejectReason.EmptyName:
                        diagnostics.IdentitySkippedEmptyName++;
                        break;
                    case IdentityRejectReason.ImpossibleDob:
                        diagnostics.IdentitySkippedImpossibleDob++;
                        break;
                }

                continue;
            }

            var attrs = PlayerAttributeReader.Read(
                reader,
                candidate.ObjectAddress,
                playerBase,
                layout);

            var contract = PlayerContractReader.Read(
                reader,
                candidate.ObjectAddress,
                playerBase,
                layout);

            players.Add(
                new DumpPlayer
                {
                    Uid = candidate.Uid,
                    Ca = candidate.Ca,
                    Pa = candidate.Pa,
                    Name = identity.Name,
                    BirthYear = identity.BirthYear,
                    BirthDayOfYear = identity.BirthDayOfYear,
                    Nationalities = identity.Nationalities,
                    HeightCm = identity.HeightCm,
                    PreferredFoot = identity.PreferredFoot,
                    Positions = identity.Positions,
                    Attributes = attrs.Attributes,
                    HiddenAttributes = attrs.HiddenAttributes,
                    Personality = attrs.Personality,
                    WeeklyWageGbp = contract.WeeklyWageGbp,
                    ContractExpiryYear = contract.ContractExpiryYear,
                    ContractExpiryDayOfYear = contract.ContractExpiryDayOfYear,
                    TransferListed = contract.TransferListed,
                    LoanListed = contract.LoanListed,
                    NotForSale = contract.NotForSale,
                    SetForRelease = contract.SetForRelease,
                    MarketValueGbp = contract.MarketValueGbp,
                    Reputation = contract.Reputation,
                });

            if (diagnostics.SampleAttributeSnapshots.Count < ScanDiagnostics.MaxSampleAttributeSnapshots)
            {
                diagnostics.SampleAttributeSnapshots.Add(
                    FormatAttributeSample(candidate.Uid, identity.Name, attrs));
            }

            if (diagnostics.SampleContractSnapshots.Count < ScanDiagnostics.MaxSampleContractSnapshots)
            {
                diagnostics.SampleContractSnapshots.Add(
                    FormatContractSample(candidate.Uid, identity.Name, contract));
            }
        }

        if (players.Count == 0)
        {
            diagnostics.FailureReason =
                "scan produced candidates but none passed identity sanity (name/DOB)";
            DiagnosticsWriter.Write(bridgeDirectory, DiagnosticsWriter.Format(diagnostics));
            return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
        }

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

    private static string FormatAttributeSample(uint uid, string name, PlayerAttributes attrs)
    {
        static string Fmt(IReadOnlyDictionary<string, int?> map, string key) =>
            map.TryGetValue(key, out var v) && v is { } n ? n.ToString() : "null";

        return
            $"uid={uid} name={name} Acceleration={Fmt(attrs.Attributes, "Acceleration")} " +
            $"Pace={Fmt(attrs.Attributes, "Pace")} Consistency={Fmt(attrs.HiddenAttributes, "Consistency")} " +
            $"Ambition={Fmt(attrs.Personality, "Ambition")}";
    }

    private static string FormatContractSample(uint uid, string name, PlayerContractFields contract)
    {
        static string FmtLong(long? v) => v is { } n ? n.ToString() : "null";
        static string FmtBool(bool? v) => v is { } b ? (b ? "true" : "false") : "null";

        return
            $"uid={uid} name={name} wage={FmtLong(contract.WeeklyWageGbp)} " +
            $"listed={FmtBool(contract.TransferListed)} value={FmtLong(contract.MarketValueGbp)} " +
            $"curRep={contract.Reputation.Current?.ToString() ?? "null"}";
    }
}

public readonly record struct CapADumpResult(bool Success, string? Error, int PlayerCount, bool DumpReplaced)
{
    public static CapADumpResult Succeeded(int playerCount) =>
        new(true, null, playerCount, DumpReplaced: true);

    public static CapADumpResult Failed(string error, bool dumpReplaced) =>
        new(false, error, 0, dumpReplaced);
}
