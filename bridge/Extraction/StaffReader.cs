using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;

namespace FmDataBridge.Extraction;

public static class StaffReader
{
    public static StaffRecord Read(
        IMemoryReader reader,
        ulong personAddress,
        ulong staffBlockBase,
        uint uid,
        int ca,
        int pa,
        IFmMemoryLayout layout,
        out ContractClubLink? clubLink)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        var (birthYear, birthDayOfYear) = TryReadBirthDate(reader, personAddress, layout);
        var contract = StaffContractReader.Read(reader, personAddress, layout);
        clubLink = ContractClubReader.TryRead(reader, personAddress, layout);

        return new StaffRecord
        {
            Uid = uid,
            Name = NameReader.TryReadDisplayName(reader, personAddress, layout),
            BirthYear = birthYear,
            BirthDayOfYear = birthDayOfYear,
            Nationalities = NationReader.TryRead(reader, personAddress, layout),
            NationUid = NationReader.TryReadUid(reader, personAddress, layout),
            Gender = PlayerGenderReader.Read(reader, personAddress, layout),
            Ca = ca,
            Pa = pa,
            Attributes = StaffAttributeReader.Read(reader, staffBlockBase, layout),
            JobId = contract.JobId,
            WeeklyWageGbp = contract.WeeklyWageGbp,
            ContractExpiryYear = contract.ContractExpiryYear,
            ContractExpiryDayOfYear = contract.ContractExpiryDayOfYear,
            Club = clubLink?.ClubName,
            Division = clubLink?.Division,
        };
    }

    private static (int? Year, int? DayOfYear) TryReadBirthDate(
        IMemoryReader reader,
        ulong personAddress,
        IFmMemoryLayout layout)
    {
        if (!TryAdd(personAddress, layout.DobOffset, out var dobAddress)
            || !reader.TryReadUInt32(dobAddress, out var rawDob))
        {
            return (null, null);
        }

        var (year, dayOfYear) = FmDateDecoder.Decode(rawDob);
        return FmDateDecoder.IsPlausible(year, dayOfYear)
            ? (year, dayOfYear)
            : (null, null);
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
