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
}
