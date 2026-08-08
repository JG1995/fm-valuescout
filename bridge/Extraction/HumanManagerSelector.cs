using FmDataBridge.Models;
using FmDataBridge.Scanning;

namespace FmDataBridge.Extraction;

public static class HumanManagerSelector
{
    public static HumanManager? Select(
        IEnumerable<PersonCandidate> candidates,
        IReadOnlyDictionary<uint, StaffRecord> staffByUid,
        IReadOnlyDictionary<uint, ContractClubLink?> contractLinksByUid,
        SquadClubIndex clubIndex)
    {
        ArgumentNullException.ThrowIfNull(candidates);
        ArgumentNullException.ThrowIfNull(staffByUid);
        ArgumentNullException.ThrowIfNull(contractLinksByUid);
        ArgumentNullException.ThrowIfNull(clubIndex);

        var selections = new List<ManagerSelection>();
        foreach (var candidate in candidates.OrderBy(candidate => candidate.Uid).ThenBy(candidate => candidate.ObjectAddress))
        {
            if (!staffByUid.TryGetValue(candidate.Uid, out var staff)
                || string.IsNullOrWhiteSpace(staff.Name))
            {
                continue;
            }

            clubIndex.TryGetHumanManager(candidate.ObjectAddress, out var graphClub);
            contractLinksByUid.TryGetValue(candidate.Uid, out var contractClub);
            selections.Add(new ManagerSelection(candidate, staff, graphClub, contractClub));
        }

        var selected = selections
            .OrderBy(selection => selection.SourceRank)
            .ThenBy(selection => selection.GraphClub?.TeamType ?? int.MaxValue)
            .ThenBy(selection => selection.Candidate.Uid)
            .ThenBy(selection => selection.Candidate.ObjectAddress)
            .FirstOrDefault();
        if (selected is null)
        {
            return null;
        }

        return new HumanManager
        {
            Uid = selected.Candidate.Uid,
            Name = selected.Staff.Name!,
            Club = selected.GraphClub?.ClubName ?? selected.ContractClub?.ClubName,
            ClubReputation = selected.GraphClub?.TeamReputation ?? selected.ContractClub?.TeamReputation,
        };
    }

    private sealed class ManagerSelection
    {
        public ManagerSelection(
            PersonCandidate candidate,
            StaffRecord staff,
            HumanManagerClubAssignment? graphClub,
            ContractClubLink? contractClub)
        {
            Candidate = candidate;
            Staff = staff;
            GraphClub = graphClub;
            ContractClub = contractClub;
        }

        public PersonCandidate Candidate { get; }

        public StaffRecord Staff { get; }

        public HumanManagerClubAssignment? GraphClub { get; }

        public ContractClubLink? ContractClub { get; }

        public int SourceRank => GraphClub switch
        {
            { TeamType: 0 } => 0,
            not null => 1,
            _ when ContractClub?.ClubName is not null => 2,
            _ => 3,
        };
    }
}
