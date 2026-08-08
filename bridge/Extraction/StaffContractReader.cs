using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

public sealed class StaffContractFields
{
    public long? WeeklyWageGbp { get; init; }

    public int? ContractExpiryYear { get; init; }

    public int? ContractExpiryDayOfYear { get; init; }

    /// <summary>FM's language-independent personJobTypes enum; null when unread or unset.</summary>
    public int? JobId { get; init; }
}

public static class StaffContractReader
{
    public static StaffContractFields Read(
        IMemoryReader reader,
        ulong personAddress,
        IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        if (!TryReadPointerAt(reader, personAddress, layout.FullContractPtrOffset, out var contract)
            || contract == 0)
        {
            return new StaffContractFields();
        }

        long? wage = null;
        if (TryReadUInt32At(reader, contract, layout.ContractWeeklyWageOffset, out var wageRaw))
        {
            wage = MoneyDecode.TryGbp(wageRaw);
        }

        int? expiryYear = null;
        int? expiryDayOfYear = null;
        if (TryReadUInt32At(reader, contract, layout.ContractExpiryOffset, out var expiryRaw))
        {
            var (year, dayOfYear) = FmDateDecoder.Decode(expiryRaw);
            if (year >= 2000
                && FmDateDecoder.IsPlausible(year, dayOfYear)
                && (dayOfYear != 366 || DateTime.IsLeapYear(year)))
            {
                expiryYear = year;
                expiryDayOfYear = dayOfYear;
            }
        }

        int? jobId = null;
        if (TryReadByteAt(reader, contract, layout.ContractJobIdOffset, out var rawJobId)
            && rawJobId != 0)
        {
            jobId = rawJobId;
        }

        return new StaffContractFields
        {
            WeeklyWageGbp = wage,
            ContractExpiryYear = expiryYear,
            ContractExpiryDayOfYear = expiryDayOfYear,
            JobId = jobId,
        };
    }

    private static bool TryReadPointerAt(
        IMemoryReader reader,
        ulong address,
        int offset,
        out ulong value)
    {
        value = 0;
        return TryAdd(address, offset, out var fieldAddress)
            && reader.TryReadUInt64(fieldAddress, out value);
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

    private static bool TryReadByteAt(
        IMemoryReader reader,
        ulong address,
        int offset,
        out byte value)
    {
        value = 0;
        return TryAdd(address, offset, out var fieldAddress)
            && reader.TryReadByte(fieldAddress, out value);
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
