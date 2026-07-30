namespace FmDataBridge.Memory;

/// <summary>
/// Win32 memory constants used by VirtualQuery region filtering.
/// </summary>
public static class MemoryConstants
{
    public const uint PageNoAccess = 0x01;
    public const uint PageReadOnly = 0x02;
    public const uint PageReadWrite = 0x04;
    public const uint PageWriteCopy = 0x08;
    public const uint PageExecute = 0x10;
    public const uint PageExecuteRead = 0x20;
    public const uint PageExecuteReadWrite = 0x40;
    public const uint PageExecuteWriteCopy = 0x80;
    public const uint PageGuard = 0x100;

    public const uint MemCommit = 0x1000;
    public const uint MemReserve = 0x2000;
    public const uint MemFree = 0x10000;

    public const uint MemPrivate = 0x20000;
    public const uint MemMapped = 0x40000;
    public const uint MemImage = 0x1000000;

    /// <summary>
    /// Default upper bound for a single candidate region (512 MiB).
    /// </summary>
    public const ulong DefaultMaxRegionSize = 512UL * 1024 * 1024;

    /// <summary>
    /// Smallest block size used when subdividing a failed large process-memory read (one page).
    /// </summary>
    public const int MinBlockReadSize = 0x1000;
}
