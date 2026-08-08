using System.Text.Json;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;
using FmDataBridge.Output;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class CapADumpTests
{
    private const ulong GameAssemblyBase = 0x180000000UL;
    private const ulong GameAssemblyEnd = 0x180100000UL;
    private const ulong VtableInAssembly = 0x180001000UL;
    private const ulong MetaInAssembly = 0x180002000UL;
    private const ulong PlayerBlockBase = 0x100000UL;
    private const int PlayerClassOffset = 0x288;
    private static readonly ulong PersonAddress = PlayerBlockBase + (ulong)PlayerClassOffset;

    [Fact]
    public void Layout_registry_resolves_26_3_from_full_game_version()
    {
        var registry = LayoutRegistry.CreateDefault();

        Assert.True(registry.TryResolveFromGameVersion("26.3.2.2329565", out var layout));
        Assert.Equal("26.3", layout.VersionKey);
        Assert.Equal(0x0C, layout.ObjectUidOffset);
        Assert.Contains(0x288, layout.PlayerClassOffsets);
        Assert.Contains(0x100, layout.StaffClassOffsets);
        Assert.Contains(0x450, layout.HumanManagerClassOffsets);
        Assert.Equal(0x264, layout.CurrentAbilityOffset);
        Assert.Equal(0x266, layout.PotentialAbilityOffset);
        Assert.Equal(0xDA, layout.StaffCurrentAbilityOffset);
        Assert.Equal(0xDC, layout.StaffPotentialAbilityOffset);
        Assert.Equal(0x19, layout.GenderOffset);
        Assert.Equal(0x10, layout.FemaleGenderBit);
        Assert.True(layout.IsProvisional);
    }

    [Fact]
    public void Layout_registry_rejects_unsupported_version()
    {
        var registry = LayoutRegistry.CreateDefault();

        Assert.False(registry.TryResolveFromGameVersion("25.4.0", out _));
    }

    [Fact]
    public void Unsupported_version_writes_diagnostics_and_preserves_existing_dump()
    {
        var bridgeDir = CreateTempBridgeDir();
        try
        {
            var prior = new DumpDocument
            {
                SchemaVersion = 2,
                GeneratedAtUtc = "2026-07-28T00:00:00Z",
                GameVersion = "26.3.0",
                SupportedGameVersion = "26.3",
                BridgeVersion = "0.1.0",
                ProtocolVersion = 1,
                PlayerCount = 1,
                Players = new[]
                {
                    new DumpPlayer
                    {
                        Uid = 1,
                        Ca = 10,
                        Pa = 20,
                        Name = "Prior Player",
                        BirthYear = 2000,
                        BirthDayOfYear = 1,
                        PreferredFoot = "right",
                    },
                },
            };
            Assert.True(DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, prior));

            var reader = new FakeMemoryReader();
            var pipeline = new CapADumpPipeline();
            var result = pipeline.Run(
                reader,
                bridgeDir,
                gameVersion: "99.0.0",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.False(result.Success);
            Assert.False(result.DumpReplaced);
            Assert.Contains("unsupported", result.Error, StringComparison.OrdinalIgnoreCase);

            var dumpJson = File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir));
            using var doc = JsonDocument.Parse(dumpJson);
            Assert.Equal(1, doc.RootElement.GetProperty("playerCount").GetInt32());

            var diagnostics = File.ReadAllText(BridgePaths.GetDiagnosticsPath(bridgeDir));
            Assert.Contains("unsupported", diagnostics, StringComparison.OrdinalIgnoreCase);
            Assert.Contains("hint=", diagnostics, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Dump_writer_does_not_clobber_good_dump_when_players_empty()
    {
        var bridgeDir = CreateTempBridgeDir();
        try
        {
            var prior = new DumpDocument
            {
                SchemaVersion = 2,
                GeneratedAtUtc = "2026-07-28T00:00:00Z",
                GameVersion = "26.3.0",
                SupportedGameVersion = "26.3",
                BridgeVersion = "0.1.0",
                ProtocolVersion = 1,
                PlayerCount = 1,
                Players = new[]
                {
                    new DumpPlayer
                    {
                        Uid = 42,
                        Ca = 100,
                        Pa = 150,
                        Name = "Kept Player",
                        BirthYear = 1995,
                        BirthDayOfYear = 10,
                        PreferredFoot = "left",
                    },
                },
            };
            Assert.True(DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, prior));

            var empty = new DumpDocument
            {
                SchemaVersion = 2,
                GeneratedAtUtc = "2026-07-28T01:00:00Z",
                GameVersion = "26.3.0",
                SupportedGameVersion = "26.3",
                BridgeVersion = "0.1.0",
                ProtocolVersion = 1,
                PlayerCount = 0,
                Players = Array.Empty<DumpPlayer>(),
            };

            Assert.False(DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, empty));

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(42u, doc.RootElement.GetProperty("players")[0].GetProperty("uid").GetUInt32());
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Dump_metadata_includes_versioned_contract_fields()
    {
        var document = new DumpDocument
        {
            SchemaVersion = BridgeProtocol.DumpSchemaVersion,
            GeneratedAtUtc = "2026-07-28T15:00:00+00:00",
            GameVersion = "26.3.2.2329565",
            SupportedGameVersion = "26.3",
            BridgeVersion = "0.1.0",
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            GameDate = "2026-08-14",
            GameDateSource = "memory",
            GameDateBasis = "next-fixture-consensus",
            PlayerDatabaseScope = "both",
            ScanTruncated = false,
            MaxAccepted = PersonScanner.DefaultMaxAccepted,
            PlayerCount = 1,
            Players = new[]
            {
                new DumpPlayer
                {
                    Uid = 7,
                    Ca = 120,
                    Pa = 160,
                    Name = "Meta Player",
                    BirthYear = 2001,
                    BirthDayOfYear = 33,
                    Nationalities = new[] { "ENG" },
                    NationUid = 44,
                    Gender = PlayerGenderValues.Male,
                    HeightCm = 180,
                    PreferredFoot = "right",
                    Positions = new Dictionary<string, int> { ["ST"] = 20 },
                    Attributes = new Dictionary<string, int?> { ["Acceleration"] = 13 },
                    HiddenAttributes = new Dictionary<string, int?> { ["Consistency"] = 12 },
                    Personality = new Dictionary<string, int?> { ["Ambition"] = 16 },
                    WeeklyWageGbp = 50_000,
                    ContractExpiryYear = 2028,
                    ContractExpiryDayOfYear = 100,
                    TransferListed = false,
                    LoanListed = false,
                    NotForSale = true,
                    SetForRelease = false,
                    MarketValueGbp = 8_000_000,
                    Reputation = new DumpReputation { Current = 4000, World = 3500 },
                    CurrentClub = "Example FC",
                    ParentClub = "Example FC",
                    OnLoan = false,
                    Division = "Premier League",
                    TeamLevel = "senior",
                    ClubReputation = 6200,
                    TeamType = 0,
                    Age = 25,
                },
            },
            StaffCount = 1,
            Staff = new[]
            {
                new DumpStaff
                {
                    Uid = 8,
                    Name = "Meta Staff",
                    BirthYear = 1985,
                    BirthDayOfYear = 100,
                    Age = 24,
                    Nationalities = new[] { "DEN" },
                    NationUid = 208,
                    Gender = PlayerGenderValues.Female,
                    Ca = 100,
                    Pa = 120,
                    Attributes = Fm263Layout.Instance.StaffAttributeEntries.ToDictionary(
                        entry => entry.Key,
                        _ => (int?)15),
                    JobId = 16,
                    WeeklyWageGbp = 20_000,
                    ContractExpiryYear = 2028,
                    ContractExpiryDayOfYear = 80,
                    Club = "Example FC",
                    Division = "Premier League",
                },
            },
            Manager = new DumpManager
            {
                Uid = 8,
                Name = "Meta Staff",
                Club = "Example FC",
                ClubReputation = 6200,
            },
        };

        var json = DumpWriter.Serialize(document);
        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;
        Assert.Equal(BridgeProtocol.DumpSchemaVersion, root.GetProperty("schemaVersion").GetInt32());
        Assert.Equal("2026-08-14", root.GetProperty("gameDate").GetString());
        Assert.Equal("memory", root.GetProperty("gameDateSource").GetString());
        Assert.Equal("next-fixture-consensus", root.GetProperty("gameDateBasis").GetString());
        Assert.Equal("both", root.GetProperty("playerDatabaseScope").GetString());
        Assert.False(root.GetProperty("scanTruncated").GetBoolean());
        Assert.Equal(
            PersonScanner.DefaultMaxAccepted,
            root.GetProperty("maxAccepted").GetInt32());
        Assert.Equal("Example FC", root.GetProperty("players")[0].GetProperty("currentClub").GetString());
        Assert.Equal(25, root.GetProperty("players")[0].GetProperty("age").GetInt32());
        Assert.Equal("26.3.2.2329565", root.GetProperty("gameVersion").GetString());
        Assert.Equal("26.3", root.GetProperty("supportedGameVersion").GetString());
        Assert.Equal("0.1.0", root.GetProperty("bridgeVersion").GetString());
        Assert.Equal(1, root.GetProperty("protocolVersion").GetInt32());
        Assert.Equal(1, root.GetProperty("playerCount").GetInt32());
        Assert.Equal(7u, root.GetProperty("players")[0].GetProperty("uid").GetUInt32());
        Assert.Equal(44u, root.GetProperty("players")[0].GetProperty("nationUid").GetUInt32());
        Assert.Equal("male", root.GetProperty("players")[0].GetProperty("gender").GetString());
        Assert.Equal(120, root.GetProperty("players")[0].GetProperty("ca").GetInt32());
        Assert.Equal(50_000, root.GetProperty("players")[0].GetProperty("weeklyWageGbp").GetInt64());
        Assert.True(root.GetProperty("players")[0].GetProperty("notForSale").GetBoolean());
        Assert.Equal(4000, root.GetProperty("players")[0].GetProperty("reputation").GetProperty("current").GetInt32());
        Assert.Equal(160, root.GetProperty("players")[0].GetProperty("pa").GetInt32());
        Assert.Equal("Meta Player", root.GetProperty("players")[0].GetProperty("name").GetString());
        Assert.Equal(2001, root.GetProperty("players")[0].GetProperty("birthYear").GetInt32());
        Assert.Equal(33, root.GetProperty("players")[0].GetProperty("birthDayOfYear").GetInt32());
        Assert.Equal(6200, root.GetProperty("players")[0].GetProperty("clubReputation").GetInt32());
        Assert.Equal(0, root.GetProperty("players")[0].GetProperty("teamType").GetInt32());
        Assert.Equal(1, root.GetProperty("staffCount").GetInt32());
        Assert.Equal(8u, root.GetProperty("staff")[0].GetProperty("uid").GetUInt32());
        Assert.Equal("female", root.GetProperty("staff")[0].GetProperty("gender").GetString());
        Assert.Equal(
            Fm263Layout.Instance.StaffAttributeEntries.Count,
            root.GetProperty("staff")[0].GetProperty("attributes").EnumerateObject().Count());
        Assert.Equal(15, root.GetProperty("staff")[0].GetProperty("attributes").GetProperty("Attacking").GetInt32());
        Assert.Equal(8u, root.GetProperty("manager").GetProperty("uid").GetUInt32());
        Assert.Equal("Example FC", root.GetProperty("manager").GetProperty("club").GetString());
        Assert.Equal("ENG", root.GetProperty("players")[0].GetProperty("nationalities")[0].GetString());
        Assert.Equal(180, root.GetProperty("players")[0].GetProperty("heightCm").GetInt32());
        Assert.Equal("right", root.GetProperty("players")[0].GetProperty("preferredFoot").GetString());
        Assert.Equal(20, root.GetProperty("players")[0].GetProperty("positions").GetProperty("ST").GetInt32());
        Assert.Equal(13, root.GetProperty("players")[0].GetProperty("attributes").GetProperty("Acceleration").GetInt32());
        Assert.Equal(12, root.GetProperty("players")[0].GetProperty("hiddenAttributes").GetProperty("Consistency").GetInt32());
        Assert.Equal(16, root.GetProperty("players")[0].GetProperty("personality").GetProperty("Ambition").GetInt32());
    }

    [Fact]
    public void Person_scanner_accepts_valid_uid_ca_pa_and_dedupes()
    {
        var layout = Fm263Layout.Instance;
        var reader = BuildReaderWithTwoIdenticalPlayers(layout);
        var regions = RegionEnumerator.GetCandidateRegions(reader);
        var diagnostics = new ScanDiagnostics();
        var gameAssembly = new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd);

        var scan = PersonScanner.Scan(
            reader,
            layout,
            gameAssembly,
            gamePlugin: null,
            regions,
            diagnostics);
        var candidates = scan.Players;

        Assert.Single(candidates);
        Assert.Equal(12345u, candidates[0].Uid);
        Assert.Equal(150, candidates[0].Ca);
        Assert.Equal(170, candidates[0].Pa);
        Assert.True(diagnostics.DuplicatesSkipped >= 1);
    }

    [Fact]
    public void Person_scanner_discovery_scales_with_blocks_not_per_word_reads()
    {
        var layout = Fm263Layout.Instance;
        const ulong regionBase = 0x300000UL;
        const ulong regionSize = 128UL * 1024;
        var inner = new FakeMemoryReader();
        AddCandidateRegion(inner, regionBase, regionSize);

        var personAddress = regionBase + (ulong)PlayerClassOffset;
        PlacePlayerBytes(
            inner,
            layout,
            personAddress,
            uid: 4242,
            ca: 110,
            pa: 140,
            playerBlockBase: regionBase);

        var reader = new CountingMemoryReader(inner);
        var diagnostics = new ScanDiagnostics();
        var scan = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics);
        var candidates = scan.Players;

        Assert.Single(candidates);
        Assert.Equal(4242u, candidates[0].Uid);
        Assert.Equal(110, candidates[0].Ca);
        Assert.Equal(140, candidates[0].Pa);

        var wordsScanned = diagnostics.BytesScanned / 8;
        Assert.True(wordsScanned > 1000, "fixture must scan a multi-kilobyte region");
        // Scalar heap walk issues ~1 ReadProcessMemory per aligned word. Block scanning must
        // keep process-memory calls well below that floor.
        Assert.True(
            reader.CallCount < wordsScanned / 4,
            $"expected block-scale calls, got CallCount={reader.CallCount} for wordsScanned={wordsScanned}");
    }

    [Fact]
    public void Person_scanner_caches_vtable_class_offset_across_hits()
    {
        var layout = Fm263Layout.Instance;
        const ulong regionBase = 0x400000UL;
        const int hitCount = 40;
        // One region large enough for hitCount aligned person headers (invalid UID → reject after resolve).
        const ulong regionSize = 0x1000;
        var inner = new FakeMemoryReader();
        AddCandidateRegion(inner, regionBase, regionSize);

        var metaBytes = new byte[8];
        WriteInt32(metaBytes, 4, PlayerClassOffset);
        inner.AddBytes(MetaInAssembly, metaBytes);
        var vtableLink = new byte[8];
        WriteUInt64(vtableLink, 0, MetaInAssembly);
        inner.AddBytes(VtableInAssembly - 8, vtableLink);

        for (var i = 0; i < hitCount; i++)
        {
            var personAddress = regionBase + (ulong)(i * 0x20);
            var personHeader = new byte[0x10];
            WriteUInt64(personHeader, 0, VtableInAssembly);
            WriteUInt32(personHeader, layout.ObjectUidOffset, 0); // invalid UID — still resolves class offset
            inner.AddBytes(personAddress, personHeader);
        }

        var reader = new CountingMemoryReader(inner);
        var diagnostics = new ScanDiagnostics();
        var scan = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics);
        var candidates = scan.Players;

        Assert.Empty(candidates);
        Assert.Equal(hitCount, diagnostics.VtableHits);
        Assert.Equal(hitCount, diagnostics.CandidatesRejected);
        // Uncached resolve costs 2 module reads per hit (~80 here). Cached resolve pays once.
        Assert.True(
            reader.CallCount < hitCount,
            $"expected cached vtable resolve; CallCount={reader.CallCount} for hits={hitCount}");
    }

    [Fact]
    public void Person_scanner_accepts_candidates_above_int_max_address()
    {
        var layout = Fm263Layout.Instance;
        // Live FM heaps sit well above int.MaxValue; the underflow guard must not cast to int.
        const ulong highPlayerBase = 0x1_0000_1000UL;
        var highPerson = highPlayerBase + (ulong)PlayerClassOffset;
        var reader = new FakeMemoryReader();
        PlacePlayerFixture(
            reader,
            layout,
            highPerson,
            uid: 55555,
            ca: 140,
            pa: 180,
            playerBlockBase: highPlayerBase);

        var scan = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            new ScanDiagnostics());
        var candidates = scan.Players;

        Assert.Single(candidates);
        Assert.Equal(55555u, candidates[0].Uid);
        Assert.Equal(140, candidates[0].Ca);
        Assert.Equal(180, candidates[0].Pa);
    }

    [Fact]
    public void Person_scanner_rejects_out_of_range_abilities()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlacePlayerFixture(reader, layout, PersonAddress, uid: 99, ca: 0, pa: 170);

        var diagnostics = new ScanDiagnostics();
        var scan = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics);
        var candidates = scan.Players;

        Assert.Empty(candidates);
        Assert.True(diagnostics.CandidatesRejected > 0);
    }

    [Fact]
    public void Person_scanner_stops_when_max_accepted_reached()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlacePlayerFixture(reader, layout, PersonAddress, uid: 101, ca: 100, pa: 110);
        PlacePlayerFixture(
            reader,
            layout,
            PersonAddress + 0x100,
            uid: 102,
            ca: 120,
            pa: 130,
            playerBlockBase: PlayerBlockBase + 0x100);
        PlacePlayerFixture(
            reader,
            layout,
            PersonAddress + 0x200,
            uid: 103,
            ca: 140,
            pa: 150,
            playerBlockBase: PlayerBlockBase + 0x200);

        var diagnostics = new ScanDiagnostics();
        var scan = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics,
            maxAccepted: 2);
        var candidates = scan.Players;

        Assert.Equal(2, candidates.Count);
        Assert.True(diagnostics.StoppedEarly);
        Assert.True(diagnostics.ClubDiscoveryIncomplete);
        Assert.True(scan.StoppedEarly);
        Assert.Equal(2, diagnostics.MaxAccepted);
        Assert.Equal(2, diagnostics.CandidatesAccepted);
    }

    [Fact]
    public void Person_scanner_exact_cap_without_extra_players_not_truncated()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlacePlayerFixture(reader, layout, PersonAddress, uid: 101, ca: 100, pa: 110);
        PlacePlayerFixture(
            reader,
            layout,
            PersonAddress + 0x100,
            uid: 102,
            ca: 120,
            pa: 130,
            playerBlockBase: PlayerBlockBase + 0x100);

        var diagnostics = new ScanDiagnostics();
        var scan = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics,
            maxAccepted: 2);
        var candidates = scan.Players;

        Assert.Equal(2, candidates.Count);
        Assert.False(diagnostics.StoppedEarly);
        Assert.False(diagnostics.ClubDiscoveryIncomplete);
        Assert.False(scan.StoppedEarly);
    }

    [Fact]
    public void Pipeline_writes_scan_truncated_when_max_accepted_reached()
    {
        var bridgeDir = CreateTempBridgeDir();
        try
        {
            var layout = Fm263Layout.Instance;
            var reader = new FakeMemoryReader();
            PlacePlayerFixture(
                reader,
                layout,
                PersonAddress,
                uid: 201,
                ca: 150,
                pa: 170,
                name: "Player One",
                birthYear: 2000,
                birthDoy: 100);
            PlacePlayerFixture(
                reader,
                layout,
                PersonAddress + 0x100,
                uid: 202,
                ca: 140,
                pa: 160,
                playerBlockBase: PlayerBlockBase + 0x100,
                name: "Player Two",
                birthYear: 2001,
                birthDoy: 101);
            PlacePlayerFixture(
                reader,
                layout,
                PersonAddress + 0x200,
                uid: 203,
                ca: 130,
                pa: 150,
                playerBlockBase: PlayerBlockBase + 0x200,
                name: "Player Three",
                birthYear: 2002,
                birthDoy: 102);

            var pipeline = new CapADumpPipeline();
            var result = pipeline.Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.1",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
                maxAccepted: 2);

            Assert.True(result.Success);
            Assert.True(result.ScanTruncated);
            Assert.Equal(2, result.MaxAccepted);
            Assert.Equal(2, result.PlayerCount);

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.True(doc.RootElement.GetProperty("scanTruncated").GetBoolean());
            Assert.Equal(2, doc.RootElement.GetProperty("maxAccepted").GetInt32());
            Assert.Equal(2, doc.RootElement.GetProperty("playerCount").GetInt32());

            var diagnostics = File.ReadAllText(BridgePaths.GetDiagnosticsPath(bridgeDir));
            Assert.Contains("scanMateriallyIncomplete=False", diagnostics, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Person_scanner_sets_cancelled_when_token_already_cancelled()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlacePlayerFixture(reader, layout, PersonAddress, uid: 101, ca: 100, pa: 110);

        using var cts = new CancellationTokenSource();
        cts.Cancel();

        var diagnostics = new ScanDiagnostics();
        var scan = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics,
            cancellationToken: cts.Token);
        var candidates = scan.Players;

        Assert.True(diagnostics.Cancelled);
        Assert.True(diagnostics.ClubDiscoveryIncomplete);
        Assert.True(scan.Cancelled);
        Assert.False(diagnostics.StoppedEarly);
        Assert.Equal(0, scan.ReadQuality.RequestedBytes);
        Assert.Equal(0, scan.ReadQuality.UnreadBytes);
        Assert.Empty(candidates);
    }

    [Fact]
    public void Pipeline_preserves_prior_dump_when_scan_cancelled()
    {
        var bridgeDir = CreateTempBridgeDir();
        try
        {
            var prior = new DumpDocument
            {
                SchemaVersion = 5,
                GeneratedAtUtc = "2026-07-28T00:00:00Z",
                GameVersion = "26.3.0",
                SupportedGameVersion = "26.3",
                BridgeVersion = "0.1.0",
                ProtocolVersion = 1,
                ScanTruncated = false,
                MaxAccepted = PersonScanner.DefaultMaxAccepted,
                PlayerCount = 1,
                Players = new[]
                {
                    new DumpPlayer
                    {
                        Uid = 9,
                        Ca = 11,
                        Pa = 12,
                        Name = "Prior Player",
                        BirthYear = 1990,
                        BirthDayOfYear = 2,
                        PreferredFoot = "right",
                    },
                },
            };
            Assert.True(DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, prior));

            var layout = Fm263Layout.Instance;
            var reader = new FakeMemoryReader();
            PlacePlayerFixture(reader, layout, PersonAddress, uid: 201, ca: 150, pa: 170);

            using var cts = new CancellationTokenSource();
            cts.Cancel();

            var pipeline = new CapADumpPipeline();
            var result = pipeline.Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.1",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
                cancellationToken: cts.Token);

            Assert.False(result.Success);
            Assert.False(result.DumpReplaced);
            Assert.Contains("cancelled", result.Error, StringComparison.OrdinalIgnoreCase);

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(1, doc.RootElement.GetProperty("playerCount").GetInt32());
            Assert.Equal("Prior Player", doc.RootElement.GetProperty("players")[0].GetProperty("name").GetString());
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Pipeline_preserves_prior_dump_when_scan_read_quality_is_materially_incomplete()
    {
        var bridgeDir = CreateTempBridgeDir();
        try
        {
            var prior = new DumpDocument
            {
                SchemaVersion = BridgeProtocol.DumpSchemaVersion,
                GeneratedAtUtc = "2026-08-08T00:00:00Z",
                GameVersion = "26.3.2",
                SupportedGameVersion = "26.3",
                BridgeVersion = "0.1.0",
                ProtocolVersion = BridgeProtocol.ProtocolVersion,
                PlayerCount = 1,
                Players = new[]
                {
                    new DumpPlayer
                    {
                        Uid = 9,
                        Ca = 11,
                        Pa = 12,
                        Name = "Prior Player",
                        BirthYear = 1990,
                        BirthDayOfYear = 2,
                        PreferredFoot = "right",
                    },
                },
            };
            Assert.True(DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, prior));

            var layout = Fm263Layout.Instance;
            var reader = new FakeMemoryReader();
            PlacePlayerFixture(reader, layout, PersonAddress, uid: 201, ca: 150, pa: 170);
            AddCandidateRegion(reader, 0x300000, 0x20000);
            reader.AddUnreadableRange(0x300000, 0x20000);

            var result = new CapADumpPipeline().Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.False(result.Success);
            Assert.False(result.DumpReplaced);
            Assert.Contains("incomplete", result.Error, StringComparison.OrdinalIgnoreCase);

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(9u, doc.RootElement.GetProperty("players")[0].GetProperty("uid").GetUInt32());

            var diagnostics = File.ReadAllText(BridgePaths.GetDiagnosticsPath(bridgeDir));
            Assert.Contains("scanReadSource=live", diagnostics, StringComparison.Ordinal);
            Assert.True(ParseDiagnosticLong(diagnostics, "scanUnreadBytes") > 0);
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Person_scanner_records_recovered_readable_and_unread_region_bytes()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        var regionSize = MemoryConstants.MinBlockReadSize * 3;
        AddCandidateRegion(reader, PlayerBlockBase, (ulong)regionSize);
        reader.AddUnreadableRange(
            PlayerBlockBase + (ulong)MemoryConstants.MinBlockReadSize,
            (ulong)MemoryConstants.MinBlockReadSize);
        PlacePlayerBytes(reader, layout, PersonAddress, uid: 201, ca: 150, pa: 170);

        var diagnostics = new ScanDiagnostics();
        var scan = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics);

        Assert.Single(scan.Players);
        Assert.Equal(regionSize, scan.ReadQuality.RequestedBytes);
        Assert.Equal(regionSize - MemoryConstants.MinBlockReadSize, scan.ReadQuality.ReadableBytes);
        Assert.Equal(MemoryConstants.MinBlockReadSize, scan.ReadQuality.UnreadBytes);
        Assert.Equal(0, scan.ReadQuality.InternalFailureBytes);
        Assert.Equal(scan.ReadQuality, diagnostics.ReadQuality);
    }

    [Fact]
    public void Person_scanner_marks_region_read_exceptions_as_internal_unread_bytes()
    {
        var reader = new FakeMemoryReader();
        AddCandidateRegion(reader, PlayerBlockBase, MemoryConstants.MinBlockReadSize);

        var diagnostics = new ScanDiagnostics();
        var scan = PersonScanner.Scan(
            new ThrowingBlockReader(reader),
            Fm263Layout.Instance,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics);

        Assert.Empty(scan.Players);
        Assert.Equal(MemoryConstants.MinBlockReadSize, scan.ReadQuality.RequestedBytes);
        Assert.Equal(0, scan.ReadQuality.ReadableBytes);
        Assert.Equal(MemoryConstants.MinBlockReadSize, scan.ReadQuality.UnreadBytes);
        Assert.Equal(MemoryConstants.MinBlockReadSize, scan.ReadQuality.InternalFailureBytes);
    }

    [Fact]
    public void Scan_read_quality_allows_exactly_ten_percent_unread_coverage()
    {
        var atThreshold = default(ScanReadQuality).Record(requestedBytes: 10, readableBytes: 9);
        var overThreshold = default(ScanReadQuality).Record(requestedBytes: 11, readableBytes: 9);

        Assert.False(atThreshold.IsMateriallyIncomplete);
        Assert.True(overThreshold.IsMateriallyIncomplete);
    }

    [Fact]
    public void Pipeline_writes_dump_when_fake_memory_has_players()
    {
        var bridgeDir = CreateTempBridgeDir();
        try
        {
            var layout = Fm263Layout.Instance;
            var reader = BuildReaderWithTwoIdenticalPlayers(layout);
            var pipeline = new CapADumpPipeline();

            var result = pipeline.Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.1",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success);
            Assert.True(result.DumpReplaced);
            Assert.Equal(1, result.PlayerCount);
            Assert.False(result.ScanTruncated);
            Assert.Null(result.MaxAccepted);

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(1, doc.RootElement.GetProperty("playerCount").GetInt32());
            Assert.False(doc.RootElement.GetProperty("scanTruncated").GetBoolean());
            Assert.Equal(JsonValueKind.Null, doc.RootElement.GetProperty("maxAccepted").ValueKind);
            Assert.True(File.Exists(BridgePaths.GetDiagnosticsPath(bridgeDir)));
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Pipeline_null_max_accepted_is_unlimited_not_default_cap()
    {
        var bridgeDir = CreateTempBridgeDir();
        try
        {
            var layout = Fm263Layout.Instance;
            var reader = BuildReaderWithTwoIdenticalPlayers(layout);
            var pipeline = new CapADumpPipeline();

            var result = pipeline.Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.1",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
                maxAccepted: null);

            Assert.True(result.Success);
            Assert.False(result.ScanTruncated);
            Assert.Null(result.MaxAccepted);

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(JsonValueKind.Null, doc.RootElement.GetProperty("maxAccepted").ValueKind);
            Assert.False(doc.RootElement.GetProperty("scanTruncated").GetBoolean());
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Pipeline_diagnostics_include_phase_timings_and_memory_read_counts()
    {
        var bridgeDir = CreateTempBridgeDir();
        try
        {
            var layout = Fm263Layout.Instance;
            var reader = BuildReaderWithTwoIdenticalPlayers(layout);
            var result = new CapADumpPipeline().Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.1",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success);

            var diagnostics = File.ReadAllText(BridgePaths.GetDiagnosticsPath(bridgeDir));
            AssertNonNegativeDiagnostic(diagnostics, "regionEnumerationMs");
            AssertNonNegativeDiagnostic(diagnostics, "candidateDiscoveryMs");
            AssertNonNegativeDiagnostic(diagnostics, "extractionMs");
            AssertNonNegativeDiagnostic(diagnostics, "clubIndexingMs");
            AssertNonNegativeDiagnostic(diagnostics, "dumpWritingMs");
            AssertNonNegativeDiagnostic(diagnostics, "totalMs");
            AssertNonNegativeDiagnostic(diagnostics, "processMemoryCalls");
            AssertNonNegativeDiagnostic(diagnostics, "processMemoryRequestedBytes");
            AssertNonNegativeDiagnostic(diagnostics, "scanRequestedBytes");
            AssertNonNegativeDiagnostic(diagnostics, "scanReadableBytes");
            AssertNonNegativeDiagnostic(diagnostics, "scanUnreadBytes");
            AssertNonNegativeDiagnostic(diagnostics, "scanInternalFailureBytes");
            Assert.Contains("scanReadSource=live", diagnostics, StringComparison.Ordinal);
            Assert.Equal(
                ParseDiagnosticLong(diagnostics, "scanRequestedBytes"),
                ParseDiagnosticLong(diagnostics, "scanReadableBytes")
                    + ParseDiagnosticLong(diagnostics, "scanUnreadBytes"));
            Assert.True(
                ParseDiagnosticLong(diagnostics, "processMemoryCalls") > 0,
                "successful fake scan should perform at least one memory read");
            Assert.True(
                ParseDiagnosticLong(diagnostics, "processMemoryRequestedBytes") > 0,
                "successful fake scan should request at least one memory byte");
            Assert.True(
                ParseDiagnosticLong(diagnostics, "totalMs")
                >= ParseDiagnosticLong(diagnostics, "regionEnumerationMs"),
                "totalMs should cover at least region enumeration");
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Pipeline_zero_candidates_preserves_prior_dump()
    {
        var bridgeDir = CreateTempBridgeDir();
        try
        {
            var prior = new DumpDocument
            {
                SchemaVersion = 2,
                GeneratedAtUtc = "2026-07-28T00:00:00Z",
                GameVersion = "26.3.0",
                SupportedGameVersion = "26.3",
                BridgeVersion = "0.1.0",
                ProtocolVersion = 1,
                PlayerCount = 1,
                Players = new[]
                {
                    new DumpPlayer
                    {
                        Uid = 9,
                        Ca = 11,
                        Pa = 12,
                        Name = "Prior Zero",
                        BirthYear = 1990,
                        BirthDayOfYear = 2,
                        PreferredFoot = "either",
                    },
                },
            };
            Assert.True(DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, prior));

            var reader = new FakeMemoryReader();
            AddCandidateRegion(reader, PlayerBlockBase, 0x1000);
            var result = new CapADumpPipeline().Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.0",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.False(result.Success);
            Assert.False(result.DumpReplaced);

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(9u, doc.RootElement.GetProperty("players")[0].GetProperty("uid").GetUInt32());

            var diagnostics = File.ReadAllText(BridgePaths.GetDiagnosticsPath(bridgeDir));
            Assert.Contains("zero player", diagnostics, StringComparison.OrdinalIgnoreCase);
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    private static FakeMemoryReader BuildReaderWithTwoIdenticalPlayers(IFmMemoryLayout layout)
    {
        var reader = new FakeMemoryReader();
        PlacePlayerFixture(
            reader,
            layout,
            PersonAddress,
            uid: 12345,
            ca: 150,
            pa: 170,
            name: "Test Player",
            birthYear: 2000,
            birthDoy: 100);
        // Second person object with the same UID (dedupe).
        PlacePlayerFixture(
            reader,
            layout,
            PersonAddress + 0x80,
            uid: 12345,
            ca: 150,
            pa: 170,
            playerBlockBase: PlayerBlockBase + 0x80,
            name: "Test Player",
            birthYear: 2000,
            birthDoy: 100);
        return reader;
    }

    private sealed class ThrowingBlockReader : IMemoryReader
    {
        private readonly IMemoryReader _inner;

        public ThrowingBlockReader(IMemoryReader inner)
        {
            _inner = inner;
        }

        public string ReadSource => _inner.ReadSource;

        public IEnumerable<MemoryRegion> EnumerateRegions() => _inner.EnumerateRegions();

        public bool TryRead(ulong address, Span<byte> destination, out int bytesRead) =>
            _inner.TryRead(address, destination, out bytesRead);

        public bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead) =>
            throw new InvalidOperationException("synthetic region failure");
    }

    private static void PlacePlayerFixture(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong personAddress,
        uint uid,
        int ca,
        int pa,
        ulong? playerBlockBase = null,
        string? name = "Fixture Player",
        int birthYear = 1999,
        int birthDoy = 50)
    {
        var playerBase = playerBlockBase ?? (personAddress - (ulong)PlayerClassOffset);
        var regionBase = Math.Min(playerBase, personAddress);
        var regionEnd = Math.Max(
            personAddress + 0xA0,
            playerBase + (ulong)Math.Max(layout.PotentialAbilityOffset, layout.HeightOffset) + 2);
        regionEnd = Math.Max(regionEnd, playerBase + (ulong)layout.AttrsOffset + 0x40);
        AddCandidateRegion(reader, regionBase, regionEnd - regionBase);
        PlacePlayerBytes(reader, layout, personAddress, uid, ca, pa, playerBase, name, birthYear, birthDoy);
    }

    /// <summary>
    /// Writes person/player bytes without adding a candidate region (caller owns the region).
    /// </summary>
    private static void PlacePlayerBytes(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong personAddress,
        uint uid,
        int ca,
        int pa,
        ulong? playerBlockBase = null,
        string? name = "Fixture Player",
        int birthYear = 1999,
        int birthDoy = 50)
    {
        var playerBase = playerBlockBase ?? (personAddress - (ulong)PlayerClassOffset);

        // Il2Cpp meta: *(vtable - 8) → meta; *(int*)(meta + 4) → class offset.
        var metaBytes = new byte[8];
        WriteInt32(metaBytes, 4, PlayerClassOffset);
        reader.AddBytes(MetaInAssembly, metaBytes);

        var vtableLink = new byte[8];
        WriteUInt64(vtableLink, 0, MetaInAssembly);
        reader.AddBytes(VtableInAssembly - 8, vtableLink);

        var personHeader = new byte[0x10];
        WriteUInt64(personHeader, 0, VtableInAssembly);
        WriteUInt32(personHeader, layout.ObjectUidOffset, uid);
        reader.AddBytes(personAddress, personHeader);

        var abilitySpan = Math.Max(layout.PotentialAbilityOffset, layout.HeightOffset) + sizeof(ushort);
        abilitySpan = Math.Max(abilitySpan, layout.AttrsOffset + 0x40);
        abilitySpan = Math.Max(abilitySpan, layout.PositionsOffset + 16);
        var playerBytes = new byte[abilitySpan];
        WriteUInt16(playerBytes, layout.CurrentAbilityOffset, (ushort)ca);
        WriteUInt16(playerBytes, layout.PotentialAbilityOffset, (ushort)pa);
        WriteUInt16(playerBytes, layout.HeightOffset, 180);
        playerBytes[layout.AttrsOffset + layout.FootLeftAttrOffset] = 25; // ≈5
        playerBytes[layout.AttrsOffset + layout.FootRightAttrOffset] = 90; // ≈18
        playerBytes[layout.PositionsOffset + 0x0C] = 20; // ST
        reader.AddBytes(playerBase, playerBytes);

        if (!string.IsNullOrEmpty(name))
        {
            PlaceNestedName(reader, personAddress, layout, name);
        }

        uint dob = birthYear == 0 && birthDoy == 0
            ? 0u
            : ((uint)birthYear << 16) | (uint)birthDoy;
        reader.AddBytes(personAddress + (ulong)layout.DobOffset, BitConverter.GetBytes(dob));
    }

    private static void PlaceNestedName(
        FakeMemoryReader reader,
        ulong personAddress,
        IFmMemoryLayout layout,
        string name)
    {
        var outer = personAddress + 0x10000;
        var inner = outer + 0x40;
        reader.AddBytes(personAddress + (ulong)layout.CommonNameOffset, BitConverter.GetBytes(outer));
        reader.AddBytes(outer, BitConverter.GetBytes(inner));
        var utf8 = System.Text.Encoding.UTF8.GetBytes(name + "\0");
        var payload = new byte[4 + utf8.Length];
        utf8.CopyTo(payload, 4);
        reader.AddBytes(inner, payload);
    }

    private static void AddCandidateRegion(FakeMemoryReader reader, ulong baseAddress, ulong size)
    {
        reader.AddRegion(
            new MemoryRegion(
                baseAddress,
                size,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));
    }

    private static void WriteUInt64(byte[] buffer, int offset, ulong value)
    {
        var span = buffer.AsSpan(offset, sizeof(ulong));
        BitConverter.TryWriteBytes(span, value);
    }

    private static void WriteUInt32(byte[] buffer, int offset, uint value)
    {
        var span = buffer.AsSpan(offset, sizeof(uint));
        BitConverter.TryWriteBytes(span, value);
    }

    private static void WriteUInt16(byte[] buffer, int offset, ushort value)
    {
        var span = buffer.AsSpan(offset, sizeof(ushort));
        BitConverter.TryWriteBytes(span, value);
    }

    private static void WriteInt32(byte[] buffer, int offset, int value)
    {
        var span = buffer.AsSpan(offset, sizeof(int));
        BitConverter.TryWriteBytes(span, value);
    }

    private static string CreateTempBridgeDir()
    {
        var path = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(path);
        return path;
    }

    private static void AssertNonNegativeDiagnostic(string diagnostics, string key)
    {
        Assert.True(
            ParseDiagnosticLong(diagnostics, key) >= 0,
            $"diagnostics must include non-negative {key}");
    }

    private static long ParseDiagnosticLong(string diagnostics, string key)
    {
        var prefix = key + "=";
        foreach (var line in diagnostics.Split('\n'))
        {
            var trimmed = line.TrimEnd('\r');
            if (!trimmed.StartsWith(prefix, StringComparison.Ordinal))
            {
                continue;
            }

            Assert.True(
                long.TryParse(trimmed.AsSpan(prefix.Length), out var value),
                $"{key} must be an integer");
            return value;
        }

        Assert.Fail($"diagnostics missing {key}");
        return -1;
    }
}
