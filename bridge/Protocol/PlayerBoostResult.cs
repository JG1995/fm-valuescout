namespace FmDataBridge.Protocol;

/// <summary>
/// Sanitized result for one closed player-boost action. It deliberately contains no player identity or memory detail.
/// </summary>
public sealed class PlayerBoostResult
{
    public string Operation { get; init; } = "";

    /// <summary><c>verified</c>, <c>failed</c>, or <c>partial-unverified</c>.</summary>
    public string Outcome { get; init; } = "";

    /// <summary><c>not-needed</c>, <c>restored</c>, or <c>unverified</c>.</summary>
    public string Rollback { get; init; } = "";

    public int? PreviousCurrentAbility { get; init; }

    public int? CurrentAbility { get; init; }

    public int? PotentialAbility { get; init; }

    public int? PreviousAmbition { get; init; }

    public int? Ambition { get; init; }

    public int? PreviousProfessionalism { get; init; }

    public int? Professionalism { get; init; }

    public int? PreviousDetermination { get; init; }

    public int? Determination { get; init; }
}
