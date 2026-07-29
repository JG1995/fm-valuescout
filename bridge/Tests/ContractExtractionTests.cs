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

public sealed class ContractExtractionTests
{
    private const ulong GameAssemblyBase = 0x180000000UL;
    private const ulong GameAssemblyEnd = 0x180100000UL;
    private const ulong VtableInAssembly = 0x180001000UL;
    private const ulong MetaInAssembly = 0x180002000UL;
    private const ulong PlayerBlockBase = 0x200000UL;
    private const int PlayerClassOffset = 0x288;
    private static readonly ulong PersonAddress = PlayerBlockBase + (ulong)PlayerClassOffset;
    // Keep near the person block so the pipeline scan region stays small.
    private static readonly ulong ContractAddress = PersonAddress + 0x3000UL;

    [Theory]
    [InlineData(0xFFFFFFFFu, null)]
    [InlineData(0u, 0L)]
    [InlineData(50_000u, 50_000L)]
    public void Money_decode_nulls_unset_sentinel(uint raw, long? expected)
    {
        Assert.Equal(expected, MoneyDecode.TryGbp(raw));
    }

    [Theory]
    [InlineData(0xFFFFFFFFu, null)]
    [InlineData(300_000_000u, null)]
    [InlineData(45_000_000u, 45_000_000L)]
    public void Market_value_nulls_unset_and_unfixed_sentinel(uint raw, long? expected)
    {
        Assert.Equal(expected, MoneyDecode.TryMarketValueGbp(raw));
    }

    [Fact]
    public void Contract_reader_decodes_wage_expiry_flags_value_and_reputation()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();

        PlaceContract(
            reader,
            layout,
            weeklyWage: 85_000,
            expiryYear: 2028,
            expiryDoy: 182,
            statusFlags: (1 << 0) | (1 << 4)); // listed + not for sale
        PlacePlayerValues(reader, layout, marketValue: 40_000_000, curRep: 5400, worldRep: 4200);

        var contract = PlayerContractReader.Read(reader, PersonAddress, PlayerBlockBase, layout);

        Assert.Equal(85_000, contract.WeeklyWageGbp);
        Assert.Equal(2028, contract.ContractExpiryYear);
        Assert.Equal(182, contract.ContractExpiryDayOfYear);
        Assert.True(contract.TransferListed);
        Assert.False(contract.LoanListed);
        Assert.True(contract.NotForSale);
        Assert.False(contract.SetForRelease);
        Assert.Equal(40_000_000, contract.MarketValueGbp);
        Assert.Equal(5400, contract.Reputation.Current);
        Assert.Equal(4200, contract.Reputation.World);
    }

    [Fact]
    public void Contract_reader_nulls_contract_fields_for_free_agent()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        // No contract pointer written
        PlacePlayerValues(reader, layout, marketValue: 1_000_000, curRep: 100, worldRep: 50);

        var contract = PlayerContractReader.Read(reader, PersonAddress, PlayerBlockBase, layout);

        Assert.Null(contract.WeeklyWageGbp);
        Assert.Null(contract.ContractExpiryYear);
        Assert.Null(contract.ContractExpiryDayOfYear);
        Assert.Null(contract.TransferListed);
        Assert.Null(contract.LoanListed);
        Assert.Null(contract.NotForSale);
        Assert.Null(contract.SetForRelease);
        Assert.Equal(1_000_000, contract.MarketValueGbp);
        Assert.Equal(100, contract.Reputation.Current);
        Assert.Equal(50, contract.Reputation.World);
    }

    [Fact]
    public void Contract_reader_treats_listed_by_request_as_transfer_listed()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlaceContract(
            reader,
            layout,
            weeklyWage: 10_000,
            expiryYear: 2027,
            expiryDoy: 1,
            statusFlags: 1 << 3); // listed by request
        PlacePlayerValues(reader, layout, marketValue: 500_000, curRep: 200, worldRep: 100);

        var contract = PlayerContractReader.Read(reader, PersonAddress, PlayerBlockBase, layout);

        Assert.True(contract.TransferListed);
        Assert.False(contract.NotForSale);
        Assert.False(contract.SetForRelease);
    }

    [Fact]
    public void Contract_reader_nulls_impossible_expiry_and_money_sentinels()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlaceContract(
            reader,
            layout,
            weeklyWageRaw: 0xFFFFFFFFu,
            expiryRaw: 0u,
            statusFlags: 1 << 1); // loan listed
        PlacePlayerValues(reader, layout, marketValueRaw: 300_000_000u, curRep: 1, worldRep: 1);

        var contract = PlayerContractReader.Read(reader, PersonAddress, PlayerBlockBase, layout);

        Assert.Null(contract.WeeklyWageGbp);
        Assert.Null(contract.ContractExpiryYear);
        Assert.Null(contract.ContractExpiryDayOfYear);
        Assert.True(contract.LoanListed);
        Assert.False(contract.TransferListed);
        Assert.Null(contract.MarketValueGbp);
    }

    [Fact]
    public void Pipeline_writes_schema_v4_contract_fields_and_null_free_agent()
    {
        var bridgeDir = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDir);
        try
        {
            var layout = Fm263Layout.Instance;
            var reader = new FakeMemoryReader();
            PlaceFullPlayerWithContract(
                reader,
                layout,
                uid: 42,
                ca: 140,
                pa: 170,
                name: "Contract Player",
                weeklyWage: 50_000,
                expiryYear: 2029,
                expiryDoy: 90,
                statusFlags: 1 << 0,
                marketValue: 12_000_000,
                curRep: 3000,
                worldRep: 2500);

            var result = new CapADumpPipeline().Run(
                reader,
                bridgeDir,
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success);
            Assert.Equal(1, result.PlayerCount);

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(4, doc.RootElement.GetProperty("schemaVersion").GetInt32());
            var player = doc.RootElement.GetProperty("players")[0];
            Assert.Equal(50_000, player.GetProperty("weeklyWageGbp").GetInt64());
            Assert.Equal(2029, player.GetProperty("contractExpiryYear").GetInt32());
            Assert.Equal(90, player.GetProperty("contractExpiryDayOfYear").GetInt32());
            Assert.True(player.GetProperty("transferListed").GetBoolean());
            Assert.False(player.GetProperty("loanListed").GetBoolean());
            Assert.False(player.GetProperty("notForSale").GetBoolean());
            Assert.False(player.GetProperty("setForRelease").GetBoolean());
            Assert.Equal(12_000_000, player.GetProperty("marketValueGbp").GetInt64());
            Assert.Equal(3000, player.GetProperty("reputation").GetProperty("current").GetInt32());
            Assert.Equal(2500, player.GetProperty("reputation").GetProperty("world").GetInt32());

            var diagnostics = File.ReadAllText(BridgePaths.GetDiagnosticsPath(bridgeDir));
            Assert.Contains("sampleContracts:", diagnostics, StringComparison.Ordinal);
            Assert.Contains("uid=42", diagnostics, StringComparison.Ordinal);
            Assert.Contains("wage=50000", diagnostics, StringComparison.Ordinal);
            Assert.Contains("contractNull=free agent or unread; money 0xFFFFFFFF/300M → null", diagnostics, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    private static void PlaceContract(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        uint weeklyWage = 0,
        int expiryYear = 2028,
        int expiryDoy = 1,
        byte statusFlags = 0,
        uint? weeklyWageRaw = null,
        uint? expiryRaw = null)
    {
        reader.AddBytes(PersonAddress + (ulong)layout.FullContractPtrOffset, BitConverter.GetBytes(ContractAddress));
        reader.AddBytes(
            ContractAddress + (ulong)layout.ContractWeeklyWageOffset,
            BitConverter.GetBytes(weeklyWageRaw ?? weeklyWage));
        var expiry = expiryRaw ?? (((uint)expiryYear << 16) | (uint)expiryDoy);
        reader.AddBytes(ContractAddress + (ulong)layout.ContractExpiryOffset, BitConverter.GetBytes(expiry));
        reader.AddBytes(ContractAddress + (ulong)layout.ContractStatusFlagsOffset, new[] { statusFlags });
    }

    private static void PlacePlayerValues(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        uint marketValue = 0,
        ushort curRep = 0,
        ushort worldRep = 0,
        uint? marketValueRaw = null)
    {
        var span = Math.Max(layout.PotentialAbilityOffset, layout.MarketValueOffset) + 4;
        span = Math.Max(span, layout.CurrentReputationOffset + 2);
        span = Math.Max(span, layout.WorldReputationOffset + 2);
        var playerBytes = new byte[span];
        BitConverter.TryWriteBytes(
            playerBytes.AsSpan(layout.MarketValueOffset),
            marketValueRaw ?? marketValue);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.CurrentReputationOffset), curRep);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.WorldReputationOffset), worldRep);
        reader.AddBytes(PlayerBlockBase, playerBytes);
    }

    private static void PlaceFullPlayerWithContract(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        uint uid,
        int ca,
        int pa,
        string name,
        uint weeklyWage,
        int expiryYear,
        int expiryDoy,
        byte statusFlags,
        uint marketValue,
        ushort curRep,
        ushort worldRep)
    {
        var regionBase = Math.Min(PlayerBlockBase, PersonAddress);
        var regionEnd = Math.Max(
            PersonAddress + 0x100,
            PlayerBlockBase + (ulong)layout.AttrsOffset + 0x40);
        regionEnd = Math.Max(regionEnd, ContractAddress + 0x80);
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
            reader.AddBytes(PersonAddress + (ulong)entry.Offset, new[] { (byte)10 });
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
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.MarketValueOffset), marketValue);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.CurrentReputationOffset), curRep);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(layout.WorldReputationOffset), worldRep);
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

        reader.AddBytes(PlayerBlockBase, playerBytes);

        var stringBase = PersonAddress + 0x10000;
        PlaceNestedString(reader, stringBase, name);
        reader.AddBytes(PersonAddress + (ulong)layout.CommonNameOffset, BitConverter.GetBytes(stringBase));

        uint dob = (2000u << 16) | 100u;
        reader.AddBytes(PersonAddress + (ulong)layout.DobOffset, BitConverter.GetBytes(dob));

        PlaceContract(reader, layout, weeklyWage, expiryYear, expiryDoy, statusFlags);
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
