namespace FmDataBridge.Extraction;

/// <summary>One squad-walk hit for a player UID.</summary>
public readonly record struct SquadHit(string ClubName, int TeamType, string? Division);

/// <summary>
/// Deterministic multi-squad conflict rules (SuperScout PickSquad).
/// Same club → lower team type wins. Different clubs → prefer the non-parent (loan current).
/// </summary>
public static class SquadPick
{
    public static SquadHit Choose(SquadHit current, SquadHit candidate, string? parentClub)
    {
        if (current.ClubName == candidate.ClubName)
        {
            return candidate.TeamType < current.TeamType ? candidate : current;
        }

        var curIsParent = parentClub != null && current.ClubName == parentClub;
        var newIsParent = parentClub != null && candidate.ClubName == parentClub;
        if (curIsParent == newIsParent)
        {
            return candidate.TeamType < current.TeamType ? candidate : current;
        }

        // Prefer the non-parent club — that is where the player currently plays on loan.
        return curIsParent ? candidate : current;
    }
}
