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
    private const ulong PersonPointerTargetBase = PointerTargetBase + 0x1000;
    private const ulong DepthTwoTargetBase = PointerTargetBase + 0x3000;
    private const ulong DepthThreeTargetBase = PointerTargetBase + 0x4000;
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
            Assert.Equal(
                2 + ProbeCaptureLimits.MaxFirstHopTargetsPerPlayer + ProbeCaptureLimits.MaxSecondHopTargetsPerPlayer,
                ranges.Count);
            Assert.Equal(
                ProbeCaptureLimits.MaxFirstHopTargetsPerPlayer,
                ranges.Count(range => range.PointerDepth == 1));
            Assert.Equal(
                ProbeCaptureLimits.MaxSecondHopTargetsPerPlayer,
                ranges.Count(range => range.PointerDepth == 2));
            Assert.All(
                ranges,
                range =>
                {
                    Assert.InRange(range.RequestedLength, 1, ProbeCaptureLimits.PlayerRootWindowBytes);
                    Assert.InRange(range.PointerDepth, 0, ProbeCaptureLimits.MaxPointerDepth);
                });
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
    public void Cohort_capture_selects_shared_paths_and_reserves_person_root_quota()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            var result = new ProbeCaptureService().RunAndWrite(
                BuildReaderWithCohortPointerPaths(),
                bridgeDirectory,
                new ProbeRequest
                {
                    ProtocolVersion = ProbeProtocol.ProtocolVersion,
                    RequestId = "cohort-paths",
                    CreatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
                    Uids = new[] { 1001u, 1002u },
                },
                gameVersion: "26.3.2.2329565",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success, result.Error);
            Assert.NotNull(result.Document);
            Assert.All(
                result.Document!.Players,
                player => Assert.Contains(
                    player.Ranges,
                    range => range.SourcePointerPath == "player-block+0x70"));
            Assert.All(
                result.Document.Players,
                player => Assert.Contains(
                    player.Ranges,
                    range => range.SourcePointerPath == "person-object+0x20"));

            var policy = Assert.IsType<ProbeCapturePolicy>(result.Document.CapturePolicy);
            Assert.Equal(2, policy.MaxPointerDepth);
            Assert.Equal(ProbeCaptureLimits.MaxBytesPerPlayer, policy.MaxBytesPerPlayer);
            Assert.Equal(ProbeCaptureLimits.MaxBytesPerRequest, policy.MaxBytesPerRequest);
            Assert.Equal(
                new[] { "player-block", "person-object", "pointer-target" },
                policy.PathQuotas.Select(quota => quota.AddressBasis).ToArray());
            Assert.All(policy.PathQuotas, quota => Assert.Equal(8, quota.MaxPaths));
            Assert.Equal(
                new[]
                {
                    "player-block+0x70",
                    "player-block+0x20",
                    "player-block+0x28",
                    "player-block+0x30",
                    "player-block+0x38",
                    "player-block+0x48",
                    "player-block+0x50",
                    "player-block+0x58",
                    "person-object+0x20",
                },
                policy.SelectedPaths.Select(path => path.SourcePointerPath).ToArray());
            Assert.Equal(
                1,
                policy.SelectedPaths.Single(path => path.SourcePointerPath == "player-block+0x20").EligiblePlayerCount);
            Assert.DoesNotContain(
                result.Document.Players.Single(player => player.Uid == 1002).Ranges,
                range => range.SourcePointerPath == "player-block+0x20");

            using var document = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetProbePath(bridgeDirectory)));
            Assert.Equal(2, document.RootElement.GetProperty("schemaVersion").GetInt32());
            var policyJson = document.RootElement.GetProperty("capturePolicy");
            Assert.Equal(2, policyJson.GetProperty("maxPointerDepth").GetInt32());
            Assert.Contains(
                policyJson.GetProperty("selectedPaths").EnumerateArray(),
                path => path.GetProperty("sourcePointerPath").GetString() == "person-object+0x20");

            var repeatDirectory = CreateTempBridgeDirectory();
            try
            {
                var repeated = new ProbeCaptureService().RunAndWrite(
                    BuildReaderWithCohortPointerPaths(),
                    repeatDirectory,
                    new ProbeRequest
                    {
                        ProtocolVersion = ProbeProtocol.ProtocolVersion,
                        RequestId = "cohort-paths-repeat",
                        CreatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
                        Uids = new[] { 1001u, 1002u },
                    },
                    gameVersion: "26.3.2.2329565",
                    bridgeVersion: "0.1.0",
                    gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

                Assert.True(repeated.Success, repeated.Error);
                Assert.NotNull(repeated.Document);
                Assert.Equal(
                    policy.SelectedPaths.Select(path => path.SourcePointerPath).ToArray(),
                    repeated.Document!.CapturePolicy!.SelectedPaths.Select(path => path.SourcePointerPath).ToArray());
            }
            finally
            {
                Directory.Delete(repeatDirectory, recursive: true);
            }
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Cohort_capture_follows_one_selected_second_hop_with_full_provenance()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            var result = new ProbeCaptureService().RunAndWrite(
                BuildReaderWithDepthTwoPointerChain(),
                bridgeDirectory,
                new ProbeRequest
                {
                    ProtocolVersion = ProbeProtocol.ProtocolVersion,
                    RequestId = "depth-two",
                    CreatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
                    Uids = new[] { 1001u, 1002u },
                },
                gameVersion: "26.3.2.2329565",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success, result.Error);
            var players = result.Document!.Players;
            Assert.Equal(new[] { 1001u, 1002u }, players.Select(player => player.Uid).ToArray());
            var depthTwoRanges = players
                .Select(player => Assert.Single(player.Ranges, range => range.PointerDepth == 2))
                .ToArray();
            Assert.All(depthTwoRanges, depthTwoRange =>
            {
                Assert.Equal("player-block+0x70->target+0x20", depthTwoRange.SourcePointerPath);
                Assert.Equal("player-block+0x70->target+0x20->target+0x0", depthTwoRange.RelativePath);
            });
            Assert.Equal(new byte[] { 42, 43 }, depthTwoRanges.Select(range => ReadCapturedBytes(range)[0x40]).ToArray());
            Assert.DoesNotContain(
                players.SelectMany(player => player.Ranges),
                range => range.PointerDepth > 2 || range.Address == DepthThreeTargetBase || range.Address == DepthThreeTargetBase + 0x80);

            var policy = Assert.IsType<ProbeCapturePolicy>(result.Document.CapturePolicy);
            Assert.Equal(2, policy.MaxPointerDepth);
            Assert.Contains(
                policy.PathQuotas,
                quota => quota.AddressBasis == "pointer-target" && quota.PointerDepth == 2 && quota.MaxPaths == 8);
            Assert.Contains(
                policy.SelectedPaths,
                path => path.SourcePointerPath == "player-block+0x70->target+0x20"
                    && path.PointerDepth == 2
                    && path.EligiblePlayerCount == 2);

            var repeatDirectory = CreateTempBridgeDirectory();
            try
            {
                var repeated = new ProbeCaptureService().RunAndWrite(
                    BuildReaderWithDepthTwoPointerChain(),
                    repeatDirectory,
                    new ProbeRequest
                    {
                        ProtocolVersion = ProbeProtocol.ProtocolVersion,
                        RequestId = "depth-two-repeat",
                        CreatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
                        Uids = new[] { 1002u, 1001u },
                    },
                    gameVersion: "26.3.2.2329565",
                    bridgeVersion: "0.1.0",
                    gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

                Assert.True(repeated.Success, repeated.Error);
                Assert.Equal(
                    policy.SelectedPaths.Select(path => (path.SourcePointerPath, path.PointerDepth, path.EligiblePlayerCount)),
                    repeated.Document!.CapturePolicy!.SelectedPaths.Select(path => (path.SourcePointerPath, path.PointerDepth, path.EligiblePlayerCount)));
            }
            finally
            {
                Directory.Delete(repeatDirectory, recursive: true);
            }
        }
        finally
        {
            Directory.Delete(bridgeDirectory, recursive: true);
        }
    }

    [Fact]
    public void Second_hop_capture_skips_cycle_and_alias_targets()
    {
        var bridgeDirectory = CreateTempBridgeDirectory();
        try
        {
            var result = new ProbeCaptureService().RunAndWrite(
                BuildReaderWithAliasedSecondHopTargets(),
                bridgeDirectory,
                CreateProbeRequest("depth-two-aliases"),
                gameVersion: "26.3.2.2329565",
                bridgeVersion: "0.1.0",
                gameAssembly: new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd));

            Assert.True(result.Success, result.Error);
            var player = Assert.Single(result.Document!.Players);
            var depthTwoRanges = player.Ranges.Where(range => range.PointerDepth == 2).ToArray();
            Assert.Equal(2, depthTwoRanges.Length);
            Assert.Equal(2, depthTwoRanges.Select(range => range.Address).Distinct().Count());
            Assert.DoesNotContain(
                depthTwoRanges,
                range => range.SourcePointerPath == "player-block+0x70->target+0x28"
                    || range.SourcePointerPath == "player-block+0x70->target+0x30");
            Assert.Equal(
                ProbeCaptureLimits.PlayerRootWindowBytes
                    + ProbeCaptureLimits.PersonRootWindowBytes
                    + (4 * ProbeCaptureLimits.PointerTargetWindowBytes),
                player.RequestedBytes);

            var policy = Assert.IsType<ProbeCapturePolicy>(result.Document.CapturePolicy);
            Assert.Contains(
                policy.SelectedPaths,
                path => path.SourcePointerPath == "player-block+0x70->target+0x28" && path.PointerDepth == 2);
            Assert.Contains(
                policy.SelectedPaths,
                path => path.SourcePointerPath == "player-block+0x70->target+0x30" && path.PointerDepth == 2);
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
    public void Probe_request_accepts_103_uids_with_a_fixed_full_export_capture_budget()
    {
        var request = new ProbeRequest
        {
            ProtocolVersion = ProbeProtocol.ProtocolVersion,
            RequestId = "full-export-request",
            CreatedAtUtc = DateTimeOffset.Parse("2026-08-07T12:00:00Z"),
            Uids = Enumerable.Range(1, 103).Select(value => (uint)value).ToArray(),
        };

        Assert.True(ProbeRequestAcceptance.TryValidateForCapture(request, out var rejectReason));
        Assert.Null(rejectReason);
        Assert.Equal(128, ProbeRequestAcceptance.MaxRequestedUids);
        Assert.Equal(507_904, ProbeRequestAcceptance.MaxRequestedUids * ProbeCaptureLimits.MaxBytesPerPlayer);
    }

    [Fact]
    public void Capture_boundary_rejects_129_uids_before_replacing_prior_probe()
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
                    Uids = Enumerable.Range(1, 129)
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
                0x5000,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));

        PlacePlayer(reader, PlayerOneBase, uid: 1001, includePointers: true);
        PlacePlayer(reader, PlayerTwoBase, uid: 1002, includePointers: false);

        var firstHopPathCount = ProbeCaptureLimits.MaxPlayerRootFirstHopPaths + 1;
        for (var index = 0; index < firstHopPathCount; index++)
        {
            var target = PointerTargetBase + (ulong)(index * ProbeCaptureLimits.PointerTargetWindowBytes);
            var personTarget = PersonPointerTargetBase + (ulong)(index * ProbeCaptureLimits.PointerTargetWindowBytes);
            var playerSecondHopTarget = DepthTwoTargetBase + (ulong)(index * ProbeCaptureLimits.PointerTargetWindowBytes);
            var personSecondHopTarget = DepthTwoTargetBase
                + (ulong)((firstHopPathCount + index) * ProbeCaptureLimits.PointerTargetWindowBytes);
            reader.AddBytes(target + 0x20, PointerBytes(playerSecondHopTarget));
            reader.AddBytes(personTarget + 0x20, PointerBytes(personSecondHopTarget));
            reader.AddBytes(playerSecondHopTarget, new byte[ProbeCaptureLimits.PointerTargetWindowBytes]);
            reader.AddBytes(personSecondHopTarget, new byte[ProbeCaptureLimits.PointerTargetWindowBytes]);
        }

        for (var index = 0; index < firstHopPathCount; index++)
        {
            var target = PointerTargetBase + (ulong)(index * ProbeCaptureLimits.PointerTargetWindowBytes);
            var personTarget = PersonPointerTargetBase + (ulong)(index * ProbeCaptureLimits.PointerTargetWindowBytes);
            reader.AddBytes(target, Enumerable.Repeat((byte)(index + 1), ProbeCaptureLimits.PointerTargetWindowBytes).ToArray());
            reader.AddBytes(personTarget, Enumerable.Repeat((byte)(index + 11), ProbeCaptureLimits.PointerTargetWindowBytes).ToArray());
        }

        return reader;

        static byte[] PointerBytes(ulong target)
        {
            var pointer = new byte[sizeof(ulong)];
            BinaryPrimitives.WriteUInt64LittleEndian(pointer, target);
            return pointer;
        }
    }

    private static FakeMemoryReader BuildReaderWithCohortPointerPaths()
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

        var targetIndex = 0;
        AddPointer(PlayerOneBase + 0x20, 1);
        AddPointer(PlayerOneBase + 0x28, 2);
        AddPointer(PlayerOneBase + 0x30, 3);
        AddPointer(PlayerOneBase + 0x38, 4);
        AddPointer(PlayerOneBase + 0x70, 5);
        AddPointer(PlayerOneBase + (ulong)PlayerClassOffset + 0x20, 6);

        AddPointer(PlayerTwoBase + 0x48, 7);
        AddPointer(PlayerTwoBase + 0x50, 8);
        AddPointer(PlayerTwoBase + 0x58, 9);
        AddPointer(PlayerTwoBase + 0x60, 10);
        AddPointer(PlayerTwoBase + 0x70, 11);
        AddPointer(PlayerTwoBase + (ulong)PlayerClassOffset + 0x20, 12);

        PlacePlayer(reader, PlayerOneBase, uid: 1001, includePointers: false);
        PlacePlayer(reader, PlayerTwoBase, uid: 1002, includePointers: false);
        return reader;

        void AddPointer(ulong sourceAddress, byte marker)
        {
            var target = PointerTargetBase + (ulong)(targetIndex * ProbeCaptureLimits.PointerTargetWindowBytes);
            var pointer = new byte[sizeof(ulong)];
            BinaryPrimitives.WriteUInt64LittleEndian(pointer, target);
            reader.AddBytes(sourceAddress, pointer);
            reader.AddBytes(target, Enumerable.Repeat(marker, ProbeCaptureLimits.PointerTargetWindowBytes).ToArray());
            targetIndex++;
        }
    }

    private static FakeMemoryReader BuildReaderWithDepthTwoPointerChain()
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
                0x5000,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));

        AddPointer(PlayerOneBase + 0x70, PointerTargetBase);
        AddPointer(PlayerTwoBase + 0x70, PointerTargetBase + 0x80);
        AddPointer(PointerTargetBase + 0x20, DepthTwoTargetBase);
        AddPointer(PointerTargetBase + 0x80 + 0x20, DepthTwoTargetBase + 0x80);
        AddPointer(DepthTwoTargetBase + 0x20, DepthThreeTargetBase);
        AddPointer(DepthTwoTargetBase + 0x80 + 0x20, DepthThreeTargetBase + 0x80);

        var firstHopBytes = new byte[ProbeCaptureLimits.PointerTargetWindowBytes];
        var secondHopBytes = new byte[ProbeCaptureLimits.PointerTargetWindowBytes];
        secondHopBytes[0x40] = 42;
        var secondPlayerFirstHopBytes = new byte[ProbeCaptureLimits.PointerTargetWindowBytes];
        var secondPlayerSecondHopBytes = new byte[ProbeCaptureLimits.PointerTargetWindowBytes];
        secondPlayerSecondHopBytes[0x40] = 43;
        reader.AddBytes(PointerTargetBase, firstHopBytes);
        reader.AddBytes(DepthTwoTargetBase, secondHopBytes);
        reader.AddBytes(PointerTargetBase + 0x80, secondPlayerFirstHopBytes);
        reader.AddBytes(DepthTwoTargetBase + 0x80, secondPlayerSecondHopBytes);
        reader.AddBytes(DepthThreeTargetBase, new byte[ProbeCaptureLimits.PointerTargetWindowBytes]);
        reader.AddBytes(DepthThreeTargetBase + 0x80, new byte[ProbeCaptureLimits.PointerTargetWindowBytes]);
        PlacePlayer(reader, PlayerOneBase, uid: 1001, includePointers: false);
        PlacePlayer(reader, PlayerTwoBase, uid: 1002, includePointers: false);
        return reader;

        void AddPointer(ulong sourceAddress, ulong targetAddress)
        {
            var pointer = new byte[sizeof(ulong)];
            BinaryPrimitives.WriteUInt64LittleEndian(pointer, targetAddress);
            reader.AddBytes(sourceAddress, pointer);
        }
    }

    private static FakeMemoryReader BuildReaderWithAliasedSecondHopTargets()
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
                0x5000,
                MemoryConstants.PageReadWrite,
                MemoryConstants.MemPrivate,
                MemoryConstants.MemCommit));

        AddPointer(PlayerOneBase + 0x70, PointerTargetBase);
        AddPointer(PlayerOneBase + 0x78, PointerTargetBase + 0x80);
        AddPointer(PointerTargetBase + 0x20, DepthTwoTargetBase);
        AddPointer(PointerTargetBase + 0x28, PointerTargetBase);
        AddPointer(PointerTargetBase + 0x30, DepthTwoTargetBase);
        AddPointer(PointerTargetBase + 0x80 + 0x20, DepthTwoTargetBase + 0x80);

        reader.AddBytes(PointerTargetBase, new byte[ProbeCaptureLimits.PointerTargetWindowBytes]);
        reader.AddBytes(PointerTargetBase + 0x80, new byte[ProbeCaptureLimits.PointerTargetWindowBytes]);
        reader.AddBytes(DepthTwoTargetBase, new byte[ProbeCaptureLimits.PointerTargetWindowBytes]);
        reader.AddBytes(DepthTwoTargetBase + 0x80, new byte[ProbeCaptureLimits.PointerTargetWindowBytes]);
        PlacePlayer(reader, PlayerOneBase, uid: 1001, includePointers: false);
        return reader;

        void AddPointer(ulong sourceAddress, ulong targetAddress)
        {
            var pointer = new byte[sizeof(ulong)];
            BinaryPrimitives.WriteUInt64LittleEndian(pointer, targetAddress);
            reader.AddBytes(sourceAddress, pointer);
        }
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
        if (includePointers)
        {
            for (var index = 0; index < ProbeCaptureLimits.MaxPlayerRootFirstHopPaths + 1; index++)
            {
                var offset = 0x20 + (index * sizeof(ulong));
                var target = PersonPointerTargetBase + (ulong)(index * ProbeCaptureLimits.PointerTargetWindowBytes);
                BinaryPrimitives.WriteUInt64LittleEndian(personBytes.AsSpan(offset), target);
            }
        }

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
            for (var index = 0; index < ProbeCaptureLimits.MaxPlayerRootFirstHopPaths + 1; index++)
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
