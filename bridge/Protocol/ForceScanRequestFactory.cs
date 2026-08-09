namespace FmDataBridge.Protocol;

/// <summary>
/// Creates a distinct provenance ID for each manual force scan so it cannot reuse a prior live candidate index.
/// </summary>
internal static class ForceScanRequestFactory
{
    public static BridgeRequest Create(DateTimeOffset createdAtUtc) =>
        new()
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            RequestId = $"force-scan-{Guid.NewGuid():N}",
            CreatedAtUtc = createdAtUtc,
            Operation = BridgeProtocol.OperationFullDump,
            PlayerDatabaseScope = PlayerDatabaseScopes.Men,
        };
}
