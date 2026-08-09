using System.Runtime.InteropServices;
using FmDataBridge.Memory;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class ProcessSnapshotTests
{
    private sealed class WindowsFactAttribute : FactAttribute
    {
        public WindowsFactAttribute()
        {
            if (!OperatingSystem.IsWindows())
            {
                Skip = "PSS VA-clone snapshots require Windows";
            }
        }
    }

    [WindowsFact]
    public void Va_clone_reads_known_memory_and_disposes_once()
    {
        var expected = Enumerable.Range(0, 64).Select(index => (byte)(0xA0 + index)).ToArray();
        var address = Marshal.AllocHGlobal(expected.Length);
        try
        {
            Marshal.Copy(expected, 0, address, expected.Length);

            var capture = new WindowsProcessSnapshotFactory().TryCapture();
            Assert.True(capture.IsSuccess, capture.FailureReason);
            var snapshot = Assert.IsAssignableFrom<IProcessSnapshot>(capture.Snapshot);
            try
            {
                var actual = new byte[expected.Length];
                var read = snapshot.Reader.TryRead(unchecked((ulong)address.ToInt64()), actual, out var bytesRead);

                Assert.True(read);
                Assert.Equal(expected.Length, bytesRead);
                Assert.Equal(expected, actual);
            }
            finally
            {
                snapshot.Dispose();
            }
        }
        finally
        {
            Marshal.FreeHGlobal(address);
        }
    }

    [WindowsFact]
    public void Va_clone_enumerates_a_region_after_the_live_process_releases_it()
    {
        const uint allocationFlags = 0x1000 | 0x2000;
        const uint pageReadWrite = 0x04;
        const uint releaseFlags = 0x8000;
        var allocation = TestNativeMethods.VirtualAlloc(
            IntPtr.Zero,
            (UIntPtr)MemoryConstants.MinBlockReadSize,
            allocationFlags,
            pageReadWrite);
        Assert.NotEqual(IntPtr.Zero, allocation);

        try
        {
            var address = unchecked((ulong)allocation.ToInt64());
            Marshal.WriteByte(allocation, 0xA5);

            var capture = new WindowsProcessSnapshotFactory().TryCapture();
            Assert.True(capture.IsSuccess, capture.FailureReason);
            var snapshot = Assert.IsAssignableFrom<IProcessSnapshot>(capture.Snapshot);
            try
            {
                Assert.True(TestNativeMethods.VirtualFree(allocation, UIntPtr.Zero, releaseFlags));
                allocation = IntPtr.Zero;

                Assert.DoesNotContain(
                    new WindowsMemoryReader().EnumerateRegions(),
                    region => ContainsCommittedAddress(region, address));
                Assert.Contains(
                    snapshot.Reader.EnumerateRegions(),
                    region => ContainsCommittedAddress(region, address));
            }
            finally
            {
                snapshot.Dispose();
            }
        }
        finally
        {
            if (allocation != IntPtr.Zero)
            {
                Assert.True(TestNativeMethods.VirtualFree(allocation, UIntPtr.Zero, releaseFlags));
            }
        }
    }

    private static bool ContainsCommittedAddress(MemoryRegion region, ulong address) =>
        region.State == MemoryConstants.MemCommit
        && region.BaseAddress <= address
        && address - region.BaseAddress < region.Size;

    private static class TestNativeMethods
    {
        [DllImport("kernel32.dll", SetLastError = true)]
        internal static extern IntPtr VirtualAlloc(
            IntPtr address,
            UIntPtr size,
            uint allocationType,
            uint protection);

        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        internal static extern bool VirtualFree(
            IntPtr address,
            UIntPtr size,
            uint freeType);
    }
}
