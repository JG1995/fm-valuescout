namespace FmDataBridge.Models;

public sealed class DumpPlayer
{
    public uint Uid { get; init; }

    public int Ca { get; init; }

    public int Pa { get; init; }
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
