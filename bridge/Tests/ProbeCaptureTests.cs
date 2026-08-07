using System.Buffers.Binary;
using System.Text;
using System.Text.Json;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;
using FmDataBridge.Output;
using FmDataBridge.Protocol;
using FmDataBridge.Research;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class ProbeCaptureTests
{
    private const ulong GameAssemblyBase = 0x180000000UL;
    private const ulong GameAssemblyEnd = 0x180100000UL;
    private const ulong VtableInAssembly = 0x180001000UL;
    private const ulong MetaInAssembly = 0x180002000UL;
    private const ulong PlayerOneBase = 0x100000UL;
    private const ulong PlayerTwoBase = 0x110000UL;
    private const ulong PointerTargetBase = 0x200000UL;
    private const int PlayerClassOffset = 0x288;

    [Fact]
    public void Accepted_request_captures_only_requested_uid_with_bounded_roots_and_pointer_targets()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            var productionFiles = WriteProductionSentinels(bridgeDirectory);
            var requestPath = BridgePaths.GetProbeRequestPath(bridgeDirectory);
            var now = DateTimeOffset.Parse("2026-08-07T12:00:00Z");
            File.WriteAllText(
                requestPath,
                "{\n"
                + $"  \"protocolVersion\": {ProbeProtocol.ProtocolVersion},\n"
                + "  \"requestId\": \"probe-capture-1\",\n"
                + $"  \"createdAtUtc\": \"{now:O}\",\n"
                + "  \"uids\": [1001]\n"
                + "}\n");

            Assert.True(
                ProbeRequestAcceptance.TryAccept(
                    requestPath,
                    now,
                    TimeSpan.FromSeconds(BridgeProtocol.RequestTtlSeconds),
                    out var request,
                    out var rejectReason,
                    out var observedRequestId));
            Assert.Null(rejectReason);
            Assert.Equal("probe-capture-1", observedRequestId);
            Assert.False(File.Exists(requestPath));

            var reader = BuildReaderWithTwoPlayersAndManyPointers();
            var result = new ProbeCaptureService().RunAndWrite(
                reader,
                bridgeDirectory,
                request,
                gameVersion: "26.3.2.2329565",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success, result.Error);
            Assert.NotNull(result.Document);
            Assert.Equal(1, result.Document!.PlayerCount);
            Assert.Equal(1001u, result.Document.Players[0].Uid);
            Assert.Equal(PlayerOneBase + (ulong)PlayerClassOffset, result.Document.Players[0].CandidateAddress);
            Assert.Equal(PlayerOneBase, result.Document.Players[0].PlayerBlockAddress);
            Assert.Equal(ProbeCaptureLimits.MaxBytesPerPlayer, result.Document.Players[0].RequestedBytes);

            var ranges = result.Document.Players[0].Ranges;
            Assert.Equal(2 + ProbeCaptureLimits.MaxPointerTargetsPerPlayer, ranges.Count);
            Assert.Equal(
                ProbeCaptureLimits.MaxPointerTargetsPerPlayer,
                ranges.Count(range => range.PointerDepth == 1));
            Assert.All(
                ranges,
                range => Assert.InRange(range.RequestedLength, 1, ProbeCaptureLimits.PlayerRootWindowBytes));
            Assert.DoesNotContain(
                ranges,
                range => range.SourcePointerPath?.EndsWith("+0x18", StringComparison.Ordinal) == true);

            var playerRoot = Assert.Single(ranges, range => range.AddressBasis == "player-block");
            Assert.Equal(ProbeCaptureLimits.PlayerRootWindowBytes, playerRoot.RequestedLength);
            var playerBytes = ReadCapturedBytes(playerRoot);
            var layout = Fm263Layout.Instance;
            var determinationOffset = layout.AttrsOffset
                + layout.AttributeEntries.Single(entry => entry.Key == "Determination").Offset;
            Assert.Equal(
                (ushort)120,
                BinaryPrimitives.ReadUInt16LittleEndian(playerBytes.AsSpan(layout.CurrentAbilityOffset)));
            Assert.Equal(
                (ushort)160,
                BinaryPrimitives.ReadUInt16LittleEndian(playerBytes.AsSpan(layout.PotentialAbilityOffset)));
            Assert.Equal(75, playerBytes[determinationOffset]);
            Assert.Equal(
                10_000_000u,
                BinaryPrimitives.ReadUInt32LittleEndian(playerBytes.AsSpan(layout.MarketValueOffset)));

            var personRoot = Assert.Single(ranges, range => range.AddressBasis == "person-object");
            Assert.Equal(ProbeCaptureLimits.PersonRootWindowBytes, personRoot.RequestedLength);
            Assert.Equal(
                ProbeCaptureLimits.PersonRootWindowBytes,
                personRoot.ReadableSpans.Sum(span => Convert.FromBase64String(span.BytesBase64).Length));

            using var document = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory)));
            Assert.Equal(ProbeProtocol.SchemaVersion, document.RootElement.GetProperty("schemaVersion").GetInt32());
            Assert.Equal("probe-capture-1", document.RootElement.GetProperty("requestId").GetString());
            Assert.Equal(1, document.RootElement.GetProperty("players").GetArrayLength());
            Assert.Equal(1001u, document.RootElement.GetProperty("players")[0].GetProperty("uid").GetUInt32());

            foreach (var (path, contents) in productionFiles)
            {
                Assert.Equal(contents, File.ReadAllText(path));
            }
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Missing_requested_uid_preserves_prior_successful_probe()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            var prior = MinimalProbeDocument("prior-success");
            Assert.True(ProbeWriter.TryWriteReplaceOnSuccess(bridgeDirectory, prior));
            var priorJson = File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory));

            var reader = BuildReaderWithTwoPlayersAndManyPointers();
            var result = new ProbeCaptureService().RunAndWrite(
                reader,
                bridgeDirectory,
                new ProbeRequest
                {
                    ProtocolVersion = ProbeProtocol.ProtocolVersion,
                    RequestId = "missing-player",
                    CreatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
                    Uids = new[] { 9999u },
                },
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.False(result.Success);
            Assert.Contains("missing", result.Error, StringComparison.OrdinalIgnoreCase);
            Assert.Equal(priorJson, File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory)));
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Partial_root_read_preserves_prior_successful_probe()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            var prior = MinimalProbeDocument("prior-partial-root");
            Assert.True(ProbeWriter.TryWriteReplaceOnSuccess(bridgeDirectory, prior));
            var priorJson = File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory));

            var result = new ProbeCaptureService().RunAndWrite(
                BuildReaderWithUnreadablePersonRoot(),
                bridgeDirectory,
                CreateProbeRequest("partial-root"),
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.False(result.Success);
            Assert.Contains("unread", result.Error, StringComparison.OrdinalIgnoreCase);
            Assert.Equal(priorJson, File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory)));
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Probe_request_rejects_duplicate_uids()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            var requestPath = BridgePaths.GetProbeRequestPath(bridgeDirectory);
            var now = DateTimeOffset.Parse("2026-08-07T12:00:00Z");
            File.WriteAllText(
                requestPath,
                "{\n"
                + $"  \"protocolVersion\": {ProbeProtocol.ProtocolVersion},\n"
                + "  \"requestId\": \"duplicate-uids\",\n"
                + $"  \"createdAtUtc\": \"{now:O}\",\n"
                + "  \"uids\": [1001, 1001]\n"
                + "}\n");

            Assert.False(
                ProbeRequestAcceptance.TryAccept(
                    requestPath,
                    now,
                    TimeSpan.FromSeconds(BridgeProtocol.RequestTtlSeconds),
                    out _,
                    out var rejectReason,
                    out var observedRequestId));
            Assert.Contains("duplicate", rejectReason, StringComparison.OrdinalIgnoreCase);
            Assert.Equal("duplicate-uids", observedRequestId);
            Assert.False(File.Exists(requestPath));
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Capture_boundary_rejects_too_many_uids_before_replacing_prior_probe()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            var prior = MinimalProbeDocument("prior-boundary");
            Assert.True(ProbeWriter.TryWriteReplaceOnSuccess(bridgeDirectory, prior));
            var priorJson = File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory));

            var result = new ProbeCaptureService().RunAndWrite(
                new FakeMemoryReader(),
                bridgeDirectory,
                new ProbeRequest
                {
                    ProtocolVersion = ProbeProtocol.ProtocolVersion,
                    RequestId = "too-many-uids",
                    CreatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
                    Uids = Enumerable.Range(1, ProbeRequestAcceptance.MaxRequestedUids + 1)
                        .Select(value => (uint)value)
                        .ToArray(),
                },
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.False(result.Success);
            Assert.Contains("uids", result.Error, StringComparison.OrdinalIgnoreCase);
            Assert.Equal(priorJson, File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory)));
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Cancelled_capture_preserves_prior_successful_probe()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            var prior = MinimalProbeDocument("prior-cancelled");
            Assert.True(ProbeWriter.TryWriteReplaceOnSuccess(bridgeDirectory, prior));
            var priorJson = File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory));
            using var cancellation = new CancellationTokenSource();
            cancellation.Cancel();

            var result = new ProbeCaptureService().RunAndWrite(
                BuildReaderWithTwoPlayersAndManyPointers(),
                bridgeDirectory,
                new ProbeRequest
                {
                    ProtocolVersion = ProbeProtocol.ProtocolVersion,
                    RequestId = "cancelled-probe",
                    CreatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
                    Uids = new[] { 1001u },
                },
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
                cancellationToken: cancellation.Token);

            Assert.False(result.Success);
            Assert.Contains("cancelled", result.Error, StringComparison.OrdinalIgnoreCase);
            Assert.Equal(priorJson, File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory)));
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Cancellation_after_final_root_read_preserves_prior_successful_probe()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            var prior = MinimalProbeDocument("prior-late-cancellation");
            Assert.True(ProbeWriter.TryWriteReplaceOnSuccess(bridgeDirectory, prior));
            var priorJson = File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory));
            using var cancellation = new CancellationTokenSource();
            var reader = new CancellingMemoryReader(
                BuildReaderWithTwoPlayersAndManyPointers(),
                cancellation,
                PlayerOneBase
                    + (ulong)PlayerClassOffset
                    + (ulong)ProbeCaptureLimits.PersonRootWindowBytes
                    - 1);

            var result = new ProbeCaptureService().RunAndWrite(
                reader,
                bridgeDirectory,
                CreateProbeRequest("late-cancellation"),
                gameVersion: "26.3.2",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
                cancellationToken: cancellation.Token);

            Assert.False(result.Success);
            Assert.Contains("cancelled", result.Error, StringComparison.OrdinalIgnoreCase);
            Assert.Equal(priorJson, File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory)));
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Probe_status_writer_uses_a_separate_status_file()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            ProbeStatusWriter.Write(
                bridgeDirectory,
                new ProbeStatus
                {
                    ProtocolVersion = ProbeProtocol.ProtocolVersion,
                    PluginVersion = "0.1.0",
                    State = ProbeProtocol.StateReady,
                    UpdatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
                    GamePluginModulePresent = true,
                    GameAssemblyModulePresent = true,
                    RequestId = "separate-status",
                    PlayersCaptured = 1,
                });

            Assert.True(File.Exists(BridgePaths.GetProbeStatusPath(bridgeDirectory)));
            Assert.False(File.Exists(BridgePaths.GetStatusPath(bridgeDirectory)));
            using var document = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetProbeStatusPath(bridgeDirectory)));
            Assert.Equal(ProbeProtocol.StateReady, document.RootElement.GetProperty("state").GetString());
            Assert.Equal(1, document.RootElement.GetProperty("playersCaptured").GetInt32());
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Probe_status_writer_records_a_rejected_request_without_a_parseable_id()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            ProbeStatusWriter.Write(
                bridgeDirectory,
                new ProbeStatus
                {
                    ProtocolVersion = ProbeProtocol.ProtocolVersion,
                    PluginVersion = "0.1.0",
                    State = ProbeProtocol.StateFailed,
                    UpdatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
                    GamePluginModulePresent = true,
                    GameAssemblyModulePresent = true,
                    Error = "requestId is required",
                });

            using var document = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetProbeStatusPath(bridgeDirectory)));
            Assert.Equal(ProbeProtocol.StateFailed, document.RootElement.GetProperty("state").GetString());
            Assert.Equal(JsonValueKind.Null, document.RootElement.GetProperty("requestId").ValueKind);
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Theory]
    [InlineData(true, false, true, 1)]
    [InlineData(false, true, true, 2)]
    [InlineData(false, false, true, 3)]
    [InlineData(false, false, false, 0)]
    public void Scan_priority_keeps_production_requests_ahead_of_probe_requests(
        bool hasDumpRequest,
        bool hasForceScan,
        bool hasProbeRequest,
        int expected)
    {
        Assert.Equal((ScanRequestKind)expected, ScanRequestPriority.Select(hasDumpRequest, hasForceScan, hasProbeRequest));
    }

    private static FakeMemoryReader BuildReaderWithTwoPlayersAndManyPointers()
    {
        var reader = new FakeMemoryReader();
        reader.AddRegion(
            new MemoryRegion(
                PlayerOneBase,
                0x30000,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));
        reader.AddRegion(
            new MemoryRegion(
                PointerTargetBase,
                0x2000,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));

        PlacePlayer(reader, PlayerOneBase, uid: 1001, includePointers: true);
        PlacePlayer(reader, PlayerTwoBase, uid: 1002, includePointers: false);

        for (var index = 0; index < ProbeCaptureLimits.MaxPointerTargetsPerPlayer + 1; index++)
        {
            var target = PointerTargetBase + (ulong)(index * ProbeCaptureLimits.PointerTargetWindowBytes);
            reader.AddBytes(target, Enumerable.Repeat((byte)(index + 1), ProbeCaptureLimits.PointerTargetWindowBytes).ToArray());
        }

        return reader;
    }

    private static FakeMemoryReader BuildReaderWithUnreadablePersonRoot()
    {
        var reader = new FakeMemoryReader();
        reader.AddRegion(
            new MemoryRegion(
                PlayerOneBase,
                0x1000,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));
        PlacePlayer(reader, PlayerOneBase, uid: 1001, includePointers: false, fullPersonRoot: false);
        return reader;
    }

    private static void PlacePlayer(
        FakeMemoryReader reader,
        ulong playerBase,
        uint uid,
        bool includePointers,
        bool fullPersonRoot = true)
    {
        var layout = Fm263Layout.Instance;
        var personAddress = playerBase + (ulong)PlayerClassOffset;

        var meta = new byte[8];
        BinaryPrimitives.WriteInt32LittleEndian(meta.AsSpan(4), PlayerClassOffset);
        reader.AddBytes(MetaInAssembly, meta);

        var vtableLink = new byte[8];
        BinaryPrimitives.WriteUInt64LittleEndian(vtableLink, MetaInAssembly);
        reader.AddBytes(VtableInAssembly - 8, vtableLink);

        var personBytes = new byte[fullPersonRoot ? ProbeCaptureLimits.PersonRootWindowBytes : 16];
        BinaryPrimitives.WriteUInt64LittleEndian(personBytes, VtableInAssembly);
        BinaryPrimitives.WriteUInt32LittleEndian(personBytes.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(personAddress, personBytes);

        var playerBytes = new byte[ProbeCaptureLimits.PlayerRootWindowBytes];
        BinaryPrimitives.WriteUInt16LittleEndian(playerBytes.AsSpan(layout.CurrentAbilityOffset), 120);
        BinaryPrimitives.WriteUInt16LittleEndian(playerBytes.AsSpan(layout.PotentialAbilityOffset), 160);
        BinaryPrimitives.WriteUInt32LittleEndian(playerBytes.AsSpan(layout.MarketValueOffset), 10_000_000);
        var determinationOffset = layout.AttrsOffset
            + layout.AttributeEntries.Single(entry => entry.Key == "Determination").Offset;
        playerBytes[determinationOffset] = 75;
        if (includePointers)
        {
            BinaryPrimitives.WriteUInt64LittleEndian(playerBytes.AsSpan(0x18), 0x400000UL);
            for (var index = 0; index < ProbeCaptureLimits.MaxPointerTargetsPerPlayer + 1; index++)
            {
                var offset = 0x20 + (index * sizeof(ulong));
                var target = PointerTargetBase + (ulong)(index * ProbeCaptureLimits.PointerTargetWindowBytes);
                BinaryPrimitives.WriteUInt64LittleEndian(playerBytes.AsSpan(offset), target);
            }
        }

        reader.AddBytes(playerBase, playerBytes);
    }

    private static ProbeRequest CreateProbeRequest(string requestId) =>
        new()
        {
            ProtocolVersion = ProbeProtocol.ProtocolVersion,
            RequestId = requestId,
            CreatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
            Uids = new[] { 1001u },
        };

    private static byte[] ReadCapturedBytes(ProbeMemoryRange range)
    {
        var bytes = new byte[range.RequestedLength];
        foreach (var span in range.ReadableSpans)
        {
            Convert.FromBase64String(span.BytesBase64).CopyTo(bytes.AsSpan(span.Offset));
        }

        return bytes;
    }

    private sealed class CancellingMemoryReader : IMemoryReader
    {
        private readonly IMemoryReader _inner;
        private readonly CancellationTokenSource _cancellation;
        private readonly ulong _cancelAfterAddress;

        public CancellingMemoryReader(
            IMemoryReader inner,
            CancellationTokenSource cancellation,
            ulong cancelAfterAddress)
        {
            _inner = inner;
            _cancellation = cancellation;
            _cancelAfterAddress = cancelAfterAddress;
        }

        public IEnumerable<MemoryRegion> EnumerateRegions() => _inner.EnumerateRegions();

        public bool TryRead(ulong address, Span<byte> destination, out int bytesRead)
        {
            var success = _inner.TryRead(address, destination, out bytesRead);
            if (success && address == _cancelAfterAddress)
            {
                _cancellation.Cancel();
            }

            return success;
        }

        public bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead) =>
            _inner.TryReadBlock(address, buffer, offset, length, out bytesRead);
    }

    private static Dictionary<string, string> WriteProductionSentinels(string bridgeDirectory)
    {
        var sentinels = new Dictionary<string, string>
        {
            [BridgePaths.GetRequestPath(bridgeDirectory)] = "production-request",
            [BridgePaths.GetStatusPath(bridgeDirectory)] = "production-status",
            [BridgePaths.GetDumpPath(bridgeDirectory)] = "production-dump",
            [BridgePaths.GetDiagnosticsPath(bridgeDirectory)] = "production-diagnostics",
        };

        foreach (var (path, contents) in sentinels)
        {
            File.WriteAllText(path, contents, Encoding.UTF8);
        }

        return sentinels;
    }

    private static ProbeDocument MinimalProbeDocument(string requestId) =>
        new()
        {
            SchemaVersion = ProbeProtocol.SchemaVersion,
            GeneratedAtUtc = "2026-08-07T11:00:00+00:00",
            GameVersion = "26.3.2",
            SupportedGameVersion = "26.3",
            BridgeVersion = "0.1.0",
            ProtocolVersion = ProbeProtocol.ProtocolVersion,
            RequestId = requestId,
            RequestedUids = new[] { 77u },
            PlayerCount = 1,
            Players = new[]
            {
                new ProbePlayer
                {
                    Uid = 77,
                    CandidateAddress = 0x1010,
                    PlayerBlockAddress = 0x0F88,
                    RequestedBytes = 1,
                    ReadableBytes = 1,
                },
            },
        };

    private static string CreateTempBridgeDirectory()
    {
        var path = Path.Combine(Path.GetTempPath(), "fm-memory-probe-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(path);
        return path;
    }
}
