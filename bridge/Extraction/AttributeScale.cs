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
        var value = (int)Math.Floor(raw / 5.0 + 0.5);
        return Math.Clamp(value, 0, 20);
    }

    /// <summary>
    /// Decode a dumped player attribute to the 1–20 scale.
    /// Returns null when the decoded value is outside 1–20 (unknown / invalid).
    /// </summary>
    public static int? TryDecodeScaled(byte raw)
    {
        var value = DecodeScaled(raw);
        return value is >= 1 and <= 20 ? value : null;
    }

    /// <summary>Personality bytes are raw 1–20; out of range is null (not zero).</summary>
    public static int? TryPersonality(byte raw) => raw is >= 1 and <= 20 ? raw : null;
}
