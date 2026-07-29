namespace FmDataBridge.Extraction;

/// <summary>
/// FM money fields: unset sentinels become null (never −1 or 0 as a stand-in for missing).
/// Values are GBP as stored in memory for the 26.3 pin.
/// </summary>
public static class MoneyDecode
{
    public const uint UnsetSentinel = 0xFFFFFFFFu;

    /// <summary>FM uses 300M on the market-value slot when the value is not fixed.</summary>
    public const uint UnfixedMarketValueSentinel = 300_000_000u;

    public static long? TryGbp(uint raw) => raw == UnsetSentinel ? null : raw;

    public static long? TryMarketValueGbp(uint raw) =>
        raw is UnsetSentinel or UnfixedMarketValueSentinel ? null : raw;
}
