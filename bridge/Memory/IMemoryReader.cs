namespace FmDataBridge.Memory;

/// <summary>
/// Safe process-memory reads. Invalid addresses return false — they must not crash the host.
/// </summary>
public interface IMemoryReader
{
    /// <summary>
    /// Attempts to read <paramref name="destination"/>.Length bytes at <paramref name="address"/>.
    /// Returns true only when the full buffer is filled. On short or failed reads,
    /// <paramref name="bytesRead"/> reports how many bytes were copied (may be zero).
    /// </summary>
    bool TryRead(ulong address, Span<byte> destination, out int bytesRead);

    /// <summary>
    /// Yields raw VirtualQuery-style regions (unfiltered). Prefer
    /// <see cref="RegionEnumerator.GetCandidateRegions"/> for scan targets.
    /// </summary>
    IEnumerable<MemoryRegion> EnumerateRegions();
}
