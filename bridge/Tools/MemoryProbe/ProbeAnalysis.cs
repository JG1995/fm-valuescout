using System.Buffers.Binary;
using System.Globalization;
using System.Text.Json;
using FmDataBridge.Models;
using FmDataBridge.Protocol;

namespace FmDataBridge.MemoryProbe;

internal sealed class ProbeCapture
{
    private const int MaximumCaptureFileBytes = 4 * 1024 * 1024;
    private const int MaximumRangeBytes = 1024 * 1024;

    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        PropertyNameCaseInsensitive = true,
    };

    private ProbeCapture(
        ProbeDocument document,
        DateTimeOffset generatedAtUtc,
        IReadOnlyDictionary<uint, CapturedPlayer> players,
        int unreadableRangeCount)
    {
        Document = document;
        GeneratedAtUtc = generatedAtUtc;
        Players = players;
        UnreadableRangeCount = unreadableRangeCount;
    }

    public ProbeDocument Document { get; }

    public DateTimeOffset GeneratedAtUtc { get; }

    public IReadOnlyDictionary<uint, CapturedPlayer> Players { get; }

    public int UnreadableRangeCount { get; }

    public static ProbeCapture Load(string path)
    {
        if (!File.Exists(path))
        {
            throw new MemoryProbeException($"probe capture does not exist: {path}");
        }

        var fileInfo = new FileInfo(path);
        if (fileInfo.Length > MaximumCaptureFileBytes)
        {
            throw new MemoryProbeException($"probe capture exceeds the {MaximumCaptureFileBytes} byte analysis limit");
        }

        ProbeDocument? document;
        try
        {
            document = JsonSerializer.Deserialize<ProbeDocument>(File.ReadAllText(path), SerializerOptions);
        }
        catch (Exception ex) when (ex is JsonException or IOException or UnauthorizedAccessException)
        {
            throw new MemoryProbeException($"could not read probe capture '{path}': {ex.Message}");
        }

        if (document is null)
        {
            throw new MemoryProbeException($"probe capture '{path}' is empty");
        }

        ValidateDocument(document);
        if (!DateTimeOffset.TryParse(
                document.GeneratedAtUtc,
                CultureInfo.InvariantCulture,
                DateTimeStyles.AllowWhiteSpaces | DateTimeStyles.AssumeUniversal | DateTimeStyles.AdjustToUniversal,
                out var generatedAtUtc))
        {
            throw new MemoryProbeException("probe capture has an invalid generatedAtUtc timestamp");
        }

        var players = new Dictionary<uint, CapturedPlayer>();
        var unreadableRangeCount = 0;
        foreach (var player in document.Players)
        {
            var capturedPlayer = CapturedPlayer.Create(player);
            if (!players.TryAdd(player.Uid, capturedPlayer))
            {
                throw new MemoryProbeException($"probe capture has duplicate player UID {player.Uid}");
            }

            unreadableRangeCount += capturedPlayer.Ranges.Count(range => !range.IsFullyReadable);
        }

        var requestedUids = document.RequestedUids.ToHashSet();
        if (!requestedUids.SetEquals(players.Keys))
        {
            throw new MemoryProbeException(
                MemoryProbeCli.DescribeUidDifference(requestedUids, players.Keys.ToHashSet(), "probe request", "probe players"));
        }

        return new ProbeCapture(document, generatedAtUtc, players, unreadableRangeCount);
    }

    public void RequireUids(IEnumerable<uint> expectedUids, string expectedName)
    {
        var expected = expectedUids.ToHashSet();
        if (expected.Count == 0)
        {
            throw new MemoryProbeException($"{expectedName} does not contain any player UIDs");
        }

        if (!expected.SetEquals(Players.Keys))
        {
            throw new MemoryProbeException(
                MemoryProbeCli.DescribeUidDifference(expected, Players.Keys.ToHashSet(), expectedName, "probe capture"));
        }
    }

    public static void RequireCompatible(ProbeCapture before, ProbeCapture after)
    {
        ArgumentNullException.ThrowIfNull(before);
        ArgumentNullException.ThrowIfNull(after);

        if (string.Equals(before.Document.RequestId, after.Document.RequestId, StringComparison.Ordinal))
        {
            throw new MemoryProbeException("before and after captures have the same request ID");
        }

        RequireSameMetadata("gameVersion", before.Document.GameVersion, after.Document.GameVersion);
        RequireSameMetadata("supportedGameVersion", before.Document.SupportedGameVersion, after.Document.SupportedGameVersion);
        RequireSameMetadata("bridgeVersion", before.Document.BridgeVersion, after.Document.BridgeVersion);
        if (!before.Players.Keys.ToHashSet().SetEquals(after.Players.Keys))
        {
            throw new MemoryProbeException(
                MemoryProbeCli.DescribeUidDifference(
                    before.Players.Keys.ToHashSet(),
                    after.Players.Keys.ToHashSet(),
                    "before capture",
                    "after capture"));
        }
    }

    private static void RequireSameMetadata(string name, string before, string after)
    {
        if (!string.Equals(before, after, StringComparison.Ordinal))
        {
            throw new MemoryProbeException($"incompatible capture metadata: {name} differs ('{before}' vs '{after}')");
        }
    }

    private static void ValidateDocument(ProbeDocument document)
    {
        if (document.SchemaVersion != ProbeProtocol.SchemaVersion)
        {
            throw new MemoryProbeException(
                $"unsupported probe schema version {document.SchemaVersion}; expected {ProbeProtocol.SchemaVersion}");
        }

        if (document.ProtocolVersion != ProbeProtocol.ProtocolVersion)
        {
            throw new MemoryProbeException(
                $"unsupported probe protocol version {document.ProtocolVersion}; expected {ProbeProtocol.ProtocolVersion}");
        }

        if (string.IsNullOrWhiteSpace(document.RequestId)
            || string.IsNullOrWhiteSpace(document.GameVersion)
            || string.IsNullOrWhiteSpace(document.SupportedGameVersion)
            || string.IsNullOrWhiteSpace(document.BridgeVersion))
        {
            throw new MemoryProbeException("probe capture is missing required request or build metadata");
        }

        if (document.RequestedUids is null || document.Players is null || document.PlayerCount != document.Players.Count)
        {
            throw new MemoryProbeException("probe capture player count or UID metadata is invalid");
        }

        if (document.PlayerCount == 0 || document.RequestedUids.Count != document.PlayerCount)
        {
            throw new MemoryProbeException("probe capture must contain one player for every requested UID");
        }

        var requestedUids = new HashSet<uint>();
        foreach (var uid in document.RequestedUids)
        {
            if (uid == 0 || uid == uint.MaxValue || !requestedUids.Add(uid))
            {
                throw new MemoryProbeException("probe capture has invalid or duplicate requested UIDs");
            }
        }
    }

    internal sealed class CapturedPlayer
    {
        private CapturedPlayer(uint uid, IReadOnlyList<CapturedRange> ranges, IReadOnlyDictionary<string, CapturedRange> rangesByKey)
        {
            Uid = uid;
            Ranges = ranges;
            RangesByKey = rangesByKey;
        }

        public uint Uid { get; }

        public IReadOnlyList<CapturedRange> Ranges { get; }

        public IReadOnlyDictionary<string, CapturedRange> RangesByKey { get; }

        public static CapturedPlayer Create(ProbePlayer player)
        {
            if (player.Uid == 0 || player.Uid == uint.MaxValue || player.Ranges is null || player.Ranges.Count == 0)
            {
                throw new MemoryProbeException($"probe player {player.Uid} has no valid memory ranges");
            }

            var ranges = new List<CapturedRange>();
            var rangesByKey = new Dictionary<string, CapturedRange>(StringComparer.Ordinal);
            var relativePaths = new HashSet<string>(StringComparer.Ordinal);
            var requestedBytes = 0;
            var readableBytes = 0;
            foreach (var range in player.Ranges)
            {
                var capturedRange = CapturedRange.Create(player.Uid, range);
                if (!relativePaths.Add(capturedRange.RelativePath) || !rangesByKey.TryAdd(capturedRange.Key, capturedRange))
                {
                    throw new MemoryProbeException($"probe player {player.Uid} has duplicate candidate path '{capturedRange.RelativePath}'");
                }

                ranges.Add(capturedRange);
                requestedBytes = checked(requestedBytes + capturedRange.RequestedLength);
                readableBytes = checked(readableBytes + capturedRange.ReadableBytes);
            }

            if (player.RequestedBytes != requestedBytes || player.ReadableBytes != readableBytes)
            {
                throw new MemoryProbeException($"probe player {player.Uid} has inconsistent range byte totals");
            }

            return new CapturedPlayer(player.Uid, ranges, rangesByKey);
        }
    }

    internal sealed class CapturedRange
    {
        private CapturedRange(
            string addressBasis,
            string relativePath,
            string? sourcePointerPath,
            int pointerDepth,
            byte[] bytes,
            bool[] readable)
        {
            AddressBasis = addressBasis;
            RelativePath = relativePath;
            SourcePointerPath = sourcePointerPath;
            PointerDepth = pointerDepth;
            Bytes = bytes;
            Readable = readable;
            Key = string.Join(
                "\u001F",
                addressBasis,
                relativePath,
                sourcePointerPath ?? string.Empty,
                pointerDepth.ToString(),
                bytes.Length.ToString());
        }

        public string AddressBasis { get; }

        public string RelativePath { get; }

        public string? SourcePointerPath { get; }

        public int PointerDepth { get; }

        public byte[] Bytes { get; }

        public bool[] Readable { get; }

        public string Key { get; }

        public int RequestedLength => Bytes.Length;

        public int ReadableBytes => Readable.Count(value => value);

        public bool IsFullyReadable => ReadableBytes == RequestedLength;

        public bool IsReadable(int offset, int length)
        {
            for (var index = offset; index < offset + length; index++)
            {
                if (!Readable[index])
                {
                    return false;
                }
            }

            return true;
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

        public static CapturedRange Create(uint uid, ProbeMemoryRange range)
        {
            if (range is null
                || string.IsNullOrWhiteSpace(range.AddressBasis)
                || string.IsNullOrWhiteSpace(range.RelativePath)
                || range.RequestedLength is <= 0 or > MaximumRangeBytes
                || range.PointerDepth < 0
                || range.ReadableSpans is null)
            {
                throw new MemoryProbeException($"probe player {uid} has an invalid memory range");
            }

            var bytes = new byte[range.RequestedLength];
            var readable = new bool[range.RequestedLength];
            if (range.ReadableSpans.Any(span => span is null))
            {
                throw new MemoryProbeException($"probe player {uid} has an invalid readable span");
            }

            foreach (var span in range.ReadableSpans.OrderBy(span => span.Offset))
            {
                if (span.Offset < 0 || string.IsNullOrEmpty(span.BytesBase64))
                {
                    throw new MemoryProbeException($"probe player {uid} has an invalid readable span");
                }

                if (span.BytesBase64.Length > ((range.RequestedLength - span.Offset + 2) / 3 * 4))
                {
                    throw new MemoryProbeException($"probe player {uid} readable span exceeds its range");
                }

                byte[] decoded;
                try
                {
                    decoded = Convert.FromBase64String(span.BytesBase64);
                }
                catch (FormatException)
                {
                    throw new MemoryProbeException($"probe player {uid} has invalid base64 memory bytes");
                }

                if (span.Offset > range.RequestedLength - decoded.Length)
                {
                    throw new MemoryProbeException($"probe player {uid} readable span exceeds its range");
                }

                for (var index = 0; index < decoded.Length; index++)
                {
                    var destination = span.Offset + index;
                    if (readable[destination])
                    {
                        throw new MemoryProbeException($"probe player {uid} has overlapping readable spans");
                    }

                    bytes[destination] = decoded[index];
                    readable[destination] = true;
                }
            }

            return new CapturedRange(
                range.AddressBasis,
                range.RelativePath,
                range.SourcePointerPath,
                range.PointerDepth,
                bytes,
                readable);
        }
    }
}

internal static class ProbeAnalysis
{
    private const int MaximumReportedCandidates = 50;

    private static readonly IReadOnlyList<ScalarEncoding> ExactEncodings = new ScalarEncoding[]
    {
        new("uint32-le", 4, 0, "exact", MatchesUInt32),
        new("int32-le", 4, 1, "exact", MatchesInt32),
        new("uint16-le", 2, 2, "exact", MatchesUInt16),
        new("int16-le", 2, 3, "exact", MatchesInt16),
        new("uint8-times-five", 1, 4, "exact", MatchesTimesFive),
        new("uint8", 1, 5, "exact", MatchesUInt8),
        new("int8", 1, 6, "exact", MatchesInt8),
    };

    public static CorrelationReport Correlate(
        ProbeCapture capture,
        CsvPlayerTable table,
        string capturePath,
        IReadOnlyList<FieldMapping> mappings)
    {
        var fields = mappings.Select(mapping => CorrelateField(capture, table, mapping)).ToArray();
        return new CorrelationReport(
            capturePath,
            capture.Document.RequestId,
            table.DelimiterName,
            table.Players.Count,
            capture.UnreadableRangeCount,
            fields);
    }

    public static DiffReport Diff(
        ProbeCapture before,
        ProbeCapture after,
        CsvPlayerTable beforeTable,
        CsvPlayerTable afterTable,
        string beforePath,
        string afterPath,
        IReadOnlyList<FieldMapping> mappings)
    {
        var uids = beforeTable.Players.Select(player => player.Uid).OrderBy(uid => uid).ToArray();
        var (changedBytes, unmatchedRanges) = CountRangeDifferences(before, after, uids);
        var fields = mappings.Select(mapping => DiffField(before, after, beforeTable, afterTable, mapping, uids)).ToArray();
        return new DiffReport(
            beforePath,
            afterPath,
            before.Document.RequestId,
            after.Document.RequestId,
            beforeTable.DelimiterName,
            afterTable.DelimiterName,
            changedBytes,
            unmatchedRanges,
            before.UnreadableRangeCount + after.UnreadableRangeCount,
            fields);
    }

    private static FieldReport CorrelateField(
        ProbeCapture capture,
        CsvPlayerTable table,
        FieldMapping mapping)
    {
        var eligiblePlayers = table.Players
            .Where(player => player.Values[mapping.Name] is not null)
            .ToArray();
        var eligibleUids = eligiblePlayers.Select(player => player.Uid).OrderBy(uid => uid).ToArray();
        var excludedUids = table.Players
            .Where(player => player.Values[mapping.Name] is null)
            .Select(player => player.Uid)
            .OrderBy(uid => uid)
            .ToArray();
        var encodings = GetEncodings(mapping.Normalization);
        var collector = new CandidateCollector();
        foreach (var csvPlayer in eligiblePlayers)
        {
            var expected = csvPlayer.Values[mapping.Name]!.Value;
            var capturePlayer = capture.Players[csvPlayer.Uid];
            foreach (var range in capturePlayer.Ranges)
            {
                FindMatches(range, expected, encodings, csvPlayer.Uid, collector);
            }
        }

        var evidenceSufficient = HasVariedMultiPlayerEvidence(
            eligiblePlayers.Select(player => player.Values[mapping.Name]!.Value),
            eligibleUids.Length);
        return BuildFieldReport(
            mapping,
            collector.Build(eligibleUids, mapping.Name),
            eligibleUids,
            excludedUids,
            evidenceSufficient);
    }

    private static DiffFieldReport DiffField(
        ProbeCapture before,
        ProbeCapture after,
        CsvPlayerTable beforeTable,
        CsvPlayerTable afterTable,
        FieldMapping mapping,
        IReadOnlyList<uint> uids)
    {
        var beforePlayers = beforeTable.Players.ToDictionary(player => player.Uid);
        var afterPlayers = afterTable.Players.ToDictionary(player => player.Uid);
        var eligibleUids = uids
            .Where(uid => beforePlayers[uid].Values[mapping.Name] is not null && afterPlayers[uid].Values[mapping.Name] is not null)
            .ToArray();
        var excludedUids = uids.Except(eligibleUids).ToArray();
        var beforeValues = eligibleUids.ToDictionary(uid => uid, uid => beforePlayers[uid].Values[mapping.Name]!.Value);
        var afterValues = eligibleUids.ToDictionary(uid => uid, uid => afterPlayers[uid].Values[mapping.Name]!.Value);
        var evidenceSufficient = HasVariedMultiPlayerEvidence(beforeValues.Values, eligibleUids.Length)
            || HasVariedMultiPlayerEvidence(afterValues.Values, eligibleUids.Length);
        var changedUids = eligibleUids.Where(uid => beforeValues[uid] != afterValues[uid]).ToArray();
        if (changedUids.Length == 0)
        {
            return new DiffFieldReport(
                mapping.Name,
                mapping.CsvColumn,
                mapping.Normalization.Name,
                "no-evidence",
                Array.Empty<uint>(),
                eligibleUids,
                eligibleUids,
                excludedUids,
                evidenceSufficient,
                0,
                0,
                0,
                false,
                Array.Empty<DiffCandidateReport>());
        }

        var collector = new CandidateCollector();
        var encodings = GetEncodings(mapping.Normalization);
        foreach (var uid in eligibleUids)
        {
            var beforePlayer = before.Players[uid];
            var afterPlayer = after.Players[uid];
            foreach (var pair in beforePlayer.RangesByKey)
            {
                if (!afterPlayer.RangesByKey.TryGetValue(pair.Key, out var afterRange))
                {
                    continue;
                }

                var beforeRange = pair.Value;
                foreach (var encoding in encodings)
                {
                    for (var offset = 0; offset <= beforeRange.RequestedLength - encoding.Width; offset++)
                    {
                        if (!beforeRange.IsReadable(offset, encoding.Width)
                            || !afterRange.IsReadable(offset, encoding.Width)
                            || !encoding.Matches(beforeRange.Bytes, offset, beforeValues[uid])
                            || !encoding.Matches(afterRange.Bytes, offset, afterValues[uid]))
                        {
                            continue;
                        }

                        if (beforeValues[uid] != afterValues[uid]
                            && !BytesChanged(beforeRange.Bytes, afterRange.Bytes, offset, encoding.Width))
                        {
                            continue;
                        }

                        collector.Add(beforeRange, offset, encoding, uid);
                    }
                }
            }
        }

        var candidates = collector.Build(eligibleUids, mapping.Name);
        var field = BuildFieldReport(mapping, candidates, eligibleUids, excludedUids, evidenceSufficient);
        var deltas = changedUids
            .Select(uid => new ScalarDelta(uid, beforeValues[uid], afterValues[uid]))
            .ToArray();
        var reportedCandidates = candidates
            .Take(MaximumReportedCandidates)
            .Select(candidate => new DiffCandidateReport(
                candidate.Path,
                candidate.AddressBasis,
                candidate.SourcePointerPath,
                candidate.PointerDepth,
                candidate.Encoding,
                candidate.EvidenceKind,
                candidate.MatchingEncodings,
                candidate.EncodingMatches,
                candidate.EncodingAmbiguous,
                candidate.Coverage,
                candidate.MatchedUids,
                candidate.MissingUids,
                candidate.DuplicatePathHits,
                deltas))
            .ToArray();
        return new DiffFieldReport(
            mapping.Name,
            mapping.CsvColumn,
            mapping.Normalization.Name,
            field.Outcome,
            changedUids,
            eligibleUids.Except(changedUids).ToArray(),
            eligibleUids,
            excludedUids,
            field.EvidenceSufficient,
            field.CandidateCount,
            field.TopCoverage,
            field.TopCandidateCount,
            field.CandidatesTruncated,
            reportedCandidates);
    }

    private static FieldReport BuildFieldReport(
        FieldMapping mapping,
        IReadOnlyList<RankedCandidate> candidates,
        IReadOnlyList<uint> eligibleUids,
        IReadOnlyList<uint> excludedUids,
        bool evidenceSufficient)
    {
        var requiredCoverage = eligibleUids.Count;
        var topCoverage = candidates.Count == 0 ? 0 : candidates[0].Coverage;
        var topCandidateCount = candidates.Count(candidate => candidate.Coverage == topCoverage);
        var outcome = candidates.Count == 0
            ? "no-evidence"
            : evidenceSufficient && topCoverage == requiredCoverage && topCandidateCount == 1
                ? "candidate"
                : "ambiguous";
        var reportedCandidates = candidates
            .Take(MaximumReportedCandidates)
            .Select(candidate => new CandidateReport(
                candidate.Path,
                candidate.AddressBasis,
                candidate.SourcePointerPath,
                candidate.PointerDepth,
                candidate.Encoding,
                candidate.EvidenceKind,
                candidate.MatchingEncodings,
                candidate.EncodingMatches,
                candidate.EncodingAmbiguous,
                candidate.Coverage,
                candidate.MatchedUids,
                candidate.MissingUids,
                candidate.DuplicatePathHits))
            .ToArray();
        return new FieldReport(
            mapping.Name,
            mapping.CsvColumn,
            mapping.Normalization.Name,
            outcome,
            eligibleUids,
            excludedUids,
            evidenceSufficient,
            candidates.Count,
            topCoverage,
            topCandidateCount,
            candidates.Count > MaximumReportedCandidates,
            reportedCandidates);
    }

    private static bool HasVariedMultiPlayerEvidence(IEnumerable<decimal> values, int playerCount) =>
        playerCount > 1 && values.Distinct().Skip(1).Any();

    private static void FindMatches(
        ProbeCapture.CapturedRange range,
        decimal expected,
        IReadOnlyList<ScalarEncoding> encodings,
        uint uid,
        CandidateCollector collector)
    {
        foreach (var encoding in encodings)
        {
            for (var offset = 0; offset <= range.RequestedLength - encoding.Width; offset++)
            {
                if (range.IsReadable(offset, encoding.Width) && encoding.Matches(range.Bytes, offset, expected))
                {
                    collector.Add(range, offset, encoding, uid);
                }
            }
        }
    }

    private static IReadOnlyList<ScalarEncoding> GetEncodings(FieldNormalization normalization)
    {
        if (normalization.Kind is FieldNormalizationKind.Integer
            or FieldNormalizationKind.AppearancesStarts
            or FieldNormalizationKind.AppearancesSubstitutes)
        {
            return ExactEncodings;
        }

        var scale = PowerOfTen(normalization.DecimalPlaces);
        var scaleName = scale.ToString("0", CultureInfo.InvariantCulture);
        return new ScalarEncoding[]
        {
            new(
                $"float32-le-rounded-{normalization.DecimalPlaces}",
                sizeof(float),
                7,
                "rounded",
                (bytes, offset, expected) => MatchesRoundedFloat32(bytes, offset, expected, normalization.DecimalPlaces)),
            new(
                $"float64-le-rounded-{normalization.DecimalPlaces}",
                sizeof(double),
                8,
                "rounded",
                (bytes, offset, expected) => MatchesRoundedFloat64(bytes, offset, expected, normalization.DecimalPlaces)),
            new(
                $"uint32-le-fixed-scale-{scaleName}",
                sizeof(uint),
                9,
                "fixed-scale",
                (bytes, offset, expected) => MatchesFixed(bytes, offset, expected, scale, MatchesUInt32)),
            new(
                $"int32-le-fixed-scale-{scaleName}",
                sizeof(int),
                10,
                "fixed-scale",
                (bytes, offset, expected) => MatchesFixed(bytes, offset, expected, scale, MatchesInt32)),
            new(
                $"uint16-le-fixed-scale-{scaleName}",
                sizeof(ushort),
                11,
                "fixed-scale",
                (bytes, offset, expected) => MatchesFixed(bytes, offset, expected, scale, MatchesUInt16)),
            new(
                $"int16-le-fixed-scale-{scaleName}",
                sizeof(short),
                12,
                "fixed-scale",
                (bytes, offset, expected) => MatchesFixed(bytes, offset, expected, scale, MatchesInt16)),
            new(
                $"uint8-fixed-scale-{scaleName}",
                sizeof(byte),
                13,
                "fixed-scale",
                (bytes, offset, expected) => MatchesFixed(bytes, offset, expected, scale, MatchesUInt8)),
            new(
                $"int8-fixed-scale-{scaleName}",
                sizeof(sbyte),
                14,
                "fixed-scale",
                (bytes, offset, expected) => MatchesFixed(bytes, offset, expected, scale, MatchesInt8)),
        };
    }

    private static (int ChangedBytes, int UnmatchedRanges) CountRangeDifferences(
        ProbeCapture before,
        ProbeCapture after,
        IReadOnlyList<uint> uids)
    {
        var changedBytes = 0;
        var unmatchedRanges = 0;
        foreach (var uid in uids)
        {
            var beforeRanges = before.Players[uid].RangesByKey;
            var afterRanges = after.Players[uid].RangesByKey;
            unmatchedRanges += beforeRanges.Keys.Except(afterRanges.Keys).Count();
            unmatchedRanges += afterRanges.Keys.Except(beforeRanges.Keys).Count();
            foreach (var pair in beforeRanges)
            {
                if (!afterRanges.TryGetValue(pair.Key, out var afterRange))
                {
                    continue;
                }

                var beforeRange = pair.Value;
                for (var offset = 0; offset < beforeRange.RequestedLength; offset++)
                {
                    if (beforeRange.Readable[offset]
                        && afterRange.Readable[offset]
                        && beforeRange.Bytes[offset] != afterRange.Bytes[offset])
                    {
                        changedBytes++;
                    }
                }
            }
        }

        return (changedBytes, unmatchedRanges);
    }

    private static bool BytesChanged(byte[] before, byte[] after, int offset, int length)
    {
        for (var index = offset; index < offset + length; index++)
        {
            if (before[index] != after[index])
            {
                return true;
            }
        }

        return false;
    }

    private static bool MatchesUInt32(byte[] bytes, int offset, decimal expected) =>
        IsWhole(expected)
        && expected >= uint.MinValue
        && expected <= uint.MaxValue
        && BinaryPrimitives.ReadUInt32LittleEndian(bytes.AsSpan(offset, sizeof(uint))) == (uint)expected;

    private static bool MatchesInt32(byte[] bytes, int offset, decimal expected) =>
        IsWhole(expected)
        && expected >= int.MinValue
        && expected <= int.MaxValue
        && BinaryPrimitives.ReadInt32LittleEndian(bytes.AsSpan(offset, sizeof(int))) == (int)expected;

    private static bool MatchesUInt16(byte[] bytes, int offset, decimal expected) =>
        IsWhole(expected)
        && expected >= ushort.MinValue
        && expected <= ushort.MaxValue
        && BinaryPrimitives.ReadUInt16LittleEndian(bytes.AsSpan(offset, sizeof(ushort))) == (ushort)expected;

    private static bool MatchesInt16(byte[] bytes, int offset, decimal expected) =>
        IsWhole(expected)
        && expected >= short.MinValue
        && expected <= short.MaxValue
        && BinaryPrimitives.ReadInt16LittleEndian(bytes.AsSpan(offset, sizeof(short))) == (short)expected;

    private static bool MatchesTimesFive(byte[] bytes, int offset, decimal expected) =>
        IsWhole(expected)
        && expected >= 0
        && expected <= byte.MaxValue / 5
        && bytes[offset] == (byte)(expected * 5);

    private static bool MatchesUInt8(byte[] bytes, int offset, decimal expected) =>
        IsWhole(expected) && expected >= byte.MinValue && expected <= byte.MaxValue && bytes[offset] == (byte)expected;

    private static bool MatchesInt8(byte[] bytes, int offset, decimal expected) =>
        IsWhole(expected)
        && expected >= sbyte.MinValue
        && expected <= sbyte.MaxValue
        && unchecked((sbyte)bytes[offset]) == (sbyte)expected;

    private static bool MatchesRoundedFloat32(byte[] bytes, int offset, decimal expected, int decimalPlaces) =>
        MatchesRounded(
            BitConverter.Int32BitsToSingle(BinaryPrimitives.ReadInt32LittleEndian(bytes.AsSpan(offset, sizeof(float)))),
            expected,
            decimalPlaces);

    private static bool MatchesRoundedFloat64(byte[] bytes, int offset, decimal expected, int decimalPlaces) =>
        MatchesRounded(
            BitConverter.Int64BitsToDouble(BinaryPrimitives.ReadInt64LittleEndian(bytes.AsSpan(offset, sizeof(double)))),
            expected,
            decimalPlaces);

    private static bool MatchesRounded(double value, decimal expected, int decimalPlaces)
    {
        if (double.IsNaN(value) || double.IsInfinity(value))
        {
            return false;
        }

        try
        {
            var halfUnit = 0.5m / PowerOfTen(decimalPlaces);
            var actual = (decimal)value;
            return actual >= expected - halfUnit && actual < expected + halfUnit;
        }
        catch (OverflowException)
        {
            return false;
        }
    }

    private static bool MatchesFixed(
        byte[] bytes,
        int offset,
        decimal expected,
        decimal scale,
        Func<byte[], int, decimal, bool> matchesScalar)
    {
        try
        {
            var scaled = expected * scale;
            return IsWhole(scaled) && matchesScalar(bytes, offset, scaled);
        }
        catch (OverflowException)
        {
            return false;
        }
    }

    private static bool IsWhole(decimal value) => decimal.Truncate(value) == value;

    private static decimal PowerOfTen(int decimalPlaces)
    {
        var scale = 1m;
        for (var index = 0; index < decimalPlaces; index++)
        {
            scale *= 10m;
        }

        return scale;
    }

    private static int EncodingRank(string metricName, ScalarEncoding encoding)
    {
        var normalized = metricName.Replace("-", string.Empty).Replace("_", string.Empty).ToLowerInvariant();
        var preferred = normalized switch
        {
            "uid" or "playeruid" => "uint32-le",
            "ca" or "currentability" or "pa" or "potentialability" => "uint16-le",
            "determination" => "uint8-times-five",
            "market" or "marketvalue" or "marketvaluegbp" => "uint32-le",
            _ => null,
        };
        return string.Equals(encoding.Name, preferred, StringComparison.Ordinal) ? -1 : encoding.Rank;
    }

    private sealed record ScalarEncoding(
        string Name,
        int Width,
        int Rank,
        string EvidenceKind,
        Func<byte[], int, decimal, bool> Matches);

    private sealed class CandidateCollector
    {
        private readonly Dictionary<string, LocationEvidence> _locations = new(StringComparer.Ordinal);

        public void Add(ProbeCapture.CapturedRange range, int offset, ScalarEncoding encoding, uint uid)
        {
            var key = $"{range.Key}\u001F{offset}";
            if (!_locations.TryGetValue(key, out var location))
            {
                location = new LocationEvidence(range, offset);
                _locations.Add(key, location);
            }

            location.Add(encoding, uid);
        }

        public IReadOnlyList<RankedCandidate> Build(IReadOnlyList<uint> allUids, string metricName)
        {
            return _locations.Values
                .Select(location => location.ToRankedCandidate(allUids, metricName))
                .OrderByDescending(candidate => candidate.Coverage)
                .ThenBy(candidate => candidate.EncodingRank)
                .ThenBy(candidate => candidate.Path, StringComparer.Ordinal)
                .ToArray();
        }
    }

    private sealed class LocationEvidence
    {
        private readonly ProbeCapture.CapturedRange _range;
        private readonly int _offset;
        private readonly Dictionary<string, EncodingEvidence> _encodings = new(StringComparer.Ordinal);

        public LocationEvidence(ProbeCapture.CapturedRange range, int offset)
        {
            _range = range;
            _offset = offset;
        }

        public void Add(ScalarEncoding encoding, uint uid)
        {
            if (!_encodings.TryGetValue(encoding.Name, out var evidence))
            {
                evidence = new EncodingEvidence(encoding);
                _encodings.Add(encoding.Name, evidence);
            }

            evidence.Add(uid);
        }

        public RankedCandidate ToRankedCandidate(IReadOnlyList<uint> allUids, string metricName)
        {
            var ordered = _encodings.Values
                .OrderByDescending(evidence => evidence.MatchedUids.Count)
                .ThenBy(evidence => EncodingRank(metricName, evidence.Encoding))
                .ToArray();
            var selected = ordered[0];
            var matchingEncodings = ordered
                .Where(evidence => evidence.MatchedUids.Count == selected.MatchedUids.Count)
                .OrderBy(evidence => EncodingRank(metricName, evidence.Encoding))
                .Select(evidence => evidence.Encoding.Name)
                .ToArray();
            var encodingMatches = ordered
                .Select(
                    evidence => new EncodingMatchReport(
                        evidence.Encoding.Name,
                        evidence.Encoding.EvidenceKind,
                        evidence.MatchedUids.Count,
                        evidence.MatchedUids.OrderBy(uid => uid).ToArray(),
                        evidence.DuplicatePathHits))
                .ToArray();
            var matchedUids = selected.MatchedUids.OrderBy(uid => uid).ToArray();
            var missingUids = allUids.Except(selected.MatchedUids).OrderBy(uid => uid).ToArray();
            return new RankedCandidate(
                _range.PathAt(_offset),
                _range.AddressBasis,
                _range.SourcePointerPath,
                _range.PointerDepth,
                selected.Encoding.Name,
                selected.Encoding.EvidenceKind,
                matchingEncodings,
                encodingMatches,
                matchingEncodings.Length > 1,
                EncodingRank(metricName, selected.Encoding),
                matchedUids.Length,
                matchedUids,
                missingUids,
                selected.DuplicatePathHits);
        }
    }

    private sealed class EncodingEvidence
    {
        public EncodingEvidence(ScalarEncoding encoding)
        {
            Encoding = encoding;
        }

        public ScalarEncoding Encoding { get; }

        public HashSet<uint> MatchedUids { get; } = new();

        public int DuplicatePathHits { get; private set; }

        public void Add(uint uid)
        {
            if (!MatchedUids.Add(uid))
            {
                DuplicatePathHits++;
            }
        }
    }

    private sealed record RankedCandidate(
        string Path,
        string AddressBasis,
        string? SourcePointerPath,
        int PointerDepth,
        string Encoding,
        string EvidenceKind,
        IReadOnlyList<string> MatchingEncodings,
        IReadOnlyList<EncodingMatchReport> EncodingMatches,
        bool EncodingAmbiguous,
        int EncodingRank,
        int Coverage,
        IReadOnlyList<uint> MatchedUids,
        IReadOnlyList<uint> MissingUids,
        int DuplicatePathHits);
}

internal sealed record CorrelationReport(
    string CapturePath,
    string RequestId,
    string Delimiter,
    int PlayerCount,
    int UnreadableRangeCount,
    IReadOnlyList<FieldReport> Fields)
{
    public string Kind => "correlation";
}

internal sealed record FieldReport(
    string Name,
    string CsvColumn,
    string Normalization,
    string Outcome,
    IReadOnlyList<uint> EligibleUids,
    IReadOnlyList<uint> ExcludedUids,
    bool EvidenceSufficient,
    int CandidateCount,
    int TopCoverage,
    int TopCandidateCount,
    bool CandidatesTruncated,
    IReadOnlyList<CandidateReport> Candidates);

internal sealed record CandidateReport(
    string Path,
    string AddressBasis,
    string? SourcePointerPath,
    int PointerDepth,
    string Encoding,
    string EvidenceKind,
    IReadOnlyList<string> MatchingEncodings,
    IReadOnlyList<EncodingMatchReport> EncodingMatches,
    bool EncodingAmbiguous,
    int Coverage,
    IReadOnlyList<uint> MatchedUids,
    IReadOnlyList<uint> MissingUids,
    int DuplicatePathHits);

internal sealed record DiffReport(
    string BeforeCapturePath,
    string AfterCapturePath,
    string BeforeRequestId,
    string AfterRequestId,
    string BeforeDelimiter,
    string AfterDelimiter,
    int ChangedByteCount,
    int UnmatchedRangeCount,
    int UnreadableRangeCount,
    IReadOnlyList<DiffFieldReport> Fields)
{
    public string Kind => "before-after-diff";
}

internal sealed record DiffFieldReport(
    string Name,
    string CsvColumn,
    string Normalization,
    string Outcome,
    IReadOnlyList<uint> ChangedUids,
    IReadOnlyList<uint> UnchangedUids,
    IReadOnlyList<uint> EligibleUids,
    IReadOnlyList<uint> ExcludedUids,
    bool EvidenceSufficient,
    int CandidateCount,
    int TopCoverage,
    int TopCandidateCount,
    bool CandidatesTruncated,
    IReadOnlyList<DiffCandidateReport> Candidates);

internal sealed record DiffCandidateReport(
    string Path,
    string AddressBasis,
    string? SourcePointerPath,
    int PointerDepth,
    string Encoding,
    string EvidenceKind,
    IReadOnlyList<string> MatchingEncodings,
    IReadOnlyList<EncodingMatchReport> EncodingMatches,
    bool EncodingAmbiguous,
    int Coverage,
    IReadOnlyList<uint> MatchedUids,
    IReadOnlyList<uint> MissingUids,
    int DuplicatePathHits,
    IReadOnlyList<ScalarDelta> Deltas);

internal sealed record EncodingMatchReport(
    string Encoding,
    string EvidenceKind,
    int Coverage,
    IReadOnlyList<uint> MatchedUids,
    int DuplicatePathHits);

internal sealed record ScalarDelta(uint Uid, decimal BeforeValue, decimal AfterValue);
