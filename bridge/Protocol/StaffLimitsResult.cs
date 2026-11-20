namespace FmDataBridge.Protocol;

/// <summary>
/// The board's staff limits for the managed club's first, reserve, and youth teams, as FM's staff
/// screens show them.
/// </summary>
public sealed class StaffLimitsResult
{
    public IReadOnlyList<StaffLimitsTeam> Teams { get; init; } = Array.Empty<StaffLimitsTeam>();
}

/// <summary>One team's staff groups; a team FM sent no groups for lists none.</summary>
public sealed class StaffLimitsTeam
{
    /// <summary>senior, reserve, or youth.</summary>
    public string Level { get; init; } = "";

    public int TeamType { get; init; }

    public IReadOnlyList<StaffLimitsGroup> Groups { get; init; } = Array.Empty<StaffLimitsGroup>();
}

/// <summary>One staff group of a team: the board's limit for the group and for each of its jobs.</summary>
public sealed class StaffLimitsGroup
{
    public const string Coaching = "coaching";
    public const string Recruitment = "recruitment";
    public const string Medical = "medical";

    /// <summary>One of the constants above.</summary>
    public string Group { get; init; } = "";

    /// <summary>FM's name for the group, such as "Coaching Team".</summary>
    public string? Name { get; init; }

    /// <summary>How many staff the group holds now.</summary>
    public int? Count { get; init; }

    /// <summary>The board's limit for the whole group.</summary>
    public int Max { get; init; }

    public IReadOnlyList<StaffLimitsJob> Jobs { get; init; } = Array.Empty<StaffLimitsJob>();
}

/// <summary>The board's limit for one job.</summary>
public sealed class StaffLimitsJob
{
    /// <summary>FM's job type, the number a staff contract holds, such as 2 for Coach.</summary>
    public int JobType { get; init; }

    /// <summary>FM's name for the job, such as "Coaches".</summary>
    public string? Name { get; init; }

    /// <summary>How many staff hold the job now.</summary>
    public int? Count { get; init; }

    public int Max { get; init; }
}
