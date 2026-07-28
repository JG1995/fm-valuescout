namespace FmDataBridge.Protocol;

public static class BridgeProtocol
{
    public const int ProtocolVersion = 1;
    public const int DumpSchemaVersion = 1;

    /// <summary>Ignore request.json older than this many seconds (SuperScout-class stale fix).</summary>
    public const int RequestTtlSeconds = 30;

    public const string StateIdle = "idle";
    public const string StateScanning = "scanning";
    public const string StateReady = "ready";
    public const string StateFailed = "failed";
    public const string OperationFullDump = "full-dump";
    public const string AppFolderName = "fm-valuescout";
    public const string BridgeFolderName = "fm-bridge";
    public const string StatusFileName = "status.json";
    public const string RequestFileName = "request.json";
    public const string DumpFileName = "dump.json";
    public const string DiagnosticsFileName = "diagnostics.txt";
    public const string ForceScanFileName = "force-scan";
}
