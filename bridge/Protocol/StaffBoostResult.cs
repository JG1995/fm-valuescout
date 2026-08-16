namespace FmDataBridge.Protocol;

/// <summary>Sanitized result for the closed staff CA boost.</summary>
public sealed class StaffBoostResult
{
    public string Operation { get; init; } = "";

    public string Outcome { get; init; } = "";

    public string Rollback { get; init; } = "";

    public int? PreviousCurrentAbility { get; init; }

    public int? CurrentAbility { get; init; }

    public int? PotentialAbility { get; init; }
}
