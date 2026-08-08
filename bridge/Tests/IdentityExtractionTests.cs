using System.Text;
using System.Text.Json;
using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;
using FmDataBridge.Output;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class IdentityExtractionTests
{
    private const ulong GameAssemblyBase = 0x180000000UL;
    private const ulong GameAssemblyEnd = 0x180100000UL;
    private const ulong VtableInAssembly = 0x180001000UL;
    private const ulong MetaInAssembly = 0x180002000UL;
    private const ulong PlayerBlockBase = 0x200000UL;
    private const int PlayerClassOffset = 0x288;
    private static readonly ulong PersonAddress = PlayerBlockBase + (ulong)PlayerClassOffset;

    [Theory]
    [InlineData(0u)]
    [InlineData(0x07D0_0000u)] // year 2000, doy 0
    [InlineData(0x07D0_016Fu)] // year 2000, doy 367
    [InlineData(0x0752_0001u)] // year 1874
    public void Fm_date_decoder_rejects_impossible_raw(uint raw)
    {
        var (year, doy) = FmDateDecoder.Decode(raw);
        Assert.Equal(0, year);
        Assert.Equal(0, doy);
        Assert.False(FmDateDecoder.IsPlausible(year, doy));
    }

    [Fact]
    public void Fm_date_decoder_accepts_valid_packed_date()
    {
        // year 2005, day-of-year 142 → (2005 << 16) | 142
        uint raw = (2005u << 16) | 142u;
        var (year, doy) = FmDateDecoder.Decode(raw);
        Assert.Equal(2005, year);
        Assert.Equal(142, doy);
        Assert.True(FmDateDecoder.IsPlausible(year, doy));
    }

    [Fact]
    public void Name_reader_prefers_common_name_and_decodes_utf8()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlaceNestedString(reader, 0x3000, "Søren");
        PlaceNestedString(reader, 0x3100, "Anders");
        PlaceNestedString(reader, 0x3200, "Nielsen");

        // person+offsets hold pointers to outer wrappers
        reader.AddBytes(PersonAddress + (ulong)layout.CommonNameOffset, BitConverter.GetBytes(0x3000UL));
        reader.AddBytes(PersonAddress + (ulong)layout.FirstNameOffset, BitConverter.GetBytes(0x3100UL));
        reader.AddBytes(PersonAddress + (ulong)layout.SecondNameOffset, BitConverter.GetBytes(0x3200UL));

        Assert.Equal("Søren", NameReader.TryReadDisplayName(reader, PersonAddress, layout));
    }

    [Fact]
    public void Name_reader_falls_back_to_first_and_second_when_common_missing()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlaceNestedString(reader, 0x3100, "José");
        PlaceNestedString(reader, 0x3200, "García");
        reader.AddBytes(PersonAddress + (ulong)layout.FirstNameOffset, BitConverter.GetBytes(0x3100UL));
        reader.AddBytes(PersonAddress + (ulong)layout.SecondNameOffset, BitConverter.GetBytes(0x3200UL));

        Assert.Equal("José García", NameReader.TryReadDisplayName(reader, PersonAddress, layout));
    }

    [Fact]
    public void Nation_reader_returns_short_name_from_nation_pointer()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        const ulong nationObj = 0x4000;
        PlaceIndirectString(reader, 0x4100, "DEN");
        reader.AddBytes(nationObj + (ulong)layout.NationShortNameOffset, BitConverter.GetBytes(0x4100UL));
        reader.AddBytes(PersonAddress + (ulong)layout.NationPtrOffset, BitConverter.GetBytes(nationObj));

        var nations = NationReader.TryRead(reader, PersonAddress, layout);
        Assert.Equal(new[] { "DEN" }, nations);
    }

    [Fact]
    public void Nation_reader_reads_uid_from_the_nation_object_header()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        const ulong nationObj = 0x4000;
        reader.AddBytes(PersonAddress + (ulong)layout.NationPtrOffset, BitConverter.GetBytes(nationObj));
        reader.AddBytes(nationObj + (ulong)layout.ObjectUidOffset, BitConverter.GetBytes(208u));

        Assert.Equal(208u, NationReader.TryReadUid(reader, PersonAddress, layout));
    }

    [Theory]
    [InlineData(0u)]
    [InlineData(uint.MaxValue)]
    public void Nation_reader_rejects_invalid_object_header_uids(uint uid)
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        const ulong nationObj = 0x4000;
        reader.AddBytes(PersonAddress + (ulong)layout.NationPtrOffset, BitConverter.GetBytes(nationObj));
        reader.AddBytes(nationObj + (ulong)layout.ObjectUidOffset, BitConverter.GetBytes(uid));

        Assert.Null(NationReader.TryReadUid(reader, PersonAddress, layout));
    }

    [Fact]
    public void Identity_reader_rejects_empty_name()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlacePlayerIdentity(
            reader,
            layout,
            name: null,
            birthYear: 2000,
            birthDoy: 100,
            heightCm: 180,
            footLeft: 20,
            footRight: 5,
            positions: new Dictionary<string, int> { ["ST"] = 20 });

        var identity = PlayerIdentityReader.TryRead(
            reader,
            PersonAddress,
            PlayerBlockBase,
            layout,
            out var rejectReason);

        Assert.Null(identity);
        Assert.Equal(IdentityRejectReason.EmptyName, rejectReason);
    }

    [Fact]
    public void Identity_reader_rejects_impossible_dob()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlacePlayerIdentity(
            reader,
            layout,
            name: "Valid Name",
            birthYear: 0,
            birthDoy: 0,
            heightCm: 180,
            footLeft: 5,
            footRight: 18,
            positions: new Dictionary<string, int> { ["MC"] = 18 });

        var identity = PlayerIdentityReader.TryRead(
            reader,
            PersonAddress,
            PlayerBlockBase,
            layout,
            out var rejectReason);

        Assert.Null(identity);
        Assert.Equal(IdentityRejectReason.ImpossibleDob, rejectReason);
    }

    [Fact]
    public void Identity_reader_extracts_name_dob_nation_height_foot_and_natural_positions()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlacePlayerIdentity(
            reader,
            layout,
            name: "Søren Nielsen",
            birthYear: 2005,
            birthDoy: 142,
            heightCm: 186,
            footLeft: 8,
            footRight: 18,
            positions: new Dictionary<string, int>
            {
                ["DC"] = 20,
                ["DM"] = 18,
                ["MC"] = 12,
                ["ST"] = 1,
            },
            nationality: "DEN");

        var identity = PlayerIdentityReader.TryRead(
            reader,
            PersonAddress,
            PlayerBlockBase,
            layout,
            out var rejectReason);

        Assert.Null(rejectReason);
        Assert.NotNull(identity);
        Assert.Equal("Søren Nielsen", identity!.Name);
        Assert.Equal(2005, identity.BirthYear);
        Assert.Equal(142, identity.BirthDayOfYear);
        Assert.Equal(new[] { "DEN" }, identity.Nationalities);
        Assert.Equal(208u, identity.NationUid);
        Assert.Equal(186, identity.HeightCm);
        Assert.Equal("right", identity.PreferredFoot);
        Assert.Equal(20, identity.Positions["DC"]);
        Assert.Equal(18, identity.Positions["DM"]);
        Assert.False(identity.Positions.ContainsKey("MC")); // below natural threshold
        Assert.False(identity.Positions.ContainsKey("ST"));
    }

    [Fact]
    public void Pipeline_writes_schema_v3_identity_fields_and_skips_bad_identity()
    {
        var bridgeDir = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDir);
        try
        {
            var layout = Fm263Layout.Instance;
            var reader = new FakeMemoryReader();

            // Good player
            PlaceFullPlayer(
                reader,
                layout,
                personAddress: PersonAddress,
                playerBlockBase: PlayerBlockBase,
                uid: 42,
                ca: 140,
                pa: 170,
                name: "Good Player",
                birthYear: 1998,
                birthDoy: 50,
                heightCm: 182,
                footLeft: 5,
                footRight: 19,
                positions: new Dictionary<string, int> { ["GK"] = 20 },
                nationality: "ENG");

            // CA/PA-valid person but empty name → identity skip
            var badPerson = PersonAddress + 0x200;
            var badPlayerBase = PlayerBlockBase + 0x200;
            PlaceFullPlayer(
                reader,
                layout,
                personAddress: badPerson,
                playerBlockBase: badPlayerBase,
                uid: 99,
                ca: 100,
                pa: 120,
                name: null,
                birthYear: 2001,
                birthDoy: 10,
                heightCm: 175,
                footLeft: 18,
                footRight: 5,
                positions: new Dictionary<string, int> { ["ST"] = 20 },
                nationality: null);

            var result = new CapADumpPipeline().Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success);
            Assert.Equal(1, result.PlayerCount);

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(5, doc.RootElement.GetProperty("schemaVersion").GetInt32());
            var player = doc.RootElement.GetProperty("players")[0];
            Assert.Equal(42u, player.GetProperty("uid").GetUInt32());
            Assert.Equal("Good Player", player.GetProperty("name").GetString());
            Assert.Equal(1998, player.GetProperty("birthYear").GetInt32());
            Assert.Equal(50, player.GetProperty("birthDayOfYear").GetInt32());
            Assert.Equal("ENG", player.GetProperty("nationalities")[0].GetString());
            Assert.Equal(182, player.GetProperty("heightCm").GetInt32());
            Assert.Equal("right", player.GetProperty("preferredFoot").GetString());
            Assert.Equal(20, player.GetProperty("positions").GetProperty("GK").GetInt32());
            Assert.True(player.TryGetProperty("attributes", out _));
            Assert.True(player.TryGetProperty("hiddenAttributes", out _));
            Assert.True(player.TryGetProperty("personality", out _));
            Assert.True(player.TryGetProperty("weeklyWageGbp", out var wage) && wage.ValueKind == JsonValueKind.Null);
            Assert.True(player.TryGetProperty("reputation", out _));

            var diagnostics = File.ReadAllText(BridgePaths.GetDiagnosticsPath(bridgeDir));
            Assert.Contains("identitySkippedEmptyName=", diagnostics, StringComparison.Ordinal);
            Assert.Contains("identitySkippedEmptyName=1", diagnostics, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    private static void PlaceFullPlayer(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong personAddress,
        ulong playerBlockBase,
        uint uid,
        int ca,
        int pa,
        string? name,
        int birthYear,
        int birthDoy,
        int heightCm,
        int footLeft,
        int footRight,
        IReadOnlyDictionary<string, int> positions,
        string? nationality)
    {
        var regionBase = Math.Min(playerBlockBase, personAddress);
        var regionEnd = Math.Max(
            personAddress + 0x100,
            playerBlockBase + (ulong)layout.HeightOffset + 2);
        reader.AddRegion(
            new MemoryRegion(
                regionBase,
                regionEnd - regionBase,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));

        var metaBytes = new byte[8];
        BitConverter.TryWriteBytes(metaBytes.AsSpan(4), PlayerClassOffset);
        reader.AddBytes(MetaInAssembly, metaBytes);

        var vtableLink = new byte[8];
        BitConverter.TryWriteBytes(vtableLink.AsSpan(), MetaInAssembly);
        reader.AddBytes(VtableInAssembly - 8, vtableLink);

        var personHeader = new byte[0x10];
        BitConverter.TryWriteBytes(personHeader.AsSpan(), VtableInAssembly);
        BitConverter.TryWriteBytes(personHeader.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(personAddress, personHeader);

        var abilitySpan = Math.Max(layout.PotentialAbilityOffset, layout.HeightOffset) + 2;
        abilitySpan = Math.Max(abilitySpan, layout.PositionsOffset + 16);
        abilitySpan = Math.Max(abilitySpan, layout.AttrsOffset + 0x40);
        var playerBytes = new byte[abilitySpan];
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.CurrentAbilityOffset), (ushort)ca);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.PotentialAbilityOffset), (ushort)pa);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.HeightOffset), (ushort)heightCm);
        playerBytes[layout.AttrsOffset + layout.FootLeftAttrOffset] = (byte)(footLeft * 5);
        playerBytes[layout.AttrsOffset + layout.FootRightAttrOffset] = (byte)(footRight * 5);
        foreach (var (key, rating) in positions)
        {
            var off = layout.PositionEntries.First(p => p.Key == key).Offset;
            playerBytes[layout.PositionsOffset + off] = (byte)rating;
        }

        reader.AddBytes(playerBlockBase, playerBytes);

        PlacePlayerIdentityCore(
            reader,
            layout,
            personAddress,
            name,
            birthYear,
            birthDoy,
            nationality);
    }

    private static void PlacePlayerIdentity(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        string? name,
        int birthYear,
        int birthDoy,
        int heightCm,
        int footLeft,
        int footRight,
        IReadOnlyDictionary<string, int> positions,
        string? nationality = "DEN")
    {
        var abilitySpan = Math.Max(layout.HeightOffset, layout.PositionsOffset + 16);
        abilitySpan = Math.Max(abilitySpan, layout.AttrsOffset + 0x40) + 2;
        var playerBytes = new byte[abilitySpan];
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.HeightOffset), (ushort)heightCm);
        playerBytes[layout.AttrsOffset + layout.FootLeftAttrOffset] = (byte)(footLeft * 5);
        playerBytes[layout.AttrsOffset + layout.FootRightAttrOffset] = (byte)(footRight * 5);
        foreach (var (key, rating) in positions)
        {
            var off = layout.PositionEntries.First(p => p.Key == key).Offset;
            playerBytes[layout.PositionsOffset + off] = (byte)rating;
        }

        reader.AddBytes(PlayerBlockBase, playerBytes);
        PlacePlayerIdentityCore(reader, layout, PersonAddress, name, birthYear, birthDoy, nationality);
    }

    private static void PlacePlayerIdentityCore(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong personAddress,
        string? name,
        int birthYear,
        int birthDoy,
        string? nationality)
    {
        if (!string.IsNullOrEmpty(name))
        {
            var stringBase = personAddress + 0x10000;
            PlaceNestedString(reader, stringBase, name);
            reader.AddBytes(personAddress + (ulong)layout.CommonNameOffset, BitConverter.GetBytes(stringBase));
        }

        uint dob = birthYear == 0 && birthDoy == 0
            ? 0u
            : ((uint)birthYear << 16) | (uint)birthDoy;
        reader.AddBytes(personAddress + (ulong)layout.DobOffset, BitConverter.GetBytes(dob));

        if (!string.IsNullOrEmpty(nationality))
        {
            var nationObj = personAddress + 0x20000;
            var nationStr = personAddress + 0x21000;
            PlaceIndirectString(reader, nationStr, nationality);
            reader.AddBytes(nationObj + (ulong)layout.NationShortNameOffset, BitConverter.GetBytes(nationStr));
            reader.AddBytes(nationObj + (ulong)layout.ObjectUidOffset, BitConverter.GetBytes(208u));
            reader.AddBytes(personAddress + (ulong)layout.NationPtrOffset, BitConverter.GetBytes(nationObj));
        }
    }

    /// <summary>
    /// Nested FM string: [slot]→outer, [outer]→inner, UTF-8 at inner+4.
    /// PlaceNestedString writes the outer wrapper at <paramref name="outerAddress"/>.
    /// </summary>
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
