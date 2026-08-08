using System.Runtime.InteropServices;

namespace FmDataBridge.Memory;

/// <summary>
/// Snapshot of available system memory used to bound scanner allocations.
/// </summary>
public readonly record struct SystemMemoryStatus(
    ulong AvailablePhysicalBytes,
    ulong AvailableCommitBytes,
    uint MemoryLoadPercent)
{
    public bool IsKnown => AvailablePhysicalBytes != 0 || AvailableCommitBytes != 0;
}

public static class SystemMemoryStatusReader
{
    public static SystemMemoryStatus Read()
    {
        if (!OperatingSystem.IsWindows())
        {
            return default;
        }

        try
        {
            var native = new NativeMethods.MemoryStatusEx
            {
                Length = (uint)Marshal.SizeOf<NativeMethods.MemoryStatusEx>(),
            };
            if (!NativeMethods.GlobalMemoryStatusEx(ref native))
            {
                return default;
            }

            return new SystemMemoryStatus(
                native.AvailablePhysicalBytes,
                native.AvailableCommitBytes,
                native.MemoryLoadPercent);
        }
        catch (DllNotFoundException)
        {
            return default;
        }
        catch (EntryPointNotFoundException)
        {
            return default;
        }
    }
}
