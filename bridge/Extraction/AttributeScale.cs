namespace FmDataBridge.Extraction;

/// <summary>
/// FM attribute encoding: player attrs stored ×5; personality bytes are already 1–20.
/// </summary>
public static class AttributeScale
{
    /// <summary>
    /// Decode a player attribute byte stored as value×5.
    /// Used for foot preference and similar comparisons (0 is a valid low score).
    /// </summary>
    public static int DecodeScaled(byte raw)
    {
        var value = DecodeScaledUnclamped(raw);
        return Math.Clamp(value, 0, 20);
    }

    /// <summary>
    /// Decode a dumped player attribute through the compatibility clamp, then return the 1–20 scale.
    /// </summary>
    public static int? TryDecodeScaled(byte raw)
    {
        var value = DecodeScaled(raw);
        return value is >= 1 and <= 20 ? value : null;
    }

    /// <summary>
    /// Decode a stored attribute without treating an out-of-range byte as a valid maximum rating.
    /// </summary>
    public static int? TryDecodeScaledStrict(byte raw)
    {
        var value = DecodeScaledUnclamped(raw);
        return value is >= 1 and <= 20 ? value : null;
    }

    /// <summary>Personality bytes are raw 1–20; out of range is null (not zero).</summary>
    public static int? TryPersonality(byte raw) => raw is >= 1 and <= 20 ? raw : null;

    private static int DecodeScaledUnclamped(byte raw) => (int)Math.Floor(raw / 5.0 + 0.5);
}
