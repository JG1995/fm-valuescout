using FmDataBridge.Memory;

namespace FmDataBridge.Tests.Fakes;

/// <summary>
/// In-memory <see cref="IMemoryReader"/> for unit tests — no process or Windows APIs.
/// </summary>
public sealed class FakeMemoryReader : IMemoryReader
{
    private readonly List<MemoryRegion> _regions = new();
    private readonly List<(ulong Address, byte[] Bytes)> _segments = new();

    public void AddRegion(MemoryRegion region) => _regions.Add(region);

    public void AddBytes(ulong address, byte[] bytes)
    {
        ArgumentNullException.ThrowIfNull(bytes);
        _segments.Add((address, bytes));
    }

    public IEnumerable<MemoryRegion> EnumerateRegions() => _regions;

    public bool TryRead(ulong address, Span<byte> destination, out int bytesRead)
    {
        bytesRead = 0;
        if (destination.IsEmpty)
        {
            return true;
        }

        foreach (var (segmentAddress, bytes) in _segments)
        {
            if (address < segmentAddress)
            {
                continue;
            }

            var offset = address - segmentAddress;
            if (offset >= (ulong)bytes.Length)
            {
                continue;
            }

            var available = bytes.Length - (int)offset;
            var toCopy = Math.Min(available, destination.Length);
            bytes.AsSpan((int)offset, toCopy).CopyTo(destination);
            bytesRead = toCopy;
            return toCopy == destination.Length;
        }

        return false;
    }

    public bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead) =>
        BlockReadHelper.TryFill(address, buffer, offset, length, out bytesRead, TryReadDirect);

    private bool TryReadDirect(ulong address, byte[] buffer, int offset, int length, out int bytesRead)
    {
        bytesRead = 0;
        if (length == 0)
        {
            return true;
        }

        // Compose every overlapping segment into the range (production RPM returns a contiguous
        // page image; tests store sparse AddBytes fragments).
        var hit = new bool[length];
        var reqEnd = address + (ulong)length;
        foreach (var (segmentAddress, bytes) in _segments)
        {
            var segEnd = segmentAddress + (ulong)bytes.Length;
            if (segEnd <= address || segmentAddress >= reqEnd)
            {
                continue;
            }

            var dstStart = segmentAddress > address ? (int)(segmentAddress - address) : 0;
            var srcStart = address > segmentAddress ? (int)(address - segmentAddress) : 0;
            var copyLen = Math.Min(bytes.Length - srcStart, length - dstStart);
            if (copyLen <= 0)
            {
                continue;
            }

            for (var i = 0; i < copyLen; i++)
            {
                if (hit[dstStart + i])
                {
                    continue;
                }

                buffer[offset + dstStart + i] = bytes[srcStart + i];
                hit[dstStart + i] = true;
            }
        }

        var filled = 0;
        for (var i = 0; i < length; i++)
        {
            if (hit[i])
            {
                filled++;
            }
        }

        bytesRead = filled;
        return filled == length;
    }
}
