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

    /// <summary>True when the last ready dump stopped at <see cref="MaxAccepted"/>.</summary>
    public bool? ScanTruncated { get; init; }

    /// <summary>Accepted-player cap used for the last ready dump; null when unlimited.</summary>
    public int? MaxAccepted { get; init; }

    /// <summary>Whether this loaded exact FM build has a live candidate index for the two boost actions.</summary>
    public bool? PlayerBoostsSupported { get; init; }

    /// <summary>Whether this exact FM build has proved staff CA writes and a live staff candidate index.</summary>
    public bool? StaffBoostsSupported { get; init; }

    /// <summary>Sanitized verified result for the last player-boost request, when applicable.</summary>
    public PlayerBoostResult? PlayerBoost { get; init; }

    /// <summary>Sanitized verified result for the last staff CA boost, when applicable.</summary>
    public StaffBoostResult? StaffBoost { get; init; }
}
