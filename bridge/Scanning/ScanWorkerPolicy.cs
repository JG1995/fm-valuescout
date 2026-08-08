using FmDataBridge.Memory;

namespace FmDataBridge.Scanning;

internal static class ScanWorkerPolicy
{
    internal const int MaximumWorkerCount = 8;
    internal const int LowMemoryWorkerCount = 2;
    internal const ulong LowMemoryThresholdBytes = MemoryPressurePolicy.LowMemoryThresholdBytes;

    internal static int GetWorkerCount(
        int candidateRegionCount,
        int processorCount,
        SystemMemoryStatus memoryStatus)
    {
        if (candidateRegionCount <= 0)
        {
            return 0;
        }

        var workerCount = Math.Min(
            candidateRegionCount,
            Math.Clamp(processorCount - 1, 1, MaximumWorkerCount));
        if (memoryStatus.IsKnown
            && memoryStatus.AvailablePhysicalBytes < LowMemoryThresholdBytes
            && workerCount > LowMemoryWorkerCount)
        {
            return LowMemoryWorkerCount;
        }

        return workerCount;
    }
}
