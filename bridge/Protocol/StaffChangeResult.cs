namespace FmDataBridge.Protocol;

/// <summary>The outcome of each requested staff change, in request order.</summary>
public sealed class StaffChangeResult
{
    public IReadOnlyList<StaffChangeOutcome> Changes { get; init; } = Array.Empty<StaffChangeOutcome>();
}

/// <summary>What FM did with one staff change, and the person's contract afterwards as read from FM.</summary>
public sealed class StaffChangeOutcome
{
    public const string Sacked = "sacked";
    public const string Hired = "hired";
    public const string Changed = "changed";

    /// <summary>A change whose sack went through but whose re-hire did not: the person is unemployed.</summary>
    public const string LeftUnemployed = "left-unemployed";
    public const string NotFound = "not-found";
    public const string NotAtClub = "not-at-club";
    public const string NoTargetTeam = "no-target-team";
    public const string NotConfirmed = "not-confirmed";

    /// <summary>One of the constants above.</summary>
    public string Outcome { get; init; } = "";

    /// <summary>The contract's club; null when the person has no contract.</summary>
    public uint? ClubUid { get; init; }

    public string? Club { get; init; }

    public string? Division { get; init; }

    public int? JobId { get; init; }

    public long? WeeklyWageGbp { get; init; }

    public int? ContractExpiryYear { get; init; }

    public int? ContractExpiryDayOfYear { get; init; }

    public int? TeamType { get; init; }

    /// <summary>The level of <see cref="TeamType"/>: senior, reserve, or youth.</summary>
    public string? TeamLevel { get; init; }
}
