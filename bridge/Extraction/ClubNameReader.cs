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

        var name = FmStringReader.TryReadIndirect(reader, clubAddress + (ulong)layout.ClubNameOffset)
                   ?? FmStringReader.TryReadIndirect(reader, clubAddress + (ulong)layout.ClubShortNameOffset);
        return ClubNamePlausibility.IsPlausible(name) ? name : null;
    }
}

public static class CompetitionNameReader
{
    public static string? TryRead(IMemoryReader reader, ulong teamAddress, IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        foreach (var toff in new[] { layout.TeamCompPtrOffset, layout.TeamCompAltPtrOffset })
        {
            if (!reader.TryReadUInt64(teamAddress + (ulong)toff, out var comp) || comp == 0)
            {
                continue;
            }

            var full = FmStringReader.TryReadIndirect(reader, comp + (ulong)layout.CompNameOffset);
            if (!ClubNamePlausibility.IsPlausible(full))
            {
                full = FmStringReader.TryReadIndirect(reader, comp + (ulong)layout.CompShortNameOffset);
            }

            if (ClubNamePlausibility.IsPlausible(full))
            {
                return full;
            }
        }

        return null;
    }
}
