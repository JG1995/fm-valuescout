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
    public const int MaxPointerTargetsPerPlayer = 4;
    public const int MaxBytesPerPlayer = PlayerRootWindowBytes
        + PersonRootWindowBytes
        + (MaxPointerTargetsPerPlayer * PointerTargetWindowBytes);
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

        var players = new List<ProbePlayer>(request.Uids.Length);
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
            var capture = CapturePlayer(
                reader,
                candidate,
                playerBlockAddress,
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

    private static ProbePlayer? CapturePlayer(
        IMemoryReader reader,
        PersonCandidate candidate,
        ulong playerBlockAddress,
        IReadOnlyList<MemoryRegion> acceptableRegions,
        CancellationToken cancellationToken,
        out string? error)
    {
        error = null;
        var ranges = new List<CapturedRange>();
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

        ranges.Add(playerRoot);
        ranges.Add(personRoot);

        if (cancellationToken.IsCancellationRequested)
        {
            error = "probe capture cancelled";
            return null;
        }

        var seenTargets = new HashSet<ulong>();
        var pointerTargets = 0;
        foreach (var root in ranges.ToArray())
        {
            foreach (var pointer in root.FindReadablePointers())
            {
                if (cancellationToken.IsCancellationRequested)
                {
                    error = "probe capture cancelled";
                    return null;
                }

                if (pointerTargets >= ProbeCaptureLimits.MaxPointerTargetsPerPlayer)
                {
                    break;
                }

                if (pointer.TargetAddress == 0
                    || pointer.TargetAddress % (ulong)sizeof(ulong) != 0
                    || !seenTargets.Add(pointer.TargetAddress)
                    || !IsWindowInsideAcceptableRegion(
                        pointer.TargetAddress,
                        ProbeCaptureLimits.PointerTargetWindowBytes,
                        acceptableRegions))
                {
                    continue;
                }

                var sourcePath = $"{root.RelativePath}+0x{pointer.Offset:X}";
                var targetRange = CaptureRange(
                    reader,
                    pointer.TargetAddress,
                    ProbeCaptureLimits.PointerTargetWindowBytes,
                    addressBasis: "pointer-target",
                    relativePath: sourcePath + "->target+0x0",
                    sourcePointerPath: sourcePath,
                    pointerDepth: ProbeCaptureLimits.MaxPointerDepth,
                    cancellationToken: cancellationToken,
                    error: out error);
                if (targetRange is null)
                {
                    return null;
                }

                ranges.Add(targetRange);
                pointerTargets++;
            }
        }

        var requestedBytes = ranges.Sum(range => range.RequestedLength);
        if (requestedBytes > ProbeCaptureLimits.MaxBytesPerPlayer)
        {
            throw new InvalidOperationException("probe capture exceeded its per-player byte ceiling");
        }

        return new ProbePlayer
        {
            Uid = candidate.Uid,
            CandidateAddress = candidate.ObjectAddress,
            ClassOffset = candidate.ClassOffset,
            PlayerBlockAddress = playerBlockAddress,
            RequestedBytes = requestedBytes,
            ReadableBytes = ranges.Sum(range => range.ReadableByteCount),
            Ranges = ranges.Select(range => range.ToDocumentRange()).ToArray(),
        };
    }

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
