namespace FmDataBridge.Models;

/// <summary>
/// Bridge-internal staff data retained until schema v6 publishes it.
/// </summary>
public sealed record StaffRecord
{
    public uint Uid { get; init; }

    public string? Name { get; init; }

    public int? BirthYear { get; init; }

    public int? BirthDayOfYear { get; init; }

    public int? Age { get; init; }

    public IReadOnlyList<string> Nationalities { get; init; } = Array.Empty<string>();

    public uint? NationUid { get; init; }

    public PlayerGender Gender { get; init; }

    public int Ca { get; init; }

    public int Pa { get; init; }

    /// <summary>Stable English keys, each on the 1–20 scale; null = unread or invalid.</summary>
    public IReadOnlyDictionary<string, int?> Attributes { get; init; } =
        new Dictionary<string, int?>();

    public int? JobId { get; init; }

    public long? WeeklyWageGbp { get; init; }

    public int? ContractExpiryYear { get; init; }

    public int? ContractExpiryDayOfYear { get; init; }

    public string? Club { get; init; }

    public string? Division { get; init; }
}
