using System.Buffers.Binary;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;
using FmDataBridge.Output;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;

namespace FmDataBridge.Research;

internal static class ProbeCaptureLimits
{
    public const int PlayerRootWindowBytes = 0x280;
    public const int PersonRootWindowBytes = 0x100;
    public const int PointerTargetWindowBytes = 128;
    public const int MaxPointerDepth = 1;
    public const int MaxPlayerRootFirstHopPaths = 8;
    public const int MaxPersonRootFirstHopPaths = 8;
    public const int MaxFirstHopTargetsPerPlayer = MaxPlayerRootFirstHopPaths + MaxPersonRootFirstHopPaths;
    public const int MaxBytesPerPlayer = PlayerRootWindowBytes
        + PersonRootWindowBytes
        + (MaxFirstHopTargetsPerPlayer * PointerTargetWindowBytes);
    public const int MaxBytesPerRequest = ProbeRequestAcceptance.MaxRequestedUids * MaxBytesPerPlayer;
}

/// <summary>
/// Locates requested players with the normal scanner, then writes one bounded research capture.
/// </summary>
public sealed class ProbeCaptureService
{
    private readonly LayoutRegistry _layouts;

    public ProbeCaptureService(LayoutRegistry? layouts = null)
    {
        _layouts = layouts ?? LayoutRegistry.CreateDefault();
    }

    public ProbeCaptureResult RunAndWrite(
        IMemoryReader reader,
        string bridgeDirectory,
        ProbeRequest request,
        string gameVersion,
        string bridgeVersion,
        ModuleBounds gameAssembly,
        ModuleBounds? gamePlugin = null,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(request);
        if (string.IsNullOrWhiteSpace(bridgeDirectory))
        {
            throw new ArgumentException("Bridge directory is required.", nameof(bridgeDirectory));
        }

        if (string.IsNullOrWhiteSpace(gameVersion))
        {
            throw new ArgumentException("Game version is required.", nameof(gameVersion));
        }

        if (string.IsNullOrWhiteSpace(bridgeVersion))
        {
            throw new ArgumentException("Bridge version is required.", nameof(bridgeVersion));
        }

        if (!ProbeRequestAcceptance.TryValidateForCapture(request, out var requestError))
        {
            return ProbeCaptureResult.Failed(requestError!);
        }

        if (!_layouts.TryResolveFromGameVersion(gameVersion, out var layout))
        {
            return ProbeCaptureResult.Failed(
                $"unsupported FM version '{gameVersion}'; no layout for major.minor key");
        }

        if (cancellationToken.IsCancellationRequested)
        {
            return ProbeCaptureResult.Failed("probe capture cancelled");
        }

        var regions = RegionEnumerator.GetCandidateRegions(reader);
        var diagnostics = new ScanDiagnostics { GameVersion = gameVersion };
        var candidates = PersonScanner.Scan(
            reader,
            layout,
            gameAssembly,
            gamePlugin,
            regions,
            diagnostics,
            maxAccepted: null,
            cancellationToken);

        if (diagnostics.Cancelled || cancellationToken.IsCancellationRequested)
        {
            return ProbeCaptureResult.Failed("probe capture cancelled");
        }

        var candidatesByUid = candidates.ToDictionary(candidate => candidate.Uid);
        var missing = request.Uids.Where(uid => !candidatesByUid.ContainsKey(uid)).ToArray();
        if (missing.Length > 0)
        {
            return ProbeCaptureResult.Failed(
                $"requested player UIDs were missing after scanner deduplication: {string.Join(", ", missing)}");
        }

        var capturedRoots = new List<CapturedPlayerRoots>(request.Uids.Length);
        foreach (var uid in request.Uids)
        {
            if (cancellationToken.IsCancellationRequested)
            {
                return ProbeCaptureResult.Failed("probe capture cancelled");
            }

            var candidate = candidatesByUid[uid];
            if (candidate.ClassOffset <= 0 || (ulong)candidate.ClassOffset > candidate.ObjectAddress)
            {
                return ProbeCaptureResult.Failed($"player {uid} had an invalid class offset");
            }

            var playerBlockAddress = candidate.ObjectAddress - (ulong)candidate.ClassOffset;
            var roots = CaptureRoots(
                reader,
                candidate,
                playerBlockAddress,
                cancellationToken,
                out var captureError);
            if (roots is null)
            {
                return ProbeCaptureResult.Failed(captureError!);
            }

            capturedRoots.Add(roots);
        }

        if (cancellationToken.IsCancellationRequested)
        {
            return ProbeCaptureResult.Failed("probe capture cancelled");
        }

        var firstHopPlan = BuildFirstHopPlan(
            capturedRoots,
            regions,
            cancellationToken,
            out var planError);
        if (firstHopPlan is null)
        {
            return ProbeCaptureResult.Failed(planError!);
        }

        var players = new List<ProbePlayer>(capturedRoots.Count);
        foreach (var roots in capturedRoots)
        {
            var capture = CaptureFirstHopTargets(
                reader,
                roots,
                firstHopPlan,
                regions,
                cancellationToken,
                out var captureError);
            if (capture is null)
            {
                return ProbeCaptureResult.Failed(captureError!);
            }

            players.Add(capture);
        }

        if (cancellationToken.IsCancellationRequested)
        {
            return ProbeCaptureResult.Failed("probe capture cancelled");
        }

        var document = new ProbeDocument
        {
            SchemaVersion = ProbeProtocol.SchemaVersion,
            GeneratedAtUtc = DateTimeOffset.UtcNow.ToString("O"),
            GameVersion = gameVersion,
            SupportedGameVersion = layout.VersionKey,
            BridgeVersion = bridgeVersion,
            ProtocolVersion = ProbeProtocol.ProtocolVersion,
            RequestId = request.RequestId,
            RequestedUids = request.Uids,
            GameAssembly = ToProbeModule(gameAssembly),
            GamePlugin = gamePlugin is { } gamePluginBounds ? ToProbeModule(gamePluginBounds) : null,
            PlayerCount = players.Count,
            CapturePolicy = CreateCapturePolicy(firstHopPlan),
            Players = players,
        };

        try
        {
            if (!ProbeWriter.TryWriteReplaceOnSuccess(bridgeDirectory, document))
            {
                return ProbeCaptureResult.Failed("probe write did not replace file");
            }
        }
        catch (Exception ex)
        {
            return ProbeCaptureResult.Failed($"probe write failed: {ex.Message}");
        }

        return ProbeCaptureResult.Succeeded(document);
    }

    private static CapturedPlayerRoots? CaptureRoots(
        IMemoryReader reader,
        PersonCandidate candidate,
        ulong playerBlockAddress,
        CancellationToken cancellationToken,
        out string? error)
    {
        error = null;
        var playerRoot = CaptureRange(
            reader,
            playerBlockAddress,
            ProbeCaptureLimits.PlayerRootWindowBytes,
            addressBasis: "player-block",
            relativePath: "player-block+0x0",
            sourcePointerPath: null,
            pointerDepth: 0,
            cancellationToken: cancellationToken,
            error: out error);
        if (playerRoot is null)
        {
            return null;
        }

        var personRoot = CaptureRange(
            reader,
            candidate.ObjectAddress,
            ProbeCaptureLimits.PersonRootWindowBytes,
            addressBasis: "person-object",
            relativePath: "person-object+0x0",
            sourcePointerPath: null,
            pointerDepth: 0,
            cancellationToken: cancellationToken,
            error: out error);
        if (personRoot is null)
        {
            return null;
        }

        return new CapturedPlayerRoots(candidate, playerBlockAddress, playerRoot, personRoot);
    }

    private static IReadOnlyList<PlannedPointerPath>? BuildFirstHopPlan(
        IReadOnlyList<CapturedPlayerRoots> players,
        IReadOnlyList<MemoryRegion> acceptableRegions,
        CancellationToken cancellationToken,
        out string? error)
    {
        error = null;
        var playerPaths = SelectPaths(
            players,
            addressBasis: "player-block",
            maxPaths: ProbeCaptureLimits.MaxPlayerRootFirstHopPaths,
            acceptableRegions,
            cancellationToken,
            out error);
        if (playerPaths is null)
        {
            return null;
        }

        var personPaths = SelectPaths(
            players,
            addressBasis: "person-object",
            maxPaths: ProbeCaptureLimits.MaxPersonRootFirstHopPaths,
            acceptableRegions,
            cancellationToken,
            out error);
        if (personPaths is null)
        {
            return null;
        }

        return playerPaths.Concat(personPaths).ToArray();
    }

    private static IReadOnlyList<PlannedPointerPath>? SelectPaths(
        IReadOnlyList<CapturedPlayerRoots> players,
        string addressBasis,
        int maxPaths,
        IReadOnlyList<MemoryRegion> acceptableRegions,
        CancellationToken cancellationToken,
        out string? error)
    {
        error = null;
        var availability = new Dictionary<string, PathAvailability>(StringComparer.Ordinal);
        foreach (var player in players)
        {
            var root = player.GetRoot(addressBasis);
            foreach (var pointer in root.FindReadablePointers())
            {
                if (cancellationToken.IsCancellationRequested)
                {
                    error = "probe capture cancelled";
                    return null;
                }

                if (!IsSafePointerTarget(pointer.TargetAddress, acceptableRegions))
                {
                    continue;
                }

                var sourcePath = root.PathAt(pointer.Offset);
                if (!availability.TryGetValue(sourcePath, out var path))
                {
                    path = new PathAvailability(addressBasis, sourcePath, pointer.Offset);
                    availability.Add(sourcePath, path);
                }

                path.EligibleUids.Add(player.Candidate.Uid);
            }
        }

        return availability.Values
            .OrderByDescending(path => path.EligibleUids.Count)
            .ThenBy(path => path.SourceOffset)
            .ThenBy(path => path.SourcePointerPath, StringComparer.Ordinal)
            .Take(maxPaths)
            .Select(
                path => new PlannedPointerPath(
                    path.AddressBasis,
                    path.SourcePointerPath,
                    path.SourceOffset,
                    ProbeCaptureLimits.MaxPointerDepth,
                    path.EligibleUids.Count))
            .ToArray();
    }

    private static ProbePlayer? CaptureFirstHopTargets(
        IMemoryReader reader,
        CapturedPlayerRoots roots,
        IReadOnlyList<PlannedPointerPath> firstHopPlan,
        IReadOnlyList<MemoryRegion> acceptableRegions,
        CancellationToken cancellationToken,
        out string? error)
    {
        error = null;
        var ranges = new List<CapturedRange> { roots.PlayerRoot, roots.PersonRoot };
        var seenTargets = new HashSet<ulong>();
        foreach (var path in firstHopPlan)
        {
            if (cancellationToken.IsCancellationRequested)
            {
                error = "probe capture cancelled";
                return null;
            }

            var root = roots.GetRoot(path.AddressBasis);
            if (!root.TryGetReadablePointer(path.SourceOffset, out var pointer)
                || !IsSafePointerTarget(pointer.TargetAddress, acceptableRegions)
                || !seenTargets.Add(pointer.TargetAddress))
            {
                continue;
            }

            var targetRange = CaptureRange(
                reader,
                pointer.TargetAddress,
                ProbeCaptureLimits.PointerTargetWindowBytes,
                addressBasis: "pointer-target",
                relativePath: path.SourcePointerPath + "->target+0x0",
                sourcePointerPath: path.SourcePointerPath,
                pointerDepth: path.PointerDepth,
                cancellationToken: cancellationToken,
                error: out error);
            if (targetRange is null)
            {
                return null;
            }

            ranges.Add(targetRange);
        }

        var requestedBytes = ranges.Sum(range => range.RequestedLength);
        if (requestedBytes > ProbeCaptureLimits.MaxBytesPerPlayer)
        {
            throw new InvalidOperationException("probe capture exceeded its per-player byte ceiling");
        }

        return new ProbePlayer
        {
            Uid = roots.Candidate.Uid,
            CandidateAddress = roots.Candidate.ObjectAddress,
            ClassOffset = roots.Candidate.ClassOffset,
            PlayerBlockAddress = roots.PlayerBlockAddress,
            RequestedBytes = requestedBytes,
            ReadableBytes = ranges.Sum(range => range.ReadableByteCount),
            Ranges = ranges.Select(range => range.ToDocumentRange()).ToArray(),
        };
    }

    private static ProbeCapturePolicy CreateCapturePolicy(IReadOnlyList<PlannedPointerPath> firstHopPlan) =>
        new()
        {
            MaxPointerDepth = ProbeCaptureLimits.MaxPointerDepth,
            TargetWindowBytes = ProbeCaptureLimits.PointerTargetWindowBytes,
            MaxBytesPerPlayer = ProbeCaptureLimits.MaxBytesPerPlayer,
            MaxBytesPerRequest = ProbeCaptureLimits.MaxBytesPerRequest,
            PathQuotas = new[]
            {
                new ProbePointerPathQuota
                {
                    AddressBasis = "player-block",
                    PointerDepth = ProbeCaptureLimits.MaxPointerDepth,
                    MaxPaths = ProbeCaptureLimits.MaxPlayerRootFirstHopPaths,
                },
                new ProbePointerPathQuota
                {
                    AddressBasis = "person-object",
                    PointerDepth = ProbeCaptureLimits.MaxPointerDepth,
                    MaxPaths = ProbeCaptureLimits.MaxPersonRootFirstHopPaths,
                },
            },
            SelectedPaths = firstHopPlan
                .Select(
                    path => new ProbeSelectedPointerPath
                    {
                        AddressBasis = path.AddressBasis,
                        SourcePointerPath = path.SourcePointerPath,
                        PointerDepth = path.PointerDepth,
                        EligiblePlayerCount = path.EligiblePlayerCount,
                    })
                .ToArray(),
        };

    private static bool IsSafePointerTarget(ulong targetAddress, IReadOnlyList<MemoryRegion> acceptableRegions) =>
        targetAddress != 0
        && targetAddress % (ulong)sizeof(ulong) == 0
        && IsWindowInsideAcceptableRegion(
            targetAddress,
            ProbeCaptureLimits.PointerTargetWindowBytes,
            acceptableRegions);

    private static CapturedRange? CaptureRange(
        IMemoryReader reader,
        ulong address,
        int length,
        string addressBasis,
        string relativePath,
        string? sourcePointerPath,
        int pointerDepth,
        CancellationToken cancellationToken,
        out string? error)
    {
        error = null;
        if (address > ulong.MaxValue - (ulong)length)
        {
            throw new ArgumentOutOfRangeException(nameof(address));
        }

        var bytes = new byte[length];
        var readable = new bool[length];
        for (var offset = 0; offset < length; offset++)
        {
            if (cancellationToken.IsCancellationRequested)
            {
                error = "probe capture cancelled";
                return null;
            }

            if (!reader.TryReadByte(address + (ulong)offset, out var value))
            {
                error = $"probe range {relativePath} had unread bytes";
                return null;
            }

            bytes[offset] = value;
            readable[offset] = true;
        }

        return new CapturedRange(
            address,
            length,
            addressBasis,
            relativePath,
            sourcePointerPath,
            pointerDepth,
            bytes,
            readable);
    }

    private static bool IsWindowInsideAcceptableRegion(
        ulong address,
        int length,
        IReadOnlyList<MemoryRegion> acceptableRegions)
    {
        foreach (var region in acceptableRegions)
        {
            if (region.Size < (ulong)length || address < region.BaseAddress)
            {
                continue;
            }

            if (address - region.BaseAddress <= region.Size - (ulong)length)
            {
                return true;
            }
        }

        return false;
    }

    private static ProbeModule ToProbeModule(ModuleBounds module) =>
        new()
        {
            Name = module.ModuleName,
            BaseAddress = module.BaseAddress,
            EndAddress = module.EndAddress,
        };

    private sealed class CapturedPlayerRoots
    {
        public CapturedPlayerRoots(
            PersonCandidate candidate,
            ulong playerBlockAddress,
            CapturedRange playerRoot,
            CapturedRange personRoot)
        {
            Candidate = candidate;
            PlayerBlockAddress = playerBlockAddress;
            PlayerRoot = playerRoot;
            PersonRoot = personRoot;
        }

        public PersonCandidate Candidate { get; }

        public ulong PlayerBlockAddress { get; }

        public CapturedRange PlayerRoot { get; }

        public CapturedRange PersonRoot { get; }

        public CapturedRange GetRoot(string addressBasis) => addressBasis switch
        {
            "player-block" => PlayerRoot,
            "person-object" => PersonRoot,
            _ => throw new ArgumentOutOfRangeException(nameof(addressBasis)),
        };
    }

    private sealed class PathAvailability
    {
        public PathAvailability(string addressBasis, string sourcePointerPath, int sourceOffset)
        {
            AddressBasis = addressBasis;
            SourcePointerPath = sourcePointerPath;
            SourceOffset = sourceOffset;
        }

        public string AddressBasis { get; }

        public string SourcePointerPath { get; }

        public int SourceOffset { get; }

        public HashSet<uint> EligibleUids { get; } = new();
    }

    private sealed record PlannedPointerPath(
        string AddressBasis,
        string SourcePointerPath,
        int SourceOffset,
        int PointerDepth,
        int EligiblePlayerCount);

    private sealed class CapturedRange
    {
        private readonly byte[] _bytes;
        private readonly bool[] _readable;

        public CapturedRange(
            ulong address,
            int requestedLength,
            string addressBasis,
            string relativePath,
            string? sourcePointerPath,
            int pointerDepth,
            byte[] bytes,
            bool[] readable)
        {
            Address = address;
            RequestedLength = requestedLength;
            AddressBasis = addressBasis;
            RelativePath = relativePath;
            SourcePointerPath = sourcePointerPath;
            PointerDepth = pointerDepth;
            _bytes = bytes;
            _readable = readable;
        }

        public ulong Address { get; }

        public int RequestedLength { get; }

        public string AddressBasis { get; }

        public string RelativePath { get; }

        public string? SourcePointerPath { get; }

        public int PointerDepth { get; }

        public int ReadableByteCount => _readable.Count(value => value);

        public IEnumerable<ReadablePointer> FindReadablePointers()
        {
            for (var offset = 0; offset <= RequestedLength - sizeof(ulong); offset++)
            {
                if ((Address + (ulong)offset) % (ulong)sizeof(ulong) != 0 || !IsFullyReadable(offset, sizeof(ulong)))
                {
                    continue;
                }

                yield return new ReadablePointer(
                    offset,
                    BinaryPrimitives.ReadUInt64LittleEndian(_bytes.AsSpan(offset, sizeof(ulong))));
            }
        }

        public string PathAt(int offset)
        {
            const string rootSuffix = "+0x0";
            if (RelativePath.EndsWith(rootSuffix, StringComparison.Ordinal))
            {
                return RelativePath[..^rootSuffix.Length] + $"+0x{offset:X}";
            }

            return $"{RelativePath}+0x{offset:X}";
        }

        public bool TryGetReadablePointer(int offset, out ReadablePointer pointer)
        {
            if (offset < 0
                || offset > RequestedLength - sizeof(ulong)
                || (Address + (ulong)offset) % (ulong)sizeof(ulong) != 0
                || !IsFullyReadable(offset, sizeof(ulong)))
            {
                pointer = default;
                return false;
            }

            pointer = new ReadablePointer(
                offset,
                BinaryPrimitives.ReadUInt64LittleEndian(_bytes.AsSpan(offset, sizeof(ulong))));
            return true;
        }

        public ProbeMemoryRange ToDocumentRange()
        {
            var readableSpans = new List<ProbeReadableSpan>();
            for (var offset = 0; offset < RequestedLength;)
            {
                if (!_readable[offset])
                {
                    offset++;
                    continue;
                }

                var start = offset;
                while (offset < RequestedLength && _readable[offset])
                {
                    offset++;
                }

                readableSpans.Add(
                    new ProbeReadableSpan
                    {
                        Offset = start,
                        BytesBase64 = Convert.ToBase64String(_bytes, start, offset - start),
                    });
            }

            return new ProbeMemoryRange
            {
                AddressBasis = AddressBasis,
                RelativePath = RelativePath,
                SourcePointerPath = SourcePointerPath,
                Address = Address,
                RequestedLength = RequestedLength,
                PointerDepth = PointerDepth,
                ReadableSpans = readableSpans,
            };
        }

        private bool IsFullyReadable(int offset, int length)
        {
            for (var index = offset; index < offset + length; index++)
            {
                if (!_readable[index])
                {
                    return false;
                }
            }

            return true;
        }
    }

    private readonly record struct ReadablePointer(int Offset, ulong TargetAddress);
}

public readonly record struct ProbeCaptureResult(bool Success, string? Error, ProbeDocument? Document)
{
    public static ProbeCaptureResult Succeeded(ProbeDocument document) => new(true, null, document);

    public static ProbeCaptureResult Failed(string error) => new(false, error, null);
}
