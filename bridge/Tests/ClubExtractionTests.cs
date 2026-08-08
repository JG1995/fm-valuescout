using System.Text;
using System.Text.Json;
using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class ClubExtractionTests
{
    private const ulong GameAssemblyBase = 0x180000000UL;
    private const ulong GameAssemblyEnd = 0x180100000UL;
    private const ulong VtableInAssembly = 0x180001000UL;
    private const ulong MetaInAssembly = 0x180002000UL;
    private const int PlayerClassOffset = 0x288;

    // Loan player (uid 77): contract → Parent FC; appears in Parent + Loan squads.
    private const ulong LoanPlayerBase = 0x200000UL;
    private static readonly ulong LoanPerson = LoanPlayerBase + (ulong)PlayerClassOffset;
    private static readonly ulong LoanContract = LoanPerson + 0x3000UL;
    private static readonly ulong ParentTeam = LoanPerson + 0x4000UL;
    private static readonly ulong ParentClub = LoanPerson + 0x5000UL;
    private static readonly ulong ParentComp = LoanPerson + 0x6000UL;
    private static readonly ulong ParentSchedule = LoanPerson + 0x7000UL;
    private static readonly ulong ParentTeamArray = LoanPerson + 0x8000UL;
    private static readonly ulong ParentSquadArray = LoanPerson + 0x9000UL;

    private static readonly ulong LoanTeam = LoanPerson + 0xA000UL;
    private static readonly ulong LoanClub = LoanPerson + 0xB000UL;
    private static readonly ulong LoanComp = LoanPerson + 0xC000UL;
    private static readonly ulong LoanSchedule = LoanPerson + 0xD000UL;
    private static readonly ulong LoanTeamArray = LoanPerson + 0xE000UL;
    private static readonly ulong LoanSquadArray = LoanPerson + 0xF000UL;

    // Seed player (uid 88): contract → Loan FC so Loan FC enters the club walk set.
    private const ulong SeedPlayerBase = 0x300000UL;
    private static readonly ulong SeedPerson = SeedPlayerBase + (ulong)PlayerClassOffset;
    private static readonly ulong SeedContract = SeedPerson + 0x3000UL;

    private const ulong GlobalPlayerBase = 0x350000UL;
    private static readonly ulong GlobalPerson = GlobalPlayerBase + (ulong)PlayerClassOffset;
    private const ulong FalseClub = 0x330000UL;
    private const ulong FalseTeam = 0x331000UL;
    private const ulong FalseComp = 0x332000UL;
    private const ulong FalseSchedule = 0x333000UL;
    private const ulong FalseTeamArray = 0x334001UL;
    private const ulong FalseSquadArray = 0x335000UL;
    private const ulong GlobalClub = 0x340000UL;
    private const ulong GlobalTeam = 0x341000UL;
    private const ulong GlobalComp = 0x342000UL;
    private const ulong GlobalSchedule = 0x343000UL;
    private const ulong GlobalTeamArray = 0x344000UL;
    private const ulong GlobalSquadArray = 0x345000UL;

    [Theory]
    [InlineData(0, "senior")]
    [InlineData(3, "reserve")]
    [InlineData(11, "youth")]
    [InlineData(-1, null)]
    public void Team_level_maps_fm_team_type(int teamType, string? expected)
    {
        Assert.Equal(expected, TeamLevelMap.FromTeamType(teamType));
    }

    [Fact]
    public void Contract_club_reader_resolves_parent_club_and_division()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlaceClubObject(reader, layout, ParentClub, "Parent FC");
        PlaceTeamObject(
            reader,
            layout,
            ParentTeam,
            ParentClub,
            ParentComp,
            "Premier League",
            teamType: 0,
            schedule: ParentSchedule,
            scheduleYear: 2026,
            scheduleDoy: 226);
        reader.AddBytes(LoanPerson + (ulong)layout.FullContractPtrOffset, BitConverter.GetBytes(LoanContract));
        reader.AddBytes(LoanContract + (ulong)layout.ContractTeamPtrOffset, BitConverter.GetBytes(ParentTeam));

        var link = ContractClubReader.TryRead(reader, LoanPerson, layout);

        Assert.NotNull(link);
        Assert.Equal("Parent FC", link!.ClubName);
        Assert.Equal("Premier League", link.Division);
        Assert.Equal(ParentClub, link.ClubAddress);
        Assert.Equal(ParentTeam, link.TeamAddress);
    }

    [Fact]
    public void Squad_pick_prefers_non_parent_when_player_in_two_clubs()
    {
        var chosen = SquadPick.Choose(
            current: new SquadHit("Parent FC", TeamType: 0, Division: "PL"),
            candidate: new SquadHit("Loan FC", TeamType: 0, Division: "Championship"),
            parentClub: "Parent FC");

        Assert.Equal("Loan FC", chosen.ClubName);
        Assert.Equal("Championship", chosen.Division);
    }

    [Fact]
    public void Squad_pick_prefers_lower_team_type_within_same_club()
    {
        var chosen = SquadPick.Choose(
            current: new SquadHit("Same FC", TeamType: 3, Division: "Reserves"),
            candidate: new SquadHit("Same FC", TeamType: 0, Division: "First"),
            parentClub: "Same FC");

        Assert.Equal(0, chosen.TeamType);
        Assert.Equal("First", chosen.Division);
    }

    [Fact]
    public void Game_date_resolver_uses_majority_memory_vote()
    {
        var votes = new Dictionary<uint, int>
        {
            [(2026u << 16) | 226u] = 12,
            [(2026u << 16) | 230u] = 2,
        };

        var resolved = GameDateResolver.Resolve(votes, youngestBirthCohortYear: 2010);

        Assert.Equal("2026-08-14", resolved.GameDate);
        Assert.Equal("memory", resolved.Source);
        Assert.Equal(2026, resolved.Year);
        Assert.Equal(226, resolved.DayOfYear);
    }

    [Fact]
    public void Player_age_uses_game_date()
    {
        Assert.Equal(25, PlayerAge.At(2000, 100, gameYear: 2026, gameDayOfYear: 50));
        Assert.Equal(26, PlayerAge.At(2000, 100, gameYear: 2026, gameDayOfYear: 100));
    }

    [Fact]
    public void Pipeline_writes_schema_v5_clubs_loan_game_date_and_age()
    {
        var bridgeDir = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDir);
        try
        {
            var layout = Fm263Layout.Instance;
            var reader = new FakeMemoryReader();
            PlaceLoanScenario(reader, layout);

            var result = new CapADumpPipeline().Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success);
            Assert.Equal(2, result.PlayerCount);

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(5, doc.RootElement.GetProperty("schemaVersion").GetInt32());
            Assert.Equal("2026-08-14", doc.RootElement.GetProperty("gameDate").GetString());
            Assert.Equal("memory", doc.RootElement.GetProperty("gameDateSource").GetString());

            var players = doc.RootElement.GetProperty("players");
            var loaned = FindPlayer(players, 77);
            Assert.Equal("Loan FC", loaned.GetProperty("currentClub").GetString());
            Assert.Equal("Parent FC", loaned.GetProperty("parentClub").GetString());
            Assert.True(loaned.GetProperty("onLoan").GetBoolean());
            Assert.Equal("Championship", loaned.GetProperty("division").GetString());
            Assert.Equal("senior", loaned.GetProperty("teamLevel").GetString());
            Assert.Equal(26, loaned.GetProperty("age").GetInt32());

            var seed = FindPlayer(players, 88);
            Assert.Equal("Loan FC", seed.GetProperty("currentClub").GetString());
            Assert.Equal("Loan FC", seed.GetProperty("parentClub").GetString());
            Assert.False(seed.GetProperty("onLoan").GetBoolean());

            var diagnostics = File.ReadAllText(BridgePaths.GetDiagnosticsPath(bridgeDir));
            Assert.Contains("sampleClubs:", diagnostics, StringComparison.Ordinal);
            Assert.Contains("uid=77", diagnostics, StringComparison.Ordinal);
            Assert.Contains("gameDateSource=memory", diagnostics, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Pipeline_uses_discovered_clubs_before_contract_seeded_fallbacks()
    {
        var bridgeDir = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDir);
        try
        {
            var layout = Fm263Layout.Instance;
            var reader = new FakeMemoryReader();
            PlaceLoanScenario(reader, layout, includeGlobalClub: true);

            var result = new CapADumpPipeline().Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success);
            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            var players = doc.RootElement.GetProperty("players");
            Assert.Equal("Global FC", FindPlayer(players, 99).GetProperty("currentClub").GetString());
            Assert.Equal("Loan FC", FindPlayer(players, 77).GetProperty("currentClub").GetString());
            Assert.Equal("Loan FC", FindPlayer(players, 88).GetProperty("currentClub").GetString());
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Reordered_discovered_clubs_keep_the_same_squad_selection()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlaceLoanScenario(reader, layout);

        var personToUid = new Dictionary<ulong, uint> { [LoanPerson] = 77 };
        var parentClubByUid = new Dictionary<uint, string?> { [77] = "Parent FC" };
        var clubs = new[]
        {
            new ClubCandidate(LoanClub, "Loan FC"),
            new ClubCandidate(ParentClub, "Parent FC"),
        };

        var forward = SquadClubIndex.Build(
            reader,
            layout,
            personToUid,
            parentClubByUid,
            clubs,
            Array.Empty<ulong>());
        var reversed = SquadClubIndex.Build(
            reader,
            layout,
            personToUid,
            parentClubByUid,
            clubs.Reverse(),
            Array.Empty<ulong>());

        Assert.True(forward.TryGet(77, out var forwardAssignment));
        Assert.True(reversed.TryGet(77, out var reversedAssignment));
        Assert.Equal("Loan FC", forwardAssignment.ClubName);
        Assert.Equal(forwardAssignment.ClubName, reversedAssignment.ClubName);
        Assert.Equal(forwardAssignment.TeamType, reversedAssignment.TeamType);
        Assert.Equal(forwardAssignment.Division, reversedAssignment.Division);
    }

    private static JsonElement FindPlayer(JsonElement players, uint uid)
    {
        foreach (var p in players.EnumerateArray())
        {
            if (p.GetProperty("uid").GetUInt32() == uid)
            {
                return p;
            }
        }

        throw new InvalidOperationException($"player uid={uid} not found in dump");
    }

    private static void PlaceLoanScenario(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        bool includeGlobalClub = false)
    {
        var regionEnd = includeGlobalClub
            ? GlobalPlayerBase + 0x12000
            : SeedContract + 0x200;
        reader.AddRegion(
            new MemoryRegion(
                LoanPlayerBase,
                regionEnd - LoanPlayerBase,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));

        PlaceClubObject(reader, layout, ParentClub, "Parent FC");
        PlaceClubObject(reader, layout, LoanClub, "Loan FC");

        PlaceTeamObject(
            reader,
            layout,
            ParentTeam,
            ParentClub,
            ParentComp,
            "Premier League",
            teamType: 0,
            schedule: ParentSchedule,
            scheduleYear: 2026,
            scheduleDoy: 226);
        PlaceTeamObject(
            reader,
            layout,
            LoanTeam,
            LoanClub,
            LoanComp,
            "Championship",
            teamType: 0,
            schedule: LoanSchedule,
            scheduleYear: 2026,
            scheduleDoy: 226);

        // Parent club → [ParentTeam]; squad lists loan person.
        reader.AddBytes(ParentClub + (ulong)layout.ClubTeamsBeginOffset, BitConverter.GetBytes(ParentTeamArray));
        reader.AddBytes(ParentClub + (ulong)layout.ClubTeamsEndOffset, BitConverter.GetBytes(ParentTeamArray + 8));
        reader.AddBytes(ParentTeamArray, BitConverter.GetBytes(ParentTeam));
        reader.AddBytes(ParentTeam + (ulong)layout.TeamSquadBeginOffset, BitConverter.GetBytes(ParentSquadArray));
        reader.AddBytes(ParentTeam + (ulong)layout.TeamSquadEndOffset, BitConverter.GetBytes(ParentSquadArray + 8));
        reader.AddBytes(ParentSquadArray, BitConverter.GetBytes(LoanPerson));

        // Loan club → [LoanTeam]; squad lists loan person + seed person.
        reader.AddBytes(LoanClub + (ulong)layout.ClubTeamsBeginOffset, BitConverter.GetBytes(LoanTeamArray));
        reader.AddBytes(LoanClub + (ulong)layout.ClubTeamsEndOffset, BitConverter.GetBytes(LoanTeamArray + 8));
        reader.AddBytes(LoanTeamArray, BitConverter.GetBytes(LoanTeam));
        reader.AddBytes(LoanTeam + (ulong)layout.TeamSquadBeginOffset, BitConverter.GetBytes(LoanSquadArray));
        reader.AddBytes(
            LoanTeam + (ulong)layout.TeamSquadEndOffset,
            BitConverter.GetBytes(LoanSquadArray + (includeGlobalClub ? 24UL : 16UL)));
        reader.AddBytes(LoanSquadArray, BitConverter.GetBytes(LoanPerson));
        reader.AddBytes(LoanSquadArray + 8, BitConverter.GetBytes(SeedPerson));
        if (includeGlobalClub)
        {
            reader.AddBytes(LoanSquadArray + 16, BitConverter.GetBytes(GlobalPerson));
        }

        PlaceScannablePlayer(
            reader,
            layout,
            playerBase: LoanPlayerBase,
            person: LoanPerson,
            contract: LoanContract,
            team: ParentTeam,
            uid: 77,
            ca: 150,
            pa: 180,
            name: "Loan Player",
            birthYear: 2000,
            birthDoy: 100);

        PlaceScannablePlayer(
            reader,
            layout,
            playerBase: SeedPlayerBase,
            person: SeedPerson,
            contract: SeedContract,
            team: LoanTeam,
            uid: 88,
            ca: 120,
            pa: 140,
            name: "Seed Player",
            birthYear: 1998,
            birthDoy: 50);

        if (!includeGlobalClub)
        {
            return;
        }

        PlaceClubObject(reader, layout, FalseClub, "False FC");
        PlaceTeamObject(
            reader,
            layout,
            FalseTeam,
            FalseClub,
            FalseComp,
            "False League",
            teamType: 0,
            schedule: FalseSchedule,
            scheduleYear: 2026,
            scheduleDoy: 226);
        reader.AddBytes(FalseClub + (ulong)layout.ClubTeamsBeginOffset, BitConverter.GetBytes(FalseTeamArray));
        reader.AddBytes(FalseClub + (ulong)layout.ClubTeamsEndOffset, BitConverter.GetBytes(FalseTeamArray + 8));
        reader.AddBytes(FalseTeamArray, BitConverter.GetBytes(FalseTeam));
        reader.AddBytes(FalseTeam + (ulong)layout.TeamSquadBeginOffset, BitConverter.GetBytes(FalseSquadArray));
        reader.AddBytes(FalseTeam + (ulong)layout.TeamSquadEndOffset, BitConverter.GetBytes(FalseSquadArray + 8));
        reader.AddBytes(FalseSquadArray, BitConverter.GetBytes(GlobalPerson));
        PlaceScannableClub(reader, layout, FalseClub, uid: 9000);

        PlaceClubObject(reader, layout, GlobalClub, "Global FC");
        PlaceTeamObject(
            reader,
            layout,
            GlobalTeam,
            GlobalClub,
            GlobalComp,
            "Global League",
            teamType: 0,
            schedule: GlobalSchedule,
            scheduleYear: 2026,
            scheduleDoy: 226);
        reader.AddBytes(GlobalClub + (ulong)layout.ClubTeamsBeginOffset, BitConverter.GetBytes(GlobalTeamArray));
        reader.AddBytes(GlobalClub + (ulong)layout.ClubTeamsEndOffset, BitConverter.GetBytes(GlobalTeamArray + 8));
        reader.AddBytes(GlobalTeamArray, BitConverter.GetBytes(GlobalTeam));
        reader.AddBytes(GlobalTeam + (ulong)layout.TeamSquadBeginOffset, BitConverter.GetBytes(GlobalSquadArray));
        reader.AddBytes(GlobalTeam + (ulong)layout.TeamSquadEndOffset, BitConverter.GetBytes(GlobalSquadArray + 8));
        reader.AddBytes(GlobalSquadArray, BitConverter.GetBytes(GlobalPerson));
        PlaceScannableClub(reader, layout, GlobalClub, uid: 9001);
        PlaceScannablePlayer(
            reader,
            layout,
            playerBase: GlobalPlayerBase,
            person: GlobalPerson,
            contract: 0,
            team: 0,
            uid: 99,
            ca: 110,
            pa: 130,
            name: "Global Player",
            birthYear: 2001,
            birthDoy: 120);
    }

    private static void PlaceScannableClub(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong club,
        uint uid)
    {
        var metadata = GameAssemblyBase + 0x3100UL;
        var vtable = GameAssemblyBase + 0x3000UL;
        var metaBytes = new byte[8];
        BitConverter.TryWriteBytes(metaBytes.AsSpan(4), 0x600);
        reader.AddBytes(metadata, metaBytes);
        reader.AddBytes(vtable - 8, BitConverter.GetBytes(metadata));

        var header = new byte[0x10];
        BitConverter.TryWriteBytes(header.AsSpan(), vtable);
        BitConverter.TryWriteBytes(header.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(club, header);
    }

    private static void PlaceScannablePlayer(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong playerBase,
        ulong person,
        ulong contract,
        ulong team,
        uint uid,
        int ca,
        int pa,
        string name,
        int birthYear,
        int birthDoy)
    {
        var metaBytes = new byte[8];
        BitConverter.TryWriteBytes(metaBytes.AsSpan(4), PlayerClassOffset);
        reader.AddBytes(MetaInAssembly, metaBytes);

        var vtableLink = new byte[8];
        BitConverter.TryWriteBytes(vtableLink.AsSpan(), MetaInAssembly);
        reader.AddBytes(VtableInAssembly - 8, vtableLink);

        var personHeader = new byte[0x10];
        BitConverter.TryWriteBytes(personHeader.AsSpan(), VtableInAssembly);
        BitConverter.TryWriteBytes(personHeader.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(person, personHeader);

        foreach (var entry in layout.PersonalityEntries)
        {
            reader.AddBytes(person + (ulong)entry.Offset, new[] { (byte)10 });
        }

        var abilitySpan = Math.Max(layout.PotentialAbilityOffset, layout.HeightOffset) + 2;
        abilitySpan = Math.Max(abilitySpan, layout.PositionsOffset + 16);
        abilitySpan = Math.Max(abilitySpan, layout.AttrsOffset + 0x40);
        abilitySpan = Math.Max(abilitySpan, layout.MarketValueOffset + 4);
        abilitySpan = Math.Max(abilitySpan, layout.WorldReputationOffset + 2);
        var playerBytes = new byte[abilitySpan];
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.CurrentAbilityOffset), (ushort)ca);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.PotentialAbilityOffset), (ushort)pa);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.HeightOffset), (ushort)180);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.MarketValueOffset), 5_000_000u);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.CurrentReputationOffset), (ushort)2000);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.WorldReputationOffset), (ushort)1500);
        playerBytes[layout.AttrsOffset + layout.FootLeftAttrOffset] = 25;
        playerBytes[layout.AttrsOffset + layout.FootRightAttrOffset] = 90;
        playerBytes[layout.PositionsOffset + layout.PositionEntries.First(p => p.Key == "ST").Offset] = 20;
        foreach (var entry in layout.AttributeEntries)
        {
            playerBytes[layout.AttrsOffset + entry.Offset] = 50;
        }

        foreach (var entry in layout.HiddenAttributeEntries)
        {
            playerBytes[layout.AttrsOffset + entry.Offset] = 40;
        }

        reader.AddBytes(playerBase, playerBytes);

        var stringBase = person + 0x10000;
        PlaceNestedString(reader, stringBase, name);
        reader.AddBytes(person + (ulong)layout.CommonNameOffset, BitConverter.GetBytes(stringBase));
        reader.AddBytes(
            person + (ulong)layout.DobOffset,
            BitConverter.GetBytes(((uint)birthYear << 16) | (uint)birthDoy));

        reader.AddBytes(person + (ulong)layout.FullContractPtrOffset, BitConverter.GetBytes(contract));
        if (contract != 0)
        {
            reader.AddBytes(contract + (ulong)layout.ContractTeamPtrOffset, BitConverter.GetBytes(team));
            reader.AddBytes(contract + (ulong)layout.ContractWeeklyWageOffset, BitConverter.GetBytes(10_000u));
            reader.AddBytes(
                contract + (ulong)layout.ContractExpiryOffset,
                BitConverter.GetBytes((2029u << 16) | 1u));
            reader.AddBytes(contract + (ulong)layout.ContractStatusFlagsOffset, new byte[] { 0 });
        }
    }

    private static void PlaceClubObject(FakeMemoryReader reader, IFmMemoryLayout layout, ulong club, string name)
    {
        PlaceIndirectString(reader, club + 0x200, name);
        reader.AddBytes(club + (ulong)layout.ClubNameOffset, BitConverter.GetBytes(club + 0x200));
    }

    private static void PlaceTeamObject(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong team,
        ulong club,
        ulong comp,
        string division,
        int teamType,
        ulong schedule,
        int scheduleYear,
        int scheduleDoy)
    {
        reader.AddBytes(team + (ulong)layout.TeamClubPtrOffset, BitConverter.GetBytes(club));
        reader.AddBytes(team + (ulong)layout.TeamTypeOffset, new[] { (byte)teamType });
        reader.AddBytes(team + (ulong)layout.TeamReputationOffset, BitConverter.GetBytes((ushort)5000));
        reader.AddBytes(team + (ulong)layout.TeamCompPtrOffset, BitConverter.GetBytes(comp));
        PlaceIndirectString(reader, comp + 0x100, division);
        reader.AddBytes(comp + (ulong)layout.CompNameOffset, BitConverter.GetBytes(comp + 0x100));
        reader.AddBytes(team + (ulong)layout.TeamSchedulePtrOffset, BitConverter.GetBytes(schedule));
        reader.AddBytes(
            schedule + (ulong)layout.ScheduleNextMatchOffset,
            BitConverter.GetBytes(((uint)scheduleYear << 16) | (uint)scheduleDoy));
    }

    private static void PlaceNestedString(FakeMemoryReader reader, ulong outerAddress, string value)
    {
        var inner = outerAddress + 0x40;
        reader.AddBytes(outerAddress, BitConverter.GetBytes(inner));
        var utf8 = Encoding.UTF8.GetBytes(value + "\0");
        var payload = new byte[4 + utf8.Length];
        utf8.CopyTo(payload, 4);
        reader.AddBytes(inner, payload);
    }

    private static void PlaceIndirectString(FakeMemoryReader reader, ulong stringObject, string value)
    {
        var utf8 = Encoding.UTF8.GetBytes(value + "\0");
        var payload = new byte[4 + utf8.Length];
        utf8.CopyTo(payload, 4);
        reader.AddBytes(stringObject, payload);
    }
}
