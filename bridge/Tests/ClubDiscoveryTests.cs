using System.Text;
using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class ClubDiscoveryTests
{
    private const ulong GameAssemblyBase = 0x180000000UL;
    private const ulong GameAssemblyEnd = 0x180100000UL;
    private const ulong RegionBase = 0x100000UL;
    private const ulong RegionSize = 0x80000UL;
    private const ulong ValidClub = 0x120000UL;
    private const ulong MisalignedClub = 0x130000UL;
    private const ulong ImplausibleClub = 0x140000UL;
    private const ulong OversizedClub = 0x150000UL;

    [Fact]
    public void Person_scanner_discovers_only_structurally_valid_clubs()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        var region = new MemoryRegion(
            RegionBase,
            RegionSize,
            MemoryConstants.PageReadWrite,
            MemoryConstants.MemPrivate,
            MemoryConstants.MemCommit);

        PlaceScannableObject(reader, layout, ValidClub, uid: 1001, slot: 1);
        PlaceClub(reader, layout, ValidClub, "Global FC", teamVector: 0x160000, teamCount: 1);

        PlaceScannableObject(reader, layout, MisalignedClub, uid: 1002, slot: 2);
        PlaceClub(
            reader,
            layout,
            MisalignedClub,
            "Broken FC",
            teamVector: 0x161001,
            teamCount: 1,
            teamsEnd: 0x161009);

        PlaceScannableObject(reader, layout, ImplausibleClub, uid: 1003, slot: 3);
        PlaceClub(reader, layout, ImplausibleClub, "!", teamVector: 0x162000, teamCount: 1);

        PlaceScannableObject(reader, layout, OversizedClub, uid: 1004, slot: 4);
        PlaceClub(
            reader,
            layout,
            OversizedClub,
            "Too Many FC",
            teamVector: 0x163000,
            teamCount: SquadClubIndex.MaxTeamsPerClub + 1);

        var diagnostics = new ScanDiagnostics();
        var scan = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            new[] { region, region },
            diagnostics);

        var club = Assert.Single(scan.Clubs);

        Assert.Equal(ValidClub, club.Address);
        Assert.Equal("Global FC", club.Name);
        Assert.Equal(1, diagnostics.ClubCandidatesAccepted);
        Assert.Equal(1, diagnostics.ClubCandidateDuplicatesSkipped);
        Assert.True(diagnostics.ClubCandidatesRejected >= 3);
    }

    [Fact]
    public void Club_name_reader_rejects_wrapped_field_address()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        var wrappedNameSlot = unchecked(ulong.MaxValue + (ulong)layout.ClubNameOffset);
        const ulong nameObject = 0x170000UL;
        reader.AddBytes(wrappedNameSlot, BitConverter.GetBytes(nameObject));
        var utf8 = Encoding.UTF8.GetBytes("Wrapped FC\0");
        var payload = new byte[sizeof(uint) + utf8.Length];
        utf8.CopyTo(payload, sizeof(uint));
        reader.AddBytes(nameObject, payload);

        Assert.Null(ClubNameReader.TryRead(reader, ulong.MaxValue, layout));
    }

    private static void PlaceScannableObject(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong address,
        uint uid,
        int slot)
    {
        var vtable = GameAssemblyBase + 0x1000UL + (ulong)(slot * 0x100);
        var metadata = GameAssemblyBase + 0x2000UL + (ulong)(slot * 0x100);
        var metadataBytes = new byte[8];
        BitConverter.TryWriteBytes(metadataBytes.AsSpan(4), 0x600);
        reader.AddBytes(metadata, metadataBytes);
        reader.AddBytes(vtable - 8, BitConverter.GetBytes(metadata));

        var header = new byte[0x10];
        BitConverter.TryWriteBytes(header.AsSpan(), vtable);
        BitConverter.TryWriteBytes(header.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(address, header);
    }

    private static void PlaceClub(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong address,
        string name,
        ulong teamVector,
        int teamCount,
        ulong? teamsEnd = null)
    {
        reader.AddBytes(address + (ulong)layout.ClubTeamsBeginOffset, BitConverter.GetBytes(teamVector));
        reader.AddBytes(
            address + (ulong)layout.ClubTeamsEndOffset,
            BitConverter.GetBytes(teamsEnd ?? teamVector + (ulong)(teamCount * sizeof(ulong))));
        reader.AddBytes(teamVector, BitConverter.GetBytes(0x160000UL));

        var nameObject = address + 0x200;
        reader.AddBytes(address + (ulong)layout.ClubNameOffset, BitConverter.GetBytes(nameObject));
        var utf8 = Encoding.UTF8.GetBytes(name + "\0");
        var payload = new byte[sizeof(uint) + utf8.Length];
        utf8.CopyTo(payload, sizeof(uint));
        reader.AddBytes(nameObject, payload);
    }
}
