namespace FmDataBridge.Protocol;

internal enum ScanRequestKind
{
    None,
    FullDump,
    ForceDump,
    Probe,
}

internal static class ScanRequestPriority
{
    public static ScanRequestKind Select(
        bool hasDumpRequest,
        bool hasForceScan,
        bool hasProbeRequest)
    {
        if (hasDumpRequest)
        {
            return ScanRequestKind.FullDump;
        }

        if (hasForceScan)
        {
            return ScanRequestKind.ForceDump;
        }

        return hasProbeRequest ? ScanRequestKind.Probe : ScanRequestKind.None;
    }
}
