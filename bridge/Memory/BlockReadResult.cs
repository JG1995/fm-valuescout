namespace FmDataBridge.Memory;

public readonly record struct BlockReadRange(int Offset, int Length);

/// <summary>
/// Exact readable coverage for one caller-owned block read.
/// </summary>
public sealed class BlockReadResult
{
    private readonly IReadOnlyList<BlockReadRange> _readableRanges;

    private BlockReadResult(
        int requestedBytes,
        IReadOnlyList<BlockReadRange> readableRanges,
        int readableBytes,
        bool hasExactCoverage)
    {
        RequestedBytes = requestedBytes;
        _readableRanges = readableRanges;
        ReadableBytes = readableBytes;
        HasExactCoverage = hasExactCoverage;
    }

    public int RequestedBytes { get; }

    public IReadOnlyList<BlockReadRange> ReadableRanges => _readableRanges;

    public int ReadableBytes { get; }

    public bool HasExactCoverage { get; }

    public bool IsComplete => ReadableBytes == RequestedBytes;

    public int CountReadableBytes(int offset, int length)
    {
        if (offset < 0 || length < 0 || offset > RequestedBytes - length)
        {
            throw new ArgumentOutOfRangeException(nameof(length));
        }

        var end = offset + length;
        var readableBytes = 0;
        foreach (var range in _readableRanges)
        {
            var start = Math.Max(offset, range.Offset);
            var rangeEnd = range.Offset + range.Length;
            var overlapEnd = Math.Min(end, rangeEnd);
            if (overlapEnd > start)
            {
                readableBytes += overlapEnd - start;
            }
        }

        return readableBytes;
    }

    public static BlockReadResult Empty { get; } =
        new(0, Array.Empty<BlockReadRange>(), 0, hasExactCoverage: true);

    public static BlockReadResult FromReadablePrefix(
        int requestedBytes,
        int readableBytes,
        bool hasExactCoverage = true)
    {
        ValidateCounts(requestedBytes, readableBytes);
        var ranges = readableBytes == 0
            ? Array.Empty<BlockReadRange>()
            : new[] { new BlockReadRange(0, readableBytes) };
        return new BlockReadResult(requestedBytes, ranges, readableBytes, hasExactCoverage);
    }

    public static BlockReadResult FromSummary(int requestedBytes, int readableBytes) =>
        FromReadablePrefix(
            requestedBytes,
            readableBytes,
            hasExactCoverage: readableBytes is 0 || readableBytes == requestedBytes);

    public static BlockReadResult FromReadabilityMask(bool[] readable)
    {
        ArgumentNullException.ThrowIfNull(readable);
        var ranges = new List<BlockReadRange>();
        var readableBytes = 0;
        for (var offset = 0; offset < readable.Length;)
        {
            if (!readable[offset])
            {
                offset++;
                continue;
            }

            var start = offset;
            while (offset < readable.Length && readable[offset])
            {
                readableBytes++;
                offset++;
            }

            ranges.Add(new BlockReadRange(start, offset - start));
        }

        return new BlockReadResult(readable.Length, ranges, readableBytes, hasExactCoverage: true);
    }

    internal static BlockReadResult Combine(
        int requestedBytes,
        BlockReadResult left,
        BlockReadResult right,
        int rightOffset)
    {
        var ranges = new List<BlockReadRange>(left.ReadableRanges.Count + right.ReadableRanges.Count);
        ranges.AddRange(left.ReadableRanges);
        foreach (var range in right.ReadableRanges)
        {
            ranges.Add(new BlockReadRange(rightOffset + range.Offset, range.Length));
        }

        return new BlockReadResult(
            requestedBytes,
            ranges,
            left.ReadableBytes + right.ReadableBytes,
            left.HasExactCoverage && right.HasExactCoverage);
    }

    private static void ValidateCounts(int requestedBytes, int readableBytes)
    {
        if (requestedBytes < 0)
        {
            throw new ArgumentOutOfRangeException(nameof(requestedBytes));
        }

        if (readableBytes < 0 || readableBytes > requestedBytes)
        {
            throw new ArgumentOutOfRangeException(nameof(readableBytes));
        }
    }
}
