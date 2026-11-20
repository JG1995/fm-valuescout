namespace FmDataBridge.Extraction;

/// <summary>
/// Maps FM's team-type byte to the dump's team level. Verified on FM 27.1 against each team's
/// competition and median age: 0 first team; 1 reserves, 3 and 15 B teams, 9 under-23s, and
/// 10 under-21s are reserve teams; 11 and 18 (under-19 and under-20) and 12 (under-18 and
/// under-17) are youth teams. Other values are rare and count as youth; 22 is the Youth
/// Candidates team (verified 2026-11-20), which holds youth intake prospects, not a squad.
/// </summary>
public static class TeamLevelMap
{
    public const string Senior = "senior";
    public const string Reserve = "reserve";
    public const string Youth = "youth";

    /// <summary>A team of a separate B club counts as its parent club's reserve team.</summary>
    public static string? FromTeamType(int teamType, bool reserveClub = false) =>
        reserveClub
            ? Reserve
            : teamType switch
            {
                < 0 => null,
                0 => Senior,
                <= 10 or 15 => Reserve,
                _ => Youth,
            };

    /// <summary>
    /// Whether the team type is one of the verified first, reserve, or youth teams above. Only
    /// these may receive a moved player or a hired staff member.
    /// </summary>
    public static bool IsSquadTeam(int teamType) =>
        teamType is 0 or 1 or 3 or 9 or 10 or 15 or 11 or 12 or 18;

    /// <summary>Senior before reserve before youth, so a player listed twice counts where he ranks highest.</summary>
    public static int Rank(int teamType, bool reserveClub = false) =>
        FromTeamType(teamType, reserveClub) switch
        {
            Senior => 0,
            Reserve => 1,
            Youth => 2,
            _ => 3,
        };
}
