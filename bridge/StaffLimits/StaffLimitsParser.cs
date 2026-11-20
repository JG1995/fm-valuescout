using FmDataBridge.Protocol;

namespace FmDataBridge.StaffLimits;

/// <summary>
/// Reads one staff group as FM's Staff overview receives it: the group's name, count, and the
/// board's limit, then a row per job with FM's job type, its count, and its limit. See the FM27
/// board staff limits note in the wiki.
/// </summary>
internal static class StaffLimitsParser
{
    // Four-character record keys, read big-endian.
    private const uint NameKey = 1684370275; // "desc"
    private const uint CountKey = 1668771444; // "cwnt"
    private const uint MaxKey = 1298233430; // "MaxV"
    private const uint JobsKey = 1400138340; // "Stkd"
    private const uint JobNameKey = 825696289; // "17 !"
    private const uint JobTypeKey = 1363900275; // "QKws"

    /// <summary>The group, or null when the value carries no limit for it.</summary>
    public static StaffLimitsGroup? ParseGroup(string group, FmValue? value)
    {
        if (value is not FmRecord record || Limit(record) is not { } max)
        {
            return null;
        }

        var jobs = new List<StaffLimitsJob>();
        foreach (var row in record.Get<FmList>(JobsKey)?.Items ?? Array.Empty<FmValue>())
        {
            if (row is FmRecord job
                && job.Get<FmNumber>(JobTypeKey)?.Value is { } jobType and > 0 and <= 255
                && Limit(job) is { } jobMax)
            {
                jobs.Add(new StaffLimitsJob
                {
                    JobType = (int)jobType,
                    Name = job.Get<FmText>(JobNameKey)?.Value,
                    Count = Count(job),
                    Max = jobMax,
                });
            }
        }

        return new StaffLimitsGroup
        {
            Group = group,
            Name = record.Get<FmText>(NameKey)?.Value,
            Count = Count(record),
            Max = max,
            Jobs = jobs,
        };
    }

    private static int? Limit(FmRecord record) =>
        record.Get<FmNumber>(MaxKey)?.Value is { } max and >= 0 and <= int.MaxValue ? (int)max : null;

    private static int? Count(FmRecord record) =>
        record.Get<FmNumber>(CountKey)?.Value is { } count and >= 0 and <= int.MaxValue ? (int)count : null;
}
