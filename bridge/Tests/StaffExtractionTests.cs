using System.Text;
using System.Text.Json;
using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class StaffExtractionTests
{
    private const ulong GameAssemblyBase = 0x180000000UL;
    private const ulong GameAssemblyEnd = 0x180100000UL;
    private const ulong ScanRegionBase = 0x200000UL;
    private const ulong ScanRegionSize = 0x20000UL;
    private const int PlayerClassOffset = 0x288;
    private const int StaffClassOffset = 0x100;
    private const int HumanManagerClassOffset = 0x450;
    private static readonly (string Key, int Offset, int? Value)[] ExpectedStaffAttributes =
    {
        ("Attacking", 0x22, 1),
        ("Defending", 0x23, 2),
        ("Fitness", 0x24, 3),
        ("Possession", 0x25, 4),
        ("Technical", 0x26, 5),
        ("Tactical", 0x27, 6),
        ("SetPieces", 0x33, 7),
        ("Determination", 0x0D, 8),
        ("ManManagement", 0x1E, 9),
        ("Motivating", 0x1F, 10),
        ("JudgingPlayerAbility", 0x1C, 11),
        ("JudgingPlayerPotential", 0x1D, 12),
        ("JudgingStaffAbility", 0x32, 13),
        ("Negotiating", 0x31, 14),
        ("TacticalKnowledge", 0x21, 15),
        ("Physiotherapy", 0x20, 16),
        ("SportsScience", 0x2F, 17),
        ("Authority", 0x30, 19),
        ("DataAnalysis", 0x2C, null),
        ("WorkingWithYoungsters", 0x0C, 18),
        ("GoalkeepingDistribution", 0x2A, 19),
        ("GoalkeepingHandling", 0x29, 20),
        ("GoalkeepingReflexes", 0x1B, 13),
    };

    [Fact]
    public void Staff_reader_decodes_all_audited_fields_and_nulls_unread_values()
    {
        const ulong staffBlock = 0x400000UL;
        const ulong person = staffBlock + StaffClassOffset;
        const ulong contract = 0x410000UL;
        const ulong team = 0x420000UL;
        const ulong club = 0x430000UL;
        const ulong competition = 0x440000UL;
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();

        PlaceIdentity(reader, layout, person, "Staff Reader", 1984, 200, nationUid: 208);
        reader.AddBytes(person + 0x70, new byte[] { 20 });
        reader.AddBytes(person + (ulong)layout.GenderOffset, new byte[] { 0x12 });
        PlaceStaffAttributes(reader, layout, staffBlock);
        PlaceTeamAndClub(
            reader,
            layout,
            team,
            club,
            competition,
            teamList: 0x450000,
            clubName: "Staff FC",
            division: "Premier League",
            teamType: 0,
            teamReputation: 6200,
            managerAddress: 0);
        PlaceContract(reader, layout, person, contract, team, weeklyWage: 75_000, expiryYear: 2029, expiryDay: 100, jobId: 16);

        var record = StaffReader.Read(reader, person, staffBlock, uid: 77, ca: 110, pa: 150, layout, out var clubLink);

        Assert.Equal(77u, record.Uid);
        Assert.Equal("Staff Reader", record.Name);
        Assert.Equal(1984, record.BirthYear);
        Assert.Equal(200, record.BirthDayOfYear);
        Assert.Equal(new[] { "DEN" }, record.Nationalities);
        Assert.Equal(208u, record.NationUid);
        Assert.Equal(PlayerGender.Female, record.Gender);
        Assert.Equal(110, record.Ca);
        Assert.Equal(150, record.Pa);
        Assert.Equal(
            ExpectedStaffAttributes.Select(attribute => $"{attribute.Key}:{attribute.Offset:X}"),
            layout.StaffAttributeEntries.Select(entry => $"{entry.Key}:{entry.Offset:X}"));
        foreach (var attribute in ExpectedStaffAttributes)
        {
            Assert.Equal(attribute.Value, record.Attributes[attribute.Key]);
        }
        Assert.Equal(20, record.Attributes["Adaptability"]);
        Assert.Equal(16, record.JobId);
        Assert.Equal(75_000, record.WeeklyWageGbp);
        Assert.Equal(2029, record.ContractExpiryYear);
        Assert.Equal(100, record.ContractExpiryDayOfYear);
        Assert.Equal("Staff FC", record.Club);
        Assert.Equal("Premier League", record.Division);
        Assert.NotNull(clubLink);
        Assert.Equal(6200, clubLink!.TeamReputation);

        var unread = StaffReader.Read(
            new FakeMemoryReader(),
            person,
            staffBlock,
            uid: 78,
            ca: 100,
            pa: 140,
            layout,
            out var unreadClub);
        Assert.Null(unread.Name);
        Assert.Null(unread.BirthYear);
        Assert.Null(unread.NationUid);
        Assert.Equal(PlayerGender.Unknown, unread.Gender);
        Assert.All(unread.Attributes.Values, Assert.Null);
        Assert.Null(unread.JobId);
        Assert.Null(unread.WeeklyWageGbp);
        Assert.Null(unread.Club);
        Assert.Null(unreadClub);
    }

    [Fact]
    public void Staff_attribute_reader_nulls_corrupt_attribute_bytes()
    {
        const ulong staffBlock = 0x400000UL;
        var layout = Fm263Layout.Instance;

        var corruptAttributes = new FakeMemoryReader();
        corruptAttributes.AddBytes(
            staffBlock + (ulong)layout.StaffAttrsOffset + 0x22,
            new[] { byte.MaxValue });

        var attributes = StaffAttributeReader.Read(corruptAttributes, 0, staffBlock, layout);

        Assert.Null(attributes["Attacking"]);
    }

    [Fact]
    public void Staff_contract_reader_nulls_non_leap_year_day_366()
    {
        const ulong person = 0x410000UL;
        const ulong contract = 0x420000UL;
        var layout = Fm263Layout.Instance;

        var impossibleExpiry = new FakeMemoryReader();
        impossibleExpiry.AddBytes(
            person + (ulong)layout.FullContractPtrOffset,
            BitConverter.GetBytes(contract));
        impossibleExpiry.AddBytes(
            contract + (ulong)layout.ContractExpiryOffset,
            BitConverter.GetBytes((2029u << 16) | 366u));

        var contractFields = StaffContractReader.Read(impossibleExpiry, person, layout);

        Assert.Null(contractFields.ContractExpiryYear);
        Assert.Null(contractFields.ContractExpiryDayOfYear);
    }

    [Fact]
    public void Pipeline_keeps_non_player_staff_and_selects_the_first_team_human_manager()
    {
        var bridgeDirectory = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDirectory);
        try
        {
            var result = new CapADumpPipeline().Run(
                BuildPipelineReader(),
                bridgeDirectory,
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success);
            Assert.Equal(1, result.PlayerCount);
            Assert.Equal(new uint[] { 200, 201, 300 }, result.Staff.Select(record => record.Uid));
            Assert.DoesNotContain(result.Staff, record => record.Uid == 10);
            Assert.NotNull(result.Manager);
            Assert.Equal(300u, result.Manager!.Uid);
            Assert.Equal("First Manager", result.Manager.Name);
            Assert.Equal("First FC", result.Manager.Club);
            Assert.Equal(7100, result.Manager.ClubReputation);
            Assert.Equal("First Manager", result.Staff.Single(record => record.Uid == 300).Name);
            Assert.Equal("First FC", result.Staff.Single(record => record.Uid == 300).Club);
            Assert.Equal(36, result.Staff.Single(record => record.Uid == 300).Age);
            Assert.DoesNotContain(typeof(StaffRecord).GetProperties(), property => property.Name.Contains("Address", StringComparison.Ordinal));
            Assert.DoesNotContain(typeof(HumanManager).GetProperties(), property => property.Name.Contains("Address", StringComparison.Ordinal));

            using var dump = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDirectory)));
            Assert.Equal(BridgeProtocol.DumpSchemaVersion, dump.RootElement.GetProperty("schemaVersion").GetInt32());
            Assert.Equal(3, dump.RootElement.GetProperty("staffCount").GetInt32());
            Assert.Equal(
                new uint[] { 200, 201, 300 },
                dump.RootElement.GetProperty("staff").EnumerateArray()
                    .Select(record => record.GetProperty("uid").GetUInt32()));
            var serializedAttributes = dump.RootElement.GetProperty("staff")[0].GetProperty("attributes");
            Assert.Equal(24, serializedAttributes.EnumerateObject().Count());
            Assert.Equal(19, serializedAttributes.GetProperty("Authority").GetInt32());
            Assert.Equal(20, serializedAttributes.GetProperty("Adaptability").GetInt32());
            Assert.Equal("First Manager", dump.RootElement.GetProperty("manager").GetProperty("name").GetString());
            Assert.Equal("First FC", dump.RootElement.GetProperty("manager").GetProperty("club").GetString());
            Assert.Equal(7100, dump.RootElement.GetProperty("manager").GetProperty("clubReputation").GetInt32());
            Assert.DoesNotContain("Address", dump.RootElement.GetRawText(), StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Human_manager_selector_uses_contract_fallback_and_rejects_missing_names()
    {
        var candidate = new PersonCandidate(
            ObjectAddress: 0x5000,
            BlockAddress: 0x4BB0,
            Uid: 77,
            Ca: 100,
            Pa: 140,
            ClassOffset: HumanManagerClassOffset,
            Facet: PersonFacet.HumanManager);
        var contract = new ContractClubLink
        {
            ClubName = "Fallback FC",
            TeamReputation = 6000,
        };
        var staff = new Dictionary<uint, StaffRecord>
        {
            [77] = new StaffRecord { Uid = 77, Name = "Fallback Manager" },
        };
        var links = new Dictionary<uint, ContractClubLink?> { [77] = contract };

        var selected = HumanManagerSelector.Select(
            new[] { candidate },
            staff,
            links,
            new SquadClubIndex());

        Assert.NotNull(selected);
        Assert.Equal("Fallback Manager", selected!.Name);
        Assert.Equal("Fallback FC", selected.Club);
        Assert.Equal(6000, selected.ClubReputation);

        staff[77] = new StaffRecord { Uid = 77, Name = null };
        Assert.Null(HumanManagerSelector.Select(new[] { candidate }, staff, links, new SquadClubIndex()));
    }

    private static FakeMemoryReader BuildPipelineReader()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        reader.AddRegion(
            new MemoryRegion(
                ScanRegionBase,
                ScanRegionSize,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));

        PlaceScannablePlayer(reader, layout, block: 0x200000, uid: 10, slot: 1, name: "Player Wins");
        PlaceScannableStaff(reader, layout, block: 0x204000, StaffClassOffset, uid: 10, slot: 2, name: "Duplicate Staff");
        PlaceScannableStaff(reader, layout, block: 0x208000, StaffClassOffset, uid: 200, slot: 3, name: "Pure Staff");
        PlaceScannableStaff(reader, layout, block: 0x20C000, HumanManagerClassOffset, uid: 201, slot: 4, name: "Reserve Manager");
        PlaceScannableStaff(reader, layout, block: 0x210000, HumanManagerClassOffset, uid: 300, slot: 5, name: "First Manager");

        const ulong reserveTeam = 0x300000;
        const ulong reserveClub = 0x310000;
        const ulong reserveCompetition = 0x320000;
        const ulong firstTeam = 0x330000;
        const ulong firstClub = 0x340000;
        const ulong firstCompetition = 0x350000;
        PlaceTeamAndClub(
            reader,
            layout,
            reserveTeam,
            reserveClub,
            reserveCompetition,
            teamList: 0x360000,
            clubName: "Reserve FC",
            division: "Reserve League",
            teamType: 3,
            teamReputation: 4200,
            managerAddress: 0x20C000UL + HumanManagerClassOffset);
        PlaceTeamAndClub(
            reader,
            layout,
            firstTeam,
            firstClub,
            firstCompetition,
            teamList: 0x370000,
            clubName: "First FC",
            division: "Premier League",
            teamType: 0,
            teamReputation: 7100,
            managerAddress: 0x210000UL + HumanManagerClassOffset);
        PlaceContract(
            reader,
            layout,
            0x20C000UL + HumanManagerClassOffset,
            contract: 0x380000,
            reserveTeam,
            weeklyWage: 50_000,
            expiryYear: 2028,
            expiryDay: 120,
            jobId: 16);
        PlaceContract(
            reader,
            layout,
            0x210000UL + HumanManagerClassOffset,
            contract: 0x390000,
            firstTeam,
            weeklyWage: 80_000,
            expiryYear: 2029,
            expiryDay: 140,
            jobId: 16);

        return reader;
    }

    private static void PlaceScannablePlayer(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong block,
        uint uid,
        int slot,
        string name)
    {
        PlaceScannableObject(reader, layout, block + PlayerClassOffset, PlayerClassOffset, uid, slot);
        var data = new byte[layout.PotentialAbilityOffset + sizeof(ushort)];
        BitConverter.TryWriteBytes(data.AsSpan(layout.CurrentAbilityOffset), (ushort)120);
        BitConverter.TryWriteBytes(data.AsSpan(layout.PotentialAbilityOffset), (ushort)160);
        reader.AddBytes(block, data);
        PlaceIdentity(reader, layout, block + PlayerClassOffset, name, 2000, 100, nationUid: 208);
    }

    private static void PlaceScannableStaff(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong block,
        int classOffset,
        uint uid,
        int slot,
        string name)
    {
        var person = block + (ulong)classOffset;
        PlaceScannableObject(reader, layout, person, classOffset, uid, slot);
        PlaceStaffAttributes(reader, layout, block);
        var data = new byte[layout.StaffPotentialAbilityOffset + sizeof(ushort)];
        BitConverter.TryWriteBytes(data.AsSpan(layout.StaffCurrentAbilityOffset), (ushort)100);
        BitConverter.TryWriteBytes(data.AsSpan(layout.StaffPotentialAbilityOffset), (ushort)140);
        reader.AddBytes(block, data);
        PlaceIdentity(reader, layout, person, name, 1980, 1, nationUid: 208);
        reader.AddBytes(person + 0x70, new byte[] { 20 });
    }

    private static void PlaceScannableObject(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong objectAddress,
        int classOffset,
        uint uid,
        int slot)
    {
        var vtable = GameAssemblyBase + 0x1000UL + (ulong)(slot * 0x100);
        var metadata = GameAssemblyBase + 0x2000UL + (ulong)(slot * 0x100);
        var metadataBytes = new byte[8];
        BitConverter.TryWriteBytes(metadataBytes.AsSpan(4), classOffset);
        reader.AddBytes(metadata, metadataBytes);
        reader.AddBytes(vtable - 8, BitConverter.GetBytes(metadata));

        var header = new byte[0x10];
        BitConverter.TryWriteBytes(header.AsSpan(), vtable);
        BitConverter.TryWriteBytes(header.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(objectAddress, header);
    }

    private static void PlaceIdentity(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong person,
        string name,
        int birthYear,
        int birthDayOfYear,
        uint nationUid)
    {
        var nameAddress = person + 0x10000;
        PlaceNestedString(reader, nameAddress, name);
        reader.AddBytes(person + (ulong)layout.CommonNameOffset, BitConverter.GetBytes(nameAddress));
        reader.AddBytes(
            person + (ulong)layout.DobOffset,
            BitConverter.GetBytes(((uint)birthYear << 16) | (uint)birthDayOfYear));

        var nation = person + 0x11000;
        PlaceIndirectString(reader, nation + 0x100, "DEN");
        reader.AddBytes(nation + (ulong)layout.NationShortNameOffset, BitConverter.GetBytes(nation + 0x100));
        reader.AddBytes(nation + (ulong)layout.ObjectUidOffset, BitConverter.GetBytes(nationUid));
        reader.AddBytes(person + (ulong)layout.NationPtrOffset, BitConverter.GetBytes(nation));
    }

    private static void PlaceStaffAttributes(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong staffBlock)
    {
        var maxOffset = ExpectedStaffAttributes.Max(attribute => attribute.Offset);
        var values = new byte[maxOffset + 1];
        foreach (var attribute in ExpectedStaffAttributes)
        {
            values[attribute.Offset] = attribute.Value is { } value ? (byte)(value * 5) : (byte)0;
        }

        reader.AddBytes(staffBlock + (ulong)layout.StaffAttrsOffset, values);
    }

    private static void PlaceContract(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong person,
        ulong contract,
        ulong team,
        uint weeklyWage,
        int expiryYear,
        int expiryDay,
        byte jobId)
    {
        reader.AddBytes(person + (ulong)layout.FullContractPtrOffset, BitConverter.GetBytes(contract));
        reader.AddBytes(contract + (ulong)layout.ContractTeamPtrOffset, BitConverter.GetBytes(team));
        reader.AddBytes(contract + (ulong)layout.ContractWeeklyWageOffset, BitConverter.GetBytes(weeklyWage));
        reader.AddBytes(
            contract + (ulong)layout.ContractExpiryOffset,
            BitConverter.GetBytes(((uint)expiryYear << 16) | (uint)expiryDay));
        reader.AddBytes(contract + (ulong)layout.ContractJobIdOffset, new[] { jobId });
    }

    private static void PlaceTeamAndClub(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong team,
        ulong club,
        ulong competition,
        ulong teamList,
        string clubName,
        string division,
        byte teamType,
        ushort teamReputation,
        ulong managerAddress)
    {
        PlaceIndirectString(reader, club + 0x100, clubName);
        reader.AddBytes(club + (ulong)layout.ClubNameOffset, BitConverter.GetBytes(club + 0x100));
        reader.AddBytes(club + (ulong)layout.ClubTeamsBeginOffset, BitConverter.GetBytes(teamList));
        reader.AddBytes(club + (ulong)layout.ClubTeamsEndOffset, BitConverter.GetBytes(teamList + 8));
        reader.AddBytes(teamList, BitConverter.GetBytes(team));

        reader.AddBytes(team + (ulong)layout.TeamClubPtrOffset, BitConverter.GetBytes(club));
        reader.AddBytes(team + (ulong)layout.TeamManagerPtrOffset, BitConverter.GetBytes(managerAddress));
        reader.AddBytes(team + (ulong)layout.TeamTypeOffset, new[] { teamType });
        reader.AddBytes(team + (ulong)layout.TeamReputationOffset, BitConverter.GetBytes(teamReputation));
        reader.AddBytes(team + (ulong)layout.TeamCompPtrOffset, BitConverter.GetBytes(competition));
        PlaceIndirectString(reader, competition + 0x100, division);
        reader.AddBytes(competition + (ulong)layout.CompNameOffset, BitConverter.GetBytes(competition + 0x100));
    }

    private static void PlaceNestedString(FakeMemoryReader reader, ulong outerAddress, string value)
    {
        var innerAddress = outerAddress + 0x40;
        reader.AddBytes(outerAddress, BitConverter.GetBytes(innerAddress));
        var bytes = Encoding.UTF8.GetBytes(value + "\0");
        var payload = new byte[4 + bytes.Length];
        bytes.CopyTo(payload, 4);
        reader.AddBytes(innerAddress, payload);
    }

    private static void PlaceIndirectString(FakeMemoryReader reader, ulong address, string value)
    {
        var bytes = Encoding.UTF8.GetBytes(value + "\0");
        var payload = new byte[4 + bytes.Length];
        bytes.CopyTo(payload, 4);
        reader.AddBytes(address, payload);
    }
}
