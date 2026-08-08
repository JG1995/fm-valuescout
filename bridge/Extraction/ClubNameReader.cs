using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

/// <summary>
/// Club / competition name plausibility (Latin-heavy, length-bounded).
/// </summary>
public static class ClubNamePlausibility
{
    public static bool IsPlausible(string? s)
    {
        if (string.IsNullOrEmpty(s) || s.Length is < 2 or > 48)
        {
            return false;
        }

        var latin = 0;
        var weird = 0;
        foreach (var c in s)
        {
            if ((c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z'))
            {
                latin++;
            }
            else if (c > 0x7F && !(char.IsLetter(c) && (c <= 0x24F || (c >= 0x1E00 && c <= 0x1EFF))))
            {
                weird++;
            }
        }

        return latin >= 2 && weird == 0;
    }
}

public static class ClubNameReader
{
    public static string? TryRead(IMemoryReader reader, ulong clubAddress, IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        var name = TryReadIndirectAt(reader, clubAddress, layout.ClubNameOffset)
                   ?? TryReadIndirectAt(reader, clubAddress, layout.ClubShortNameOffset);
        return ClubNamePlausibility.IsPlausible(name) ? name : null;
    }

    private static string? TryReadIndirectAt(IMemoryReader reader, ulong address, int offset) =>
        offset >= 0 && (ulong)offset <= ulong.MaxValue - address
            ? FmStringReader.TryReadIndirect(reader, address + (ulong)offset)
            : null;
}

public static class CompetitionNameReader
{
    public static string? TryRead(IMemoryReader reader, ulong teamAddress, IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        foreach (var toff in new[] { layout.TeamCompPtrOffset, layout.TeamCompAltPtrOffset })
        {
            if (!TryReadPointerAt(reader, teamAddress, toff, out var comp) || comp == 0)
            {
                continue;
            }

            var full = TryReadIndirectAt(reader, comp, layout.CompNameOffset);
            if (!ClubNamePlausibility.IsPlausible(full))
            {
                full = TryReadIndirectAt(reader, comp, layout.CompShortNameOffset);
            }

            if (ClubNamePlausibility.IsPlausible(full))
            {
                return full;
            }
        }

        return null;
    }

    private static bool TryReadPointerAt(
        IMemoryReader reader,
        ulong address,
        int offset,
        out ulong value)
    {
        value = 0;
        return offset >= 0
            && (ulong)offset <= ulong.MaxValue - address
            && reader.TryReadUInt64(address + (ulong)offset, out value);
    }

    private static string? TryReadIndirectAt(IMemoryReader reader, ulong address, int offset) =>
        offset >= 0 && (ulong)offset <= ulong.MaxValue - address
            ? FmStringReader.TryReadIndirect(reader, address + (ulong)offset)
            : null;
}
