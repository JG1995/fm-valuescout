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
        ModuleBounds? gamePlugin = null,
        int? maxAccepted = null,
        CancellationToken cancellationToken = default)
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
        var scanCap = maxAccepted ?? PersonScanner.DefaultMaxAccepted;
        var candidates = PersonScanner.Scan(
            reader,
            layout,
            gameAssembly,
            gamePlugin,
            regions,
            diagnostics,
            scanCap,
            cancellationToken);

        if (diagnostics.Cancelled)
        {
            diagnostics.FailureReason = "scan cancelled";
            DiagnosticsWriter.Write(bridgeDirectory, DiagnosticsWriter.Format(diagnostics));
            return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
        }

        if (candidates.Count == 0)
        {
            diagnostics.FailureReason = "scan produced zero player candidates";
            DiagnosticsWriter.Write(bridgeDirectory, DiagnosticsWriter.Format(diagnostics));
            return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
        }

        var drafts = new List<PlayerDraft>(candidates.Count);
        var personToUid = new Dictionary<ulong, uint>();
        var parentClubByUid = new Dictionary<uint, string?>();
        var clubAddresses = new HashSet<ulong>();

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

            var parentLink = ContractClubReader.TryRead(reader, candidate.ObjectAddress, layout);
            if (parentLink is { ClubAddress: not 0 })
            {
                clubAddresses.Add(parentLink.ClubAddress);
            }

            personToUid[candidate.ObjectAddress] = candidate.Uid;
            parentClubByUid[candidate.Uid] = parentLink?.ClubName;

            drafts.Add(
                new PlayerDraft(
                    candidate,
                    identity,
                    attrs,
                    contract,
                    parentLink));

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

        if (drafts.Count == 0)
        {
            diagnostics.FailureReason =
                "scan produced candidates but none passed identity sanity (name/DOB)";
            DiagnosticsWriter.Write(bridgeDirectory, DiagnosticsWriter.Format(diagnostics));
            return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
        }

        var squadIndex = SquadClubIndex.Build(
            reader,
            layout,
            personToUid,
            parentClubByUid,
            clubAddresses);

        foreach (var sample in squadIndex.MultiClubSamples.Take(ScanDiagnostics.MaxSampleClubSnapshots))
        {
            diagnostics.MultiClubSamples.Add(sample);
        }

        diagnostics.ClubsWalked = squadIndex.ClubsWalked;
        diagnostics.PlayersLinkedViaSquad = squadIndex.PlayersLinked;

        var gameDate = GameDateResolver.Resolve(
            squadIndex.DateVotes,
            GameDateResolver.YoungestBirthCohortYear(
                drafts.Select(d => d.Identity.BirthYear),
                minCohortSize: Math.Min(30, Math.Max(1, drafts.Count))));

        diagnostics.GameDateSource = gameDate.Source;
        diagnostics.GameDate = gameDate.GameDate;

        var players = new List<DumpPlayer>(drafts.Count);
        foreach (var draft in drafts)
        {
            var parentName = draft.ParentLink?.ClubName;
            string? currentName = null;
            string? division = draft.ParentLink?.Division;
            string? teamLevel = null;
            int? teamType = null;

            if (squadIndex.TryGet(draft.Candidate.Uid, out var squad))
            {
                currentName = squad.ClubName;
                division = squad.Division ?? division;
                teamType = squad.TeamType;
                teamLevel = TeamLevelMap.FromTeamType(squad.TeamType);
            }
            else if (parentName is not null)
            {
                currentName = parentName;
            }

            bool? onLoan = null;
            if (currentName is not null && parentName is not null)
            {
                onLoan = !string.Equals(currentName, parentName, StringComparison.Ordinal);
            }

            if (currentName is null && parentName is null)
            {
                diagnostics.ClubUnresolved++;
            }

            int? age = null;
            if (gameDate.Year > 0 && gameDate.DayOfYear > 0)
            {
                age = PlayerAge.At(
                    draft.Identity.BirthYear,
                    draft.Identity.BirthDayOfYear,
                    gameDate.Year,
                    gameDate.DayOfYear);
            }

            players.Add(
                new DumpPlayer
                {
                    Uid = draft.Candidate.Uid,
                    Ca = draft.Candidate.Ca,
                    Pa = draft.Candidate.Pa,
                    Name = draft.Identity.Name,
                    BirthYear = draft.Identity.BirthYear,
                    BirthDayOfYear = draft.Identity.BirthDayOfYear,
                    Nationalities = draft.Identity.Nationalities,
                    HeightCm = draft.Identity.HeightCm,
                    PreferredFoot = draft.Identity.PreferredFoot,
                    Positions = draft.Identity.Positions,
                    Attributes = draft.Attrs.Attributes,
                    HiddenAttributes = draft.Attrs.HiddenAttributes,
                    Personality = draft.Attrs.Personality,
                    WeeklyWageGbp = draft.Contract.WeeklyWageGbp,
                    ContractExpiryYear = draft.Contract.ContractExpiryYear,
                    ContractExpiryDayOfYear = draft.Contract.ContractExpiryDayOfYear,
                    TransferListed = draft.Contract.TransferListed,
                    LoanListed = draft.Contract.LoanListed,
                    NotForSale = draft.Contract.NotForSale,
                    SetForRelease = draft.Contract.SetForRelease,
                    MarketValueGbp = draft.Contract.MarketValueGbp,
                    Reputation = draft.Contract.Reputation,
                    CurrentClub = currentName,
                    ParentClub = parentName,
                    OnLoan = onLoan,
                    Division = division,
                    TeamLevel = teamLevel,
                    Age = age,
                });

            if (diagnostics.SampleClubSnapshots.Count < ScanDiagnostics.MaxSampleClubSnapshots)
            {
                diagnostics.SampleClubSnapshots.Add(
                    FormatClubSample(
                        draft.Candidate.Uid,
                        draft.Identity.Name,
                        currentName,
                        parentName,
                        onLoan,
                        teamLevel,
                        teamType));
            }
        }

        if (players.Count > 0
            && diagnostics.ClubUnresolved * 2 >= players.Count)
        {
            diagnostics.ClubResolutionWarning =
                $"club resolution failed for {diagnostics.ClubUnresolved}/{players.Count} players";
        }

        var document = new DumpDocument
        {
            SchemaVersion = BridgeProtocol.DumpSchemaVersion,
            GeneratedAtUtc = DateTimeOffset.UtcNow.ToString("O"),
            GameVersion = gameVersion,
            SupportedGameVersion = layout.VersionKey,
            BridgeVersion = bridgeVersion,
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            GameDate = gameDate.GameDate,
            GameDateSource = gameDate.Source,
            ScanTruncated = diagnostics.StoppedEarly,
            MaxAccepted = diagnostics.MaxAccepted,
            PlayerCount = players.Count,
            Players = players,
        };

        var replaced = DumpWriter.TryWriteReplaceOnSuccess(bridgeDirectory, document);
        DiagnosticsWriter.Write(bridgeDirectory, DiagnosticsWriter.Format(diagnostics));

        return replaced
            ? CapADumpResult.Succeeded(
                players.Count,
                scanTruncated: diagnostics.StoppedEarly,
                maxAccepted: diagnostics.MaxAccepted)
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

    private static string FormatClubSample(
        uint uid,
        string name,
        string? current,
        string? parent,
        bool? onLoan,
        string? teamLevel,
        int? teamType)
    {
        static string Fmt(string? v) => v ?? "null";
        static string FmtBool(bool? v) => v is { } b ? (b ? "true" : "false") : "null";

        return
            $"uid={uid} name={name} current={Fmt(current)} parent={Fmt(parent)} " +
            $"onLoan={FmtBool(onLoan)} level={Fmt(teamLevel)} tt={teamType?.ToString() ?? "null"}";
    }

    private sealed record PlayerDraft(
        PersonCandidate Candidate,
        PlayerIdentity Identity,
        PlayerAttributes Attrs,
        PlayerContractFields Contract,
        ContractClubLink? ParentLink);
}

public readonly record struct CapADumpResult(
    bool Success,
    string? Error,
    int PlayerCount,
    bool DumpReplaced,
    bool ScanTruncated = false,
    int? MaxAccepted = null)
{
    public static CapADumpResult Succeeded(
        int playerCount,
        bool scanTruncated = false,
        int? maxAccepted = null) =>
        new(true, null, playerCount, DumpReplaced: true, scanTruncated, maxAccepted);

    public static CapADumpResult Failed(string error, bool dumpReplaced) =>
        new(false, error, 0, dumpReplaced);
}
