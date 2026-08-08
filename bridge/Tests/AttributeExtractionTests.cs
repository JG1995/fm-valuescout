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

public sealed class AttributeExtractionTests
{
    private const ulong GameAssemblyBase = 0x180000000UL;
    private const ulong GameAssemblyEnd = 0x180100000UL;
    private const ulong VtableInAssembly = 0x180001000UL;
    private const ulong MetaInAssembly = 0x180002000UL;
    private const ulong PlayerBlockBase = 0x200000UL;
    private const int PlayerClassOffset = 0x288;
    private static readonly ulong PersonAddress = PlayerBlockBase + (ulong)PlayerClassOffset;

    [Theory]
    [InlineData(0, null)] // decode → 0 → outside 1–20
    [InlineData(5, 1)]
    [InlineData(65, 13)] // Acceleration fixture: 13 on 1–20
    [InlineData(67, 13)] // rounds to nearest
    [InlineData(68, 14)]
    [InlineData(100, 20)]
    [InlineData(255, 20)] // clamp to 20 stays in range
    public void Attribute_scale_try_decode_scaled_null_outside_one_through_twenty(byte raw, int? expected)
    {
        Assert.Equal(expected, AttributeScale.TryDecodeScaled(raw));
    }

    [Theory]
    [InlineData(0, 0)]
    [InlineData(5, 1)]
    [InlineData(65, 13)]
    [InlineData(100, 20)]
    [InlineData(255, 20)]
    public void Attribute_scale_decode_scaled_keeps_zero_for_comparisons(byte raw, int expected)
    {
        Assert.Equal(expected, AttributeScale.DecodeScaled(raw));
    }

    [Theory]
    [InlineData(0, null)]
    [InlineData(1, 1)]
    [InlineData(16, 16)]
    [InlineData(20, 20)]
    [InlineData(21, null)]
    [InlineData(255, null)]
    public void Attribute_scale_personality_null_outside_one_through_twenty(byte raw, int? expected)
    {
        Assert.Equal(expected, AttributeScale.TryPersonality(raw));
    }

    [Fact]
    public void Attribute_reader_uses_null_for_unread_and_invalid_values()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        // Only write one visible attr; others unread → null
        WriteScaledAttr(reader, layout, "Acceleration", 13);
        // Personality out of range
        WritePersonality(reader, layout, "Ambition", 0);
        WritePersonality(reader, layout, "Loyalty", 21);

        var attrs = PlayerAttributeReader.Read(reader, PersonAddress, PlayerBlockBase, layout);

        Assert.Equal(13, attrs.Attributes["Acceleration"]);
        Assert.Null(attrs.Attributes["Pace"]);
        Assert.Null(attrs.Personality["Ambition"]);
        Assert.Null(attrs.Personality["Loyalty"]);
        Assert.Null(attrs.HiddenAttributes["Consistency"]);
    }

    [Fact]
    public void Attribute_reader_decodes_visible_hidden_and_personality_groups()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();

        // Visible: Acceleration=13, Pace=14, Passing=11 (stored ×5)
        WriteScaledAttr(reader, layout, "Acceleration", 13);
        WriteScaledAttr(reader, layout, "Pace", 14);
        WriteScaledAttr(reader, layout, "Passing", 11);

        // Hidden: Consistency=12, ImportantMatches=14, InjuryProneness=7
        WriteScaledAttr(reader, layout, "Consistency", 12, hidden: true);
        WriteScaledAttr(reader, layout, "ImportantMatches", 14, hidden: true);
        WriteScaledAttr(reader, layout, "InjuryProneness", 7, hidden: true);

        // Personality: Ambition=16, Professionalism=15, Pressure=13, Loyalty=10 (raw 1–20)
        WritePersonality(reader, layout, "Ambition", 16);
        WritePersonality(reader, layout, "Professionalism", 15);
        WritePersonality(reader, layout, "Pressure", 13);
        WritePersonality(reader, layout, "Loyalty", 10);

        var attrs = PlayerAttributeReader.Read(reader, PersonAddress, PlayerBlockBase, layout);

        Assert.Equal(13, attrs.Attributes["Acceleration"]);
        Assert.Equal(14, attrs.Attributes["Pace"]);
        Assert.Equal(11, attrs.Attributes["Passing"]);
        Assert.True(attrs.Attributes.ContainsKey("Crossing")); // stable key set

        Assert.Equal(12, attrs.HiddenAttributes["Consistency"]);
        Assert.Equal(14, attrs.HiddenAttributes["ImportantMatches"]);
        Assert.Equal(7, attrs.HiddenAttributes["InjuryProneness"]);

        Assert.Equal(16, attrs.Personality["Ambition"]);
        Assert.Equal(15, attrs.Personality["Professionalism"]);
        Assert.Equal(13, attrs.Personality["Pressure"]);
        Assert.Equal(10, attrs.Personality["Loyalty"]);
        Assert.True(attrs.Personality.ContainsKey("Adaptability"));
    }

    [Fact]
    public void Pipeline_writes_schema_v3_attribute_maps_and_sample_diagnostics()
    {
        var bridgeDir = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDir);
        try
        {
            var layout = Fm263Layout.Instance;
            var reader = new FakeMemoryReader();
            PlaceFullPlayerWithAttrs(
                reader,
                layout,
                uid: 42,
                ca: 140,
                pa: 170,
                name: "Attr Player",
                acceleration: 13,
                pace: 14,
                consistency: 12,
                ambition: 16);

            var result = new CapADumpPipeline().Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success);
            Assert.Equal(1, result.PlayerCount);

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(BridgeProtocol.DumpSchemaVersion, doc.RootElement.GetProperty("schemaVersion").GetInt32());
            var player = doc.RootElement.GetProperty("players")[0];
            Assert.Equal(13, player.GetProperty("attributes").GetProperty("Acceleration").GetInt32());
            Assert.Equal(14, player.GetProperty("attributes").GetProperty("Pace").GetInt32());
            Assert.Equal(12, player.GetProperty("hiddenAttributes").GetProperty("Consistency").GetInt32());
            Assert.Equal(16, player.GetProperty("personality").GetProperty("Ambition").GetInt32());
            Assert.True(player.TryGetProperty("weeklyWageGbp", out _));
            Assert.True(player.TryGetProperty("marketValueGbp", out _));
            Assert.True(player.TryGetProperty("reputation", out _));

            var diagnostics = File.ReadAllText(BridgePaths.GetDiagnosticsPath(bridgeDir));
            Assert.Contains("sampleAttributes:", diagnostics, StringComparison.Ordinal);
            Assert.Contains("uid=42", diagnostics, StringComparison.Ordinal);
            Assert.Contains("Acceleration=13", diagnostics, StringComparison.Ordinal);
            Assert.Contains("attrsStoredTimesFive=decode floor(raw/5+0.5); null if unread or outside 1..20", diagnostics, StringComparison.Ordinal);
            Assert.Contains("personalityRaw=1..20 or null", diagnostics, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    private static void WriteScaledAttr(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        string key,
        int value,
        bool hidden = false)
    {
        var entries = hidden ? layout.HiddenAttributeEntries : layout.AttributeEntries;
        var off = entries.First(e => e.Key == key).Offset;
        reader.AddBytes(
            PlayerBlockBase + (ulong)layout.AttrsOffset + (ulong)off,
            new[] { (byte)(value * 5) });
    }

    private static void WritePersonality(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        string key,
        int value)
    {
        var off = layout.PersonalityEntries.First(e => e.Key == key).Offset;
        reader.AddBytes(PersonAddress + (ulong)off, new[] { (byte)value });
    }

    private static void PlaceFullPlayerWithAttrs(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        uint uid,
        int ca,
        int pa,
        string name,
        int acceleration,
        int pace,
        int consistency,
        int ambition)
    {
        var regionBase = Math.Min(PlayerBlockBase, PersonAddress);
        var regionEnd = Math.Max(
            PersonAddress + 0x100,
            PlayerBlockBase + (ulong)layout.AttrsOffset + 0x40);
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
        reader.AddBytes(PersonAddress, personHeader);

        foreach (var entry in layout.PersonalityEntries)
        {
            var v = entry.Key == "Ambition" ? ambition : 10;
            reader.AddBytes(PersonAddress + (ulong)entry.Offset, new[] { (byte)v });
        }

        var abilitySpan = Math.Max(layout.PotentialAbilityOffset, layout.HeightOffset) + 2;
        abilitySpan = Math.Max(abilitySpan, layout.PositionsOffset + 16);
        abilitySpan = Math.Max(abilitySpan, layout.AttrsOffset + 0x40);
        var playerBytes = new byte[abilitySpan];
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.CurrentAbilityOffset), (ushort)ca);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.PotentialAbilityOffset), (ushort)pa);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.HeightOffset), (ushort)180);
        playerBytes[layout.AttrsOffset + layout.FootLeftAttrOffset] = 25;
        playerBytes[layout.AttrsOffset + layout.FootRightAttrOffset] = 90;
        playerBytes[layout.PositionsOffset + layout.PositionEntries.First(p => p.Key == "ST").Offset] = 20;

        foreach (var entry in layout.AttributeEntries)
        {
            var v = entry.Key switch
            {
                "Acceleration" => acceleration,
                "Pace" => pace,
                _ => 10,
            };
            playerBytes[layout.AttrsOffset + entry.Offset] = (byte)(v * 5);
        }

        foreach (var entry in layout.HiddenAttributeEntries)
        {
            var v = entry.Key == "Consistency" ? consistency : 8;
            playerBytes[layout.AttrsOffset + entry.Offset] = (byte)(v * 5);
        }

        reader.AddBytes(PlayerBlockBase, playerBytes);

        var stringBase = PersonAddress + 0x10000;
        PlaceNestedString(reader, stringBase, name);
        reader.AddBytes(PersonAddress + (ulong)layout.CommonNameOffset, BitConverter.GetBytes(stringBase));

        uint dob = (2000u << 16) | 100u;
        reader.AddBytes(PersonAddress + (ulong)layout.DobOffset, BitConverter.GetBytes(dob));
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
}
