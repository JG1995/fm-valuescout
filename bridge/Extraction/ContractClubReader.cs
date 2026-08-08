using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

public sealed class ContractClubLink
{
    public ulong ClubAddress { get; init; }

    public ulong TeamAddress { get; init; }

    public string? ClubName { get; init; }

    public string? Division { get; init; }

    public int? TeamReputation { get; init; }
}

/// <summary>
/// Parent club via person → full contract → team → club (SuperScout ResolveClub).
/// </summary>
public static class ContractClubReader
{
    public static ContractClubLink? TryRead(
        IMemoryReader reader,
        ulong personAddress,
        IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        if (!TryReadPointerAt(reader, personAddress, layout.FullContractPtrOffset, out var contract)
            || contract == 0)
        {
            return null;
        }

        if (!TryReadPointerAt(reader, contract, layout.ContractTeamPtrOffset, out var team)
            || team == 0)
        {
            return null;
        }

        int? rep = null;
        if (TryReadUInt16At(reader, team, layout.TeamReputationOffset, out var trep)
            && trep <= 12000)
        {
            rep = trep;
        }

        if (!TryReadPointerAt(reader, team, layout.TeamClubPtrOffset, out var club) || club == 0)
        {
            return new ContractClubLink
            {
                TeamAddress = team,
                Division = CompetitionNameReader.TryRead(reader, team, layout),
                TeamReputation = rep,
            };
        }

        return new ContractClubLink
        {
            ClubAddress = club,
            TeamAddress = team,
            ClubName = ClubNameReader.TryRead(reader, club, layout),
            Division = CompetitionNameReader.TryRead(reader, team, layout),
            TeamReputation = rep,
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

    private static bool TryReadUInt16At(
        IMemoryReader reader,
        ulong address,
        int offset,
        out ushort value)
    {
        value = 0;
        return TryAdd(address, offset, out var fieldAddress)
            && reader.TryReadUInt16(fieldAddress, out value);
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
