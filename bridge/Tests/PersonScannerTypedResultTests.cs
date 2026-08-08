using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class PersonScannerTypedResultTests
{
    private const ulong GameAssemblyBase = 0x180000000UL;
    private const ulong GameAssemblyEnd = 0x180100000UL;
    private const ulong RegionBase = 0x100000UL;
    private const ulong RegionSize = 0x20000UL;
    private const int PurePlayerClassOffset = 0x288;
    private const int PlayerStaffClassOffset = 0x380;
    private const int PureStaffClassOffset = 0x100;
    private const int HumanManagerClassOffset = 0x450;

    [Fact]
    public void Person_scanner_returns_deterministic_typed_people_and_keeps_player_facets()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        reader.AddRegion(
            new MemoryRegion(
                RegionBase,
                RegionSize,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));

        PlacePlayer(reader, layout, 0x101000, PurePlayerClassOffset, uid: 101, ca: 120, pa: 160, slot: 1);
        PlacePlayer(reader, layout, 0x103000, PlayerStaffClassOffset, uid: 102, ca: 130, pa: 170, slot: 2);
        PlaceStaff(reader, layout, 0x105000, PureStaffClassOffset, uid: 102, ca: 130, pa: 170, slot: 3);
        PlaceStaff(reader, layout, 0x107000, PureStaffClassOffset, uid: 201, ca: 100, pa: 140, slot: 4);
        PlaceStaff(reader, layout, 0x109000, HumanManagerClassOffset, uid: 301, ca: 110, pa: 150, slot: 5);
        PlaceStaff(reader, layout, 0x10B000, PureStaffClassOffset, uid: 401, ca: 0, pa: 150, slot: 6);
        PlaceObject(reader, layout, 0x10D000, 0x180, uid: 501, slot: 7);
        PlaceStaff(reader, layout, 0x10F000, HumanManagerClassOffset, uid: 250, ca: 115, pa: 155, slot: 8);

        var diagnostics = new ScanDiagnostics();
        var result = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics);

        Assert.Equal(new uint[] { 101, 102 }, result.Players.Select(candidate => candidate.Uid));
        Assert.Equal(new uint[] { 201, 250, 301 }, result.Staff.Select(candidate => candidate.Uid));
        Assert.Equal(new uint[] { 250, 301 }, result.HumanManagers.Select(candidate => candidate.Uid));
        Assert.Equal(new uint[] { 102 }, result.PlayerStaffOverlapUids);
        Assert.Equal(0x101000UL, result.Players[0].BlockAddress);
        Assert.Equal(PersonFacet.Player, result.Players[0].Facet);
        Assert.Equal(PersonFacet.HumanManager, result.Staff[1].Facet);
        Assert.Equal(2, diagnostics.CandidatesAccepted);
        Assert.Equal(4, diagnostics.StaffCandidatesAccepted);
        Assert.Equal(2, diagnostics.HumanManagerCandidatesAccepted);
        Assert.Equal(1, diagnostics.PlayerStaffOverlapCount);
        Assert.True(diagnostics.ClassOffsetHistogram.ContainsKey(PurePlayerClassOffset));
        Assert.True(diagnostics.ClassOffsetHistogram.ContainsKey(PlayerStaffClassOffset));
        Assert.True(diagnostics.ClassOffsetHistogram.ContainsKey(PureStaffClassOffset));
        Assert.True(diagnostics.ClassOffsetHistogram.ContainsKey(HumanManagerClassOffset));
    }

    [Fact]
    public void Person_scanner_orders_by_uid_and_keeps_the_lowest_same_facet_address()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlacePlayer(reader, layout, 0x310000, PurePlayerClassOffset, uid: 20, ca: 120, pa: 160, slot: 1);
        PlacePlayer(reader, layout, 0x210000, PurePlayerClassOffset, uid: 10, ca: 120, pa: 160, slot: 2);
        PlacePlayer(reader, layout, 0x110000, PurePlayerClassOffset, uid: 20, ca: 120, pa: 160, slot: 3);

        var result = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            new[]
            {
                CandidateRegion(0x310000),
                CandidateRegion(0x210000),
                CandidateRegion(0x110000),
            },
            new ScanDiagnostics());

        Assert.Equal(new uint[] { 10, 20 }, result.Players.Select(candidate => candidate.Uid));
        Assert.Equal(0x110000UL, result.Players[1].BlockAddress);
    }

    [Fact]
    public void Person_scanner_rejects_wrapped_dynamic_metadata_address()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        reader.AddRegion(CandidateRegion(RegionBase));
        PlacePlayer(
            reader,
            layout,
            0x101000,
            PurePlayerClassOffset,
            uid: 101,
            ca: 120,
            pa: 160,
            slot: 1,
            metadataAddress: ulong.MaxValue - 2);
        reader.AddBytes(1, BitConverter.GetBytes(PurePlayerClassOffset));

        Assert.False(PersonScanner.TryResolveDynamicOffset(reader, GameAssemblyBase + 0x1100, out _));

        var result = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            new ScanDiagnostics());

        Assert.Empty(result.Players);
    }

    [Fact]
    public void Person_scanner_filters_only_players_by_the_requested_database_scope()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        reader.AddRegion(
            new MemoryRegion(
                RegionBase,
                RegionSize,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));

        PlacePlayer(
            reader,
            layout,
            0x101000,
            PurePlayerClassOffset,
            uid: 101,
            ca: 120,
            pa: 160,
            slot: 1,
            gender: 0x02);
        PlacePlayer(
            reader,
            layout,
            0x103000,
            PurePlayerClassOffset,
            uid: 102,
            ca: 120,
            pa: 160,
            slot: 2,
            gender: 0x12);
        PlacePlayer(
            reader,
            layout,
            0x105000,
            PurePlayerClassOffset,
            uid: 103,
            ca: 120,
            pa: 160,
            slot: 3);
        PlaceStaff(reader, layout, 0x107000, PureStaffClassOffset, uid: 201, ca: 100, pa: 140, slot: 4);
        reader.AddBytes(
            0x107000UL + (ulong)PureStaffClassOffset + (ulong)layout.GenderOffset,
            new byte[] { 0x12 });

        var gameAssembly = new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd);
        var regions = RegionEnumerator.GetCandidateRegions(reader);

        var menDiagnostics = new ScanDiagnostics();
        var men = PersonScanner.Scan(
            reader,
            layout,
            gameAssembly,
            gamePlugin: null,
            regions,
            menDiagnostics,
            playerDatabaseScope: PlayerDatabaseScope.Men);
        Assert.Equal(new uint[] { 101, 103 }, men.Players.Select(candidate => candidate.Uid));
        Assert.Equal(new uint[] { 201 }, men.Staff.Select(candidate => candidate.Uid));
        Assert.Equal(1, menDiagnostics.PlayersExcludedByDatabaseScope);

        var womenDiagnostics = new ScanDiagnostics();
        var women = PersonScanner.Scan(
            reader,
            layout,
            gameAssembly,
            gamePlugin: null,
            regions,
            womenDiagnostics,
            playerDatabaseScope: PlayerDatabaseScope.Women);
        Assert.Equal(new uint[] { 102 }, women.Players.Select(candidate => candidate.Uid));
        Assert.Equal(new uint[] { 201 }, women.Staff.Select(candidate => candidate.Uid));
        Assert.Equal(2, womenDiagnostics.PlayersExcludedByDatabaseScope);

        var bothDiagnostics = new ScanDiagnostics();
        var both = PersonScanner.Scan(
            reader,
            layout,
            gameAssembly,
            gamePlugin: null,
            regions,
            bothDiagnostics,
            playerDatabaseScope: PlayerDatabaseScope.Both);
        Assert.Equal(new uint[] { 101, 102, 103 }, both.Players.Select(candidate => candidate.Uid));
        Assert.Equal(new uint[] { 201 }, both.Staff.Select(candidate => candidate.Uid));
        Assert.Equal(0, bothDiagnostics.PlayersExcludedByDatabaseScope);
    }

    private static void PlacePlayer(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong blockAddress,
        int classOffset,
        uint uid,
        int ca,
        int pa,
        int slot,
        ulong? metadataAddress = null,
        byte? gender = null)
    {
        PlaceObject(reader, layout, blockAddress, classOffset, uid, slot, metadataAddress);
        var bytes = new byte[Math.Max(layout.CurrentAbilityOffset, layout.PotentialAbilityOffset) + sizeof(ushort)];
        BitConverter.TryWriteBytes(bytes.AsSpan(layout.CurrentAbilityOffset), (ushort)ca);
        BitConverter.TryWriteBytes(bytes.AsSpan(layout.PotentialAbilityOffset), (ushort)pa);
        reader.AddBytes(blockAddress, bytes);
        if (gender is { } value)
        {
            reader.AddBytes(
                blockAddress + (ulong)classOffset + (ulong)layout.GenderOffset,
                new[] { value });
        }
    }

    private static void PlaceStaff(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong blockAddress,
        int classOffset,
        uint uid,
        int ca,
        int pa,
        int slot)
    {
        PlaceObject(reader, layout, blockAddress, classOffset, uid, slot);
        var bytes = new byte[layout.StaffPotentialAbilityOffset + sizeof(ushort)];
        BitConverter.TryWriteBytes(bytes.AsSpan(layout.StaffCurrentAbilityOffset), (ushort)ca);
        BitConverter.TryWriteBytes(bytes.AsSpan(layout.StaffPotentialAbilityOffset), (ushort)pa);
        reader.AddBytes(blockAddress, bytes);
    }

    private static void PlaceObject(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong blockAddress,
        int classOffset,
        uint uid,
        int slot,
        ulong? metadataAddress = null)
    {
        var objectAddress = blockAddress + (ulong)classOffset;
        var vtable = GameAssemblyBase + 0x1000UL + (ulong)(slot * 0x100);
        var meta = metadataAddress ?? GameAssemblyBase + 0x2000UL + (ulong)(slot * 0x100);
        if (metadataAddress is null)
        {
            var metaBytes = new byte[8];
            BitConverter.TryWriteBytes(metaBytes.AsSpan(4), classOffset);
            reader.AddBytes(meta, metaBytes);
        }

        reader.AddBytes(vtable - 8, BitConverter.GetBytes(meta));

        var header = new byte[0x10];
        BitConverter.TryWriteBytes(header.AsSpan(), vtable);
        BitConverter.TryWriteBytes(header.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(objectAddress, header);
    }

    private static MemoryRegion CandidateRegion(ulong baseAddress) => new(
        baseAddress,
        0x1000,
        MemoryConstants.PageReadWrite,
        MemoryConstants.MemPrivate,
        MemoryConstants.MemCommit);
}
