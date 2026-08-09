namespace FmDataBridge.Memory;

/// <summary>
/// Internal process-memory write seam for the approved bridge mutation service.
/// Invalid addresses return false so callers can report a verified failure instead of crashing FM.
/// </summary>
internal interface IMemoryWriter
{
    /// <summary>
    /// Attempts to write one byte at <paramref name="address"/>.
    /// Returns true only when the byte was written. On a failed write,
    /// <paramref name="bytesWritten"/> reports the bytes reported by the OS.
    /// </summary>
    bool TryWriteByte(ulong address, byte value, out int bytesWritten);

    /// <summary>
    /// Attempts to write one unsigned 16-bit value at <paramref name="address"/>.
    /// Returns true only when both bytes were written. On a short or failed write,
    /// <paramref name="bytesWritten"/> reports the bytes reported by the OS.
    /// </summary>
    bool TryWriteUInt16(ulong address, ushort value, out int bytesWritten);
}
