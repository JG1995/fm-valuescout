namespace FmDataBridge.Protocol;

/// <summary>
/// The outcome of each requested squad move, in request order. It carries no player identity or
/// memory detail.
/// </summary>
public sealed class SquadMoveResult
{
    public IReadOnlyList<SquadMoveOutcome> Moves { get; init; } = Array.Empty<SquadMoveOutcome>();
}

public sealed class SquadMoveOutcome
{
    public const string Moved = "moved";
    public const string AlreadyThere = "already-there";
    public const string NotFound = "not-found";
    public const string NotInClubTeams = "not-in-club-teams";
    public const string NoTargetTeam = "no-target-team";
    public const string NotConfirmed = "not-confirmed";

    /// <summary>One of the constants above.</summary>
    public string Outcome { get; init; } = "";

    /// <summary>FM's team type the player is in afterwards, when read.</summary>
    public int? TeamType { get; init; }

    /// <summary>The level of <see cref="TeamType"/>: senior, reserve, or youth.</summary>
    public string? TeamLevel { get; init; }
}
