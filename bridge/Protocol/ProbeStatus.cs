namespace FmDataBridge.Protocol;

/// <summary>
/// Terminal or in-progress state for a developer memory-probe request.
/// </summary>
public sealed class ProbeStatus
{
    public int ProtocolVersion { get; init; }

    public string PluginVersion { get; init; } = "";

    public string State { get; init; } = "";

    public DateTimeOffset UpdatedAtUtc { get; init; }

    public bool GamePluginModulePresent { get; init; }

    public bool GameAssemblyModulePresent { get; init; }

    public string? RequestId { get; init; }

    public int? PlayersCaptured { get; init; }

    public string? Error { get; init; }
}
