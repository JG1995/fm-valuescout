using System.Buffers;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

public static class StaffAttributeReader
{
    public static IReadOnlyDictionary<string, int?> Read(
        IMemoryReader reader,
        ulong staffBlockBase,
        IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        var entries = layout.StaffAttributeEntries;
        var min = entries.Min(entry => entry.Offset);
        var max = entries.Max(entry => entry.Offset);
        var length = max - min + 1;
        var buffer = ArrayPool<byte>.Shared.Rent(length);
        try
        {
            Array.Clear(buffer, 0, length);
            if (layout.StaffAttrsOffset <= int.MaxValue - min
                && TryAdd(staffBlockBase, layout.StaffAttrsOffset + min, out var attrsAddress))
            {
                reader.TryReadBlock(attrsAddress, buffer, 0, length, out _);
            }

            var attributes = new Dictionary<string, int?>(entries.Count, StringComparer.Ordinal);
            foreach (var entry in entries)
            {
                attributes[entry.Key] = AttributeScale.TryDecodeScaledStrict(buffer[entry.Offset - min]);
            }

            return attributes;
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }

    private static bool TryAdd(ulong address, int offset, out ulong result)
    {
        result = 0;
        if (offset < 0 || (ulong)offset > ulong.MaxValue - address)
        {
            return false;
        }

        result = address + (ulong)offset;
        return true;
    }
}
