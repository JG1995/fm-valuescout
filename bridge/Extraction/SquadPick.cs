namespace FmDataBridge.Extraction;

/// <summary>One squad-walk hit for a player UID.</summary>
public readonly record struct SquadHit(
    string ClubName,
    int TeamType,
    string? Division,
    int? TeamReputation = null,
    uint? ClubUid = null,
    bool ReserveClub = false);

/// <summary>
/// Deterministic multi-squad conflict rules (SuperScout PickSquad).
/// Same club → the higher team level wins (senior, then reserve, then youth). Different clubs →
/// prefer the non-parent (loan current).
/// </summary>
public static class SquadPick
{
    public static SquadHit Choose(SquadHit current, SquadHit candidate, string? parentClub)
    {
        if (current.ClubName == candidate.ClubName)
        {
            return Ranks(candidate).CompareTo(Ranks(current)) < 0 ? candidate : current;
        }

        var curIsParent = parentClub != null && current.ClubName == parentClub;
        var newIsParent = parentClub != null && candidate.ClubName == parentClub;
        if (curIsParent == newIsParent)
        {
            return Ranks(candidate).CompareTo(Ranks(current)) < 0 ? candidate : current;
        }

        // Prefer the non-parent club — that is where the player currently plays on loan.
        return curIsParent ? candidate : current;
    }

    private static (int Level, int Type) Ranks(SquadHit hit) =>
        (TeamLevelMap.Rank(hit.TeamType, hit.ReserveClub), hit.TeamType);
}
