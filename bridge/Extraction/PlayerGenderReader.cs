using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;

namespace FmDataBridge.Extraction;

public static class PlayerGenderReader
{
    public static PlayerGender Read(
        IMemoryReader reader,
        ulong personAddress,
        IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        if (!TryAdd(personAddress, layout.GenderOffset, out var genderAddress)
            || !reader.TryReadByte(genderAddress, out var raw))
        {
            return PlayerGender.Unknown;
        }

        return (raw & layout.FemaleGenderBit) != 0
            ? PlayerGender.Female
            : PlayerGender.Male;
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
