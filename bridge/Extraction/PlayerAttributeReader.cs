using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

public sealed class PlayerAttributes
{
    public IReadOnlyDictionary<string, int?> Attributes { get; init; } =
        new Dictionary<string, int?>();

    public IReadOnlyDictionary<string, int?> HiddenAttributes { get; init; } =
        new Dictionary<string, int?>();

    public IReadOnlyDictionary<string, int?> Personality { get; init; } =
        new Dictionary<string, int?>();
}

public static class PlayerAttributeReader
{
    public static PlayerAttributes Read(
        IMemoryReader reader,
        ulong personAddress,
        ulong playerBlockBase,
        IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        return new PlayerAttributes
        {
            Attributes = ReadScaledGroup(
                reader,
                playerBlockBase + (ulong)layout.AttrsOffset,
                layout.AttributeEntries),
            HiddenAttributes = ReadScaledGroup(
                reader,
                playerBlockBase + (ulong)layout.AttrsOffset,
                layout.HiddenAttributeEntries),
            Personality = ReadPersonalityGroup(reader, personAddress, layout.PersonalityEntries),
        };
    }

    private static IReadOnlyDictionary<string, int?> ReadScaledGroup(
        IMemoryReader reader,
        ulong attrsBase,
        IReadOnlyList<AttributeLayoutEntry> entries)
    {
        var map = new Dictionary<string, int?>(entries.Count, StringComparer.Ordinal);
        foreach (var entry in entries)
        {
            if (!reader.TryReadByte(attrsBase + (ulong)entry.Offset, out var raw))
            {
                map[entry.Key] = null;
                continue;
            }

            map[entry.Key] = AttributeScale.TryDecodeScaled(raw);
        }

        return map;
    }

    private static IReadOnlyDictionary<string, int?> ReadPersonalityGroup(
        IMemoryReader reader,
        ulong personAddress,
        IReadOnlyList<AttributeLayoutEntry> entries)
    {
        var map = new Dictionary<string, int?>(entries.Count, StringComparer.Ordinal);
        foreach (var entry in entries)
        {
            if (!reader.TryReadByte(personAddress + (ulong)entry.Offset, out var raw))
            {
                map[entry.Key] = null;
                continue;
            }

            map[entry.Key] = AttributeScale.TryPersonality(raw);
        }

        return map;
    }
}
