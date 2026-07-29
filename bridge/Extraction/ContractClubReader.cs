using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

public sealed class ContractClubLink
{
    public ulong ClubAddress { get; init; }

    public ulong TeamAddress { get; init; }

    public string? ClubName { get; init; }

    public string? Division { get; init; }

    public int TeamReputation { get; init; }
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

        if (!reader.TryReadUInt64(personAddress + (ulong)layout.FullContractPtrOffset, out var contract)
            || contract == 0)
        {
            return null;
        }

        if (!reader.TryReadUInt64(contract + (ulong)layout.ContractTeamPtrOffset, out var team)
            || team == 0)
        {
            return null;
        }

        var rep = 0;
        if (reader.TryReadUInt16(team + (ulong)layout.TeamReputationOffset, out var trep)
            && trep is >= 0 and <= 12000)
        {
            rep = trep;
        }

        if (!reader.TryReadUInt64(team + (ulong)layout.TeamClubPtrOffset, out var club) || club == 0)
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
}
