namespace FmDataBridge.Extraction;

/// <summary>
/// Maps FM team-type byte to dump team-level labels.
/// </summary>
public static class TeamLevelMap
{
    /// <summary>0 = first team; 1–9 ≈ reserves; ≥10 = youth (SuperScout pin).</summary>
    public static string? FromTeamType(int teamType) =>
        teamType switch
        {
            < 0 => null,
            0 => "senior",
            < 10 => "reserve",
            _ => "youth",
        };
}
