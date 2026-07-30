namespace FmDataBridge.Protocol;

/// <summary>
/// Versioned scan request written by the Tauri app under the bridge data directory.
/// </summary>
public sealed class BridgeRequest
{
    public int ProtocolVersion { get; init; }

    public string RequestId { get; init; } = "";

    public DateTimeOffset CreatedAtUtc { get; init; }

    /// <summary>Supported: <see cref="BridgeProtocol.OperationFullDump"/>.</summary>
    public string Operation { get; init; } = "";

    /// <summary>
    /// Optional accepted-player cap. <c>null</c> (or omitted) means unlimited;
    /// a positive integer stops after that many accepted players.
    /// </summary>
    public int? MaxAccepted { get; init; }
}
