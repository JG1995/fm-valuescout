using System.Buffers;
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

        var attrsBase = playerBlockBase + (ulong)layout.AttrsOffset;
        var attrSpan = ContiguousByteSpan(layout.AttributeEntries, layout.HiddenAttributeEntries);
        var attrBuffer = ArrayPool<byte>.Shared.Rent(attrSpan.Length);
        try
        {
            reader.TryReadBlock(attrsBase + (ulong)attrSpan.Start, attrBuffer, 0, attrSpan.Length, out _);
            return new PlayerAttributes
            {
                Attributes = DecodeScaledGroup(attrBuffer, attrSpan.Start, layout.AttributeEntries),
                HiddenAttributes = DecodeScaledGroup(
                    attrBuffer,
                    attrSpan.Start,
                    layout.HiddenAttributeEntries),
                Personality = ReadPersonalityGroup(reader, personAddress, layout.PersonalityEntries),
            };
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(attrBuffer);
        }
    }

    private static IReadOnlyDictionary<string, int?> ReadPersonalityGroup(
        IMemoryReader reader,
        ulong personAddress,
        IReadOnlyList<AttributeLayoutEntry> entries)
    {
        var span = ContiguousByteSpan(entries);
        var buffer = ArrayPool<byte>.Shared.Rent(span.Length);
        try
        {
            reader.TryReadBlock(
                personAddress + (ulong)span.Start,
                buffer,
                0,
                span.Length,
                out _);
            return DecodePersonalityGroup(buffer, span.Start, entries);
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }

    private static IReadOnlyDictionary<string, int?> DecodeScaledGroup(
        byte[] buffer,
        int spanStart,
        IReadOnlyList<AttributeLayoutEntry> entries)
    {
        var map = new Dictionary<string, int?>(entries.Count, StringComparer.Ordinal);
        foreach (var entry in entries)
        {
            map[entry.Key] = AttributeScale.TryDecodeScaled(buffer[entry.Offset - spanStart]);
        }

        return map;
    }

    private static IReadOnlyDictionary<string, int?> DecodePersonalityGroup(
        byte[] buffer,
        int spanStart,
        IReadOnlyList<AttributeLayoutEntry> entries)
    {
        var map = new Dictionary<string, int?>(entries.Count, StringComparer.Ordinal);
        foreach (var entry in entries)
        {
            map[entry.Key] = AttributeScale.TryDecodeRawStrict(buffer[entry.Offset - spanStart]);
        }

        return map;
    }

    private static (int Start, int Length) ContiguousByteSpan(
        IReadOnlyList<AttributeLayoutEntry> entries)
    {
        var min = int.MaxValue;
        var max = 0;
        foreach (var entry in entries)
        {
            if (entry.Offset < min)
            {
                min = entry.Offset;
            }

            if (entry.Offset > max)
            {
                max = entry.Offset;
            }
        }

        return (min, max - min + 1);
    }

    private static (int Start, int Length) ContiguousByteSpan(
        IReadOnlyList<AttributeLayoutEntry> first,
        IReadOnlyList<AttributeLayoutEntry> second)
    {
        var a = ContiguousByteSpan(first);
        var b = ContiguousByteSpan(second);
        var start = Math.Min(a.Start, b.Start);
        var end = Math.Max(a.Start + a.Length, b.Start + b.Length);
        return (start, end - start);
    }
}
