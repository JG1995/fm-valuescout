namespace FmDataBridge.Memory;

/// <summary>
/// Safe process-memory reads. Invalid addresses return false — they must not crash the host.
/// </summary>
public interface IMemoryReader
{
    /// <summary>
    /// Stable diagnostics label for the memory image currently being read.
    /// </summary>
    string ReadSource => "live";

    /// <summary>
    /// Whether block and scalar reads may run concurrently on this reader.
    /// </summary>
    bool SupportsConcurrentReads => false;

    /// <summary>
    /// Attempts to read <paramref name="destination"/>.Length bytes at <paramref name="address"/>.
    /// Returns true only when the full buffer is filled. On short or failed reads,
    /// <paramref name="bytesRead"/> reports how many bytes were copied (may be zero).
    /// </summary>
    bool TryRead(ulong address, Span<byte> destination, out int bytesRead);

    /// <summary>
    /// Fills a caller-owned buffer from a contiguous address range without an intermediate allocation.
    /// Failed large reads are subdivided down to <see cref="MemoryConstants.MinBlockReadSize"/> so
    /// accessible pages within the range can still be filled. Returns true only when all
    /// <paramref name="length"/> bytes were filled. On short or failed reads,
    /// <paramref name="bytesRead"/> is the count of bytes successfully copied — not necessarily a
    /// contiguous prefix. Callers that recover around holes must inspect the full
    /// <paramref name="length"/> (cleared gaps stay zero); do not treat
    /// <c>buffer[offset..offset+bytesRead)</c> as the readable span the way <see cref="TryRead"/> does.
    /// </summary>
    bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead);

    /// <summary>
    /// Reads a block with exact readable ranges when the implementation can report them.
    /// </summary>
    bool TryReadBlockWithCoverage(
        ulong address,
        byte[] buffer,
        int offset,
        int length,
        out BlockReadResult result)
    {
        var completed = TryReadBlock(address, buffer, offset, length, out var bytesRead);
        result = BlockReadResult.FromSummary(length, bytesRead);
        return completed;
    }

    /// <summary>
    /// Yields raw VirtualQuery-style regions (unfiltered). Prefer
    /// <see cref="RegionEnumerator.GetCandidateRegions"/> for scan targets.
    /// </summary>
    IEnumerable<MemoryRegion> EnumerateRegions();
}
