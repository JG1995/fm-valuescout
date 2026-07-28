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
}
