namespace FmDataBridge.Scanning;

/// <summary>
/// Byte coverage for the region blocks that drive typed person discovery.
/// </summary>
public readonly record struct ScanReadQuality(
    long RequestedBytes,
    long ReadableBytes,
    long UnreadBytes,
    long InternalFailureBytes)
{
    public const int MaximumUnreadPercent = 10;

    public bool IsMateriallyIncomplete =>
        RequestedBytes > 0
        && UnreadBytes * 100 > RequestedBytes * MaximumUnreadPercent;

    public ScanReadQuality Record(int requestedBytes, int readableBytes, bool internalFailure = false)
    {
        if (requestedBytes <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(requestedBytes));
        }

        if (readableBytes < 0 || readableBytes > requestedBytes)
        {
            throw new ArgumentOutOfRangeException(nameof(readableBytes));
        }

        var unreadBytes = requestedBytes - readableBytes;
        return new ScanReadQuality(
            RequestedBytes + requestedBytes,
            ReadableBytes + readableBytes,
            UnreadBytes + unreadBytes,
            InternalFailureBytes + (internalFailure ? unreadBytes : 0));
    }

}
