using System.Diagnostics;
using System.Runtime.InteropServices;

namespace FmDataBridge.Memory;

/// <summary>
/// Production reader for the current process using ReadProcessMemory / VirtualQuery.
/// Invalid addresses fail the read; they must not hard-crash via raw pointer deref.
/// </summary>
public sealed class WindowsMemoryReader : IMemoryReader
{
    private readonly IntPtr _processHandle;

    public WindowsMemoryReader()
        : this(NativeMethods.GetCurrentProcess())
    {
    }

    internal WindowsMemoryReader(IntPtr processHandle)
    {
        _processHandle = processHandle;
    }

    public bool TryRead(ulong address, Span<byte> destination, out int bytesRead)
    {
        bytesRead = 0;
        if (destination.IsEmpty)
        {
            return true;
        }

        var buffer = new byte[destination.Length];
        // ponytail: allocate per read
        // Upgrade to ArrayPool<byte>.Shared if scan hot path shows GC pressure from TryRead
        var ok = NativeMethods.ReadProcessMemory(
            _processHandle,
            (IntPtr)address,
            buffer,
            (IntPtr)buffer.Length,
            out var read);

        bytesRead = (int)read;
        if (bytesRead > 0)
        {
            buffer.AsSpan(0, bytesRead).CopyTo(destination);
        }

        return ok && bytesRead == destination.Length;
    }

    public IEnumerable<MemoryRegion> EnumerateRegions()
    {
        var address = 0UL;
        while (true)
        {
            var result = NativeMethods.VirtualQuery(
                (IntPtr)address,
                out var info,
                (UIntPtr)Marshal.SizeOf<NativeMethods.MemoryBasicInformation>());

            if (result == UIntPtr.Zero)
            {
                yield break;
            }

            var baseAddress = unchecked((ulong)info.BaseAddress.ToInt64());
            var regionSize = info.RegionSize.ToUInt64();
            if (regionSize == 0)
            {
                yield break;
            }

            yield return new MemoryRegion(
                baseAddress,
                regionSize,
                info.Protect,
                info.Type,
                info.State);

            var next = baseAddress + regionSize;
            if (next <= address)
            {
                yield break;
            }

            address = next;
        }
    }

    /// <summary>
    /// Snapshot of loaded modules for the current process.
    /// </summary>
    public static IReadOnlyList<ProcessModuleInfo> GetCurrentProcessModules()
    {
        var results = new List<ProcessModuleInfo>();
        foreach (ProcessModule module in Process.GetCurrentProcess().Modules)
        {
            if (module.ModuleName is not { } name)
            {
                continue;
            }

            results.Add(new ProcessModuleInfo(
                name,
                (ulong)module.BaseAddress.ToInt64(),
                (ulong)(uint)module.ModuleMemorySize));
        }

        return results;
    }

    public ModulePresenceBounds LocateKnownModules() =>
        ModuleLocator.LocateKnownModules(GetCurrentProcessModules());
}

internal static class NativeMethods
{
    [DllImport("kernel32.dll", SetLastError = true)]
    internal static extern IntPtr GetCurrentProcess();

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static extern bool ReadProcessMemory(
        IntPtr hProcess,
        IntPtr lpBaseAddress,
        [Out] byte[] lpBuffer,
        IntPtr nSize,
        out IntPtr lpNumberOfBytesRead);

    [DllImport("kernel32.dll", SetLastError = true)]
    internal static extern UIntPtr VirtualQuery(
        IntPtr lpAddress,
        out MemoryBasicInformation lpBuffer,
        UIntPtr dwLength);

    [StructLayout(LayoutKind.Sequential)]
    internal struct MemoryBasicInformation
    {
        public IntPtr BaseAddress;
        public IntPtr AllocationBase;
        public uint AllocationProtect;
        public UIntPtr RegionSize;
        public uint State;
        public uint Protect;
        public uint Type;
    }
}
