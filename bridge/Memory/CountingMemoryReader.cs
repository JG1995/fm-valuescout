namespace FmDataBridge.Memory;

/// <summary>
/// Counts <see cref="IMemoryReader.TryRead"/> / <see cref="IMemoryReader.TryReadBlock"/> calls
/// and requested byte totals for scan diagnostics.
/// </summary>
public sealed class CountingMemoryReader : IMemoryReader
{
    private readonly IMemoryReader _inner;
    private long _callCount;
    private long _requestedBytes;

    public CountingMemoryReader(IMemoryReader inner)
    {
        _inner = inner ?? throw new ArgumentNullException(nameof(inner));
    }

    public long CallCount => Interlocked.Read(ref _callCount);

    public long RequestedBytes => Interlocked.Read(ref _requestedBytes);

    public string ReadSource => _inner.ReadSource;

    public bool SupportsConcurrentReads => _inner.SupportsConcurrentReads;

    public bool TryRead(ulong address, Span<byte> destination, out int bytesRead)
    {
        Interlocked.Increment(ref _callCount);
        Interlocked.Add(ref _requestedBytes, destination.Length);
        return _inner.TryRead(address, destination, out bytesRead);
    }

    public bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead)
    {
        Interlocked.Increment(ref _callCount);
        Interlocked.Add(ref _requestedBytes, length);
        return _inner.TryReadBlock(address, buffer, offset, length, out bytesRead);
    }

    public bool TryReadBlockWithCoverage(
        ulong address,
        byte[] buffer,
        int offset,
        int length,
        out BlockReadResult result)
    {
        Interlocked.Increment(ref _callCount);
        Interlocked.Add(ref _requestedBytes, length);
        return _inner.TryReadBlockWithCoverage(address, buffer, offset, length, out result);
    }

    public IEnumerable<MemoryRegion> EnumerateRegions() => _inner.EnumerateRegions();
}
