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

        if (!reader.TryReadUInt64(personAddress + (ulong)layout.NationPtrOffset, out var nation)
            || nation == 0)
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
}
