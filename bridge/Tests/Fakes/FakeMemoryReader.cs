using FmDataBridge.Memory;

namespace FmDataBridge.Tests.Fakes;

/// <summary>
/// In-memory <see cref="IMemoryReader"/> for unit tests — no process or Windows APIs.
/// </summary>
public sealed class FakeMemoryReader : IMemoryReader, IMemoryWriter
{
    private readonly List<MemoryRegion> _regions = new();
    private readonly List<(ulong Address, byte[] Bytes)> _segments = new();
    private readonly List<(ulong Address, ulong Size)> _unreadableRanges = new();

    public void AddRegion(MemoryRegion region) => _regions.Add(region);

    public bool SupportsConcurrentReads => true;

    /// <summary>
    /// Marks part of an added region unreadable while leaving its zero-filled bytes distinct from readable zero data.
    /// </summary>
    public void AddUnreadableRange(ulong address, ulong size)
    {
        if (size == 0)
        {
            throw new ArgumentOutOfRangeException(nameof(size));
        }

        _unreadableRanges.Add((address, size));
    }

    public void AddBytes(ulong address, byte[] bytes)
    {
        ArgumentNullException.ThrowIfNull(bytes);
        _segments.Add((address, bytes));
    }

    public IEnumerable<MemoryRegion> EnumerateRegions() => _regions;

    public bool TryRead(ulong address, Span<byte> destination, out int bytesRead)
    {
        if (destination.IsEmpty)
        {
            bytesRead = 0;
            return true;
        }

        var result = Fill(address, destination);
        bytesRead = result.ReadableBytes;
        return bytesRead == destination.Length;
    }

    public bool TryWriteByte(ulong address, byte value, out int bytesWritten) =>
        TryWriteScalar(address, new[] { value }, out bytesWritten);

    public bool TryWriteUInt16(ulong address, ushort value, out int bytesWritten)
    {
        Span<byte> bytes = stackalloc byte[sizeof(ushort)];
        BitConverter.TryWriteBytes(bytes, value);
        return TryWriteScalar(address, bytes, out bytesWritten);
    }

    private bool TryWriteScalar(ulong address, ReadOnlySpan<byte> source, out int bytesWritten)
    {
        bytesWritten = 0;
        if (source.IsEmpty)
        {
            return true;
        }

        var destinations = new (byte[] Bytes, int Offset)[source.Length];
        for (var i = 0; i < source.Length; i++)
        {
            if (address > ulong.MaxValue - (ulong)i
                || !TryFindWritableByte(address + (ulong)i, out var bytes, out var offset))
            {
                return false;
            }

            destinations[i] = (bytes, offset);
        }

        for (var i = 0; i < source.Length; i++)
        {
            destinations[i].Bytes[destinations[i].Offset] = source[i];
        }

        bytesWritten = source.Length;
        return true;
    }

    public bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead)
    {
        var completed = TryReadBlockWithCoverage(address, buffer, offset, length, out var result);
        bytesRead = result.ReadableBytes;
        return completed;
    }

    public bool TryReadBlockWithCoverage(
        ulong address,
        byte[] buffer,
        int offset,
        int length,
        out BlockReadResult result) =>
        BlockReadHelper.TryFill(address, buffer, offset, length, out result, TryReadDirect);

    private BlockReadResult TryReadDirect(ulong address, byte[] buffer, int offset, int length)
    {
        if (length == 0)
        {
            return BlockReadResult.Empty;
        }

        return Fill(address, buffer.AsSpan(offset, length));
    }

    private BlockReadResult Fill(ulong address, Span<byte> destination)
    {
        destination.Clear();
        var readable = new bool[destination.Length];
        var unreadable = new bool[destination.Length];
        var written = new bool[destination.Length];

        foreach (var region in _regions)
        {
            MarkRange(readable, address, region.BaseAddress, region.Size, value: true);
        }

        foreach (var (unreadableAddress, unreadableSize) in _unreadableRanges)
        {
            MarkRange(readable, address, unreadableAddress, unreadableSize, value: false);
            MarkRange(unreadable, address, unreadableAddress, unreadableSize, value: true);
        }

        var requestEnd = address + (ulong)destination.Length;
        foreach (var (segmentAddress, bytes) in _segments)
        {
            var segmentEnd = segmentAddress + (ulong)bytes.Length;
            if (segmentEnd <= address || segmentAddress >= requestEnd)
            {
                continue;
            }

            var destinationStart = segmentAddress > address ? (int)(segmentAddress - address) : 0;
            var sourceStart = address > segmentAddress ? (int)(address - segmentAddress) : 0;
            var copyLength = Math.Min(bytes.Length - sourceStart, destination.Length - destinationStart);
            for (var i = 0; i < copyLength; i++)
            {
                var destinationIndex = destinationStart + i;
                if (unreadable[destinationIndex] || written[destinationIndex])
                {
                    continue;
                }

                destination[destinationIndex] = bytes[sourceStart + i];
                readable[destinationIndex] = true;
                written[destinationIndex] = true;
            }
        }

        return BlockReadResult.FromReadabilityMask(readable);
    }

    private bool TryFindWritableByte(ulong address, out byte[] bytes, out int offset)
    {
        foreach (var (segmentAddress, segmentBytes) in _segments)
        {
            if (address < segmentAddress)
            {
                continue;
            }

            var relativeOffset = address - segmentAddress;
            if (relativeOffset < (ulong)segmentBytes.Length)
            {
                bytes = segmentBytes;
                offset = (int)relativeOffset;
                return true;
            }
        }

        bytes = Array.Empty<byte>();
        offset = 0;
        return false;
    }

    private static void MarkRange(
        bool[] mask,
        ulong requestAddress,
        ulong rangeAddress,
        ulong rangeSize,
        bool value)
    {
        var requestEnd = requestAddress + (ulong)mask.Length;
        var rangeEnd = rangeAddress + rangeSize;
        if (requestEnd < requestAddress
            || rangeEnd < rangeAddress
            || rangeEnd <= requestAddress
            || rangeAddress >= requestEnd)
        {
            return;
        }

        var start = Math.Max(requestAddress, rangeAddress);
        var end = Math.Min(requestEnd, rangeEnd);
        Array.Fill(mask, value, (int)(start - requestAddress), (int)(end - start));
    }
}
