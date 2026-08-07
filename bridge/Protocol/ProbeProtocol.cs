namespace FmDataBridge.Protocol;

/// <summary>
/// Versioned developer-only protocol for bounded FM memory research captures.
/// </summary>
public static class ProbeProtocol
{
    public const int ProtocolVersion = 1;
    public const int SchemaVersion = 2;

    public const string StateScanning = "scanning";
    public const string StateReady = "ready";
    public const string StateFailed = "failed";

    public const string RequestFileName = "probe-request.json";
    public const string StatusFileName = "probe-status.json";
    public const string ProbeFileName = "probe.json";
}
