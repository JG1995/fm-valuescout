namespace FmDataBridge.Memory;

/// <summary>
/// Counts <see cref="IMemoryReader.TryRead"/> calls and requested byte totals for scan diagnostics.
/// </summary>
public sealed class CountingMemoryReader : IMemoryReader
{
    private readonly IMemoryReader _inner;

    public CountingMemoryReader(IMemoryReader inner)
    {
        _inner = inner ?? throw new ArgumentNullException(nameof(inner));
    }

    public long CallCount { get; private set; }

    public long RequestedBytes { get; private set; }

    public bool TryRead(ulong address, Span<byte> destination, out int bytesRead)
    {
        CallCount++;
        RequestedBytes += destination.Length;
        return _inner.TryRead(address, destination, out bytesRead);
    }

    public IEnumerable<MemoryRegion> EnumerateRegions() => _inner.EnumerateRegions();
}
