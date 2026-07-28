namespace FmDataBridge.Protocol;

public static class BridgePaths
{
    public static string GetBridgeDirectory()
    {
        var localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        return Path.Combine(
            localAppData,
            BridgeProtocol.AppFolderName,
            BridgeProtocol.BridgeFolderName);
    }

    public static string EnsureBridgeDirectory()
    {
        var directory = GetBridgeDirectory();
        Directory.CreateDirectory(directory);
        return directory;
    }

    public static string GetStatusPath(string bridgeDirectory) =>
        Path.Combine(bridgeDirectory, BridgeProtocol.StatusFileName);

    public static string GetRequestPath(string bridgeDirectory) =>
        Path.Combine(bridgeDirectory, BridgeProtocol.RequestFileName);

    public static string GetDumpPath(string bridgeDirectory) =>
        Path.Combine(bridgeDirectory, BridgeProtocol.DumpFileName);

    public static string GetDiagnosticsPath(string bridgeDirectory) =>
        Path.Combine(bridgeDirectory, BridgeProtocol.DiagnosticsFileName);

    public static string GetForceScanPath(string bridgeDirectory) =>
        Path.Combine(bridgeDirectory, BridgeProtocol.ForceScanFileName);
}
