using System.Diagnostics;
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

    private readonly IProcessSnapshotFactory _snapshotFactory;

    private readonly Func<SystemMemoryStatus> _memoryStatusReader;

    public CapADumpPipeline(LayoutRegistry? layouts = null)
        : this(
            layouts,
            new WindowsProcessSnapshotFactory(),
            SystemMemoryStatusReader.Read)
    {
    }

    internal CapADumpPipeline(
        LayoutRegistry? layouts,
        IProcessSnapshotFactory snapshotFactory,
        Func<SystemMemoryStatus> memoryStatusReader)
    {
        _layouts = layouts ?? LayoutRegistry.CreateDefault();
        _snapshotFactory = snapshotFactory ?? throw new ArgumentNullException(nameof(snapshotFactory));
        _memoryStatusReader = memoryStatusReader ?? throw new ArgumentNullException(nameof(memoryStatusReader));
    }

    public CapADumpResult Run(
        IMemoryReader reader,
        string bridgeDirectory,
        string gameVersion,
        string bridgeVersion,
        ModuleBounds gameAssembly,
        ModuleBounds? gamePlugin = null,
        int? maxAccepted = null,
        PlayerDatabaseScope playerDatabaseScope = PlayerDatabaseScope.Men,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(reader);
        Directory.CreateDirectory(bridgeDirectory);

        using var snapshotScope = new SnapshotScope();
        var totalSw = Stopwatch.StartNew();
        var counting = reader as CountingMemoryReader ?? new CountingMemoryReader(reader);
        reader = counting;

        var diagnostics = new ScanDiagnostics
        {
            GameVersion = gameVersion,
            ReadSource = reader.ReadSource,
            GamePlugin = gamePlugin is { } gp
                ? new ModuleBoundsSnapshot(gp.BaseAddress, gp.EndAddress)
                : null,
        };

        if (!_layouts.TryResolveFromGameVersion(gameVersion, out var layout))
        {
            diagnostics.FailureReason =
                $"unsupported FM version '{gameVersion}'; no layout for major.minor key";
            WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
            return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
        }

        var phaseSw = Stopwatch.StartNew();
        var regions = RegionEnumerator.GetCandidateRegions(reader);
        diagnostics.RegionEnumerationMs = phaseSw.ElapsedMilliseconds;

        phaseSw.Restart();
        var scan = PersonScanner.Scan(
            reader,
            layout,
            gameAssembly,
            gamePlugin,
            regions,
            diagnostics,
            maxAccepted,
            playerDatabaseScope,
            cancellationToken);
        diagnostics.CandidateDiscoveryMs = phaseSw.ElapsedMilliseconds;
        var candidates = scan.Players;

        if (diagnostics.Cancelled)
        {
            diagnostics.FailureReason = "scan cancelled";
            WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
            return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
        }

        if (scan.ReadQuality.IsMateriallyIncomplete)
        {
            diagnostics.ScanRetryCount = 1;
            var memoryStatus = _memoryStatusReader();
            if (memoryStatus.IsKnown)
            {
                diagnostics.SnapshotAvailableCommitBytes = memoryStatus.AvailableCommitBytes;
            }

            if (!memoryStatus.IsKnown)
            {
                diagnostics.FailureReason =
                    "live scan read quality incomplete; snapshot retry skipped because available commit memory could not be measured";
                WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
                return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
            }

            if (!ProcessSnapshotPolicy.HasSufficientAvailableCommit(memoryStatus))
            {
                diagnostics.FailureReason =
                    "live scan read quality incomplete; snapshot retry skipped because available commit memory is below the safety threshold";
                WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
                return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
            }

            ProcessSnapshotCaptureResult capture;
            try
            {
                capture = _snapshotFactory.TryCapture();
            }
            catch (Exception exception)
            {
                diagnostics.SnapshotFailureReason =
                    $"snapshot factory threw {exception.GetType().Name}: {exception.Message}";
                diagnostics.FailureReason =
                    $"live scan read quality incomplete; snapshot retry failed: {diagnostics.SnapshotFailureReason}";
                WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
                return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
            }

            diagnostics.SnapshotCaptureMs = capture.CaptureMilliseconds;
            if (!capture.IsSuccess)
            {
                diagnostics.SnapshotFailureReason = capture.FailureReason;
                diagnostics.FailureReason =
                    $"live scan read quality incomplete; snapshot retry failed: {capture.FailureReason}";
                WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
                return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
            }

            snapshotScope.Set(capture.Snapshot!);
            reader = snapshotScope.Reader;
            counting = reader as CountingMemoryReader ?? new CountingMemoryReader(reader);
            reader = counting;
            diagnostics = new ScanDiagnostics
            {
                GameVersion = gameVersion,
                ReadSource = reader.ReadSource,
                GamePlugin = gamePlugin is { } snapshotGamePlugin
                    ? new ModuleBoundsSnapshot(snapshotGamePlugin.BaseAddress, snapshotGamePlugin.EndAddress)
                    : null,
                ScanRetryCount = 1,
                SnapshotCaptureMs = capture.CaptureMilliseconds,
                SnapshotAvailableCommitBytes = memoryStatus.AvailableCommitBytes,
            };

            phaseSw.Restart();
            regions = RegionEnumerator.GetCandidateRegions(reader);
            diagnostics.RegionEnumerationMs = phaseSw.ElapsedMilliseconds;

            phaseSw.Restart();
            scan = PersonScanner.Scan(
                reader,
                layout,
                gameAssembly,
                gamePlugin,
                regions,
                diagnostics,
                maxAccepted,
                playerDatabaseScope,
                cancellationToken);
            diagnostics.CandidateDiscoveryMs = phaseSw.ElapsedMilliseconds;
            candidates = scan.Players;

            if (diagnostics.Cancelled)
            {
                diagnostics.FailureReason = "snapshot retry cancelled";
                WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
                return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
            }

            if (scan.ReadQuality.IsMateriallyIncomplete)
            {
                diagnostics.FailureReason =
                    $"snapshot retry read quality incomplete: {scan.ReadQuality.UnreadBytes}/{scan.ReadQuality.RequestedBytes} region bytes unread";
                WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
                return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
            }
        }

        if (cancellationToken.IsCancellationRequested)
        {
            return Cancelled(bridgeDirectory, diagnostics, counting, totalSw);
        }

        if (candidates.Count == 0)
        {
            diagnostics.FailureReason = "scan produced zero player candidates";
            WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
            return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
        }

        var drafts = new List<PlayerDraft>(candidates.Count);
        var staffDrafts = new List<StaffDraft>(scan.Staff.Count);
        var personToUid = new Dictionary<ulong, uint>();
        var parentClubByUid = new Dictionary<uint, string?>();
        var clubAddresses = new HashSet<ulong>();

        phaseSw.Restart();
        foreach (var candidate in candidates)
        {
            if (cancellationToken.IsCancellationRequested)
            {
                return Cancelled(bridgeDirectory, diagnostics, counting, totalSw);
            }

            var playerBase = candidate.ObjectAddress - (ulong)candidate.ClassOffset;
            var identity = PlayerIdentityReader.TryRead(
                reader,
                candidate.ObjectAddress,
                playerBase,
                layout,
                out var rejectReason);

            if (cancellationToken.IsCancellationRequested)
            {
                return Cancelled(bridgeDirectory, diagnostics, counting, totalSw);
            }

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
            var gender = PlayerGenderReader.Read(reader, candidate.ObjectAddress, layout);
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
                    parentLink,
                    gender));

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

        var playerUids = candidates.Select(candidate => candidate.Uid).ToHashSet();
        foreach (var candidate in scan.Staff)
        {
            if (cancellationToken.IsCancellationRequested)
            {
                return Cancelled(bridgeDirectory, diagnostics, counting, totalSw);
            }

            if (playerUids.Contains(candidate.Uid))
            {
                continue;
            }

            var record = StaffReader.Read(
                reader,
                candidate.ObjectAddress,
                candidate.BlockAddress,
                candidate.Uid,
                candidate.Ca,
                candidate.Pa,
                layout,
                out var clubLink);

            if (cancellationToken.IsCancellationRequested)
            {
                return Cancelled(bridgeDirectory, diagnostics, counting, totalSw);
            }

            if (clubLink is { ClubAddress: not 0 })
            {
                clubAddresses.Add(clubLink.ClubAddress);
            }

            staffDrafts.Add(new StaffDraft(candidate, record, clubLink));
        }

        diagnostics.ExtractionMs = phaseSw.ElapsedMilliseconds;

        if (drafts.Count == 0)
        {
            diagnostics.FailureReason =
                "scan produced candidates but none passed identity sanity (name/DOB)";
            WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
            return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
        }

        if (cancellationToken.IsCancellationRequested)
        {
            return Cancelled(bridgeDirectory, diagnostics, counting, totalSw);
        }

        phaseSw.Restart();
        var squadIndex = SquadClubIndex.Build(
            reader,
            layout,
            personToUid,
            parentClubByUid,
            scan.Clubs,
            clubAddresses,
            scan.HumanManagers.Select(candidate => candidate.ObjectAddress));

        if (cancellationToken.IsCancellationRequested)
        {
            return Cancelled(bridgeDirectory, diagnostics, counting, totalSw);
        }

        foreach (var sample in squadIndex.MultiClubSamples.Take(ScanDiagnostics.MaxSampleClubSnapshots))
        {
            diagnostics.MultiClubSamples.Add(sample);
        }

        diagnostics.ClubsWalked = squadIndex.ClubsWalked;
        diagnostics.PlayersLinkedViaSquad = squadIndex.PlayersLinked;
        diagnostics.ClubIndexingMs = phaseSw.ElapsedMilliseconds;

        var gameDate = GameDateResolver.Resolve(
            squadIndex.DateVotes,
            GameDateResolver.YoungestBirthCohortYear(
                drafts.Select(d => d.Identity.BirthYear),
                minCohortSize: Math.Min(30, Math.Max(1, drafts.Count))));

        diagnostics.GameDateSource = gameDate.Source;
        diagnostics.GameDateBasis = gameDate.Basis;
        diagnostics.GameDate = gameDate.GameDate;

        var staff = staffDrafts
            .Select(draft => draft.Record with
            {
                Age = draft.Record.BirthYear is { } birthYear
                    && draft.Record.BirthDayOfYear is { } birthDayOfYear
                    ? PlayerAge.At(birthYear, birthDayOfYear, gameDate.Year, gameDate.DayOfYear)
                    : null,
            })
            .ToList();
        var staffByUid = staff.ToDictionary(record => record.Uid);
        var staffContractLinks = staffDrafts.ToDictionary(
            draft => draft.Candidate.Uid,
            draft => draft.ClubLink);
        var manager = HumanManagerSelector.Select(
            scan.HumanManagers,
            staffByUid,
            staffContractLinks,
            squadIndex);

        var players = new List<DumpPlayer>(drafts.Count);
        foreach (var draft in drafts)
        {
            if (cancellationToken.IsCancellationRequested)
            {
                return Cancelled(bridgeDirectory, diagnostics, counting, totalSw);
            }

            var parentName = draft.ParentLink?.ClubName;
            string? currentName = null;
            string? division = draft.ParentLink?.Division;
            string? teamLevel = null;
            int? teamType = null;
            int? clubReputation = null;

            if (squadIndex.TryGet(draft.Candidate.Uid, out var squad))
            {
                currentName = squad.ClubName;
                division = squad.Division ?? division;
                teamType = squad.TeamType;
                clubReputation = squad.TeamReputation;
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
                    NationUid = draft.Identity.NationUid,
                    Gender = PlayerGenderValues.ToWireValue(draft.Gender),
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
                    ClubReputation = clubReputation,
                    TeamType = teamType,
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
                        teamType,
                        clubReputation,
                        draft.Identity.NationUid,
                        draft.Gender));
            }
        }

        if (players.Count > 0
            && diagnostics.ClubUnresolved * 2 >= players.Count)
        {
            diagnostics.ClubResolutionWarning =
                $"club resolution failed for {diagnostics.ClubUnresolved}/{players.Count} players";
        }

        var dumpStaff = staff.Select(record => new DumpStaff
        {
            Uid = record.Uid,
            Name = record.Name,
            BirthYear = record.BirthYear,
            BirthDayOfYear = record.BirthDayOfYear,
            Age = record.Age,
            Nationalities = record.Nationalities,
            NationUid = record.NationUid,
            Gender = PlayerGenderValues.ToWireValue(record.Gender),
            Ca = record.Ca,
            Pa = record.Pa,
            Attributes = record.Attributes,
            JobId = record.JobId,
            WeeklyWageGbp = record.WeeklyWageGbp,
            ContractExpiryYear = record.ContractExpiryYear,
            ContractExpiryDayOfYear = record.ContractExpiryDayOfYear,
            Club = record.Club,
            Division = record.Division,
        }).ToList();

        var dumpManager = manager is null
            ? null
            : new DumpManager
            {
                Uid = manager.Uid,
                Name = manager.Name,
                Club = manager.Club,
                ClubReputation = manager.ClubReputation,
            };

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
            GameDateBasis = gameDate.Basis,
            PlayerDatabaseScope = PlayerDatabaseScopes.ToWireValue(playerDatabaseScope),
            ScanTruncated = diagnostics.StoppedEarly,
            MaxAccepted = diagnostics.MaxAccepted,
            PlayerCount = players.Count,
            Players = players,
            StaffCount = dumpStaff.Count,
            Staff = dumpStaff,
            Manager = dumpManager,
        };

        if (cancellationToken.IsCancellationRequested)
        {
            return Cancelled(bridgeDirectory, diagnostics, counting, totalSw);
        }

        phaseSw.Restart();
        bool replaced;
        try
        {
            replaced = DumpWriter.TryWriteReplaceOnSuccess(
                bridgeDirectory,
                document,
                cancellationToken);
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            diagnostics.DumpWritingMs = phaseSw.ElapsedMilliseconds;
            return Cancelled(bridgeDirectory, diagnostics, counting, totalSw);
        }

        diagnostics.DumpWritingMs = phaseSw.ElapsedMilliseconds;
        WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);

        if (!replaced)
        {
            return CapADumpResult.Failed("dump write did not replace file", dumpReplaced: false);
        }

        var result = CapADumpResult.Succeeded(
                players.Count,
                scanTruncated: diagnostics.StoppedEarly,
                maxAccepted: diagnostics.MaxAccepted,
                staff: staff,
                manager: manager);
        return string.Equals(reader.ReadSource, "live", StringComparison.Ordinal)
            ? result with { LivePlayerCandidates = drafts.Select(draft => draft.Candidate).ToArray() }
            : result;
    }

    private static CapADumpResult Cancelled(
        string bridgeDirectory,
        ScanDiagnostics diagnostics,
        CountingMemoryReader counting,
        Stopwatch totalSw)
    {
        diagnostics.Cancelled = true;
        diagnostics.FailureReason = "scan cancelled";
        WriteDiagnostics(bridgeDirectory, diagnostics, counting, totalSw);
        return CapADumpResult.Failed(diagnostics.FailureReason, dumpReplaced: false);
    }

    private sealed class SnapshotScope : IDisposable
    {
        private IProcessSnapshot? _snapshot;

        public IMemoryReader Reader => _snapshot?.Reader
            ?? throw new InvalidOperationException("Snapshot reader is unavailable.");

        public void Set(IProcessSnapshot snapshot)
        {
            ArgumentNullException.ThrowIfNull(snapshot);
            if (_snapshot is not null)
            {
                throw new InvalidOperationException("A scan can use only one snapshot retry.");
            }

            _snapshot = snapshot;
        }

        public void Dispose()
        {
            var snapshot = _snapshot;
            _snapshot = null;
            snapshot?.Dispose();
        }
    }

    private static void WriteDiagnostics(
        string bridgeDirectory,
        ScanDiagnostics diagnostics,
        CountingMemoryReader counting,
        Stopwatch totalSw)
    {
        diagnostics.ProcessMemoryCalls = counting.CallCount;
        diagnostics.ProcessMemoryRequestedBytes = counting.RequestedBytes;
        diagnostics.TotalMs = totalSw.ElapsedMilliseconds;
        DiagnosticsWriter.Write(bridgeDirectory, DiagnosticsWriter.Format(diagnostics));
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
        int? teamType,
        int? clubReputation,
        uint? nationUid,
        PlayerGender gender)
    {
        static string Fmt(string? v) => v ?? "null";
        static string FmtBool(bool? v) => v is { } b ? (b ? "true" : "false") : "null";

        return
            $"uid={uid} name={name} current={Fmt(current)} parent={Fmt(parent)} " +
            $"onLoan={FmtBool(onLoan)} level={Fmt(teamLevel)} tt={teamType?.ToString() ?? "null"} " +
            $"clubRep={clubReputation?.ToString() ?? "null"} nationUid={nationUid?.ToString() ?? "null"} " +
            $"gender={gender.ToString().ToLowerInvariant()}";
    }

    private sealed record PlayerDraft(
        PersonCandidate Candidate,
        PlayerIdentity Identity,
        PlayerAttributes Attrs,
        PlayerContractFields Contract,
        ContractClubLink? ParentLink,
        PlayerGender Gender);

    private sealed record StaffDraft(
        PersonCandidate Candidate,
        StaffRecord Record,
        ContractClubLink? ClubLink);
}

public readonly record struct CapADumpResult(
    bool Success,
    string? Error,
    int PlayerCount,
    bool DumpReplaced,
    bool ScanTruncated,
    int? MaxAccepted,
    IReadOnlyList<StaffRecord> Staff,
    HumanManager? Manager)
{
    /// <summary>
    /// Candidate locations from a successful live read only. This remains internal and never enters a dump or status.
    /// </summary>
    internal IReadOnlyList<PersonCandidate> LivePlayerCandidates { get; init; } = Array.Empty<PersonCandidate>();

    public static CapADumpResult Succeeded(
        int playerCount,
        bool scanTruncated = false,
        int? maxAccepted = null,
        IReadOnlyList<StaffRecord>? staff = null,
        HumanManager? manager = null) =>
        new(
            true,
            null,
            playerCount,
            DumpReplaced: true,
            scanTruncated,
            maxAccepted,
            staff ?? Array.Empty<StaffRecord>(),
            manager);

    public static CapADumpResult Failed(string error, bool dumpReplaced) =>
        new(false, error, 0, dumpReplaced, false, null, Array.Empty<StaffRecord>(), null);
}
