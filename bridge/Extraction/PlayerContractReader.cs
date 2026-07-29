using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;

namespace FmDataBridge.Extraction;

public sealed class PlayerContractFields
{
    public long? WeeklyWageGbp { get; init; }

    public int? ContractExpiryYear { get; init; }

    public int? ContractExpiryDayOfYear { get; init; }

    public bool? TransferListed { get; init; }

    public bool? LoanListed { get; init; }

    public bool? NotForSale { get; init; }

    public bool? SetForRelease { get; init; }

    public long? MarketValueGbp { get; init; }

    public DumpReputation Reputation { get; init; } = new();
}

public static class PlayerContractReader
{
    // Status flag bits on contract+ContractStatusFlagsOffset (SuperScout Fields.cs).
    private const byte FlagListed = 1 << 0;
    private const byte FlagLoanListed = 1 << 1;
    private const byte FlagListedByRequest = 1 << 3;
    private const byte FlagNotForSale = 1 << 4;
    private const byte FlagSetForRelease = 1 << 5;

    public static PlayerContractFields Read(
        IMemoryReader reader,
        ulong personAddress,
        ulong playerBlockBase,
        IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        var marketValue = TryReadMarketValue(reader, playerBlockBase, layout);
        var reputation = TryReadReputation(reader, playerBlockBase, layout);

        if (!reader.TryReadUInt64(personAddress + (ulong)layout.FullContractPtrOffset, out var contractPtr)
            || contractPtr == 0)
        {
            return new PlayerContractFields
            {
                MarketValueGbp = marketValue,
                Reputation = reputation,
            };
        }

        long? wage = null;
        if (reader.TryReadUInt32(contractPtr + (ulong)layout.ContractWeeklyWageOffset, out var wageRaw))
        {
            wage = MoneyDecode.TryGbp(wageRaw);
        }

        int? expiryYear = null;
        int? expiryDoy = null;
        if (reader.TryReadUInt32(contractPtr + (ulong)layout.ContractExpiryOffset, out var expiryRaw))
        {
            var (year, doy) = FmDateDecoder.Decode(expiryRaw);
            // Contract dates in FM26 are 2000+; earlier packed years are unset/sentinel.
            if (year >= 2000 && FmDateDecoder.IsPlausible(year, doy))
            {
                expiryYear = year;
                expiryDoy = doy;
            }
        }

        bool? transferListed = null;
        bool? loanListed = null;
        bool? notForSale = null;
        bool? setForRelease = null;
        if (reader.TryReadByte(contractPtr + (ulong)layout.ContractStatusFlagsOffset, out var flags))
        {
            transferListed = (flags & (FlagListed | FlagListedByRequest)) != 0;
            loanListed = (flags & FlagLoanListed) != 0;
            notForSale = (flags & FlagNotForSale) != 0;
            setForRelease = (flags & FlagSetForRelease) != 0;
        }

        return new PlayerContractFields
        {
            WeeklyWageGbp = wage,
            ContractExpiryYear = expiryYear,
            ContractExpiryDayOfYear = expiryDoy,
            TransferListed = transferListed,
            LoanListed = loanListed,
            NotForSale = notForSale,
            SetForRelease = setForRelease,
            MarketValueGbp = marketValue,
            Reputation = reputation,
        };
    }

    private static long? TryReadMarketValue(
        IMemoryReader reader,
        ulong playerBlockBase,
        IFmMemoryLayout layout)
    {
        if (!reader.TryReadUInt32(playerBlockBase + (ulong)layout.MarketValueOffset, out var raw))
        {
            return null;
        }

        return MoneyDecode.TryMarketValueGbp(raw);
    }

    private static DumpReputation TryReadReputation(
        IMemoryReader reader,
        ulong playerBlockBase,
        IFmMemoryLayout layout)
    {
        int? current = null;
        int? world = null;
        if (reader.TryReadUInt16(playerBlockBase + (ulong)layout.CurrentReputationOffset, out var cur))
        {
            current = cur;
        }

        if (reader.TryReadUInt16(playerBlockBase + (ulong)layout.WorldReputationOffset, out var wrld))
        {
            world = wrld;
        }

        return new DumpReputation { Current = current, World = world };
    }
}
