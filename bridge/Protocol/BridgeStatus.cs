namespace FmDataBridge.Protocol;

public sealed class BridgeStatus
{
    public int ProtocolVersion { get; init; }

    public string PluginVersion { get; init; } = "";

    public string State { get; init; } = "";

    public DateTimeOffset UpdatedAtUtc { get; init; }

    public bool GamePluginModulePresent { get; init; }

    public bool GameAssemblyModulePresent { get; init; }

    /// <summary>Request id currently being served, when known.</summary>
    public string? RequestId { get; init; }

    /// <summary>Players accepted into the last successful dump (or mid-scan count when cheap).</summary>
    public int? PlayersFound { get; init; }

    /// <summary>Human-readable failure when <see cref="State"/> is failed.</summary>
    public string? Error { get; init; }
}
