namespace FmDataBridge.Memory;

public static class RegionEnumerator
{
    public static IReadOnlyList<MemoryRegion> GetCandidateRegions(
        IMemoryReader reader,
        ulong maxRegionSize = MemoryConstants.DefaultMaxRegionSize)
    {
        ArgumentNullException.ThrowIfNull(reader);
        return SelectCandidates(reader.EnumerateRegions(), maxRegionSize);
    }

    public static IReadOnlyList<MemoryRegion> SelectCandidates(
        IEnumerable<MemoryRegion> regions,
        ulong maxRegionSize = MemoryConstants.DefaultMaxRegionSize)
    {
        ArgumentNullException.ThrowIfNull(regions);

        var results = new List<MemoryRegion>();
        foreach (var region in regions)
        {
            if (IsCandidate(region, maxRegionSize))
            {
                results.Add(region);
            }
        }

        return results;
    }

    public static bool IsCandidate(MemoryRegion region, ulong maxRegionSize)
    {
        if (region.Size == 0 || region.Size > maxRegionSize)
        {
            return false;
        }

        if (region.State != MemoryConstants.MemCommit)
        {
            return false;
        }

        if (region.Type != MemoryConstants.MemPrivate)
        {
            return false;
        }

        if ((region.Protect & MemoryConstants.PageGuard) != 0)
        {
            return false;
        }

        if ((region.Protect & MemoryConstants.PageNoAccess) != 0)
        {
            return false;
        }

        return IsWritable(region.Protect);
    }

    private static bool IsWritable(uint protect)
    {
        const uint writableMask =
            MemoryConstants.PageReadWrite
            | MemoryConstants.PageWriteCopy
            | MemoryConstants.PageExecuteReadWrite
            | MemoryConstants.PageExecuteWriteCopy;

        return (protect & writableMask) != 0;
    }
}
