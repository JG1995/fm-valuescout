using System.Text.Json;
using FmDataBridge.Protocol;

namespace FmDataBridge.Output;

public static class StatusWriter
{
    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true,
    };

    public static string Serialize(BridgeStatus status) =>
        JsonSerializer.Serialize(status, SerializerOptions);

    public static void Write(string bridgeDirectory, BridgeStatus status)
    {
        Directory.CreateDirectory(bridgeDirectory);
        var path = BridgePaths.GetStatusPath(bridgeDirectory);
        var tempPath = path + ".tmp";
        File.WriteAllText(tempPath, Serialize(status));
        File.Move(tempPath, path, overwrite: true);
    }
}
