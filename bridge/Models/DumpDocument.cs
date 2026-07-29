namespace FmDataBridge.Models;

public sealed class DumpPlayer
{
    public uint Uid { get; init; }

    public int Ca { get; init; }

    public int Pa { get; init; }

    public string Name { get; init; } = "";

    public int BirthYear { get; init; }

    public int BirthDayOfYear { get; init; }

    public IReadOnlyList<string> Nationalities { get; init; } = Array.Empty<string>();

    public int? HeightCm { get; init; }

    public string PreferredFoot { get; init; } = "";

    public IReadOnlyDictionary<string, int> Positions { get; init; } =
        new Dictionary<string, int>();

    /// <summary>Visible technical/mental/physical attrs on the 1–20 scale; null = unread/invalid.</summary>
    public IReadOnlyDictionary<string, int?> Attributes { get; init; } =
        new Dictionary<string, int?>();

    /// <summary>Hidden scouting attrs on the 1–20 scale; null = unread/invalid.</summary>
    public IReadOnlyDictionary<string, int?> HiddenAttributes { get; init; } =
        new Dictionary<string, int?>();

    /// <summary>Personality attrs on the 1–20 scale (raw in memory, not ×5); null = unread/invalid.</summary>
    public IReadOnlyDictionary<string, int?> Personality { get; init; } =
        new Dictionary<string, int?>();

    /// <summary>Weekly wage in GBP as stored in memory; null = free agent / unread / unset sentinel.</summary>
    public long? WeeklyWageGbp { get; init; }

    /// <summary>Contract expiry year; null = free agent / unread / impossible date.</summary>
    public int? ContractExpiryYear { get; init; }

    /// <summary>Contract expiry day-of-year (1–366); null with <see cref="ContractExpiryYear"/>.</summary>
    public int? ContractExpiryDayOfYear { get; init; }

    /// <summary>Transfer-listed or listed-by-request; null = free agent / unread flags.</summary>
    public bool? TransferListed { get; init; }

    /// <summary>Loan-listed; null = free agent / unread flags.</summary>
    public bool? LoanListed { get; init; }

    /// <summary>Not-for-sale; null = free agent / unread flags.</summary>
    public bool? NotForSale { get; init; }

    /// <summary>Set for release; null = free agent / unread flags.</summary>
    public bool? SetForRelease { get; init; }

    /// <summary>FM market value in GBP; null = unread / unset / unfixed sentinel.</summary>
    public long? MarketValueGbp { get; init; }

    /// <summary>Player reputation (current / world); field nulls when unread.</summary>
    public DumpReputation Reputation { get; init; } = new();

    /// <summary>Club where the player currently appears in a squad; null = unresolved / free agent.</summary>
    public string? CurrentClub { get; init; }

    /// <summary>Parent / owning club from the full-contract chain; null = unresolved / free agent.</summary>
    public string? ParentClub { get; init; }

    /// <summary>True when current and parent clubs both resolve and differ (loan).</summary>
    public bool? OnLoan { get; init; }

    /// <summary>Competition / division name for the current team; null when unread.</summary>
    public string? Division { get; init; }

    /// <summary>senior / reserve / youth from FM team type; null when unknown.</summary>
    public string? TeamLevel { get; init; }

    /// <summary>Age at dump game date; null when DOB or game date missing.</summary>
    public int? Age { get; init; }
}

public sealed class DumpReputation
{
    public int? Current { get; init; }

    public int? World { get; init; }
}

public sealed class DumpDocument
{
    public int SchemaVersion { get; init; }

    public string GeneratedAtUtc { get; init; } = "";

    public string GameVersion { get; init; } = "";

    public string SupportedGameVersion { get; init; } = "";

    public string BridgeVersion { get; init; } = "";

    public int ProtocolVersion { get; init; }

    /// <summary>In-game date as yyyy-MM-dd when known.</summary>
    public string? GameDate { get; init; }

    /// <summary>memory | derived | unknown</summary>
    public string GameDateSource { get; init; } = "unknown";

    /// <summary>True when the person scanner stopped because <see cref="MaxAccepted"/> was reached.</summary>
    public bool ScanTruncated { get; init; }

    /// <summary>Accepted-player cap used for this dump; null when the walk was unlimited.</summary>
    public int? MaxAccepted { get; init; }

    public int PlayerCount { get; init; }

    public IReadOnlyList<DumpPlayer> Players { get; init; } = Array.Empty<DumpPlayer>();
}
