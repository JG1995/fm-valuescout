namespace FmDataBridge.Memory;

/// <summary>
/// Counts <see cref="IMemoryReader.TryRead"/> / <see cref="IMemoryReader.TryReadBlock"/> calls
/// and requested byte totals for scan diagnostics.
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

    public string ReadSource => _inner.ReadSource;

    public bool TryRead(ulong address, Span<byte> destination, out int bytesRead)
    {
        CallCount++;
        RequestedBytes += destination.Length;
        return _inner.TryRead(address, destination, out bytesRead);
    }

    public bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead)
    {
        CallCount++;
        RequestedBytes += length;
        return _inner.TryReadBlock(address, buffer, offset, length, out bytesRead);
    }

    public bool TryReadBlockWithCoverage(
        ulong address,
        byte[] buffer,
        int offset,
        int length,
        out BlockReadResult result)
    {
        CallCount++;
        RequestedBytes += length;
        return _inner.TryReadBlockWithCoverage(address, buffer, offset, length, out result);
    }

    public IEnumerable<MemoryRegion> EnumerateRegions() => _inner.EnumerateRegions();
}
