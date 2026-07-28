namespace FmDataBridge.Protocol;

public sealed class BridgeStatus
{
    public int ProtocolVersion { get; init; }

    public string PluginVersion { get; init; } = "";

    public string State { get; init; } = "";

    public DateTimeOffset UpdatedAtUtc { get; init; }

    public bool GamePluginModulePresent { get; init; }

    public bool GameAssemblyModulePresent { get; init; }
}
