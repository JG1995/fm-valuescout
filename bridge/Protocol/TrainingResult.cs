namespace FmDataBridge.Protocol;

/// <summary>
/// The outcome of each requested training change, in request order, with the training FM holds
/// for the player afterwards. It carries no player identity or memory detail.
/// </summary>
public sealed class TrainingResult
{
    public IReadOnlyList<TrainingOutcome> Players { get; init; } = Array.Empty<TrainingOutcome>();
}

public sealed class TrainingOutcome
{
    public const string Applied = "applied";
    public const string NotFound = "not-found";
    public const string NotAtClub = "not-at-club";
    public const string NotConfirmed = "not-confirmed";

    /// <summary>One of the constants above. Applied means FM holds every requested part.</summary>
    public string Outcome { get; init; } = "";

    /// <summary>FM's in-possession position mask afterwards, when read.</summary>
    public int? IpPosition { get; init; }

    /// <summary>FM's in-possession role value afterwards; 0 trains the position itself.</summary>
    public ulong? IpRole { get; init; }

    /// <summary>FM's name for the in-possession training, such as "Half Back".</summary>
    public string? IpRoleName { get; init; }

    public int? OopRole { get; init; }

    public string? OopRoleName { get; init; }

    public int? Focus { get; init; }

    public string? FocusName { get; init; }
}
