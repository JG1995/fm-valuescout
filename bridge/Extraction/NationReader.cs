using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

public static class NationReader
{
    public static IReadOnlyList<string> TryRead(
        IMemoryReader reader,
        ulong personAddress,
        IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        if (!TryReadNationPointer(reader, personAddress, layout, out var nation))
        {
            return Array.Empty<string>();
        }

        var name = FmStringReader.TryReadIndirect(
                        reader,
                        nation + (ulong)layout.NationShortNameOffset)
                    ?? FmStringReader.TryReadIndirect(
                        reader,
                        nation + (ulong)layout.NationNameOffset);

        return string.IsNullOrEmpty(name)
            ? Array.Empty<string>()
            : new[] { name };
    }

    public static uint? TryReadUid(
        IMemoryReader reader,
        ulong personAddress,
        IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        if (!TryReadNationPointer(reader, personAddress, layout, out var nation)
            || !TryReadUInt32At(reader, nation, layout.ObjectUidOffset, out var uid)
            || uid is 0 or uint.MaxValue)
        {
            return null;
        }

        return uid;
    }

    private static bool TryReadNationPointer(
        IMemoryReader reader,
        ulong personAddress,
        IFmMemoryLayout layout,
        out ulong nation)
    {
        nation = 0;
        return TryAdd(personAddress, layout.NationPtrOffset, out var nationPointerAddress)
            && reader.TryReadUInt64(nationPointerAddress, out nation)
            && nation != 0;
    }

    private static bool TryReadUInt32At(
        IMemoryReader reader,
        ulong address,
        int offset,
        out uint value)
    {
        value = 0;
        return TryAdd(address, offset, out var fieldAddress)
            && reader.TryReadUInt32(fieldAddress, out value);
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
