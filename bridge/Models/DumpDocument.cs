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
}

public sealed class DumpDocument
{
    public int SchemaVersion { get; init; }

    public string GeneratedAtUtc { get; init; } = "";

    public string GameVersion { get; init; } = "";

    public string SupportedGameVersion { get; init; } = "";

    public string BridgeVersion { get; init; } = "";

    public int ProtocolVersion { get; init; }

    public int PlayerCount { get; init; }

    public IReadOnlyList<DumpPlayer> Players { get; init; } = Array.Empty<DumpPlayer>();
}
