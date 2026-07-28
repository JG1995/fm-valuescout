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
        Assert.Equal(0x264, layout.CurrentAbilityOffset);
        Assert.Equal(0x266, layout.PotentialAbilityOffset);
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
                SchemaVersion = 1,
                GeneratedAtUtc = "2026-07-28T00:00:00Z",
                GameVersion = "26.3.0",
                SupportedGameVersion = "26.3",
                BridgeVersion = "0.1.0",
                ProtocolVersion = 1,
                PlayerCount = 1,
                Players = new[] { new DumpPlayer { Uid = 1, Ca = 10, Pa = 20 } },
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
                SchemaVersion = 1,
                GeneratedAtUtc = "2026-07-28T00:00:00Z",
                GameVersion = "26.3.0",
                SupportedGameVersion = "26.3",
                BridgeVersion = "0.1.0",
                ProtocolVersion = 1,
                PlayerCount = 1,
                Players = new[] { new DumpPlayer { Uid = 42, Ca = 100, Pa = 150 } },
            };
            Assert.True(DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, prior));

            var empty = new DumpDocument
            {
                SchemaVersion = 1,
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
            PlayerCount = 1,
            Players = new[] { new DumpPlayer { Uid = 7, Ca = 120, Pa = 160 } },
        };

        var json = DumpWriter.Serialize(document);
        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;
        Assert.Equal(1, root.GetProperty("schemaVersion").GetInt32());
        Assert.Equal("26.3.2.2329565", root.GetProperty("gameVersion").GetString());
        Assert.Equal("26.3", root.GetProperty("supportedGameVersion").GetString());
        Assert.Equal("0.1.0", root.GetProperty("bridgeVersion").GetString());
        Assert.Equal(1, root.GetProperty("protocolVersion").GetInt32());
        Assert.Equal(1, root.GetProperty("playerCount").GetInt32());
        Assert.Equal(7u, root.GetProperty("players")[0].GetProperty("uid").GetUInt32());
        Assert.Equal(120, root.GetProperty("players")[0].GetProperty("ca").GetInt32());
        Assert.Equal(160, root.GetProperty("players")[0].GetProperty("pa").GetInt32());
    }

    [Fact]
    public void Person_scanner_accepts_valid_uid_ca_pa_and_dedupes()
    {
        var layout = Fm263Layout.Instance;
        var reader = BuildReaderWithTwoIdenticalPlayers(layout);
        var regions = RegionEnumerator.GetCandidateRegions(reader);
        var diagnostics = new ScanDiagnostics();
        var gameAssembly = new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd);

        var candidates = PersonScanner.Scan(
            reader,
            layout,
            gameAssembly,
            gamePlugin: null,
            regions,
            diagnostics);

        Assert.Single(candidates);
        Assert.Equal(12345u, candidates[0].Uid);
        Assert.Equal(150, candidates[0].Ca);
        Assert.Equal(170, candidates[0].Pa);
        Assert.True(diagnostics.DuplicatesSkipped >= 1);
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

        var candidates = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            new ScanDiagnostics());

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
        var candidates = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics);

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
        var candidates = PersonScanner.Scan(
            reader,
            layout,
            new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
            gamePlugin: null,
            RegionEnumerator.GetCandidateRegions(reader),
            diagnostics,
            maxAccepted: 2);

        Assert.Equal(2, candidates.Count);
        Assert.True(diagnostics.StoppedEarly);
        Assert.Equal(2, diagnostics.MaxAccepted);
        Assert.Equal(2, diagnostics.CandidatesAccepted);
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

            using var doc = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir)));
            Assert.Equal(1, doc.RootElement.GetProperty("playerCount").GetInt32());
            Assert.True(File.Exists(BridgePaths.GetDiagnosticsPath(bridgeDir)));
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
                SchemaVersion = 1,
                GeneratedAtUtc = "2026-07-28T00:00:00Z",
                GameVersion = "26.3.0",
                SupportedGameVersion = "26.3",
                BridgeVersion = "0.1.0",
                ProtocolVersion = 1,
                PlayerCount = 1,
                Players = new[] { new DumpPlayer { Uid = 9, Ca = 11, Pa = 12 } },
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
        PlacePlayerFixture(reader, layout, PersonAddress, uid: 12345, ca: 150, pa: 170);
        // Second person object with the same UID (dedupe).
        PlacePlayerFixture(
            reader,
            layout,
            PersonAddress + 0x80,
            uid: 12345,
            ca: 150,
            pa: 170,
            playerBlockBase: PlayerBlockBase + 0x80);
        return reader;
    }

    private static void PlacePlayerFixture(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong personAddress,
        uint uid,
        int ca,
        int pa,
        ulong? playerBlockBase = null)
    {
        var playerBase = playerBlockBase ?? (personAddress - (ulong)PlayerClassOffset);
        var regionBase = Math.Min(playerBase, personAddress);
        var regionEnd = Math.Max(personAddress + 0x10, playerBase + (ulong)layout.PotentialAbilityOffset + 2);
        AddCandidateRegion(reader, regionBase, regionEnd - regionBase);

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

        var abilitySpan = layout.PotentialAbilityOffset + sizeof(ushort);
        var playerBytes = new byte[abilitySpan];
        WriteUInt16(playerBytes, layout.CurrentAbilityOffset, (ushort)ca);
        WriteUInt16(playerBytes, layout.PotentialAbilityOffset, (ushort)pa);
        reader.AddBytes(playerBase, playerBytes);
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
}
